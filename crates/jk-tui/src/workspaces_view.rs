//! Public workspace list view and action contract.
//!
//! This module is provider-neutral. Callers map their workspace source into
//! [`WorkspaceViewSnapshot`] rows, translate input into [`WorkspacesAction`], and handle returned
//! [`WorkspacesActionResult`] values for effects such as refresh, status, diff, and navigation.

use std::path::{Path, PathBuf};

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::Paragraph;

use crate::chrome::{ViewChrome, render_help_overlay};
use crate::keymap::{BindingContext, adaptive_hotbar, help_lines, help_title};
use crate::selected_row::paint_subtle_selected_row;

const DEFAULT_TITLE: &str = "jj workspace list";

/// A provider-neutral snapshot of known workspaces.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceViewSnapshot {
    title: String,
    rows: Vec<WorkspaceViewRow>,
}

impl WorkspaceViewSnapshot {
    /// Creates a workspace snapshot from display rows.
    #[must_use]
    pub fn new(rows: Vec<WorkspaceViewRow>) -> Self {
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

    /// Returns the human-readable command context for this view.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the snapshot rows in display order.
    #[must_use]
    pub fn rows(&self) -> &[WorkspaceViewRow] {
        &self.rows
    }

    fn current_index(&self) -> Option<usize> {
        self.rows.iter().position(|row| row.current)
    }

    fn named_index(&self, name: &str) -> Option<usize> {
        self.rows.iter().position(|row| row.name == name)
    }
}

/// One display row in the workspace list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceViewRow {
    /// Workspace name.
    pub name: String,
    /// Workspace root path used for selected-workspace commands.
    pub root: Option<PathBuf>,
    /// Human-readable workspace root path.
    pub root_display: String,
    /// Whether this row is the current workspace.
    pub current: bool,
    /// Current working-copy change id, when known.
    pub change_id: Option<String>,
    /// Current working-copy commit id, when known.
    pub commit_id: Option<String>,
}

impl WorkspaceViewRow {
    /// Creates a workspace display row.
    #[must_use]
    pub fn new(name: impl Into<String>, root_display: impl Into<String>, current: bool) -> Self {
        Self {
            name: name.into(),
            root: None,
            root_display: root_display.into(),
            current,
            change_id: None,
            commit_id: None,
        }
    }

    /// Sets the row's workspace root path for follow-up commands.
    #[must_use]
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Returns the workspace root path used for follow-up commands.
    #[must_use]
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Sets the row's current working-copy change id.
    #[must_use]
    pub fn with_change_id(mut self, change_id: impl Into<String>) -> Self {
        self.change_id = Some(change_id.into());
        self
    }

    /// Sets the row's current working-copy commit id.
    #[must_use]
    pub fn with_commit_id(mut self, commit_id: impl Into<String>) -> Self {
        self.commit_id = Some(commit_id.into());
        self
    }
}

/// The effect requested after applying an input action to the workspace view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorkspacesActionResult {
    /// Continue running the application.
    Continue,
    /// Refresh the workspace list from the data source.
    Refresh,
    /// Open log for the selected workspace.
    OpenLog,
    /// Open status for the selected workspace.
    OpenStatus,
    /// Open diff for the selected workspace.
    OpenDiff,
    /// Update stale metadata for the selected workspace.
    UpdateStale,
    /// Return to the previous view.
    ReturnBack,
    /// Open the view-options surface.
    ViewOptions,
    /// Exit the application.
    Quit,
}

/// Input actions understood by the workspace view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorkspacesAction {
    /// Move to the previous workspace.
    Previous,
    /// Move to the next workspace.
    Next,
    /// Scroll one rendered line earlier.
    ScrollPreviousLine,
    /// Scroll one rendered line later.
    ScrollNextLine,
    /// Move to the first workspace.
    First,
    /// Move to the last workspace.
    Last,
    /// Refresh the workspace list.
    Refresh,
    /// Open log for the selected workspace.
    OpenLog,
    /// Open status for the selected workspace.
    OpenStatus,
    /// Open diff for the selected workspace.
    OpenDiff,
    /// Update stale metadata for the selected workspace.
    UpdateStale,
    /// Toggle mode-specific help.
    ToggleHelp,
    /// Return to the previous view.
    ReturnBack,
    /// Open the view-options surface.
    ViewOptions,
    /// Quit the TUI.
    Quit,
}

