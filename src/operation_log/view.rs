//! Operation-log view state, rendering, and item-based navigation.
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

use super::{OperationLogItem, load_operation_log_entries};
use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::ViewSpec;
use crate::menus::{ActionKind, ActionMenu, ActionMenuItem, FollowUp, SafetyTier};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::{Selection, restore_by_key_or_index};
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
    /// View identity used to reload the operation log.
    pub(super) spec: ViewSpec,
    /// Rendered operation-log items paired with exact operation ids when metadata matches.
    pub(super) entries: Vec<OperationLogItem>,
    /// Current selected row within the operation-log item list.
    pub(super) selection: Selection,
}

impl OperationLogView {
    /// Loads rendered operation-log rows and initializes selection at the first row.
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

    /// Renders the operation-log list with the active selection and search highlights.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(entry_list(&self.entries, search), area, &mut state);
    }

    /// Returns the key bindings owned by the operation-log view.
    pub fn bindings(&self) -> &'static [Binding] {
        super::BINDINGS
    }

    /// Applies selection, navigation, search, copy, and recovery-menu commands.
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

    /// Reloads rendered rows while preserving the selected exact operation id when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_operation_log_entries)
    }

    /// Clamps the current selection to the available rows.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    /// Returns the view spec that identifies this operation-log surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the number of selectable operation-log rows.
    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the total rendered line count across all operation-log rows.
    pub fn line_count(&self) -> usize {
        self.entries.iter().map(OperationLogItem::line_count).sum()
    }

    /// Counts rows whose rendered text matches the current search query.
    pub(super) fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    /// Advances selection to the next matching row if one exists.
    pub(super) fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.entries.len())
            .chain(0..self.selection.index().min(self.entries.len()))
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    /// Moves selection to the previous matching row if one exists.
    pub(super) fn previous_match(&mut self, query: &SearchQuery) -> bool {
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

    /// Returns copyable identifiers and the selected rendered row text.
    pub(super) fn copy_options(&self) -> Vec<CopyOption> {
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

    /// Returns the exact operation id for the selected row, if metadata is present.
    fn selected_operation_id(&self) -> Option<String> {
        self.entries
            .get(self.selection.index())
            .and_then(OperationLogItem::operation_id)
            .map(str::to_owned)
    }

    /// Reloads rows and restores selection by exact operation id before falling back to index.
    pub(super) fn refresh_with_loader(
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
        restore_by_key_or_index(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_operation_id.as_deref(),
            OperationLogItem::operation_id,
        );
        Ok(())
    }
}

/// Builds the recovery-action menu for one exact selected operation id.
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

/// Projects rendered operation-log items into the selectable list widget.
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

/// Truncates a full operation id for compact action-menu labels.
fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}
