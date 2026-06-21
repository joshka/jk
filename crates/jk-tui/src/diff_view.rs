//! Public selected-change diff view and action contract.

use jk_core::DiffSnapshot;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use crate::chrome::{DIFF_STATUS, ViewChrome};
use crate::diff_state::DiffState;
use crate::rendered_log::rendered_text;
use crate::selected_row::paint_subtle_selected_row;

/// The effect requested after applying an input action to the diff view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiffActionResult {
    /// Continue running the application.
    Continue,

    /// Refresh the diff from the data source.
    Refresh,

    /// Return to the log view.
    ReturnToLog,

    /// Exit the application.
    Quit,
}

/// Input actions understood by the selected-change diff view.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiffAction {
    /// Leave the diff view unchanged.
    Ignore,

    /// Scroll one line earlier in the diff.
    ScrollPrevious,

    /// Scroll one line later in the diff.
    ScrollNext,

    /// Scroll one visible page earlier in the diff.
    PagePrevious,

    /// Scroll one visible page later in the diff.
    PageNext,

    /// Move to the first visible diff line.
    First,

    /// Move to the last visible diff page.
    Last,

    /// Jump to the previous file section.
    PreviousFile,

    /// Jump to the next file section.
    NextFile,

    /// Jump to the previous hunk.
    PreviousHunk,

    /// Jump to the next hunk.
    NextHunk,

    /// Fold the selected file section.
    FoldFile,

    /// Unfold the selected file section.
    UnfoldFile,

    /// Fold every file section.
    FoldAll,

    /// Unfold every file section.
    UnfoldAll,

    /// Scroll wide diff lines toward the start.
    ScrollLeft,

    /// Scroll wide diff lines toward the end.
    ScrollRight,

    /// Search visible diff lines for text.
    Search(String),

    /// Jump to the next search match.
    SearchNext,

    /// Jump to the previous search match.
    SearchPrevious,

    /// Refresh the diff.
    Refresh,

    /// Return to the log view.
    ReturnToLog,

    /// Quit the TUI.
    Quit,
}

/// Interactive diff view for rendered `jj diff` output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiffView {
    state: DiffState,
    status_message: Option<String>,
}

impl DiffView {
    /// Creates a diff view with the initial snapshot loaded.
    #[must_use]
    pub fn new(snapshot: DiffSnapshot) -> Self {
        Self {
            state: DiffState::new(snapshot),
            status_message: None,
        }
    }

    /// Returns the target change identifier for refresh requests.
    #[must_use]
    pub fn change_id(&self) -> &str {
        self.state.change_id()
    }

    /// Replaces diff output after a successful refresh.
    pub fn refresh(&mut self, snapshot: DiffSnapshot) {
        self.state.refresh(snapshot);
        self.status_message = None;
    }

    /// Shows a refresh or integration error without replacing the current diff.
    pub fn show_error(&mut self, error: impl Into<String>) {
        self.status_message = Some(error.into());
    }

    /// Applies a single input action.
    #[must_use]
    pub fn apply(&mut self, action: DiffAction) -> DiffActionResult {
        match action {
            DiffAction::Ignore => DiffActionResult::Continue,
            DiffAction::ScrollPrevious => {
                self.state.scroll_previous_line();
                DiffActionResult::Continue
            }
            DiffAction::ScrollNext => {
                self.state.scroll_next_line();
                DiffActionResult::Continue
            }
            DiffAction::PagePrevious => {
                self.state.select_page_previous();
                DiffActionResult::Continue
            }
            DiffAction::PageNext => {
                self.state.select_page_next();
                DiffActionResult::Continue
            }
            DiffAction::First => {
                self.state.select_first();
                DiffActionResult::Continue
            }
            DiffAction::Last => {
                self.state.select_last();
                DiffActionResult::Continue
            }
            DiffAction::PreviousFile => {
                self.state.select_previous_file();
                DiffActionResult::Continue
            }
            DiffAction::NextFile => {
                self.state.select_next_file();
                DiffActionResult::Continue
            }
            DiffAction::PreviousHunk => {
                self.state.select_previous_hunk();
                DiffActionResult::Continue
            }
            DiffAction::NextHunk => {
                self.state.select_next_hunk();
                DiffActionResult::Continue
            }
            DiffAction::FoldFile => {
                self.state.fold_selected_file();
                DiffActionResult::Continue
            }
            DiffAction::UnfoldFile => {
                self.state.unfold_selected_file();
                DiffActionResult::Continue
            }
            DiffAction::FoldAll => {
                self.state.fold_all_files();
                DiffActionResult::Continue
            }
            DiffAction::UnfoldAll => {
                self.state.unfold_all_files();
                DiffActionResult::Continue
            }
            DiffAction::ScrollLeft => {
                self.state.scroll_left();
                DiffActionResult::Continue
            }
            DiffAction::ScrollRight => {
                self.state.scroll_right();
                DiffActionResult::Continue
            }
            DiffAction::Search(query) => {
                self.state.search(&query);
                DiffActionResult::Continue
            }
            DiffAction::SearchNext => {
                self.state.search_next();
                DiffActionResult::Continue
            }
            DiffAction::SearchPrevious => {
                self.state.search_previous();
                DiffActionResult::Continue
            }
            DiffAction::Refresh => DiffActionResult::Refresh,
            DiffAction::ReturnToLog => DiffActionResult::ReturnToLog,
            DiffAction::Quit => DiffActionResult::Quit,
        }
    }

