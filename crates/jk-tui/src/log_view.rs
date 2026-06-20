//! Public log view and action contract.
//!
//! This module is the TUI crate's current public surface. Callers provide a [`LogSnapshot`],
//! translate their input source into [`LogAction`] values, and handle [`ActionResult`] requests for
//! effects the view intentionally does not perform itself, such as refreshing from `jj` or quitting
//! the terminal app.

use jk_core::LogSnapshot;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use crate::chrome::{LOG_STATUS, ViewChrome};
use crate::log_state::LogState;
use crate::rendered_log::{ExpandedDetails, RenderedLog, rendered_text};
use crate::selected_row::paint_selected_row;

/// The effect requested after applying an input action to the log view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ActionResult {
    /// Continue running the application.
    Continue,

    /// Refresh the log from the data source.
    Refresh,

    /// Switch to the home view backed by bare `jj`.
    SwitchHome,

    /// Switch to the explicit `jj log` view.
    SwitchLog,

    /// Exit the application.
    Quit,
}

/// Input actions understood by the log view.
///
/// Keyboard bindings live in the binary crate; this enum is the backend-neutral action contract for
/// the TUI state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LogAction {
    /// Move to the previous visible change.
    Previous,

    /// Move to the next visible change.
    Next,

    /// Move one visible page toward older changes.
    PagePrevious,

    /// Move one visible page toward newer changes.
    PageNext,

    /// Move to the first visible change.
    First,

    /// Move to the last visible change.
    Last,

    /// Toggle inline details for the selected change.
    ToggleExpanded,

    /// Collapse inline details for the selected change.
    CollapseExpanded,

    /// Refresh the log.
    Refresh,

    /// Switch to the home view backed by bare `jj`.
    Home,

    /// Switch to the explicit `jj log` view.
    Log,

    /// Quit the TUI.
    Quit,
}

/// Interactive log view for rendered `jj` output.
///
/// The view keeps the rendered log body borderless and opaque. It owns only the interaction state
/// needed to move by semantic log entry, refresh snapshots, and show inline details for the
/// selected change.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LogView {
    state: LogState,
    status_message: Option<String>,
}

impl LogView {
    /// Creates a log view with the initial snapshot loaded.
    #[must_use]
    pub fn new(snapshot: LogSnapshot) -> Self {
        Self {
            state: LogState::new(snapshot),
            status_message: None,
        }
    }

    /// Replaces log output after a successful refresh.
    ///
    /// Selection and scroll position are preserved when the selected change and rendered line still
    /// exist. Any previous status error is cleared.
    pub fn refresh(&mut self, snapshot: LogSnapshot) {
        self.state.refresh(snapshot);
        self.status_message = None;
    }

    /// Shows a refresh or integration error without replacing the current log.
    pub fn show_error(&mut self, error: impl Into<String>) {
        self.status_message = Some(error.into());
    }

    /// Applies a single input action.
    ///
    /// [`ActionResult::Refresh`] asks the caller to load a new [`LogSnapshot`]. The view does not
    /// perform I/O directly.
    #[must_use]
    pub fn apply(&mut self, action: LogAction) -> ActionResult {
        match action {
            LogAction::Previous => {
                self.state.select_previous();
                ActionResult::Continue
            }
            LogAction::Next => {
                self.state.select_next();
                ActionResult::Continue
            }
            LogAction::PagePrevious => {
                self.state.select_page_previous();
                ActionResult::Continue
            }
            LogAction::PageNext => {
                self.state.select_page_next();
                ActionResult::Continue
            }
            LogAction::First => {
                self.state.select_first();
                ActionResult::Continue
            }
            LogAction::Last => {
                self.state.select_last();
                ActionResult::Continue
            }
            LogAction::ToggleExpanded => {
                self.state.toggle_expanded();
                ActionResult::Continue
            }
            LogAction::CollapseExpanded => {
                self.state.collapse_expanded();
                ActionResult::Continue
            }
            LogAction::Refresh => ActionResult::Refresh,
            LogAction::Home => ActionResult::SwitchHome,
            LogAction::Log => ActionResult::SwitchLog,
            LogAction::Quit => ActionResult::Quit,
        }
    }

