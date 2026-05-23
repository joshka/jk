use super::{OperationLogItem, OperationLogView};
use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::jj::ViewSpec;
use crate::menus::{ActionKind, ActionMenu, ActionMenuItem, CopyOption, FollowUp, SafetyTier};
use crate::search::{SearchQuery, entry_matches};

impl OperationLogView {
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

    /// Counts rows whose rendered text matches the current search query.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    /// Advances selection to the next matching row if one exists.
    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
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
    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
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
    pub fn copy_options(&self) -> Vec<CopyOption> {
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

/// Truncates a full operation id for compact action-menu labels.
fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}
