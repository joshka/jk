//! `jj operation log` view state, rendering, and item-based navigation.
//!
//! The first pass keeps the operation log close to rendered `jj` output while
//! carrying exact operation ids separately for copy, search, and refresh
//! stability. Recovery includes global `jj undo`/`jj redo` on the repository cursor
//! and selected-row `jj operation restore`/`jj operation revert` flows using exact
//! operation ids.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

use crate::action_menu::{ActionKind, ActionMenu, ActionMenuItem, FollowUp, SafetyTier};
use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::ViewSpec;
use crate::jj_rows::{OperationLogItem, load_operation_log_entries};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;
use crate::theme;

pub const BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Down),
        Command::View(ViewCommand::MoveDown),
    ),
    Binding::new(KeyPattern::char('k'), Command::View(ViewCommand::MoveUp)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Up),
        Command::View(ViewCommand::MoveUp),
    ),
    Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Home),
        Command::View(ViewCommand::MoveFirst),
    ),
    Binding::new(KeyPattern::char('G'), Command::View(ViewCommand::MoveLast)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::End),
        Command::View(ViewCommand::MoveLast),
    ),
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenShow)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Enter),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(KeyPattern::char('d'), Command::View(ViewCommand::OpenDiff)),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
    Binding::new(KeyPattern::char('u'), Command::OperationUndo),
    Binding::new(
        KeyPattern::modified_char('r', crossterm::event::KeyModifiers::CONTROL),
        Command::OperationRedo,
    ),
];

/// Selectable rendered operation-log output.
pub struct OperationLogView {
    spec: ViewSpec,
    entries: Vec<OperationLogItem>,
    selection: Selection,
}

impl OperationLogView {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: load_operation_log_entries(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    #[cfg(test)]
    pub(crate) fn test_new(entries: Vec<OperationLogItem>) -> Self {
        Self {
            spec: ViewSpec::new(crate::jj::JjCommand::OperationLog, Vec::new()),
            entries,
            selection: Selection::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(entry_list(&self.entries, search), area, &mut state);
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::MoveDown => {
                self.selection.next(self.entries.len());
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.selection.previous();
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.selection.first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.selection.last(self.entries.len());
                ViewEffect::Handled
            }
            ViewCommand::OpenShow => self
                .selected_operation_id()
                .map(|operation_id| ViewEffect::OpenView(ViewSpec::operation_show(operation_id)))
                .unwrap_or_else(|| {
                    ViewEffect::StatusMessage(
                        "operation show unavailable: selected row has no operation id".to_owned(),
                    )
                }),
            ViewCommand::OpenDiff => self
                .selected_operation_id()
                .map(|operation_id| ViewEffect::OpenView(ViewSpec::operation_diff(operation_id)))
                .unwrap_or_else(|| {
                    ViewEffect::StatusMessage(
                        "operation diff unavailable: selected row has no operation id".to_owned(),
                    )
                }),
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::OpenActionMenu => self
                .selected_operation_id()
                .map(operation_action_menu)
                .map(ViewEffect::OpenActionMenu)
                .unwrap_or_else(|| {
                    ViewEffect::StatusMessage(
                        "operation recovery actions unavailable: selected row has no operation id"
                            .to_owned(),
                    )
                }),
            ViewCommand::ToggleSelect => ViewEffect::Ignored,
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::PageDown
            | ViewCommand::PageUp
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_operation_log_entries)
    }

    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    pub fn line_count(&self) -> usize {
        self.entries.iter().map(OperationLogItem::line_count).sum()
    }

    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.entries.len())
            .chain(0..self.selection.index().min(self.entries.len()))
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.entries.len()).rev())
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    fn copy_options(&self) -> Vec<CopyOption> {
        let Some(entry) = self.entries.get(self.selection.index()) else {
            return Vec::new();
        };

        let mut options = Vec::new();
        if let Some(operation_id) = entry.operation_id() {
            options.push(CopyOption::new("operation id", operation_id));
        }
        options.push(CopyOption::new("row text", entry.row_text()));
        options
    }

    fn selected_operation_id(&self) -> Option<String> {
        self.entries
            .get(self.selection.index())
            .and_then(OperationLogItem::operation_id)
            .map(str::to_owned)
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<OperationLogItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_operation_id = self
            .entries
            .get(previous_index)
            .and_then(OperationLogItem::operation_id)
            .map(str::to_owned);
        self.entries = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_operation_id,
        );
        Ok(())
    }
}

fn operation_action_menu(operation_id: String) -> ActionMenu {
    let short_operation_id = short_id(&operation_id).to_owned();
    ActionMenu::new(vec![
        ActionMenuItem::new(
            ActionKind::Restore,
            format!("restore repository to operation {short_operation_id}"),
            SafetyTier::PreviewFirst,
            FollowUp::OperationRestoreExactTarget {
                operation_id: operation_id.clone(),
            },
        ),
        ActionMenuItem::new(
            ActionKind::Revert,
            format!("revert operation {short_operation_id}"),
            SafetyTier::PreviewFirst,
            FollowUp::OperationRevertExactTarget { operation_id },
        ),
    ])
}

