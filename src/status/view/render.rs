use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

use crate::search::{SearchQuery, highlight_line};
use crate::status::rows::StatusRow;
use crate::status::view::StatusView;
use crate::tui::theme;

impl StatusView {
    /// Render the current status rows with search highlighting and active-row styling.
    pub fn render(&self, frame: &mut Frame, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(row_list(&self.rows, search), area, &mut state);
    }
}

/// Build the rendered row list with active-row styling and optional search highlighting.
fn row_list(rows: &[StatusRow], search: Option<&SearchQuery>) -> List<'static> {
    let items = rows
        .iter()
        .map(|row| ListItem::new(highlight_line(row.line().clone(), search)))
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}
