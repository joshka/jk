use super::WorkspacesView;
use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::menus::CopyOption;
use crate::search::{SearchQuery, entry_matches};

impl WorkspacesView {
    /// Applies selection, search, and copy commands to the read-only workspace view.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::MoveDown => {
                self.selection.next(self.context.entries().len());
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
                self.selection.last(self.context.entries().len());
                ViewEffect::Handled
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
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff
            | ViewCommand::ToggleSelect
            | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
        }
    }

    /// Counts rows whose rendered text matches the current search query.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.context
            .entries()
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    /// Advances selection to the next matching row if one exists.
    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.context.entries().len())
            .chain(0..self.selection.index().min(self.context.entries().len()))
            .find(|index| entry_matches(&self.context.entries()[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.context.entries().len());
        true
    }

    /// Moves selection to the previous matching row if one exists.
    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.context.entries().len()).rev())
            .find(|index| entry_matches(&self.context.entries()[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.context.entries().len());
        true
    }

    /// Returns copyable root context and selected-row identifiers for the workspace surface.
    pub fn copy_options(&self) -> Vec<CopyOption> {
        let mut options = Vec::new();
        if let Some(root) = self.context.root() {
            options.push(CopyOption::new("current root", root));
        }

        if let Some(entry) = self.selected_entry() {
            if let Some(name) = entry.name() {
                options.push(CopyOption::new("workspace name", name));
            }
            if let Some(change_id) = entry.target_change_id() {
                options.push(CopyOption::new("change id", change_id));
            }
            if let Some(commit_id) = entry.target_commit_id() {
                options.push(CopyOption::new("commit id", commit_id));
            }
            options.push(CopyOption::new("row text", entry.row_text()));
        }

        options
    }
}
