use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::jj::ViewSpec;
use crate::menus::CopyOption;
use crate::rendered_rows::document_plain_text;
use crate::search::{SearchQuery, line_matches};

use super::StatusView;
use crate::status::actions::StatusFileAction;

impl StatusView {
    /// Execute one view-local command against the status screen.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff => ViewEffect::Ignored,
            ViewCommand::MoveDown => {
                self.move_down(1);
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.move_up(1);
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.move_down(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.move_up(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.selection.first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.selection.last(self.rows.len());
                ViewEffect::Handled
            }
            ViewCommand::OpenFiles => ViewEffect::OpenView(ViewSpec::file_list(None, None)),
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(context.viewport_height, query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(context.viewport_height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(context.viewport_height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::OpenActionMenu => self
                .selected_file_action()
                .map_or_else(ViewEffect::StatusError, |_| ViewEffect::Ignored),
            ViewCommand::ToggleSelect => ViewEffect::Ignored,
            ViewCommand::OpenItem => ViewEffect::Ignored,
        }
    }

    /// Count search matches across rendered status rows.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.rows
            .iter()
            .filter(|row| line_matches(row.line(), query))
            .count()
    }

    /// Move to the next matching status row, wrapping once without reselecting the current row.
    pub fn next_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.rows.len())
            .chain(0..self.selection.index().min(self.rows.len()))
            .find(|index| line_matches(self.rows[*index].line(), query))
        else {
            return false;
        };
        self.selection.set(index, self.rows.len());
        true
    }

    /// Move to the previous matching status row, wrapping once without reselecting the current row.
    pub fn previous_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.rows.len()).rev())
            .find(|index| line_matches(self.rows[*index].line(), query))
        else {
            return false;
        };
        self.selection.set(index, self.rows.len());
        true
    }

    #[cfg(test)]
    pub fn selected_exact_path(&self) -> std::result::Result<&str, String> {
        let Some(row) = self.rows.get(self.selection.index()) else {
            return Err("status file action unavailable: status output is empty".to_owned());
        };
        row.exact_path()
    }

    pub fn selected_file_action(&self) -> std::result::Result<StatusFileAction, String> {
        let Some(row) = self.rows.get(self.selection.index()) else {
            return Err("status file action unavailable: status output is empty".to_owned());
        };
        row.file_action()
    }

    /// Return copy options for the whole rendered status document.
    pub fn copy_options(&self) -> Vec<CopyOption> {
        let lines = self
            .rows
            .iter()
            .map(|row| row.line().clone())
            .collect::<Vec<_>>();
        let text = document_plain_text(&lines);
        if text.is_empty() {
            Vec::new()
        } else {
            vec![CopyOption::new("status text", text)]
        }
    }

    /// Move selection down by a fixed number of rows.
    fn move_down(&mut self, amount: usize) {
        for _ in 0..amount {
            self.selection.next(self.rows.len());
        }
    }

    /// Move selection up by a fixed number of rows.
    fn move_up(&mut self, amount: usize) {
        for _ in 0..amount {
            self.selection.previous();
        }
    }
}
