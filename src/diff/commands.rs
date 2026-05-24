use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::diff::{DiffView, HORIZONTAL_SCROLL_AMOUNT};
use crate::jj::{self, ViewSpec};
use crate::menus::CopyOption;
use crate::search::SearchQuery;

impl DiffView {
    /// Applies a view command to diff-specific navigation, search, and drill-down state.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode => ViewEffect::Ignored,
            ViewCommand::NewTrunk => ViewEffect::Ignored,
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
            ViewCommand::NextFile => {
                self.next_file();
                ViewEffect::Handled
            }
            ViewCommand::PreviousFile => {
                self.previous_file();
                ViewEffect::Handled
            }
            ViewCommand::OpenFiles => {
                let spec = ViewSpec::file_list(
                    self.spec.navigation_revset(),
                    self.document.current_file_label().map(str::to_owned),
                );
                let spec = if self.spec.has_exact_change_target() {
                    spec.with_exact_change_target()
                } else {
                    spec
                };
                ViewEffect::OpenView(spec)
            }
            ViewCommand::OpenShow => self
                .spec
                .navigation_revset()
                .map(|revset| ViewEffect::OpenDetail(jj::Command::Show, revset))
                .unwrap_or(ViewEffect::Ignored),
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
            ViewCommand::OpenItem | ViewCommand::OpenDiff => ViewEffect::Ignored,
        }
    }

    /// Counts rendered matches for the current search query.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.document.search_matches(query)
    }

    /// Advances to the next rendered search match if one exists.
    pub fn next_match(&mut self, viewport_height: u16, query: &SearchQuery) -> bool {
        self.document.next_match(viewport_height, query)
    }

    /// Moves to the previous rendered search match if one exists.
    pub fn previous_match(&mut self, viewport_height: u16, query: &SearchQuery) -> bool {
        self.document.previous_match(viewport_height, query)
    }

    /// Selects the next detected file heading in the rendered document.
    pub fn next_file(&mut self) {
        self.document.next_file();
    }

    /// Selects the previous detected file heading in the rendered document.
    pub fn previous_file(&mut self) {
        self.document.previous_file();
    }

    /// Returns copyable identifiers and the current visible file context for the diff surface.
    pub fn copy_options(&self) -> Vec<CopyOption> {
        let mut options = Vec::new();
        if let Some(target) = self.spec.target() {
            options.push(CopyOption::new("change id", target));
        }
        if let Some(file) = self.document.current_file_label() {
            options.push(CopyOption::new("file path", file));
        }
        options
    }
}