    /// Renders the diff view.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.render_area(frame, area, None);
    }

    /// Renders the diff view with a temporary status-line override.
    pub fn render_with_status(&mut self, frame: &mut Frame<'_>, status: &str) {
        let area = frame.area();
        self.render_area(frame, area, Some(status));
    }

    fn render_area(&mut self, frame: &mut Frame<'_>, area: Rect, status_override: Option<&str>) {
        let areas = ViewChrome::layout(area);
        let height = usize::from(areas.content.height);
        self.state.keep_selected_in_view(height);
        self.state
            .set_viewport_width(usize::from(areas.content.width));
        let sticky_header = self.state.sticky_header();
        let body_area = if sticky_header.is_some() {
            area_below_sticky_header(areas.content)
        } else {
            areas.content
        };
        self.state
            .keep_selected_in_view(usize::from(body_area.height));
        self.state.set_viewport_width(usize::from(body_area.width));

        let search_status = self.state.search_status();
        let horizontal_status = self.state.horizontal_status();
        let status = status_override
            .or(self.status_message.as_deref())
            .or(search_status.as_deref())
            .or(horizontal_status.as_deref())
            .unwrap_or(DIFF_STATUS);
        let chrome = ViewChrome::new(self.state.title(), status);
        chrome.render(frame, areas);

        let rendered = self.state.visible_rendered();
        let text = rendered_text(&rendered);
        let scroll = u16::try_from(self.state.scroll_offset()).unwrap_or(u16::MAX);
        let horizontal_scroll = u16::try_from(self.state.horizontal_offset()).unwrap_or(u16::MAX);
        let paragraph = Paragraph::new(text).scroll((scroll, horizontal_scroll));
        frame.render_widget(paragraph, body_area);

        if let Some(header) = sticky_header {
            let sticky_area = Rect {
                height: 1,
                ..areas.content
            };
            let paragraph = Paragraph::new(rendered_text(&header)).scroll((0, horizontal_scroll));
            frame.render_widget(paragraph, sticky_area);
            paint_subtle_selected_row(frame, sticky_area, 0, 0);
        }

        if let Some(line) = self.state.selected_visible_line() {
            paint_subtle_selected_row(frame, body_area, line, self.state.scroll_offset());
        }
    }
}

/// Returns the portion of a content area left after a sticky file header row.
const fn area_below_sticky_header(area: Rect) -> Rect {
    if area.height <= 1 {
        return area;
    }

    Rect {
        y: area.y + 1,
        height: area.height - 1,
        ..area
    }
}

#[cfg(test)]
mod tests {
    use jk_core::DiffSnapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn refresh_and_return_actions_request_outer_loop_effects() {
        let mut view = DiffView::new(snapshot("aaa", "Modified regular file src/a.rs:\n a\n"));

        assert_eq!(view.apply(DiffAction::Refresh), DiffActionResult::Refresh);
        assert_eq!(
            view.apply(DiffAction::ReturnToLog),
            DiffActionResult::ReturnToLog
        );
        assert_eq!(view.apply(DiffAction::Quit), DiffActionResult::Quit);
    }