fn entry_list(entries: &[OperationLogItem], search: Option<&SearchQuery>) -> List<'static> {
    let items = entries
        .iter()
        .map(|entry| {
            let lines = entry
                .lines()
                .into_iter()
                .map(|line| highlight_line(line, search))
                .collect::<Vec<_>>();
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}

fn restore_selection(
    selection: &mut Selection,
    entries: &[OperationLogItem],
    previous_index: usize,
    previous_operation_id: Option<String>,
) {
    if let Some(operation_id) = previous_operation_id
        && let Some(index) = entries
            .iter()
            .position(|entry| entry.operation_id() == Some(operation_id.as_str()))
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use ratatui::text::Line;

    use super::*;

    fn operation_item(text: &[&str], operation_id: Option<&str>) -> OperationLogItem {
        OperationLogItem::new(
            text.iter()
                .map(|line| Line::from((*line).to_owned()))
                .collect::<Vec<_>>(),
            operation_id.map(str::to_owned),
        )
    }

    fn operation_log_view(entries: Vec<OperationLogItem>) -> OperationLogView {
        OperationLogView::test_new(entries)
    }

    #[test]
    fn copy_options_include_exact_operation_id_when_known() {
        let view = operation_log_view(vec![operation_item(
            &["@  current", "│  describe commit"],
            Some(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ),
        )]);

        let options = view.copy_options();

        assert_eq!(options.len(), 2);
        assert_eq!(options[0].label(), "operation id");
        assert_eq!(options[0].value().len(), 128);
        assert_eq!(options[1].value(), "@  current\n│  describe commit");
    }

    #[test]
    fn movement_is_operation_item_based() {
        let mut view = operation_log_view(vec![
            operation_item(&["@  current", "│  args: jj describe"], Some("a")),
            operation_item(&["○  previous"], Some("b")),
        ]);

        view.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );

        assert_eq!(view.selection.index(), 1);
        view.execute(
            ViewCommand::MoveUp,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn refresh_preserves_selected_operation_id() {
        let mut view = operation_log_view(vec![
            operation_item(&["@  current"], Some("first")),
            operation_item(&["○  previous"], Some("second")),
        ]);
        view.selection.set(1, view.entries.len());

        view.refresh_with_loader(|_| {
            Ok(vec![
                operation_item(&["@  second"], Some("second")),
                operation_item(&["○  third"], Some("third")),
            ])
        })
        .unwrap();

        assert_eq!(view.selection.index(), 0);
        assert_eq!(view.entries[0].operation_id(), Some("second"));
    }

    #[test]
    fn refresh_clamps_when_selected_operation_disappears() {
        let mut view = operation_log_view(vec![
            operation_item(&["@  current"], Some("first")),
            operation_item(&["○  previous"], Some("second")),
        ]);
        view.selection.set(1, view.entries.len());

        view.refresh_with_loader(|_| Ok(vec![operation_item(&["@  current"], Some("first"))]))
            .unwrap();

        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn search_wraps_by_operation_item() {
        let mut view = operation_log_view(vec![
            operation_item(&["@  current", "│  args: jj describe"], Some("first")),
            operation_item(&["○  previous", "│  snapshot working copy"], Some("second")),
            operation_item(&["○  oldest", "│  snapshot before describe"], Some("third")),
        ]);
        view.selection.set(1, view.entries.len());
        let query = SearchQuery::new("describe".to_owned()).unwrap();

        assert_eq!(view.search_matches(&query), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn operation_show_and_diff_open_selected_operation_detail() {
        let mut view = operation_log_view(vec![operation_item(&["@  current"], Some("first"))]);

        assert_eq!(
            view.execute(
                ViewCommand::OpenShow,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::OpenView(ViewSpec::operation_show("first".to_owned()))
        );
        assert_eq!(
            view.execute(
                ViewCommand::OpenDiff,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::OpenView(ViewSpec::operation_diff("first".to_owned()))
        );
    }

    #[test]
    fn operation_detail_actions_are_disabled_without_operation_id() {
        let mut view = operation_log_view(vec![operation_item(&["@  current"], None)]);

        assert_eq!(
            view.execute(
                ViewCommand::OpenShow,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::StatusMessage(
                "operation show unavailable: selected row has no operation id".to_owned()
            )
        );
        assert_eq!(
            view.execute(
                ViewCommand::OpenDiff,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::StatusMessage(
                "operation diff unavailable: selected row has no operation id".to_owned()
            )
        );
    }

    #[test]
    fn operation_recovery_action_menu_requires_exact_operation_id() {
        let mut view = operation_log_view(vec![operation_item(&["@  current"], None)]);

        assert_eq!(
            view.execute(
                ViewCommand::OpenActionMenu,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::StatusMessage(
                "operation recovery actions unavailable: selected row has no operation id"
                    .to_owned()
            )
        );
    }

    #[test]
    fn operation_recovery_action_menu_uses_selected_operation_id() {
        let operation_id = "b".repeat(128);
        let mut view =
            operation_log_view(vec![operation_item(&["@  current"], Some(&operation_id))]);

        let effect = view.execute(
            ViewCommand::OpenActionMenu,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );

        let ViewEffect::OpenActionMenu(menu) = effect else {
            panic!("expected operation action menu");
        };
        assert_eq!(menu.items().len(), 2);
        assert_eq!(menu.items()[0].action(), ActionKind::Restore);
        assert_eq!(
            menu.items()[0].label(),
            "restore repository to operation bbbbbbbb"
        );
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::OperationRestoreExactTarget { operation_id: id } if id == &operation_id
        ));
        assert_eq!(menu.items()[1].action(), ActionKind::Revert);
        assert_eq!(menu.items()[1].label(), "revert operation bbbbbbbb");
        assert!(matches!(
            menu.items()[1].follow_up(),
            FollowUp::OperationRevertExactTarget { operation_id: id } if id == &operation_id
        ));
    }

    #[test]
    fn bindings_expose_global_recovery_without_view_execution_target() {
        assert_eq!(
            BINDINGS
                .iter()
                .find(|binding| binding.command() == Command::OperationUndo)
                .map(|binding| binding.command()),
            Some(Command::OperationUndo)
        );
        assert_eq!(
            BINDINGS
                .iter()
                .find(|binding| binding.command() == Command::OperationRedo)
                .map(|binding| binding.command()),
            Some(Command::OperationRedo)
        );
    }
}