    /// Renders the log view.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.render_area(frame, area);
    }

    fn render_area(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let areas = ViewChrome::layout(area);
        let height = usize::from(areas.content.height);
        self.state.keep_selected_in_view(height);

        let status = self.status_message.as_deref().unwrap_or(LOG_STATUS);
        let chrome = ViewChrome::new(self.state.title(), status);
        chrome.render(frame, areas);

        let expanded_details = self
            .state
            .expanded_insertion_line()
            .zip(self.state.expanded_details())
            .map(|(line, description)| ExpandedDetails::new(line, description));
        let rendered =
            RenderedLog::new(self.state.rendered()).with_expanded_details(expanded_details);
        let rendered = rendered.render_with_width(usize::from(areas.content.width));
        let text = rendered_text(&rendered);
        let scroll = u16::try_from(self.state.scroll_offset()).unwrap_or(u16::MAX);
        let paragraph = Paragraph::new(text).scroll((scroll, 0));
        frame.render_widget(paragraph, areas.content);

        if let Some(line) = self.state.selected_rendered_line() {
            paint_selected_row(frame, areas.content, line, self.state.scroll_offset());
        }
    }
}

#[cfg(test)]
mod tests {
    use jk_core::LogEntry;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::prelude::Color;

    use super::*;

    #[test]
    fn refresh_and_quit_actions_request_outer_loop_effects() {
        let mut view = LogView::new(snapshot(["aaa"]));

        assert_eq!(view.apply(LogAction::Refresh), ActionResult::Refresh);
        assert_eq!(view.apply(LogAction::Home), ActionResult::SwitchHome);
        assert_eq!(view.apply(LogAction::Log), ActionResult::SwitchLog);
        assert_eq!(view.apply(LogAction::Quit), ActionResult::Quit);
    }

    #[test]
    fn refresh_errors_replace_status_without_replacing_log() {
        let mut view = LogView::new(snapshot(["aaa"]));
        view.show_error("jj failed");
        let backend = TestBackend::new(48, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("aaa summary"));
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("jj failed"));

