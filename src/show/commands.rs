use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::documents;
use crate::jj::{JjCommand, ViewSpec};
use crate::menus::CopyOption;
use crate::search::SearchQuery;

use super::{HORIZONTAL_SCROLL_AMOUNT, ShowView};

impl ShowView {
    /// Applies a view command to show-specific navigation, search, and drill-down state.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode => ViewEffect::Ignored,
            ViewCommand::NewTrunk => ViewEffect::Ignored,
            ViewCommand::ToggleWrap => {
                self.toggle_wrap(context.viewport_width);
                ViewEffect::Handled
            }
            ViewCommand::ScrollLeft => {
                self.scroll_left(HORIZONTAL_SCROLL_AMOUNT);
                ViewEffect::Handled
            }
            ViewCommand::ScrollRight => {
                self.scroll_right(context.viewport_width, HORIZONTAL_SCROLL_AMOUNT);
                ViewEffect::Handled
            }
            ViewCommand::MoveDown => {
                self.scroll_down(context.viewport_height, 1);
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.scroll_up(context.viewport_height, 1);
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.scroll_down(context.viewport_height, context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.scroll_up(context.viewport_height, context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.scroll_to_top();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.scroll_to_bottom(context.viewport_height);
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
            ViewCommand::OpenDiff => self
                .spec
                .navigation_revset()
                .map(|revset| ViewEffect::OpenDetail(JjCommand::Diff, revset))
                .unwrap_or(ViewEffect::Ignored),
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
            ViewCommand::ToggleSelect | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
            ViewCommand::OpenItem | ViewCommand::OpenShow => ViewEffect::Ignored,
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

    /// Returns copyable identifiers and the current visible context for the show surface.
    pub fn copy_options(&self) -> Vec<CopyOption> {
        let mut options = Vec::new();
        if let Some(target) = self.spec.target() {
            options.push(CopyOption::new("change id", target));
        }
        if let Some(file) = self.document.current_file_label() {
            options.push(CopyOption::new("file path", file));
        }
        options.push(CopyOption::new(
            "visible context",
            documents::lines_text(self.projection().fixed_lines()),
        ));
        options
    }
}
