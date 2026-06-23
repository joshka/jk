//! Provider-neutral operation log list view.
//!
//! Callers map operation history into [`OperationLogSnapshot`] rows, translate terminal input into
//! [`OperationLogAction`], and handle returned [`OperationLogActionResult`] values for effects such
//! as refresh, operation show, operation diff, back navigation, and quit.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::Paragraph;

use crate::chrome::{ViewChrome, render_help_overlay};
use crate::keymap::{BindingContext, adaptive_hotbar, help_lines, help_title};
use crate::selected_row::paint_subtle_selected_row;

const DEFAULT_TITLE: &str = "jj op log";

/// A provider-neutral snapshot of operation log rows.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OperationLogSnapshot {
    title: String,
    rows: Vec<OperationLogRow>,
}

impl OperationLogSnapshot {
    /// Creates an operation-log snapshot from display rows.
    #[must_use]
    pub fn new(rows: Vec<OperationLogRow>) -> Self {
        Self {
            title: DEFAULT_TITLE.to_owned(),
            rows,
        }
    }

    /// Sets the command context shown in the title bar.
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
    pub fn rows(&self) -> &[OperationLogRow] {
        &self.rows
    }

    fn current_index(&self) -> Option<usize> {
        self.rows.iter().position(|row| row.current)
    }

    fn operation_index(&self, operation_id: &str) -> Option<usize> {
        self.rows
            .iter()
            .position(|row| row.operation_id == operation_id)
    }
}

/// One display row in the operation log.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationLogRow {
    /// Stable operation id used for follow-up actions and refresh selection.
    pub operation_id: String,
    /// Human-readable operation id, usually a short id.
    pub display_id: String,
    /// Human-readable operation description or title.
    pub title: String,
    /// Whether this row is the current operation.
    pub current: bool,
}

impl OperationLogRow {
    /// Creates an operation log row.
    #[must_use]
    pub fn new(
        operation_id: impl Into<String>,
        display_id: impl Into<String>,
        title: impl Into<String>,
        current: bool,
    ) -> Self {
        Self {
            operation_id: operation_id.into(),
            display_id: display_id.into(),
            title: title.into(),
            current,
        }
    }
}

/// The effect requested after applying an input action to the operation log view.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationLogActionResult {
    /// Continue running the application.
    Continue,
    /// Refresh the operation log from the data source.
    Refresh,
    /// Open `jj op show` for the selected operation id.
    OperationShow {
        /// Stable operation id selected for `jj op show`.
        operation_id: String,
    },
    /// Open `jj op diff` for the selected operation id.
    OperationDiff {
        /// Stable operation id selected for `jj op diff`.
        operation_id: String,
    },
    /// Return to the previous view.
    ReturnBack,
    /// Exit the application.
    Quit,
}

/// Input actions understood by the operation log view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationLogAction {
    /// Move to the previous operation.
    Previous,
    /// Move to the next operation.
    Next,
    /// Page toward newer operations.
    PagePrevious,
    /// Page toward older operations.
    PageNext,
    /// Move to the newest visible operation.
    First,
    /// Move to the oldest visible operation.
    Last,
    /// Refresh the operation log.
    Refresh,
    /// Open operation details for the selected operation.
    OpenShow,
    /// Open operation diff for the selected operation.
    OpenDiff,
    /// Toggle mode-specific help.
    ToggleHelp,
    /// Return to the previous view.
    ReturnBack,
    /// Quit the TUI.
    Quit,
}

/// Interactive operation log list view.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OperationLogView {
    snapshot: OperationLogSnapshot,
    selected: Option<usize>,
    scroll_offset: usize,
    help_visible: bool,
}

impl OperationLogView {
    /// Creates an operation log view with the initial snapshot loaded.
    #[must_use]
    pub fn new(snapshot: OperationLogSnapshot) -> Self {
        let selected = initial_selection(&snapshot);
        Self {
            snapshot,
            selected,
            scroll_offset: 0,
            help_visible: false,
        }
    }

    /// Replaces rows after a successful refresh.
    ///
    /// Selection is preserved by stable operation id when possible, then falls back to the current
    /// operation row, then clamps to the nearest available row.
    pub fn refresh(&mut self, snapshot: OperationLogSnapshot) {
        let previous_operation_id = self.selected_row().map(|row| row.operation_id.clone());
        let previous_selected = self.selected;
        self.snapshot = snapshot;
        self.selected = previous_operation_id
            .as_deref()
            .and_then(|operation_id| self.snapshot.operation_index(operation_id))
            .or_else(|| self.snapshot.current_index())
            .or_else(|| clamp_index(previous_selected, self.snapshot.rows.len()));
        self.scroll_offset = clamp_scroll(self.scroll_offset, self.snapshot.rows.len());
    }