        view.refresh(snapshot(["bbb"]));
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("bbb summary"));
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("r refresh"));
    }

    #[test]
    fn empty_log_renders_chrome_without_selection() {
        let mut view = LogView::new(LogSnapshot::new("", Vec::new()).with_title("jj log"));
        let backend = TestBackend::new(48, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        assert!(buffer_line(buffer, 0).contains("jk jj log"));
        assert!(buffer_line(buffer, 3).contains("r refresh"));
    }

    #[test]
    fn render_scrolls_to_keep_selected_row_visible() {
        let mut view = LogView::new(snapshot(["aaa", "bbb", "ccc", "ddd", "eee"]));
        let _ = view.apply(LogAction::Next);
        let _ = view.apply(LogAction::Next);
        let _ = view.apply(LogAction::Next);
        let _ = view.apply(LogAction::Next);
        let backend = TestBackend::new(48, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        assert_eq!(view.state.scroll_offset(), 2);
        assert!(buffer_to_string(terminal.backend().buffer()).contains("eee summary"));
    }

    #[test]
    fn renders_jj_output_with_title_and_status_bars_but_no_border() {
        let mut view = LogView::new(
            LogSnapshot::new(
                "@  aaaabbbb summary\n│  body\n~\n",
                vec![LogEntry::new("aaaabbbb", "11112222", "summary")],
            )
            .with_title("jj log -n 3"),
        );
        let backend = TestBackend::new(48, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        let rendered = buffer_to_string(buffer);
        assert!(buffer_line(buffer, 0).contains("jk jj log -n 3"));
        assert!(buffer_line(buffer, 1).contains("@  aaaabbbb summary"));
        assert!(buffer_line(buffer, 5).contains("r refresh"));
        assert!(rendered.contains("jk jj log -n 3"));
        assert!(rendered.contains("@  aaaabbbb summary"));
        assert!(rendered.contains("│  body"));
        assert!(rendered.contains('~'));
        assert!(rendered.contains("r refresh"));
        assert!(!rendered.contains("┌"));
    }

    #[test]
    fn renders_jj_ansi_styles_as_tui_styles() {
        let mut view = LogView::new(LogSnapshot::new(
            "\u{1b}[1m\u{1b}[38;5;2m@\u{1b}[0m  aaaabbbb summary\n",
            vec![LogEntry::new("aaaabbbb", "11112222", "summary")],
        ));
        let backend = TestBackend::new(48, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let cell = &terminal.backend().buffer()[(0, 1)];
        assert_eq!(cell.symbol(), "@");
        assert_eq!(cell.fg, Color::Rgb(15, 20, 31));
        assert_eq!(cell.bg, Color::Rgb(82, 196, 192));
        assert_eq!(
            terminal.backend().buffer()[(23, 1)].bg,
            Color::Rgb(12, 32, 38)
        );
        assert_eq!(
            terminal.backend().buffer()[(35, 1)].bg,
            Color::Rgb(12, 32, 38)
        );
        assert_eq!(terminal.backend().buffer()[(36, 1)].bg, Color::Reset);
        assert!(cell.modifier.contains(ratatui::prelude::Modifier::BOLD));
    }

    #[test]
    fn selected_background_moves_with_selected_change() {
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(1),
            ],
        ));
        let _ = view.apply(LogAction::Next);
        let backend = TestBackend::new(48, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 1)].bg, Color::Reset);
        assert_eq!(buffer[(0, 2)].bg, Color::Rgb(82, 196, 192));
        assert_eq!(buffer[(0, 2)].fg, Color::Rgb(15, 20, 31));
        assert_eq!(buffer[(23, 2)].bg, Color::Rgb(12, 32, 38));
        assert_eq!(buffer[(35, 2)].bg, Color::Rgb(12, 32, 38));
        assert_eq!(buffer[(36, 2)].bg, Color::Reset);
    }

    #[test]
    fn renders_expanded_details_inline_after_selected_row() {
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(1),
            ],
        ));
        let _ = view.apply(LogAction::ToggleExpanded);
        let backend = TestBackend::new(64, 8);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        let first = rendered.find("@  aaa first").unwrap_or_default();
        let body = rendered.find("│  body").unwrap_or_default();
        let second = rendered.find("○  bbb second").unwrap_or_default();
        assert!(first < body);
        assert!(body < second);
    }

    #[test]
    fn wrapped_expanded_details_keep_selected_row_visible() {
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\none two three four five six")
                    .with_details("one two three four five six")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(1),
            ],
        ));
        let _ = view.apply(LogAction::ToggleExpanded);
        let backend = TestBackend::new(24, 8);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        let rendered = buffer_to_string(buffer);
        let selected = rendered.find("@  aaa first").unwrap_or_default();
        let wrapped = rendered.find("five six").unwrap_or_default();
        let second = rendered.find("○  bbb second").unwrap_or_default();
        assert_eq!(view.state.scroll_offset(), 0);
        assert_eq!(buffer[(0, 1)].bg, Color::Rgb(82, 196, 192));
        assert!(selected < wrapped);
        assert!(wrapped < second);
    }

    #[test]
    fn collapse_action_hides_expanded_details() {
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(1),
            ],
        ));
        let _ = view.apply(LogAction::ToggleExpanded);
        let _ = view.apply(LogAction::CollapseExpanded);
        let backend = TestBackend::new(64, 8);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(!rendered.contains("│  body"));
    }

    #[test]
    fn narrow_title_and_status_are_clipped_to_terminal_width() {
        let mut view = LogView::new(
            LogSnapshot::new(
                "@  aaa first\n",
                vec![LogEntry::new("aaa", "111", "first").with_rendered_line(0)],
            )
            .with_title("jj log --revisions very-long-revision-name"),
        );
        view.show_error("refresh failed because the status message is long");
        let backend = TestBackend::new(16, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer_line(buffer, 0).chars().count(), 16);
        assert_eq!(buffer_line(buffer, 3).chars().count(), 16);
        assert!(buffer_line(buffer, 0).contains("jk jj"));
        assert!(buffer_line(buffer, 3).contains("refresh failed"));
    }

    fn snapshot<const N: usize>(change_ids: [&str; N]) -> LogSnapshot {
        let entries = change_ids
            .into_iter()
            .enumerate()
            .map(|(index, change_id)| {
                LogEntry::new(change_id, "commit", format!("{change_id} summary"))
                    .with_rendered_line(index)
            })
            .collect::<Vec<_>>();
        let mut rendered = String::new();
        for entry in &entries {
            rendered.push_str("○  ");
            rendered.push_str(entry.summary());
            rendered.push('\n');
        }
        LogSnapshot::new(rendered, entries)
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