/// Interactive workspace list view.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspacesView {
    snapshot: WorkspaceViewSnapshot,
    selected: Option<usize>,
    scroll_offset: usize,
    status_message: Option<String>,
    help_visible: bool,
}

impl WorkspacesView {
    /// Creates a workspace view with the initial snapshot loaded.
    #[must_use]
    pub fn new(snapshot: WorkspaceViewSnapshot) -> Self {
        let selected = initial_selection(&snapshot);
        Self {
            snapshot,
            selected,
            scroll_offset: 0,
            status_message: None,
            help_visible: false,
        }
    }

    /// Replaces rows after a successful refresh.
    ///
    /// Selection is preserved by workspace name when possible, then falls back to the current
    /// workspace row, then clamps to the nearest available row.
    pub fn refresh(&mut self, snapshot: WorkspaceViewSnapshot) {
        let previous_name = self.selected_row().map(|row| row.name.clone());
        let previous_selected = self.selected;
        self.snapshot = snapshot;
        self.selected = previous_name
            .as_deref()
            .and_then(|name| self.snapshot.named_index(name))
            .or_else(|| self.snapshot.current_index())
            .or_else(|| clamp_index(previous_selected, self.snapshot.rows.len()));
        self.scroll_offset = clamp_scroll(self.scroll_offset, self.snapshot.rows.len());
        self.status_message = None;
    }

    /// Shows a refresh or integration error without replacing the current rows.
    pub fn show_error(&mut self, error: impl Into<String>) {
        self.status_message = Some(error.into());
    }

    /// Shows a short status message without replacing the current rows.
    pub fn show_status(&mut self, status: impl Into<String>) {
        self.status_message = Some(status.into());
    }

    /// Returns the selected row, if any.
    #[must_use]
    pub fn selected_row(&self) -> Option<&WorkspaceViewRow> {
        self.selected
            .and_then(|index| self.snapshot.rows.get(index))
    }

    /// Returns the selected workspace name, if any.
    #[must_use]
    pub fn selected_workspace_name(&self) -> Option<&str> {
        self.selected_row().map(|row| row.name.as_str())
    }

