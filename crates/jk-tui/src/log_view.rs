//! Public log view and action contract.
//!
//! This module is the TUI crate's current public surface. Callers provide a [`LogSnapshot`],
//! translate their input source into [`LogAction`] values, and handle [`ActionResult`] requests for
//! effects the view intentionally does not perform itself, such as refreshing from `jj` or quitting
//! the terminal app.

use jk_core::LogSnapshot;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;

use crate::chrome::{ViewChrome, render_help_overlay};
use crate::keymap::{BindingContext, adaptive_hotbar, help_lines, help_title};
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

    /// Open the selected change's diff.
    OpenDiff,

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

    /// Scroll one rendered line toward newer changes.
    ScrollPreviousLine,

    /// Scroll one rendered line toward older changes.
    ScrollNextLine,

    /// Move one visible page toward older changes.
    PagePrevious,

    /// Move one visible page toward newer changes.
    PageNext,

    /// Move to the first visible change.
    First,

    /// Move to the last visible change.
    Last,

    /// Move to the previous file section in views that support file sections.
    PreviousFile,

    /// Move to the next file section in views that support file sections.
    NextFile,

    /// Move to the previous hunk in views that support diff hunks.
    PreviousHunk,

    /// Move to the next hunk in views that support diff hunks.
    NextHunk,

    /// Fold the selected hunk in views that support diff hunks.
    FoldHunk,

    /// Unfold the selected hunk in views that support diff hunks.
    UnfoldHunk,

    /// Scroll horizontally toward the start in views that support wide content.
    HorizontalPrevious,

    /// Scroll horizontally toward the end in views that support wide content.
    HorizontalNext,

    /// Fold all collapsible sections in views that support sections.
    FoldAll,

    /// Unfold all collapsible sections in views that support sections.
    UnfoldAll,

    /// Toggle inline details for the selected change.
    ToggleExpanded,

    /// Collapse inline details for the selected change.
    CollapseExpanded,

    /// Toggle the selected change in ordered revision marks.
    ToggleMark,

    /// Clear ordered revision marks.
    ClearMarks,

    /// Refresh the log.
    Refresh,

    /// Switch to the home view backed by bare `jj`.
    Home,

    /// Switch to the explicit `jj log` view.
    Log,

    /// Open the selected change's diff.
    OpenDiff,

    /// Toggle mode-specific help.
    ToggleHelp,

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
    help_visible: bool,
}

