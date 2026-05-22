use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::search::{SearchQuery, highlight_line};
use crate::tui::theme;

use super::{WorkspaceContext, WorkspacesView};

impl WorkspacesView {
    /// Renders the root header plus the selectable workspace list.
    pub fn render(&self, frame: &mut Frame, area: Rect, search: Option<&SearchQuery>) {
        let header_lines = self.header_lines();
        let header_height = u16::try_from(header_lines.len())
            .unwrap_or(u16::MAX)
            .min(area.height);
        let [header, rows] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .areas(area);

        frame.render_widget(Paragraph::new(header_lines), header);

        let selected = (!self.context.entries().is_empty()).then_some(self.selection.index());
        let mut state = ListState::default().with_selected(selected);
        frame.render_stateful_widget(workspace_list(&self.context, search), rows, &mut state);
    }

    /// Returns the rendered root and degraded-load diagnostics shown above the list.
    pub fn header_lines(&self) -> Vec<Line<'static>> {
        workspace_header_lines(&self.context)
    }
}

/// Builds the rendered header for the workspace root surface and any degraded-load diagnostics.
fn workspace_header_lines(context: &WorkspaceContext) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    match context.root() {
        Some(root) => lines.push(Line::from(format!("current root: {root}"))),
        None => lines.push(Line::from("current root: unavailable")),
    }
    if let Some(error) = context.root_error() {
        lines.push(Line::from(format!("root error: {error}")));
    }
    if let Some(error) = context.list_error() {
        lines.push(Line::from(format!("workspace list error: {error}")));
    }
    if let Some(error) = context.metadata_error() {
        lines.push(Line::from(format!("workspace metadata warning: {error}")));
    }
    lines
}

/// Projects workspace rows into the selectable list widget.
fn workspace_list(context: &WorkspaceContext, search: Option<&SearchQuery>) -> List<'static> {
    let items = if context.entries().is_empty() {
        vec![ListItem::new("no workspaces listed")]
    } else {
        context
            .entries()
            .iter()
            .map(|entry| {
                let lines = entry
                    .lines()
                    .into_iter()
                    .map(|line| highlight_line(line, search))
                    .collect::<Vec<_>>();
                ListItem::new(lines)
            })
            .collect::<Vec<_>>()
    };

    List::new(items).highlight_style(theme::active_row_style())
}
