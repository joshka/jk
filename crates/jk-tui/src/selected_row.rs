//! Interaction markers outside jj-owned output.
//!
//! Marker-only selection preserves every foreground, background, and attribute in the user's jj
//! palette, including light and monochrome terminals. Cursor and marks occupy independent cells.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Style;

use crate::styles::FOCUS;

const GUTTER_WIDTH: u16 = 3;

/// Returns the width available to jj after reserving the interaction gutter.
#[must_use]
pub const fn content_width(terminal_width: u16) -> u16 {
    terminal_width.saturating_sub(GUTTER_WIDTH)
}

/// Separates cursor/mark cells from repository content without overwriting either.
pub fn interaction_areas(area: Rect) -> (Rect, Rect) {
    let width = GUTTER_WIDTH.min(area.width);
    let gutter = Rect { width, ..area };
    let content = Rect {
        x: area.x.saturating_add(width),
        width: area.width.saturating_sub(width),
        ..area
    };
    (gutter, content)
}

/// Paints a cursor anchor; an unfocused view retains its selection without the focus accent.
pub fn paint_cursor(
    frame: &mut Frame<'_>,
    gutter: Rect,
    rendered_line: usize,
    scroll_offset: usize,
    focused: bool,
) {
    paint_marker(
        frame,
        gutter,
        rendered_line,
        scroll_offset,
        (0, "›"),
        focused,
    );
}

/// Paints the selected object's continuation in the same cursor column.
pub fn paint_extent(
    frame: &mut Frame<'_>,
    gutter: Rect,
    rendered_line: usize,
    scroll_offset: usize,
    focused: bool,
) {
    paint_marker(
        frame,
        gutter,
        rendered_line,
        scroll_offset,
        (0, "·"),
        focused,
    );
}

/// Paints a persistent mark separately from the cursor and jj's working-copy node.
pub fn paint_mark(frame: &mut Frame<'_>, gutter: Rect, rendered_line: usize, scroll_offset: usize) {
    paint_marker(frame, gutter, rendered_line, scroll_offset, (1, "*"), false);
}

fn paint_marker(
    frame: &mut Frame<'_>,
    gutter: Rect,
    rendered_line: usize,
    scroll_offset: usize,
    marker: (u16, &str),
    focused: bool,
) {
    let (column, symbol) = marker;
    if gutter.is_empty() || column >= gutter.width {
        return;
    }
    let Some(visible_line) = rendered_line.checked_sub(scroll_offset) else {
        return;
    };
    let Ok(visible_line) = u16::try_from(visible_line) else {
        return;
    };
    if visible_line >= gutter.height {
        return;
    }
    let style = if focused { FOCUS } else { Style::default() };
    frame
        .buffer_mut()
        .set_string(gutter.x + column, gutter.y + visible_line, symbol, style);
}

#[cfg(test)]
mod tests {
    use ratatui::prelude::{Color, Modifier};

    use super::*;

    #[test]
    fn cursor_and_marks_preserve_repository_styles_and_symbols() {
        let backend = ratatui::backend::TestBackend::new(6, 1);
        let mut terminal = ratatui::Terminal::new(backend).expect("test backend should initialize");
        terminal
            .draw(|frame| {
                let (gutter, content) = interaction_areas(frame.area());
                let style = Style::new()
                    .fg(Color::Red)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::ITALIC);
                frame
                    .buffer_mut()
                    .set_string(content.x, content.y, "@ +", style);
                paint_cursor(frame, gutter, 0, 0, true);
                paint_mark(frame, gutter, 0, 0);
            })
            .expect("frame renders");
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].symbol(), "›");
        assert_eq!(buffer[(1, 0)].symbol(), "*");
        assert_eq!(buffer[(3, 0)].symbol(), "@");
        for x in 3..6 {
            assert_eq!(buffer[(x, 0)].fg, Color::Red);
            assert_eq!(buffer[(x, 0)].bg, Color::Blue);
            assert!(buffer[(x, 0)].modifier.contains(Modifier::ITALIC));
        }
    }

    #[test]
    fn markers_stay_in_gutter_at_small_sizes_and_scroll_offsets() {
        for width in 0..5 {
            let backend = ratatui::backend::TestBackend::new(width, 2);
            let mut terminal =
                ratatui::Terminal::new(backend).expect("test backend should initialize");
            terminal
                .draw(|frame| {
                    let (gutter, content) = interaction_areas(frame.area());
                    assert_eq!(content.width, content_width(width));
                    paint_cursor(frame, gutter, 0, 1, true);
                    paint_cursor(frame, gutter, 4, 1, true);
                    paint_cursor(frame, gutter, usize::MAX, 0, true);
                    paint_mark(frame, gutter, 1, 1);
                })
                .expect("tiny frame renders");
            if width > 0 {
                assert_eq!(terminal.backend().buffer()[(0, 0)].symbol(), " ");
            }
        }
    }

    #[test]
    fn covered_list_retains_cursor_without_focus_color() {
        let backend = ratatui::backend::TestBackend::new(3, 1);
        let mut terminal = ratatui::Terminal::new(backend).expect("test backend should initialize");
        terminal
            .draw(|frame| paint_cursor(frame, frame.area(), 0, 0, false))
            .expect("frame renders");
        assert_eq!(terminal.backend().buffer()[(0, 0)].symbol(), "›");
        assert_eq!(terminal.backend().buffer()[(0, 0)].fg, Color::Reset);
    }
}