    /// Applies a single input action.
    #[must_use]
    pub fn apply(&mut self, action: WorkspacesAction) -> WorkspacesActionResult {
        match action {
            WorkspacesAction::Previous => {
                self.select_previous();
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::Next => {
                self.select_next();
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::ScrollPreviousLine => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::ScrollNextLine => {
                self.scroll_offset = clamp_scroll(
                    self.scroll_offset.saturating_add(1),
                    self.snapshot.rows.len(),
                );
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::First => {
                if !self.snapshot.rows.is_empty() {
                    self.selected = Some(0);
                }
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::Last => {
                if !self.snapshot.rows.is_empty() {
                    self.selected = Some(self.snapshot.rows.len() - 1);
                }
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::Refresh => WorkspacesActionResult::Refresh,
            WorkspacesAction::OpenLog if self.selected.is_some() => WorkspacesActionResult::OpenLog,
            WorkspacesAction::OpenStatus if self.selected.is_some() => {
                WorkspacesActionResult::OpenStatus
            }
            WorkspacesAction::OpenDiff if self.selected.is_some() => {
                WorkspacesActionResult::OpenDiff
            }
            WorkspacesAction::UpdateStale if self.selected.is_some() => {
                WorkspacesActionResult::UpdateStale
            }
            WorkspacesAction::OpenLog
            | WorkspacesAction::OpenStatus
            | WorkspacesAction::OpenDiff
            | WorkspacesAction::UpdateStale => WorkspacesActionResult::Continue,
            WorkspacesAction::ToggleHelp => {
                self.help_visible = !self.help_visible;
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::ReturnBack => WorkspacesActionResult::ReturnBack,
            WorkspacesAction::ViewOptions => WorkspacesActionResult::ViewOptions,
            WorkspacesAction::Quit if self.help_visible => {
                self.help_visible = false;
                WorkspacesActionResult::Continue
            }
            WorkspacesAction::Quit => WorkspacesActionResult::Quit,
        }
    }

    /// Renders the workspace view.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.render_area(frame, area, None);
    }

    /// Renders the workspace view with a temporary status-line override.
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

        let fallback_status = adaptive_hotbar(BindingContext::Workspaces, areas.status_width());
        let status = status_override
            .or(self.status_message.as_deref())
            .unwrap_or(&fallback_status);
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
                help_title(BindingContext::Workspaces),
                &help_lines(BindingContext::Workspaces),
            );
        }
    }

    fn visible_text(&self) -> Text<'_> {
        if self.snapshot.rows.is_empty() {
            return Text::from(vec![
                Line::from(Span::styled(
                    "No workspaces found.",
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
            .map(workspace_line)
            .collect::<Vec<_>>();
        Text::from(rows)
    }
}

fn workspace_line(row: &WorkspaceViewRow) -> Line<'_> {
    let marker = if row.current { "*" } else { " " };
    let summary = workspace_summary(row);
    Line::from(vec![
        Span::styled(
            marker,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(&row.name, Style::new().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::raw(&row.root_display),
        Span::raw(summary),
    ])
}

fn workspace_summary(row: &WorkspaceViewRow) -> String {
    match (&row.change_id, &row.commit_id) {
        (Some(change_id), Some(commit_id)) => format!("  change {change_id}  commit {commit_id}"),
        (Some(change_id), None) => format!("  change {change_id}"),
        (None, Some(commit_id)) => format!("  commit {commit_id}"),
        (None, None) => String::new(),
    }
}

fn initial_selection(snapshot: &WorkspaceViewSnapshot) -> Option<usize> {
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
    fn selection_starts_on_current_workspace() {
        let view = WorkspacesView::new(snapshot([
            row("default", false),
            row("dogfood", true),
            row("docs", false),
        ]));

        assert_eq!(view.selected_workspace_name(), Some("dogfood"));
    }

    #[test]
    fn refresh_preserves_selected_workspace_by_name() {
        let mut view = WorkspacesView::new(snapshot([
            row("default", true),
            row("dogfood", false),
            row("docs", false),
        ]));
        let _ = view.apply(WorkspacesAction::Next);

        view.refresh(snapshot([
            row("default", true),
            row("other", false),
            row("dogfood", false),
        ]));

        assert_eq!(view.selected_workspace_name(), Some("dogfood"));
    }

    #[test]
    fn refresh_falls_back_to_current_then_clamps() {
        let mut view = WorkspacesView::new(snapshot([
            row("default", true),
            row("dogfood", false),
            row("docs", false),
        ]));
        let _ = view.apply(WorkspacesAction::Next);

        view.refresh(snapshot([row("default", true), row("other", false)]));

        assert_eq!(view.selected_workspace_name(), Some("default"));

        let mut view = WorkspacesView::new(snapshot([
            row("default", false),
            row("dogfood", false),
            row("docs", false),
        ]));
        let _ = view.apply(WorkspacesAction::Next);
        let _ = view.apply(WorkspacesAction::Next);

        view.refresh(snapshot([row("solo", false)]));

        assert_eq!(view.selected_workspace_name(), Some("solo"));
    }

    #[test]
    fn empty_snapshot_is_safe() {
        let mut view = WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new()));

        assert_eq!(view.selected_workspace_name(), None);
        assert_eq!(
            view.apply(WorkspacesAction::Previous),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::Next),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::OpenStatus),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::OpenLog),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::OpenDiff),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::UpdateStale),
            WorkspacesActionResult::Continue
        );

        let backend = TestBackend::new(48, 6);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("No workspaces found."));
        assert!(rendered.contains("Press r to refresh."));
    }

    #[test]
    fn actions_return_expected_results() {
        let mut view = WorkspacesView::new(snapshot([row("default", true)]));

        assert_eq!(
            view.apply(WorkspacesAction::Previous),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::Next),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::ScrollPreviousLine),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::ScrollNextLine),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::First),
            WorkspacesActionResult::Continue
        );
        assert_eq!(view.selected_workspace_name(), Some("default"));
        assert_eq!(
            view.apply(WorkspacesAction::Last),
            WorkspacesActionResult::Continue
        );
        assert_eq!(view.selected_workspace_name(), Some("default"));
        assert_eq!(
            view.apply(WorkspacesAction::Refresh),
            WorkspacesActionResult::Refresh
        );
        assert_eq!(
            view.apply(WorkspacesAction::OpenStatus),
            WorkspacesActionResult::OpenStatus
        );
        assert_eq!(
            view.apply(WorkspacesAction::OpenLog),
            WorkspacesActionResult::OpenLog
        );
        assert_eq!(
            view.apply(WorkspacesAction::OpenDiff),
            WorkspacesActionResult::OpenDiff
        );
        assert_eq!(
            view.apply(WorkspacesAction::UpdateStale),
            WorkspacesActionResult::UpdateStale
        );
        assert_eq!(
            view.apply(WorkspacesAction::ViewOptions),
            WorkspacesActionResult::ViewOptions
        );
        assert_eq!(
            view.apply(WorkspacesAction::ReturnBack),
            WorkspacesActionResult::ReturnBack
        );
        assert_eq!(
            view.apply(WorkspacesAction::Quit),
            WorkspacesActionResult::Quit
        );
    }

    #[test]
    fn quit_closes_workspace_help_before_quitting() {
        let mut view = WorkspacesView::new(snapshot([row("default", true)]));
        let _ = view.apply(WorkspacesAction::ToggleHelp);

        assert_eq!(
            view.apply(WorkspacesAction::Quit),
            WorkspacesActionResult::Continue
        );
        assert_eq!(
            view.apply(WorkspacesAction::Quit),
            WorkspacesActionResult::Quit
        );
    }

    #[test]
    fn basic_render_contains_title_current_marker_root_and_summary() {
        let mut view = WorkspacesView::new(WorkspaceViewSnapshot::new(vec![
            WorkspaceViewRow::new("default", "/repo/default", true)
                .with_change_id("abc123")
                .with_commit_id("def456"),
        ]));
        let backend = TestBackend::new(80, 5);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("jk jj workspace list"));
        assert!(rendered.contains("* default"));
        assert!(rendered.contains("/repo/default"));
        assert!(rendered.contains("change abc123"));
        assert!(rendered.contains("commit def456"));
        assert!(rendered.contains("r refresh"));
    }

    #[test]
    fn help_overlay_uses_workspace_bindings() {
        let mut view = WorkspacesView::new(snapshot([row("default", true)]));
        let _ = view.apply(WorkspacesAction::ToggleHelp);
        let backend = TestBackend::new(72, 32);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Workspaces keys"));
        assert!(rendered.contains("Open and inspect:"));
        assert!(rendered.contains("open selected workspace status"));
        assert!(rendered.contains("Session:"));
        assert!(rendered.contains("update selected stale workspace"));
    }

    fn snapshot<const N: usize>(rows: [WorkspaceViewRow; N]) -> WorkspaceViewSnapshot {
        WorkspaceViewSnapshot::new(rows.into())
    }

    fn row(name: &str, current: bool) -> WorkspaceViewRow {
        WorkspaceViewRow::new(name, format!("/repo/{name}"), current)
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
