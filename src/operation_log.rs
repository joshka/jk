//! `jj operation log` view state, rendering, and item-based navigation.
//!
//! The first pass keeps the operation log close to rendered `jj` output while
//! carrying exact operation ids separately for copy, search, and refresh
//! stability. Recovery actions stay out of scope until previews and
//! confirmations exist.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{OperationLogItem, ViewSpec, load_operation_log_entries};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;

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
            ViewCommand::OpenShow => {
                ViewEffect::StatusMessage("operation show not implemented yet".to_owned())
            }
            ViewCommand::OpenDiff => {
                ViewEffect::StatusMessage("operation diff not implemented yet".to_owned())
            }
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
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::PageDown
            | ViewCommand::PageUp
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile => ViewEffect::Ignored,
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

    List::new(items).highlight_style(
        Style::default()
            .bg(Color::Rgb(52, 54, 62))
            .add_modifier(Modifier::BOLD),
    )
}

fn restore_selection(
    selection: &mut Selection,
    entries: &[OperationLogItem],
    previous_index: usize,
    previous_operation_id: Option<String>,
) {
    if let Some(operation_id) = previous_operation_id {
        if let Some(index) = entries
            .iter()
            .position(|entry| entry.operation_id() == Some(operation_id.as_str()))
        {
            selection.set(index, entries.len());
            return;
        }
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
mod tests {
    use ratatui::text::Line;

    use super::*;
    use crate::jj::JjCommand;

    fn operation_item(text: &[&str], operation_id: Option<&str>) -> OperationLogItem {
        OperationLogItem::new(
            text.iter()
                .map(|line| Line::from((*line).to_owned()))
                .collect::<Vec<_>>(),
            operation_id.map(str::to_owned),
        )
    }

    fn operation_log_view(entries: Vec<OperationLogItem>) -> OperationLogView {
        OperationLogView {
            spec: ViewSpec::new(JjCommand::OperationLog, Vec::new()),
            entries,
            selection: Selection::default(),
        }
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
                search: None,
            },
        );

        assert_eq!(view.selection.index(), 1);
        view.execute(
            ViewCommand::MoveUp,
            CommandContext {
                viewport_height: 10,
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
    fn placeholders_are_non_mutating_status_messages() {
        let mut view = operation_log_view(vec![operation_item(&["@  current"], Some("first"))]);

        assert_eq!(
            view.execute(
                ViewCommand::OpenShow,
                CommandContext {
                    viewport_height: 10,
                    search: None,
                },
            ),
            ViewEffect::StatusMessage("operation show not implemented yet".to_owned())
        );
        assert_eq!(
            view.execute(
                ViewCommand::OpenDiff,
                CommandContext {
                    viewport_height: 10,
                    search: None,
                },
            ),
            ViewEffect::StatusMessage("operation diff not implemented yet".to_owned())
        );
    }
}