    /// Returns the selected row, if any.
    #[must_use]
    pub fn selected_row(&self) -> Option<&OperationLogRow> {
        self.selected
            .and_then(|index| self.snapshot.rows.get(index))
    }

    /// Returns the selected stable operation id, if any.
    #[must_use]
    pub fn selected_operation_id(&self) -> Option<&str> {
        self.selected_row().map(|row| row.operation_id.as_str())
    }

    /// Applies a single input action.
    #[must_use]
    pub fn apply(&mut self, action: OperationLogAction) -> OperationLogActionResult {
        match action {
            OperationLogAction::Previous => {
                self.select_previous();
                OperationLogActionResult::Continue
            }
            OperationLogAction::Next => {
                self.select_next();
                OperationLogActionResult::Continue
            }
            OperationLogAction::PagePrevious => {
                self.page_previous();
                OperationLogActionResult::Continue
            }
            OperationLogAction::PageNext => {
                self.page_next();
                OperationLogActionResult::Continue
            }
            OperationLogAction::First => {
                if !self.snapshot.rows.is_empty() {
                    self.selected = Some(0);
                }
                OperationLogActionResult::Continue
            }
            OperationLogAction::Last => {
                if !self.snapshot.rows.is_empty() {
                    self.selected = Some(self.snapshot.rows.len() - 1);
                }
                OperationLogActionResult::Continue
            }
            OperationLogAction::Refresh => OperationLogActionResult::Refresh,
            OperationLogAction::OpenShow => self.selected_operation_id().map_or(
                OperationLogActionResult::Continue,
                |operation_id| OperationLogActionResult::OperationShow {
                    operation_id: operation_id.to_owned(),
                },
            ),
            OperationLogAction::OpenDiff => self.selected_operation_id().map_or(
                OperationLogActionResult::Continue,
                |operation_id| OperationLogActionResult::OperationDiff {
                    operation_id: operation_id.to_owned(),
                },
            ),
            OperationLogAction::ToggleHelp => {
                self.help_visible = !self.help_visible;
                OperationLogActionResult::Continue
            }
            OperationLogAction::ReturnBack => OperationLogActionResult::ReturnBack,
            OperationLogAction::Quit if self.help_visible => {
                self.help_visible = false;
                OperationLogActionResult::Continue
            }
            OperationLogAction::Quit => OperationLogActionResult::Quit,
        }
    }

    /// Renders the operation log view.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.render_area(frame, area, None);
    }

    /// Renders the operation log view with a temporary status-line override.
    pub fn render_with_status(&mut self, frame: &mut Frame<'_>, status: &str) {
        let area = frame.area();
        self.render_area(frame, area, Some(status));
    }

    const fn select_previous(&mut self) {
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

    const fn page_previous(&mut self) {
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

        let fallback_status = adaptive_hotbar(BindingContext::OperationLog, areas.status_width());
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
                help_title(BindingContext::OperationLog),
                &help_lines(BindingContext::OperationLog),
            );
        }
    }

    fn visible_text(&self) -> Text<'_> {
        if self.snapshot.rows.is_empty() {
            return Text::from(vec![
                Line::from(Span::styled(
                    "No operations found.",
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Press r to refresh."),
            ]);
        }

        let rows = self
            .snapshot
            .rows
            .iter()
            .skip(self.scroll_offset)
            .map(operation_line)
            .collect::<Vec<_>>();
        Text::from(rows)
    }
}

fn operation_line(row: &OperationLogRow) -> Line<'_> {
    let marker = if row.current { "*" } else { " " };
    Line::from(vec![
        Span::styled(
            marker,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            format!("{:<12}", row.display_id),
            Style::new().add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::raw(operation_title(row)),
    ])
}

fn operation_title(row: &OperationLogRow) -> &str {
    if row.title.trim().is_empty() {
        "(no description)"
    } else {
        &row.title
    }
}

