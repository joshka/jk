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

use crate::chrome::{StatusTone, ViewChrome, render_help_overlay};
use crate::keymap::{BindingContext, adaptive_hotbar, help_lines, help_title};
use crate::log_state::LogState;
use crate::rendered_log::{ExpandedDetails, RenderedLog, rendered_text};
use crate::selected_row::{interaction_areas, paint_cursor, paint_extent, paint_mark};

/// A visible revision eligible for explicit destination selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevisionChoice {
    /// Full commit identifier; never a potentially ambiguous change prefix.
    pub revision: String,
    /// Human-readable description used for filtering and display.
    pub summary: String,
}

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

    /// Drill into the selected graph elision.
    DrillElision,

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
    status_message: Option<StatusMessage>,
    help_visible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatusMessage {
    text: String,
    tone: StatusTone,
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

    /// Shows that a refresh is running without replacing the current log body.
    pub fn show_loading(&mut self) {
        self.status_message = Some(StatusMessage {
            text: "Refreshing…".to_owned(),
            tone: StatusTone::Loading,
        });
    }

    /// Removes pending-refresh feedback when its work is retired.
    pub fn clear_loading(&mut self) {
        if self
            .status_message
            .as_ref()
            .is_some_and(|status| status.tone == StatusTone::Loading)
        {
            self.status_message = None;
        }
    }

    /// Shows a refresh or integration error without replacing the current log.
    pub fn show_error(&mut self, error: impl Into<String>) {
        self.status_message = Some(StatusMessage {
            text: error.into(),
            tone: StatusTone::Failure,
        });
    }

    /// Shows a non-error status message without replacing the current log.
    pub fn show_status(&mut self, status: impl Into<String>) {
        self.status_message = Some(StatusMessage {
            text: status.into(),
            tone: StatusTone::Neutral,
        });
    }

    /// Returns the selected change identifier for follow-up inspection commands.
    pub fn selected_change_id(&self) -> Option<&str> {
        self.state
            .selected_entry()
            .map(jk_core::LogEntry::change_id)
    }

    /// Returns the selected revision identifier for follow-up commands.
    #[must_use]
    pub fn selected_revision_id(&self) -> Option<&str> {
        self.state.selected_revision_id()
    }

    /// Returns the selected revision's full commit id for exact mutation targeting.
    #[must_use]
    pub fn selected_commit_id(&self) -> Option<&str> {
        self.state.selected_commit_id()
    }

    /// Returns exact visible revisions in display order for destination selection.
    #[must_use]
    pub fn revision_choices(&self) -> Vec<RevisionChoice> {
        self.state
            .entries()
            .iter()
            .map(|entry| RevisionChoice {
                revision: entry.commit_id().to_owned(),
                summary: entry.description().to_owned(),
            })
            .collect()
    }

    /// Returns the full commit id for a visible, non-divergent stable change id.
    #[must_use]
    pub fn commit_id_for_change_id(&self, change_id: &str) -> Option<&str> {
        self.state.commit_id_for_change_id(change_id)
    }
    /// Selects the visible entry with the given change identifier.
    #[must_use]
    pub fn select_change_id(&mut self, change_id: &str) -> bool {
        self.state.select_change_id(change_id)
    }

    /// Selects the first visible log entry.
    pub fn select_first(&mut self) {
        self.state.select_first();
    }

    /// Returns the visible change before the selected graph elision.
    #[must_use]
    pub fn selected_elision_before_change_id(&self) -> Option<&str> {
        self.state.selected_elision_before_change_id()
    }

    /// Selects the first entry rendered after the given visible change.
    #[must_use]
    pub fn select_first_entry_after_change_id(&mut self, change_id: &str) -> bool {
        self.state.select_first_entry_after_change_id(change_id)
    }

    /// Returns the selected change's full description for editing commands.
    pub fn selected_description(&self) -> Option<&str> {
        self.state
            .selected_entry()
            .map(jk_core::LogEntry::description)
    }

    /// Returns whether the log has ordered revision marks.
    #[must_use]
    pub fn has_marks(&self) -> bool {
        self.state.has_marks()
    }

    /// Returns marked change ids in insertion order.
    #[must_use]
    pub fn marked_change_ids(&self) -> &[String] {
        self.state.marked_change_ids()
    }

    /// Returns marked revision identifiers shortened for follow-up commands.
    #[must_use]
    pub fn marked_revision_ids(&self) -> Vec<String> {
        self.state.marked_revision_ids()
    }

    /// Returns the selected change's zero-based mark index, if marked.
    #[must_use]
    pub fn selected_mark_index(&self) -> Option<usize> {
        self.state.selected_mark_index()
    }

    /// Returns the revset that reveals the selected graph elision.
    #[must_use]
    pub fn selected_elision_revset(&self) -> Option<String> {
        self.state.selected_elision_revset()
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
                if self.selected_elision_revset().is_some() {
                    return ActionResult::DrillElision;
                }
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
        self.render_area(frame, area, None, true);
    }

    /// Renders the log view with a caller-owned status line.
    pub fn render_with_status(&mut self, frame: &mut Frame<'_>, status: &str) {
        let area = frame.area();
        self.render_area(frame, area, Some(status), true);
    }

    /// Renders the log view with a centered selector overlay.
    pub fn render_with_selector(&mut self, frame: &mut Frame<'_>, title: &str, lines: &[String]) {
        let area = frame.area();
        self.render_area(frame, area, None, false);
        let areas = ViewChrome::layout(area);
        render_help_overlay(frame, areas.content, title, lines);
    }

    fn render_area(
        &mut self,
        frame: &mut Frame<'_>,
        area: Rect,
        status: Option<&str>,
        focused: bool,
    ) {
        let areas = ViewChrome::layout(area);
        let (gutter, content) = interaction_areas(areas.content);
        let height = usize::from(content.height);
        self.state.keep_selected_in_view(height);

        let status = status.map_or_else(
            || {
                composed_status(
                    self.status_message
                        .as_ref()
                        .map(|message| message.text.as_str()),
                    BindingContext::Log,
                    areas.status_width(),
                )
            },
            ToOwned::to_owned,
        );
        let status = marked_status(&self.state, &status, areas.status_width());
        let status_tone = self
            .status_message
            .as_ref()
            .map_or(StatusTone::Neutral, |message| message.tone);
        let chrome = ViewChrome::new(self.state.title(), &status).with_status_tone(status_tone);
        chrome.render(frame, areas);

        let expanded_details = self
            .state
            .expanded_insertion_line()
            .zip(self.state.expanded_details())
            .map(|(line, description)| ExpandedDetails::new(line, description));
        let rendered_log =
            RenderedLog::new(self.state.rendered()).with_expanded_details(expanded_details);
        let content_width = usize::from(content.width);
        let rendered = rendered_log.render_with_width(content_width);
        let text = rendered_text(&rendered);
        let scroll = u16::try_from(self.state.scroll_offset()).unwrap_or(u16::MAX);
        let paragraph = Paragraph::new(text).scroll((scroll, 0));
        frame.render_widget(paragraph, content);

        paint_mark_overlays(frame, gutter, content_width, &self.state, &rendered_log);
        if let Some(line) = self.state.selected_rendered_line() {
            let focused = focused && !self.help_visible;
            let end = self.state.selected_entry_end_line().unwrap_or(line);
            let end = rendered_log.line_after_insertions(end.saturating_add(1), content_width);
            for continuation in line.saturating_add(1)..end {
                paint_extent(
                    frame,
                    gutter,
                    continuation,
                    self.state.scroll_offset(),
                    focused,
                );
            }
            paint_cursor(frame, gutter, line, self.state.scroll_offset(), focused);
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

fn composed_status(message: Option<&str>, context: BindingContext, width: u16) -> String {
    const SEPARATOR_WIDTH: usize = 2;
    const MIN_HOTBAR_WIDTH: usize = 4;
    const MAX_MESSAGE_WIDTH: usize = 28;

    let terminal_width = width;
    let width = usize::from(terminal_width);
    let hotbar = adaptive_hotbar(context, terminal_width);
    let Some(message) = message else {
        return hotbar;
    };

    let message_width = width
        .saturating_sub(SEPARATOR_WIDTH + MIN_HOTBAR_WIDTH)
        .min(MAX_MESSAGE_WIDTH);
    if message_width == 0 {
        return hotbar;
    }

    let message = truncate_status(message, message_width);
    let hotbar_width = width.saturating_sub(message.chars().count() + SEPARATOR_WIDTH);
    let hotbar_width = u16::try_from(hotbar_width).unwrap_or(terminal_width);
    let hotbar = adaptive_hotbar(context, hotbar_width);
    if hotbar.is_empty() {
        message
    } else {
        format!("{message}  {hotbar}")
    }
}

fn truncate_status(status: &str, width: usize) -> String {
    if status.chars().count() <= width {
        return status.to_owned();
    }
    if width == 1 {
        return "…".to_owned();
    }

    let mut truncated = status.chars().take(width - 1).collect::<String>();
    truncated.push('…');
    truncated
}

fn marked_status(state: &LogState, status: &str, width: u16) -> String {
    let count = state.marked_change_ids().len();
    if count == 0 {
        return status.to_owned();
    }
    let marks = state.selected_mark_index().map_or_else(
        || format!("{count} marked"),
        |index| format!("mark {}/{count}", index + 1),
    );
    let available = usize::from(width).saturating_sub(marks.len() + 2);
    if available == 0 {
        marks
    } else {
        format!("{marks}  {}", truncate_status(status, available))
    }
}

fn paint_mark_overlays(
    frame: &mut Frame<'_>,
    gutter: Rect,
    content_width: usize,
    state: &LogState,
    rendered_log: &RenderedLog<'_>,
) {
    for change_id in state.marked_change_ids() {
        let Some(rendered_line) = state.rendered_line_for_change_id(change_id) else {
            continue;
        };
        let rendered_line = rendered_log.line_after_insertions(rendered_line, content_width);
        paint_mark(frame, gutter, rendered_line, state.scroll_offset());
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
    fn toggle_expanded_on_elision_requests_drill_in() {
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n~  (elided revisions)\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
            ],
        ));

        let _ = view.apply(LogAction::Next);

        assert_eq!(
            view.apply(LogAction::ToggleExpanded),
            ActionResult::DrillElision
        );
        assert_eq!(
            view.selected_elision_revset(),
            Some("(222::111) | 111 | 222".to_owned())
        );
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
        for point in [(0, 3), (47, 3)] {
            assert_eq!(terminal.backend().buffer()[point].fg, Color::Yellow);
            assert_eq!(terminal.backend().buffer()[point].bg, Color::Reset);
        }

        view.refresh(snapshot(["bbb"]));
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("bbb summary"));
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("r refresh"));
    }

    #[test]
    fn loading_is_visible_and_refresh_keeps_the_open_help_overlay() {
        let mut view = LogView::new(snapshot(["aaa"]));
        let _ = view.apply(LogAction::ToggleHelp);
        view.show_loading();
        let backend = TestBackend::new(72, 56);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        assert!(buffer_line(terminal.backend().buffer(), 55).contains("Refreshing…"));
        assert!(buffer_to_string(terminal.backend().buffer()).contains("Log keys"));
        for point in [(0, 55), (71, 55)] {
            assert_eq!(terminal.backend().buffer()[point].fg, Color::Reset);
            assert!(
                terminal.backend().buffer()[point]
                    .modifier
                    .contains(ratatui::prelude::Modifier::BOLD)
            );
            assert_eq!(terminal.backend().buffer()[point].bg, Color::Reset);
        }

        view.refresh(snapshot(["bbb"]));
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("bbb summary"));
        assert!(rendered.contains("Log keys"));
    }

    #[test]
    fn status_messages_share_space_with_default_hotbar_until_refresh() {
        let mut view = LogView::new(snapshot(["aaa"]));
        view.show_status("✓ Created new change");
        let backend = TestBackend::new(80, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        let status = buffer_line(terminal.backend().buffer(), 3);
        assert!(status.contains("✓ Created new change"));
        assert!(status.contains("? help"));
        assert!(status.contains("q quit"));

        view.refresh(snapshot(["bbb"]));
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        let status = buffer_line(terminal.backend().buffer(), 3);
        assert!(!status.contains("Created new change"));
        assert!(status.contains("r refresh"));
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
        for point in [(0, 0), (3, 0), (47, 0), (0, 3), (47, 3)] {
            assert_eq!(buffer[point].fg, Color::Reset);
            assert_eq!(buffer[point].bg, Color::Reset);
        }
    }

    #[test]
    fn help_action_shows_log_specific_keys() {
        let mut view = LogView::new(snapshot(["aaa"]));
        let _ = view.apply(LogAction::ToggleHelp);
        let backend = TestBackend::new(72, 56);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Log keys"));
        assert!(rendered.contains("Contextual help for the current screen."));
        assert!(rendered.contains("Open and inspect:"));
        assert!(rendered.contains("open selected-change diff"));
        assert!(rendered.contains("Change actions:"));
        assert!(rendered.contains("describe selected revision"));
        assert!(rendered.contains("preview jj abandon"));
        assert!(rendered.contains("expand change / drill into ~"));
        assert!(rendered.contains("History and recovery:"));
        assert!(rendered.contains("run jj undo"));
        assert!(rendered.contains("Session:"));
        assert!(rendered.contains("close help"));
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
                "@  aaaabbbb summary\n│  body\n~  (elided revisions)\n",
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

        let buffer = terminal.backend().buffer();
        let cell = &buffer[(3, 1)];
        assert_eq!(cell.symbol(), "@");
        assert_eq!(cell.fg, Color::Indexed(2));
        assert_eq!(cell.bg, Color::Reset);
        assert!(cell.modifier.contains(ratatui::prelude::Modifier::BOLD));
        assert_eq!(buffer[(0, 1)].symbol(), "›");
        for x in 4..48 {
            assert_eq!(buffer[(x, 1)].fg, Color::Reset);
            assert_eq!(buffer[(x, 1)].bg, Color::Reset);
        }
    }

    #[test]
    fn cursor_moves_independently_of_jj_working_copy_node() {
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
        assert_eq!(buffer[(0, 1)].symbol(), " ");
        assert_eq!(buffer[(0, 2)].symbol(), "›");
        assert_eq!(buffer[(3, 1)].symbol(), "@");
        assert_eq!(buffer[(3, 2)].symbol(), "○");
        assert_eq!(buffer[(0, 2)].fg, Color::Reset);
        assert!(
            buffer[(0, 2)]
                .modifier
                .contains(ratatui::prelude::Modifier::BOLD)
        );
        for x in 3..48 {
            assert_eq!(buffer[(x, 2)].bg, Color::Reset);
            assert_eq!(buffer[(x, 2)].fg, Color::Reset);
        }
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
        assert_eq!(buffer[(1, 1)].symbol(), "*");
        assert_eq!(buffer[(1, 2)].symbol(), " ");
        assert_eq!(buffer[(1, 3)].symbol(), "*");
        assert!(buffer_line(buffer, 5).starts_with("mark 2/2"));
    }

    #[test]
    fn selected_marked_row_keeps_separate_cursor_and_mark() {
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
        assert_eq!(buffer[(0, 1)].symbol(), "›");
        assert_eq!(buffer[(1, 1)].symbol(), "*");
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
        assert_eq!(buffer[(1, 5)].symbol(), "*");
        assert_eq!(buffer[(1, 1)].symbol(), " ");
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

        assert_eq!(terminal.backend().buffer()[(1, 1)].symbol(), " ");
        assert!(!buffer_line(terminal.backend().buffer(), 4).contains("marked"));
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
        assert_eq!(buffer[(0, 1)].symbol(), "›");
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
        assert!(buffer_line(buffer, 3).contains("refresh"));
    }

    #[test]
    fn marks_remain_counted_when_their_rows_scroll_out_of_view() {
        let mut view = LogView::new(snapshot(["aaa", "bbb", "ccc", "ddd"]));
        let _ = view.apply(LogAction::ToggleMark);
        let _ = view.apply(LogAction::Last);
        let mut terminal = Terminal::new(TestBackend::new(48, 4)).expect("terminal");
        terminal.draw(|frame| view.render(frame)).expect("draw");
        let buffer = terminal.backend().buffer();
        assert!(!buffer_to_string(buffer).contains("aaa summary"));
        assert!(buffer_line(buffer, 3).starts_with("1 marked"));
        assert_eq!(buffer[(1, 1)].symbol(), " ");
    }

    #[test]
    fn multiline_selection_has_a_gutter_extent_without_recoloring_text() {
        let mut view = LogView::new(LogSnapshot::new(
            "@  aaa first\n│  details\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
            ],
        ));
        let mut terminal = Terminal::new(TestBackend::new(48, 6)).expect("terminal");
        terminal.draw(|frame| view.render(frame)).expect("draw");
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 1)].symbol(), "›");
        assert_eq!(buffer[(0, 2)].symbol(), "·");
        assert_eq!(buffer[(0, 3)].symbol(), " ");
        assert_eq!(buffer[(3, 2)].symbol(), "│");
        assert_eq!(buffer[(3, 2)].fg, Color::Reset);
    }

    #[test]
    fn cursor_and_mark_do_not_clip_content_at_representative_widths() {
        for width in [60, 80, 120, 160] {
            let text = format!(
                "@ {}",
                "x".repeat(usize::from(crate::content_width(width)) - 2)
            );
            let mut view = LogView::new(LogSnapshot::new(
                &text,
                vec![LogEntry::new("aaa", "111", "selected").with_rendered_line(0)],
            ));
            let _ = view.apply(LogAction::ToggleMark);
            let mut terminal = Terminal::new(TestBackend::new(width, 4)).expect("terminal");
            terminal.draw(|frame| view.render(frame)).expect("draw");
            let buffer = terminal.backend().buffer();
            assert_eq!(buffer[(0, 1)].symbol(), "›");
            assert_eq!(buffer[(1, 1)].symbol(), "*");
            assert_eq!(buffer[(width - 1, 1)].symbol(), "x");
            assert_eq!(
                buffer_line(buffer, 1).strip_prefix("›* "),
                Some(text.as_str())
            );
        }
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
