//! Provider-neutral command-history list view.
//!
//! Callers map retained command records into [`CommandHistorySnapshot`]. The view only owns
//! selection, scrolling, and rendering for a read-only first history surface.

use jk_core::{CommandRecord, SourceAction, SourceView, StreamSummary};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::Paragraph;

use crate::ansi_text::strip_ansi;
use crate::chrome::{ViewChrome, render_help_overlay};
use crate::keymap::{BindingContext, adaptive_hotbar, help_lines, help_title};
use crate::selected_row::paint_subtle_selected_row;

const DEFAULT_TITLE: &str = "Command History";
const DEFAULT_ROW_LIMIT: usize = 128;
const SUMMARY_LIMIT: usize = 96;

/// A provider-neutral snapshot of retained command history.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandHistorySnapshot {
    title: String,
    rows: Vec<CommandHistoryRow>,
}

impl CommandHistorySnapshot {
    /// Creates a command-history snapshot from display rows.
    #[must_use]
    pub fn new(rows: Vec<CommandHistoryRow>) -> Self {
        Self {
            title: DEFAULT_TITLE.to_owned(),
            rows,
        }
    }

    /// Creates a newest-first snapshot from retained command records.
    #[must_use]
    pub fn from_records<'a>(records: impl DoubleEndedIterator<Item = &'a CommandRecord>) -> Self {
        Self::new(
            records
                .rev()
                .take(DEFAULT_ROW_LIMIT)
                .map(CommandHistoryRow::from_record)
                .collect(),
        )
    }

    /// Sets the title shown in the view chrome.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Returns the title shown in the view chrome.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns rows in display order.
    #[must_use]
    pub fn rows(&self) -> &[CommandHistoryRow] {
        &self.rows
    }
}

/// One display row in the command-history list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandHistoryRow {
    /// Stable record id from the current in-memory history.
    pub id: u64,
    /// Compact status marker.
    pub status: String,
    /// Source view and action that triggered the command.
    pub source: String,
    /// Command title or preview.
    pub command: String,
    /// Compact output or failure summary.
    pub summary: String,
}

impl CommandHistoryRow {
    /// Creates a display row.
    #[must_use]
    pub fn new(
        id: u64,
        status: impl Into<String>,
        source: impl Into<String>,
        command: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id,
            status: status.into(),
            source: source.into(),
            command: command.into(),
            summary: summary.into(),
        }
    }

    /// Maps a retained command record into a display row.
    #[must_use]
    pub fn from_record(record: &CommandRecord) -> Self {
        Self::new(
            record.id.get(),
            status_marker(record),
            source_label(record.source.view.clone(), record.source.action.clone()),
            command_label(record),
            result_summary(record),
        )
    }
}

/// The effect requested after applying an input action to the history view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommandHistoryActionResult {
    /// Continue running the application.
    Continue,
    /// Rebuild the snapshot from current in-memory history.
    Refresh,
    /// Return to the previous view.
    ReturnBack,
    /// Exit the application.
    Quit,
}

/// Input actions understood by the command-history view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommandHistoryAction {
    /// Move to the previous command record.
    Previous,
    /// Move to the next command record.
    Next,
    /// Page toward older command records.
    PageNext,
    /// Page toward newer command records.
    PagePrevious,
    /// Move to the newest command record.
    First,
    /// Move to the oldest visible command record.
    Last,
    /// Refresh the snapshot from the current history.
    Refresh,
    /// Toggle mode-specific help.
    ToggleHelp,
    /// Return to the previous view.
    ReturnBack,
    /// Quit the TUI.
    Quit,
}

/// Interactive read-only command-history list.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandHistoryView {
    snapshot: CommandHistorySnapshot,
    selected: Option<usize>,
    scroll_offset: usize,
    help_visible: bool,
}

impl CommandHistoryView {
    /// Creates a command-history view with the initial snapshot loaded.
    #[must_use]
    pub fn new(snapshot: CommandHistorySnapshot) -> Self {
        let selected = clamp_index(Some(0), snapshot.rows.len());
        Self {
            snapshot,
            selected,
            scroll_offset: 0,
            help_visible: false,
        }
    }

    /// Replaces rows after a refresh.
    pub fn refresh(&mut self, snapshot: CommandHistorySnapshot) {
        let previous_selected = self.selected;
        self.snapshot = snapshot;
        self.selected = clamp_index(previous_selected.or(Some(0)), self.snapshot.rows.len());
        self.scroll_offset = clamp_scroll(self.scroll_offset, self.snapshot.rows.len());
    }