    #[test]
    fn refresh_errors_replace_status_without_replacing_diff() {
        let mut view = DiffView::new(snapshot("aaa", "Modified regular file src/a.rs:\n a\n"));
        view.show_error("jj failed");
        let backend = TestBackend::new(56, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Modified regular file src/a.rs:"));
        assert!(buffer_line(terminal.backend().buffer(), 4).contains("jj failed"));

        view.refresh(snapshot("aaa", "Modified regular file src/b.rs:\n b\n"));
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Modified regular file src/b.rs:"));
        assert!(buffer_line(terminal.backend().buffer(), 4).contains("r refresh"));
    }

    #[test]
    fn toggles_file_section_collapse() {
        let mut view = DiffView::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n removed body text\n new\nModified regular file src/b.rs:\n b\n",
        ));
        let _ = view.apply(DiffAction::FoldFile);
        let backend = TestBackend::new(64, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Modified regular file src/a.rs: | folded"));
        assert!(!rendered.contains("removed body text"));
    }

    #[test]
    fn fold_all_and_unfold_all_actions_update_rendered_sections() {
        let mut view = DiffView::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a\nModified regular file src/b.rs:\n b\n",
        ));

        let _ = view.apply(DiffAction::FoldAll);
        assert!(view.state.visible_rendered().contains(" | folded"));

        let _ = view.apply(DiffAction::UnfoldAll);
        assert!(!view.state.visible_rendered().contains(" | folded"));
    }

    #[test]
    fn search_status_replaces_default_status_after_search() {
        let mut view = DiffView::new(snapshot("aaa", "Modified regular file src/a.rs:\n alpha\n"));
        let _ = view.apply(DiffAction::Search("alpha".to_owned()));
        let backend = TestBackend::new(64, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        assert!(buffer_line(terminal.backend().buffer(), 4).contains("/alpha  1/1"));
    }

    #[test]
    fn horizontal_scroll_shifts_body_and_reports_column() {
        let mut view = DiffView::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n 1234567890123456789012345678901234567890\n",
        ));
        let backend = TestBackend::new(32, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        let _ = view.apply(DiffAction::ScrollRight);
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        assert_eq!(view.state.horizontal_offset(), 8);
        assert!(buffer_line(terminal.backend().buffer(), 4).contains("col 9"));
    }

    #[test]
    fn left_action_unfolds_selected_file() {
        let mut view = DiffView::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a\nModified regular file src/b.rs:\n b\n",
        ));

        let _ = view.apply(DiffAction::FoldAll);
        let _ = view.apply(DiffAction::UnfoldFile);

        let rendered = view.state.visible_rendered();
        assert!(rendered.contains("\n a\n"));
        assert!(!rendered.contains("\n b\n"));
    }

    #[test]
    fn scrolls_line_by_line_without_jumping_files() {
        let mut view = DiffView::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a1\n a2\nModified regular file src/b.rs:\n b\n",
        ));
        view.state.keep_selected_in_view(2);

        let _ = view.apply(DiffAction::ScrollNext);
        let _ = view.apply(DiffAction::ScrollNext);

        assert_eq!(view.state.scroll_offset(), 2);
        assert_eq!(view.state.selected_visible_line(), Some(0));
    }

    #[test]
    fn render_pins_current_file_header_after_it_scrolls_offscreen() {
        let mut view = DiffView::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a1\n a2\n a3\nModified regular file src/b.rs:\n b\n",
        ));
        let backend = TestBackend::new(64, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        let _ = view.apply(DiffAction::ScrollNext);

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        assert_eq!(view.state.scroll_offset(), 1);
        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(
            buffer_line(terminal.backend().buffer(), 1).contains("Modified regular file src/a.rs:")
        );
        assert!(rendered.contains("a1"));
    }

    fn snapshot(change_id: &str, rendered: &str) -> DiffSnapshot {
        DiffSnapshot::new(change_id, rendered).with_title(format!("jj diff -r {change_id}"))
    }

    fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
        let area = buffer.area;
        let mut text = String::new();

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }

        text
    }

    fn buffer_line(buffer: &ratatui::buffer::Buffer, y: u16) -> String {
        let area = buffer.area;
        let mut text = String::new();

        for x in area.left()..area.right() {
            text.push_str(buffer[(x, y)].symbol());
        }

        text
    }
}
