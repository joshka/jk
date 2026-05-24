use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::documents::{next_matching_line, previous_matching_line};
use crate::files::show::{FileShowView, HORIZONTAL_SCROLL_AMOUNT};
use crate::menus::CopyOption;
use crate::search::{SearchQuery, line_matches};

impl FileShowView {
    /// Execute one view-local command against the file show document.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff => ViewEffect::Ignored,
            ViewCommand::ToggleWrap => {
                self.toggle_wrap(context.size.width);
                ViewEffect::Handled
            }
            ViewCommand::ScrollLeft => {
                self.scroll_left(HORIZONTAL_SCROLL_AMOUNT);
                ViewEffect::Handled
            }
            ViewCommand::ScrollRight => {
                self.scroll_right(context.size.width, HORIZONTAL_SCROLL_AMOUNT);
                ViewEffect::Handled
            }
            ViewCommand::MoveDown => {
                self.scroll_down(context.size.height, 1);
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.scroll_up(context.size.height, 1);
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.scroll_down(context.size.height, context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.scroll_up(context.size.height, context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.scroll_to_top();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.scroll_to_bottom(context.size.height);
                ViewEffect::Handled
            }
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(context.size.height, query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(context.size.height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(context.size.height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::ToggleSelect | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
        }
    }

    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.document
            .lines()
            .iter()
            .filter(|line| line_matches(line, query))
            .count()
    }

    pub fn next_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = next_matching_line(&self.document, self.scroll_offset, query) else {
            return false;
        };
        self.scroll_offset = offset;
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
        true
    }

    pub fn previous_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = previous_matching_line(&self.document, self.scroll_offset, query) else {
            return false;
        };
        self.scroll_offset = offset;
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
        true
    }

    pub fn copy_options(&self) -> Vec<CopyOption> {
        vec![CopyOption::new("file path", self.path.as_str())]
    }
}