impl LogView {
    /// Creates a log view with the initial snapshot loaded.
    #[must_use]
    pub fn new(snapshot: LogSnapshot) -> Self {
        Self {
            state: LogState::new(snapshot),
            status_message: None,
            help_visible: false,
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

    /// Shows a non-error status message without replacing the current log.
    pub fn show_status(&mut self, status: impl Into<String>) {
        self.status_message = Some(status.into());
    }

    /// Returns the selected change identifier for follow-up inspection commands.
    pub fn selected_change_id(&self) -> Option<&str> {
        self.state
            .selected_entry()
            .map(jk_core::LogEntry::change_id)
    }

    /// Returns whether the log has ordered revision marks.
    pub fn has_marks(&self) -> bool {
        self.state.has_marks()
    }

    /// Returns marked change ids in insertion order.
    pub fn marked_change_ids(&self) -> &[String] {
        self.state.marked_change_ids()
    }

    /// Returns the selected change's zero-based mark index, if marked.
    pub fn selected_mark_index(&self) -> Option<usize> {
        self.state.selected_mark_index()
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
            LogAction::ScrollPreviousLine => {
                self.state.scroll_previous_line();
                ActionResult::Continue
            }
            LogAction::ScrollNextLine => {
                self.state.scroll_next_line();
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
            LogAction::PreviousFile
            | LogAction::NextFile
            | LogAction::PreviousHunk
            | LogAction::NextHunk
            | LogAction::FoldHunk
            | LogAction::UnfoldHunk
            | LogAction::HorizontalPrevious
            | LogAction::HorizontalNext
            | LogAction::FoldAll
            | LogAction::UnfoldAll => ActionResult::Continue,
            LogAction::ToggleExpanded => {
                self.state.toggle_expanded();
                ActionResult::Continue
            }
            LogAction::CollapseExpanded => {
                self.state.collapse_expanded();
                ActionResult::Continue
            }
            LogAction::ToggleMark => {
                self.state.toggle_selected_mark();
                ActionResult::Continue
            }
            LogAction::ClearMarks => {
                self.state.clear_marks();
                ActionResult::Continue
            }
            LogAction::Refresh => ActionResult::Refresh,
            LogAction::Home => ActionResult::SwitchHome,
            LogAction::Log => ActionResult::SwitchLog,
            LogAction::OpenDiff => {
                if self.selected_change_id().is_some() {
                    ActionResult::OpenDiff
                } else {
                    ActionResult::Continue
                }
            }
            LogAction::ToggleHelp => {
                self.help_visible = !self.help_visible;
                ActionResult::Continue
            }
            LogAction::Quit if self.help_visible => {
                self.help_visible = false;
                ActionResult::Continue
            }
            LogAction::Quit => ActionResult::Quit,
        }
    }

    /// Renders the log view.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.render_area(frame, area, None);
    }

    /// Renders the log view with a caller-owned status line.
    pub fn render_with_status(&mut self, frame: &mut Frame<'_>, status: &str) {
        let area = frame.area();
        self.render_area(frame, area, Some(status));
    }

    /// Renders the log view with a centered selector overlay.
    pub fn render_with_selector(&mut self, frame: &mut Frame<'_>, title: &str, lines: &[String]) {
        let area = frame.area();
        self.render_area(frame, area, None);
        let areas = ViewChrome::layout(area);
        render_help_overlay(frame, areas.content, title, lines);
    }

    fn render_area(&mut self, frame: &mut Frame<'_>, area: Rect, status: Option<&str>) {
        let areas = ViewChrome::layout(area);
        let height = usize::from(areas.content.height);
        self.state.keep_selected_in_view(height);

        let status = status
            .map(ToOwned::to_owned)
            .or_else(|| self.status_message.clone())
            .unwrap_or_else(|| adaptive_hotbar(BindingContext::Log, areas.status_width()));
        let chrome = ViewChrome::new(self.state.title(), &status);
        chrome.render(frame, areas);

        let expanded_details = self
            .state
            .expanded_insertion_line()
            .zip(self.state.expanded_details())
            .map(|(line, description)| ExpandedDetails::new(line, description));
        let rendered_log =
            RenderedLog::new(self.state.rendered()).with_expanded_details(expanded_details);
        let content_width = usize::from(areas.content.width);
        let rendered = rendered_log.render_with_width(content_width);
        let text = rendered_text(&rendered);
        let scroll = u16::try_from(self.state.scroll_offset()).unwrap_or(u16::MAX);
        let paragraph = Paragraph::new(text).scroll((scroll, 0));
        frame.render_widget(paragraph, areas.content);

        paint_mark_overlays(frame, areas.content, &self.state, &rendered_log);
        if let Some(line) = self.state.selected_rendered_line() {
            paint_selected_row(frame, areas.content, line, self.state.scroll_offset());
        }

        if self.help_visible {
            render_help_overlay(
                frame,
                areas.content,
                help_title(BindingContext::Log),
                &help_lines(BindingContext::Log),
            );
        }
    }
}

fn paint_mark_overlays(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &LogState,
    rendered_log: &RenderedLog<'_>,
) {
    if area.is_empty() || area.width < 3 {
        return;
    }

    let content_width = usize::from(area.width);
    for change_id in state.marked_change_ids() {
        let Some(mark_index) = state.mark_index_for_change_id(change_id) else {
            continue;
        };
        let Some(rendered_line) = state.rendered_line_for_change_id(change_id) else {
            continue;
        };
        let rendered_line = rendered_log.line_after_insertions(rendered_line, content_width);
        let Some(visible_line) = rendered_line.checked_sub(state.scroll_offset()) else {
            continue;
        };
        let Ok(visible_line) = u16::try_from(visible_line) else {
            continue;
        };
        if visible_line >= area.height {
            continue;
        }

        let label = format!("[{}]", mark_index + 1);
        let label_width = u16::try_from(label.chars().count()).unwrap_or(u16::MAX);
        if label_width > area.width {
            continue;
        }

        let y = area.y + visible_line;
        let x = area.right() - label_width;
        frame.buffer_mut().set_string(
            x,
            y,
            label,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
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
        assert_eq!(view.apply(LogAction::OpenDiff), ActionResult::OpenDiff);
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
    fn status_messages_replace_default_hotbar_until_refresh() {
        let mut view = LogView::new(snapshot(["aaa"]));
        view.show_status("u undo  U redo  o operation  C history");
        let backend = TestBackend::new(56, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("u undo"));
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("C history"));

        view.refresh(snapshot(["bbb"]));
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        assert!(!buffer_line(terminal.backend().buffer(), 3).contains("u undo"));
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
    fn help_action_shows_log_specific_keys() {
        let mut view = LogView::new(snapshot(["aaa"]));
        let _ = view.apply(LogAction::ToggleHelp);
        let backend = TestBackend::new(72, 28);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Log keys"));
        assert!(rendered.contains("d                    open selected-change diff"));
        assert!(rendered.contains("m                    describe selected revision"));
        assert!(rendered.contains("s                    open repository status"));
        assert!(rendered.contains("u                    preview jj undo"));
        assert!(rendered.contains("U                    preview jj redo"));
        assert!(rendered.contains("?, q, Esc            close help"));
    }

    #[test]
    fn quit_closes_log_help_before_quitting() {
        let mut view = LogView::new(snapshot(["aaa"]));
        let _ = view.apply(LogAction::ToggleHelp);

        assert_eq!(view.apply(LogAction::Quit), ActionResult::Continue);
        assert_eq!(view.apply(LogAction::Quit), ActionResult::Quit);
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
    fn marked_rows_render_ordered_affordances() {
        let mut view = LogView::new(snapshot(["aaa", "bbb", "ccc"]));
        let _ = view.apply(LogAction::ToggleMark);
        let _ = view.apply(LogAction::Next);
        let _ = view.apply(LogAction::Next);
        let _ = view.apply(LogAction::ToggleMark);
        let backend = TestBackend::new(48, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        assert!(buffer_line(buffer, 1).ends_with("[1]"));
        assert!(!buffer_line(buffer, 2).contains("["));
        assert!(buffer_line(buffer, 3).ends_with("[2]"));
    }

    #[test]
    fn selected_marked_row_keeps_selected_background() {
        let mut view = LogView::new(snapshot(["aaa", "bbb"]));
        let _ = view.apply(LogAction::ToggleMark);
        let backend = TestBackend::new(48, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 1)].bg, Color::Rgb(82, 196, 192));
        assert!(buffer_line(buffer, 1).contains("[1]"));
    }

    #[test]
    fn marked_rows_after_expanded_details_render_on_shifted_line() {
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(1),
            ],
        ));
        let _ = view.apply(LogAction::Next);
        let _ = view.apply(LogAction::ToggleMark);
        let _ = view.apply(LogAction::Previous);
        let _ = view.apply(LogAction::ToggleExpanded);
        let backend = TestBackend::new(64, 8);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let buffer = terminal.backend().buffer();
        assert!(buffer_line(buffer, 5).contains("○  bbb second"));
        assert!(buffer_line(buffer, 5).ends_with("[1]"));
        assert!(!buffer_line(buffer, 1).contains("[1]"));
    }

    #[test]
    fn clear_marks_removes_visible_affordances() {
        let mut view = LogView::new(snapshot(["aaa", "bbb"]));
        let _ = view.apply(LogAction::ToggleMark);
        let _ = view.apply(LogAction::ClearMarks);
        let backend = TestBackend::new(48, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        assert!(!buffer_to_string(terminal.backend().buffer()).contains("[1]"));
    }

    #[test]
    fn narrow_mark_overlay_rendering_does_not_panic() {
        let mut view = LogView::new(snapshot(["aaa"]));
        let _ = view.apply(LogAction::ToggleMark);
        let backend = TestBackend::new(2, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));

        assert!(draw_result.is_ok());
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