fn initial_selection(snapshot: &OperationLogSnapshot) -> Option<usize> {
    snapshot
        .current_index()
        .or_else(|| clamp_index(Some(0), snapshot.rows.len()))
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
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn selection_starts_on_current_operation() {
        let view = OperationLogView::new(snapshot([
            row("op1", "op1", "initial checkout", false),
            row("op2", "op2", "describe change", true),
            row("op3", "op3", "previous operation", false),
        ]));

        assert_eq!(view.selected_operation_id(), Some("op2"));
    }

    #[test]
    fn refresh_preserves_selected_operation_by_id() {
        let mut view = OperationLogView::new(snapshot([
            row("op1", "op1", "initial checkout", true),
            row("op2", "op2", "describe change", false),
            row("op3", "op3", "previous operation", false),
        ]));
        let _ = view.apply(OperationLogAction::Next);

        view.refresh(snapshot([
            row("op4", "op4", "newer operation", true),
            row("op2", "op2", "describe change", false),
            row("op1", "op1", "initial checkout", false),
        ]));

        assert_eq!(view.selected_operation_id(), Some("op2"));
    }

    #[test]
    fn refresh_falls_back_to_current_then_clamps() {
        let mut view = OperationLogView::new(snapshot([
            row("op1", "op1", "initial checkout", true),
            row("op2", "op2", "describe change", false),
            row("op3", "op3", "previous operation", false),
        ]));
        let _ = view.apply(OperationLogAction::Next);

        view.refresh(snapshot([
            row("op4", "op4", "new current", true),
            row("op1", "op1", "initial checkout", false),
        ]));

        assert_eq!(view.selected_operation_id(), Some("op4"));

        let mut view = OperationLogView::new(snapshot([
            row("op1", "op1", "initial checkout", false),
            row("op2", "op2", "describe change", false),
            row("op3", "op3", "previous operation", false),
        ]));
        let _ = view.apply(OperationLogAction::Last);

        view.refresh(snapshot([row("op4", "op4", "only operation", false)]));

        assert_eq!(view.selected_operation_id(), Some("op4"));
    }

    #[test]
    fn show_and_diff_results_carry_selected_operation_id() {
        let mut view = OperationLogView::new(snapshot([
            row("op1-full", "op1", "initial checkout", true),
            row("op2-full", "op2", "describe change", false),
        ]));
        let _ = view.apply(OperationLogAction::Next);

        assert_eq!(
            view.apply(OperationLogAction::OpenShow),
            OperationLogActionResult::OperationShow {
                operation_id: "op2-full".to_owned(),
            }
        );
        assert_eq!(
            view.apply(OperationLogAction::OpenDiff),
            OperationLogActionResult::OperationDiff {
                operation_id: "op2-full".to_owned(),
            }
        );
    }

    #[test]
    fn empty_snapshot_is_safe_and_renders_empty_state() {
        let mut view = OperationLogView::new(OperationLogSnapshot::new(Vec::new()));

        assert_eq!(view.selected_operation_id(), None);
        assert_eq!(
            view.apply(OperationLogAction::Previous),
            OperationLogActionResult::Continue
        );
        assert_eq!(
            view.apply(OperationLogAction::Next),
            OperationLogActionResult::Continue
        );
        assert_eq!(
            view.apply(OperationLogAction::OpenShow),
            OperationLogActionResult::Continue
        );
        assert_eq!(
            view.apply(OperationLogAction::OpenDiff),
            OperationLogActionResult::Continue
        );

        let backend = TestBackend::new(64, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("No operations found."));
        assert!(rendered.contains("Press r to refresh."));
    }

    #[test]
    fn basic_render_contains_title_hotbar_marker_id_and_title() {
        let mut view = OperationLogView::new(snapshot([
            row("op1-full", "op1", "initial checkout", false),
            row("op2-full", "op2", "describe change", true),
        ]));
        let backend = TestBackend::new(88, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("jk jj op log"));
        assert!(rendered.contains("* op2"));
        assert!(rendered.contains("describe change"));
        assert!(rendered.contains("enter show"));
        assert!(rendered.contains("d diff"));
        assert!(rendered.contains("Esc back"));
    }

    #[test]
    fn help_overlay_uses_operation_log_keymap() {
        let mut view =
            OperationLogView::new(snapshot([row("op1-full", "op1", "initial checkout", true)]));
        let _ = view.apply(OperationLogAction::ToggleHelp);
        let backend = TestBackend::new(88, 12);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Operation Log keys"));
        assert!(rendered.contains("enter"));
        assert!(rendered.contains("open selected operation show"));
        assert!(rendered.contains('d'));
        assert!(rendered.contains("open selected operation diff"));
    }

    fn snapshot<const N: usize>(rows: [OperationLogRow; N]) -> OperationLogSnapshot {
        OperationLogSnapshot::new(rows.into())
    }

    fn row(operation_id: &str, display_id: &str, title: &str, current: bool) -> OperationLogRow {
        OperationLogRow::new(operation_id, display_id, title, current)
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
