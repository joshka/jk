//! Generic rendered-output inspection view.

use jk_core::InspectionSnapshot;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use crate::chrome::{ViewChrome, render_help_overlay};
use crate::keymap::{BindingContext, help_lines, help_title, hotbar};
use crate::rendered_log::rendered_text;
use crate::rendered_state::RenderedState;

/// The effect requested after applying an input action to a rendered inspection view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderedActionResult {
    /// Continue running the application.
    Continue,

    /// Refresh the rendered output from the data source.
    Refresh,

    /// Return to the previous view.
    ReturnToLog,

    /// Exit the application.
    Quit,
}

/// Input actions understood by a rendered inspection view.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderedAction {
    /// Leave the view unchanged.
    Ignore,

    /// Scroll one line earlier.
    ScrollPrevious,

    /// Scroll one line later.
    ScrollNext,

    /// Scroll one visible page earlier.
    PagePrevious,

    /// Scroll one visible page later.
    PageNext,

    /// Move to the first rendered line.
    First,

    /// Move to the last visible page.
    Last,

    /// Search rendered lines for text.
    Search(String),

    /// Jump to the next search match.
    SearchNext,

    /// Jump to the previous search match.
    SearchPrevious,

    /// Toggle mode-specific help.
    ToggleHelp,

    /// Refresh the rendered output.
    Refresh,

    /// Return to the previous view.
    ReturnToLog,

    /// Quit the TUI.
    Quit,
}

/// Interactive view for read-only `jj` output such as `show` and `status`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderedView {
    state: RenderedState,
    status_message: Option<String>,
    help_visible: bool,
}

impl RenderedView {
    /// Creates a view with the initial snapshot loaded.
    #[must_use]
    pub fn new(snapshot: InspectionSnapshot) -> Self {
        Self {
            state: RenderedState::new(snapshot),
            status_message: None,
            help_visible: false,
        }
    }

    /// Creates a view that starts in an error state for an unloaded target.
    #[must_use]
    pub fn from_error(target: impl Into<String>, title: impl Into<String>, error: String) -> Self {
        let snapshot = InspectionSnapshot::new(target, "").with_title(title);
        let mut view = Self::new(snapshot);
        view.show_error(error);
        view
    }

    /// Replaces rendered output after a successful refresh.
    pub fn refresh(&mut self, snapshot: InspectionSnapshot) {
        self.state.refresh(snapshot);
        self.status_message = None;
    }

    /// Shows an integration error without replacing the current body.
    pub fn show_error(&mut self, error: impl Into<String>) {
        self.status_message = Some(error.into());
    }

    /// Applies a single input action.
    #[must_use]
    pub fn apply(&mut self, action: RenderedAction) -> RenderedActionResult {
        match action {
            RenderedAction::Ignore => {}
            RenderedAction::ScrollPrevious => {
                self.state.scroll_previous_line();
            }
            RenderedAction::ScrollNext => {
                self.state.scroll_next_line();
            }
            RenderedAction::PagePrevious => {
                self.state.select_page_previous();
            }
            RenderedAction::PageNext => {
                self.state.select_page_next();
            }
            RenderedAction::First => {
                self.state.select_first();
            }
            RenderedAction::Last => {
                self.state.select_last();
            }
            RenderedAction::Search(query) => {
                self.state.search(&query, self.status_message.as_deref());
            }
            RenderedAction::SearchNext => {
                self.state.search_next();
            }
            RenderedAction::SearchPrevious => {
                self.state.search_previous();
            }
            RenderedAction::ToggleHelp => {
                self.help_visible = !self.help_visible;
            }
            RenderedAction::Refresh => return RenderedActionResult::Refresh,
            RenderedAction::ReturnToLog => return RenderedActionResult::ReturnToLog,
            RenderedAction::Quit if self.help_visible => {
                self.help_visible = false;
            }
            RenderedAction::Quit => return RenderedActionResult::Quit,
        }
        RenderedActionResult::Continue
    }

    /// Renders the inspection view.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.render_area(frame, area, None);
    }

    /// Renders the inspection view with a temporary status-line override.
    pub fn render_with_status(&mut self, frame: &mut Frame<'_>, status: &str) {
        let area = frame.area();
        self.render_area(frame, area, Some(status));
    }

    fn render_area(&mut self, frame: &mut Frame<'_>, area: Rect, status_override: Option<&str>) {
        let areas = ViewChrome::layout(area);
        self.state
            .set_viewport_height(usize::from(areas.content.height));

        let search_status = self.state.search_status();
        let fallback_status = hotbar(BindingContext::Inspection);
        let status = status_override
            .or(self.status_message.as_deref())
            .or(search_status.as_deref())
            .unwrap_or(&fallback_status);
        let chrome = ViewChrome::new(self.state.title(), status);
        chrome.render(frame, areas);

        let body = self.state.visible_body(self.status_message.as_deref());
        let paragraph = Paragraph::new(rendered_text(&body)).scroll((
            u16::try_from(self.state.scroll_offset()).unwrap_or(u16::MAX),
            0,
        ));
        frame.render_widget(paragraph, areas.content);

        if self.help_visible {
            render_help_overlay(
                frame,
                areas.content,
                help_title(BindingContext::Inspection),
                &help_lines(BindingContext::Inspection),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn refresh_and_return_actions_request_outer_loop_effects() {
        let mut view = RenderedView::new(snapshot("aaa", "first\n"));

        assert_eq!(
            view.apply(RenderedAction::Refresh),
            RenderedActionResult::Refresh
        );
        assert_eq!(
            view.apply(RenderedAction::ReturnToLog),
            RenderedActionResult::ReturnToLog
        );
        assert_eq!(view.apply(RenderedAction::Quit), RenderedActionResult::Quit);
    }

    #[test]
    fn refresh_errors_replace_status_without_replacing_body() {
        let mut view = RenderedView::new(snapshot("aaa", "first line\n"));
        view.show_error("jj failed");
        let backend = TestBackend::new(48, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("first line"));
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("jj failed"));

        view.refresh(snapshot("aaa", "second line\n"));
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("second line"));
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("r refresh"));
    }

    #[test]
    fn initial_error_renders_retryable_message() {
        let mut view = RenderedView::from_error(
            "missing",
            "jj show missing",
            "Revision missing doesn't exist".to_owned(),
        );
        let backend = TestBackend::new(72, 8);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Unable to load inspection for missing."));
        assert!(rendered.contains("Revision missing doesn't exist"));
        assert!(rendered.contains("Press r to retry"));
    }

    #[test]
    fn search_status_replaces_default_status_after_search() {
        let mut view = RenderedView::new(snapshot("aaa", "alpha\nbeta\nalpha\n"));
        let _ = view.apply(RenderedAction::Search("alpha".to_owned()));
        let backend = TestBackend::new(64, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        assert!(buffer_line(terminal.backend().buffer(), 4).contains("/alpha  1/2"));
    }

    #[test]
    fn help_action_shows_inspection_specific_keys() {
        let mut view = RenderedView::new(snapshot("aaa", "alpha\n"));
        let _ = view.apply(RenderedAction::ToggleHelp);
        let backend = TestBackend::new(72, 18);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Inspection keys"));
        assert!(rendered.contains("/, n, N              search, next, previous"));
    }

    fn snapshot(target: &str, rendered: &str) -> InspectionSnapshot {
        InspectionSnapshot::new(target, rendered).with_title(format!("jj show {target}"))
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
