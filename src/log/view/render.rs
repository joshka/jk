use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState};

use super::{LogItem, LogView, explicit_selection_style};
use crate::search::{SearchQuery, highlight_line};
use crate::tui::theme;

impl LogView {
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(
            entry_list(&self.entries, search, &self.selected_change_ids),
            area,
            &mut state,
        );
    }
}

fn entry_list(
    entries: &[LogItem],
    search: Option<&SearchQuery>,
    selected_change_ids: &[String],
) -> List<'static> {
    let items = entries
        .iter()
        .map(|entry| {
            let is_selected = entry.action_id().is_some_and(|action_id| {
                selected_change_ids
                    .iter()
                    .any(|selected| selected == action_id)
            });
            let lines = entry_lines(entry, search, is_selected);
            ListItem::new(lines).style(if is_selected {
                explicit_selection_style()
            } else {
                Style::default()
            })
        })
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}

fn entry_lines(
    entry: &LogItem,
    search: Option<&SearchQuery>,
    is_selected: bool,
) -> Vec<Line<'static>> {
    let lines = entry
        .lines()
        .into_iter()
        .map(|line| highlight_line(line, search))
        .collect::<Vec<_>>();
    if is_selected {
        lines
            .into_iter()
            .map(|line| line.patch_style(explicit_selection_style()))
            .collect()
    } else {
        lines
    }
}

#[cfg(test)]
pub(super) fn test_entry_lines(
    entry: &LogItem,
    search: Option<&SearchQuery>,
    is_selected: bool,
) -> Vec<Line<'static>> {
    entry_lines(entry, search, is_selected)
}