    /// Returns the selected row, if any.
    #[must_use]
    pub fn selected_row(&self) -> Option<&CommandHistoryRow> {
        self.selected
            .and_then(|index| self.snapshot.rows.get(index))
    }

    /// Applies a single input action.
    #[must_use]
    pub fn apply(&mut self, action: CommandHistoryAction) -> CommandHistoryActionResult {
        match action {
            CommandHistoryAction::Previous => {
                self.select_previous();
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::Next => {
                self.select_next();
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::PagePrevious => {
                self.page_previous();
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::PageNext => {
                self.page_next();
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::First => {
                if !self.snapshot.rows.is_empty() {
                    self.selected = Some(0);
                }
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::Last => {
                if !self.snapshot.rows.is_empty() {
                    self.selected = Some(self.snapshot.rows.len() - 1);
                }
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::Refresh => CommandHistoryActionResult::Refresh,
            CommandHistoryAction::ToggleHelp => {
                self.help_visible = !self.help_visible;
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::ReturnBack => CommandHistoryActionResult::ReturnBack,
            CommandHistoryAction::Quit if self.help_visible => {
                self.help_visible = false;
                CommandHistoryActionResult::Continue
            }
            CommandHistoryAction::Quit => CommandHistoryActionResult::Quit,
        }
    }

    /// Renders the command-history view.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.render_area(frame, area, None);
    }

    /// Renders the command-history view with a temporary status-line override.
    pub fn render_with_status(&mut self, frame: &mut Frame<'_>, status: &str) {
        let area = frame.area();
        self.render_area(frame, area, Some(status));
    }

    fn select_previous(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };
        self.selected = Some(selected.saturating_sub(1));
    }

    fn select_next(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };
        let last = self.snapshot.rows.len().saturating_sub(1);
        self.selected = Some(selected.saturating_add(1).min(last));
    }

    fn page_previous(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };
        self.selected = Some(selected.saturating_sub(10));
    }

    fn page_next(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };
        let last = self.snapshot.rows.len().saturating_sub(1);
        self.selected = Some(selected.saturating_add(10).min(last));
    }

    fn keep_selected_in_view(&mut self, height: usize) {
        self.scroll_offset = clamp_scroll(self.scroll_offset, self.snapshot.rows.len());
        let Some(selected) = self.selected else {
            return;
        };
        if height == 0 {
            return;
        }
        if selected < self.scroll_offset {
            self.scroll_offset = selected;
        } else if selected >= self.scroll_offset.saturating_add(height) {
            self.scroll_offset = selected.saturating_add(1).saturating_sub(height);
        }
    }

    fn render_area(&mut self, frame: &mut Frame<'_>, area: Rect, status_override: Option<&str>) {
        let areas = ViewChrome::layout(area);
        self.keep_selected_in_view(usize::from(areas.content.height));

        let fallback_status = adaptive_hotbar(BindingContext::CommandHistory, areas.status_width());
        let status = status_override.unwrap_or(&fallback_status);
        let chrome = ViewChrome::new(self.snapshot.title(), status);
        chrome.render(frame, areas);

        let paragraph = Paragraph::new(self.visible_text());
        frame.render_widget(paragraph, areas.content);

        if let Some(selected) = self.selected {
            paint_subtle_selected_row(frame, areas.content, selected, self.scroll_offset);
        }

        if self.help_visible {
            render_help_overlay(
                frame,
                areas.content,
                help_title(BindingContext::CommandHistory),
                &help_lines(BindingContext::CommandHistory),
            );
        }
    }

    fn visible_text(&self) -> Text<'_> {
        if self.snapshot.rows.is_empty() {
            return Text::from(vec![
                Line::from(Span::styled(
                    "No command history yet.",
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Commands run by jk will appear here."),
            ]);
        }

        let rows = self
            .snapshot
            .rows
            .iter()
            .skip(self.scroll_offset)
            .map(history_line)
            .collect::<Vec<_>>();
        Text::from(rows)
    }
}

fn history_line(row: &CommandHistoryRow) -> Line<'_> {
    Line::from(vec![
        Span::styled(
            format!("{:<4}", row.status),
            Style::new().fg(status_color(&row.status)),
        ),
        Span::raw(" "),
        Span::styled(format!("{:<22}", row.source), Style::new().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(&row.command, Style::new().add_modifier(Modifier::BOLD)),
        Span::raw(summary_suffix(&row.summary)),
    ])
}

fn status_color(status: &str) -> Color {
    match status {
        "ok" => Color::Green,
        "fail" | "err" => Color::Red,
        _ => Color::Yellow,
    }
}

fn status_marker(record: &CommandRecord) -> &'static str {
    if record.result.spawn_error.is_some() {
        return "err";
    }
    match record.result.exit_status {
        Some(status) if status.success => "ok",
        Some(_) => "fail",
        None => "run",
    }
}

fn source_label(view: SourceView, action: SourceAction) -> String {
    format!("{} {}", view_label(view), action_label(action))
}

fn view_label(view: SourceView) -> String {
    match view {
        SourceView::Log => "log".to_owned(),
        SourceView::Diff => "diff".to_owned(),
        SourceView::Show => "show".to_owned(),
        SourceView::Status => "status".to_owned(),
        SourceView::Evolog => "evolog".to_owned(),
        SourceView::Workspaces => "workspaces".to_owned(),
        SourceView::WorkspaceStatus => "workspace status".to_owned(),
        SourceView::WorkspaceDiff => "workspace diff".to_owned(),
        SourceView::CommandHistory => "history".to_owned(),
        SourceView::OperationLog => "operation log".to_owned(),
        SourceView::OperationShow => "operation show".to_owned(),
        SourceView::OperationDiff => "operation diff".to_owned(),
        SourceView::Other(label) => label,
        _ => "unknown".to_owned(),
    }
}

fn action_label(action: SourceAction) -> String {
    match action {
        SourceAction::InitialLoad => "load".to_owned(),
        SourceAction::Refresh => "refresh".to_owned(),
        SourceAction::OpenDiff => "diff".to_owned(),
        SourceAction::OpenShow => "show".to_owned(),
        SourceAction::OpenStatus => "status".to_owned(),
        SourceAction::OpenEvolog => "evolog".to_owned(),
        SourceAction::DescribeRevision => "describe".to_owned(),
        SourceAction::WorkspaceList => "list".to_owned(),
        SourceAction::WorkspaceStatus => "status".to_owned(),
        SourceAction::WorkspaceDiff => "diff".to_owned(),
        SourceAction::WorkspaceUpdateStale => "update-stale".to_owned(),
        SourceAction::OperationLog => "op log".to_owned(),
        SourceAction::OperationShow => "op show".to_owned(),
        SourceAction::OperationDiff => "op diff".to_owned(),
        SourceAction::Other(label) => label,
        _ => "unknown".to_owned(),
    }
}

fn command_label(record: &CommandRecord) -> String {
    if record.command.title.is_empty() {
        record.command.spec_preview.clone()
    } else {
        record.command.title.clone()
    }
}

fn result_summary(record: &CommandRecord) -> String {
    if let Some(error) = &record.result.spawn_error {
        return compact(format!("error: {error}"));
    }
    if let Some(status) = record.result.exit_status
        && !status.success
    {
        let status = exit_label(status.code, status.signal);
        if let Some(output) =
            stream_snippet(&record.result.stderr).or_else(|| stream_snippet(&record.result.stdout))
        {
            return compact(format!("{status}: {output}"));
        }
        return status;
    }
    if let Some(stderr) = stream_snippet(&record.result.stderr) {
        return compact(format!("stderr: {stderr}"));
    }
    if let Some(stdout) = stream_snippet(&record.result.stdout) {
        return compact(stdout);
    }
    String::new()
}

fn exit_label(code: Option<i32>, signal: Option<i32>) -> String {
    if let Some(code) = code {
        format!("exit {code}")
    } else if let Some(signal) = signal {
        format!("signal {signal}")
    } else {
        "failed".to_owned()
    }
}

fn stream_snippet(stream: &StreamSummary) -> Option<String> {
    if stream.snippet.trim().is_empty() {
        return None;
    }
    let plain = strip_ansi(&stream.snippet);
    Some(plain.lines().next().unwrap_or("").trim().to_owned())
}

fn summary_suffix(summary: &str) -> String {
    if summary.is_empty() {
        String::new()
    } else {
        format!("  {summary}")
    }
}

fn compact(text: impl AsRef<str>) -> String {
    let text = text.as_ref().trim();
    if text.chars().count() <= SUMMARY_LIMIT {
        return text.to_owned();
    }

    let mut truncated = text.chars().take(SUMMARY_LIMIT - 3).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn clamp_index(index: Option<usize>, len: usize) -> Option<usize> {
    let index = index?;
    if len == 0 {
        None
    } else {
        Some(index.min(len - 1))
    }
}

fn clamp_scroll(scroll_offset: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        scroll_offset.min(len - 1)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use jk_core::{CommandRecordFinish, CommandRecordStart, JjCommandSpec};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn snapshot_maps_records_newest_first() {
        let snapshot = CommandHistorySnapshot::from_records(history_with_records().records());

        assert_eq!(
            snapshot
                .rows()
                .iter()
                .map(|row| row.command.as_str())
                .collect::<Vec<_>>(),
            vec!["jj status", "jj log"]
        );
        assert_eq!(snapshot.rows()[0].status, "fail");
        assert_eq!(snapshot.rows()[0].source, "log status");
        assert_eq!(snapshot.rows()[0].summary, "exit 1: workspace is stale");
    }

    #[test]
    fn actions_move_and_request_back_refresh_or_quit() {
        let mut view = CommandHistoryView::new(CommandHistorySnapshot::new(vec![
            CommandHistoryRow::new(1, "ok", "log load", "jj log", ""),
            CommandHistoryRow::new(2, "fail", "log status", "jj status", "exit 1"),
        ]));

        assert_eq!(view.selected_row().map(|row| row.id), Some(1));
        assert_eq!(
            view.apply(CommandHistoryAction::Next),
            CommandHistoryActionResult::Continue
        );
        assert_eq!(view.selected_row().map(|row| row.id), Some(2));
        assert_eq!(
            view.apply(CommandHistoryAction::Previous),
            CommandHistoryActionResult::Continue
        );
        assert_eq!(view.selected_row().map(|row| row.id), Some(1));
        assert_eq!(
            view.apply(CommandHistoryAction::Refresh),
            CommandHistoryActionResult::Refresh
        );
        assert_eq!(
            view.apply(CommandHistoryAction::ReturnBack),
            CommandHistoryActionResult::ReturnBack
        );
        assert_eq!(
            view.apply(CommandHistoryAction::Quit),
            CommandHistoryActionResult::Quit
        );
    }

    #[test]
    fn basic_render_contains_title_status_source_command_and_summary() {
        let mut view = CommandHistoryView::new(CommandHistorySnapshot::from_records(
            history_with_records().records(),
        ));
        let backend = TestBackend::new(96, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("jk Command History"));
        assert!(rendered.contains("fail log status"));
        assert!(rendered.contains("jj status"));
        assert!(rendered.contains("exit 1: workspace is stale"));
        assert!(rendered.contains("ok   log load"));
        assert!(rendered.contains("jj log"));
        assert!(rendered.contains("Esc back"));
    }

    #[test]
    fn empty_render_explains_history_is_empty() {
        let mut view = CommandHistoryView::new(CommandHistorySnapshot::new(Vec::new()));
        let backend = TestBackend::new(72, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("No command history yet."));
        assert!(rendered.contains("Commands run by jk will appear here."));
    }

    #[test]
    fn snapshot_strips_ansi_from_output_summary() {
        let mut history = jk_core::CommandHistory::new(8);
        append_record(
            &mut history,
            JjCommandSpec::render_read_only(["log"]),
            SourceView::Log,
            SourceAction::InitialLoad,
            0,
            "\u{1b}[1m\u{1b}[38;5;2m@\u{1b}[0m current\n",
            "",
        );

        let snapshot = CommandHistorySnapshot::from_records(history.records());

        assert_eq!(snapshot.rows()[0].summary, "@ current");
    }

    fn history_with_records() -> jk_core::CommandHistory {
        let mut history = jk_core::CommandHistory::new(8);
        append_record(
            &mut history,
            JjCommandSpec::render_read_only(["log"]),
            SourceView::Log,
            SourceAction::InitialLoad,
            0,
            "log output\n",
            "",
        );
        append_record(
            &mut history,
            JjCommandSpec::render_read_only(["status"]),
            SourceView::Log,
            SourceAction::OpenStatus,
            1,
            "",
            "workspace is stale\nmore detail\n",
        );
        history
    }

    fn append_record(
        history: &mut jk_core::CommandHistory,
        spec: JjCommandSpec,
        view: SourceView,
        action: SourceAction,
        code: i32,
        stdout: &str,
        stderr: &str,
    ) {
        let start = CommandRecordStart::from_spec(&spec, jk_core::CommandSource::new(view, action))
            .with_started_at(SystemTime::UNIX_EPOCH);
        let finish = CommandRecordFinish::from_exit_code(
            code,
            stdout,
            stderr,
            SystemTime::UNIX_EPOCH + Duration::from_millis(1),
        );
        history.append(start, finish);
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
}
