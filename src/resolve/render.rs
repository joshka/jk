use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState};

use crate::search::{SearchQuery, highlight_line};
use crate::tui::theme;

use super::{ResolveEntry, ResolveView};

impl ResolveView {
    /// Renders the resolve list with the active selection and search highlights.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(entry_list(&self.entries, search), area, &mut state);
    }

    /// Returns the total rendered line count across all resolve rows.
    pub fn line_count(&self) -> usize {
        self.entries.iter().map(entry_line_count).sum()
    }
}

/// Projects resolve rows into the selectable list widget.
fn entry_list(entries: &[ResolveEntry], search: Option<&SearchQuery>) -> List<'static> {
    let items = entries
        .iter()
        .map(|entry| {
            let lines = entry
                .lines()
                .into_iter()
                .map(|line| highlight_line(line, search))
                .collect::<Vec<_>>();
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}

/// Returns the rendered line count for one resolve row.
fn entry_line_count(entry: &ResolveEntry) -> usize {
    entry.lines().len()
}

impl ResolveEntry {
    /// Projects one resolve entry into visible lines for the list surface.
    pub(super) fn lines(&self) -> Vec<Line<'static>> {
        if let Some(raw_line) = self.raw_line() {
            return vec![
                Line::from("unparsed conflict metadata"),
                Line::from(raw_line.to_owned()),
            ];
        }

        vec![
            Line::from(self.path().unwrap_or("(path unavailable)").to_owned()),
            Line::from(format!(
                "type: {}  sides: {}",
                self.file_type().unwrap_or("unknown"),
                side_count_label(self.side_count()),
            )),
        ]
    }

    /// Returns plain rendered row text for copy surfaces.
    pub(super) fn row_text(&self) -> String {
        self.lines()
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.into_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Formats the conflict side count for the degraded second line of a parsed row.
fn side_count_label(side_count: Option<usize>) -> String {
    match side_count {
        Some(1) => "1".to_owned(),
        Some(count) => count.to_string(),
        None => "unknown".to_owned(),
    }
}
