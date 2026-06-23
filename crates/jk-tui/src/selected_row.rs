//! Selected-row highlighting for rendered log output.
//!
//! The highlight is painted directly into the Ratatui buffer after the log body renders. This keeps
//! the `jj` text intact while making selection visible with one high-contrast row color.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Color;

const SELECTED_ROW_BG: Color = Color::Rgb(82, 196, 192);
const SELECTED_ROW_FG: Color = Color::Rgb(15, 20, 31);
const SUBTLE_SELECTED_ROW_BG: Color = Color::Rgb(34, 40, 44);

/// Paints the visible selected row background in the content area.
pub fn paint_selected_row(
    frame: &mut Frame<'_>,
    area: Rect,
    rendered_line: usize,
    scroll_offset: usize,
) {
    if area.is_empty() {
        return;
    }

    let Some(visible_line) = rendered_line.checked_sub(scroll_offset) else {
        return;
    };
    let Ok(visible_line) = u16::try_from(visible_line) else {
        return;
    };

    if visible_line >= area.height {
        return;
    }

    let y = area.y + visible_line;
    for x in area.left()..area.right() {
        frame.buffer_mut()[(x, y)].set_bg(SELECTED_ROW_BG);
        frame.buffer_mut()[(x, y)].set_fg(SELECTED_ROW_FG);
    }
}

/// Paints a quiet full-width selected row for non-graph content.
pub fn paint_subtle_selected_row(
    frame: &mut Frame<'_>,
    area: Rect,
    rendered_line: usize,
    scroll_offset: usize,
) {
    if area.is_empty() {
        return;
    }

    let Some(visible_line) = rendered_line.checked_sub(scroll_offset) else {
        return;
    };
    let Ok(visible_line) = u16::try_from(visible_line) else {
        return;
    };

    if visible_line >= area.height {
        return;
    }

    let y = area.y + visible_line;
    for x in area.left()..area.right() {
        frame.buffer_mut()[(x, y)].set_bg(SUBTLE_SELECTED_ROW_BG);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_row_uses_flat_high_contrast_colors() {
        let backend = ratatui::backend::TestBackend::new(3, 1);
        let mut terminal = ratatui::Terminal::new(backend).expect("test backend should initialize");

        let draw_result = terminal.draw(|frame| {
            paint_selected_row(frame, Rect::new(0, 0, 3, 1), 0, 0);
        });

        assert!(draw_result.is_ok());
        assert_eq!(terminal.backend().buffer()[(0, 0)].bg, SELECTED_ROW_BG);
        assert_eq!(terminal.backend().buffer()[(0, 0)].fg, SELECTED_ROW_FG);
        assert_eq!(terminal.backend().buffer()[(2, 0)].bg, SELECTED_ROW_BG);
        assert_eq!(terminal.backend().buffer()[(2, 0)].fg, SELECTED_ROW_FG);
    }

    #[test]
    fn selected_row_paint_ignores_empty_areas() {
        let backend = ratatui::backend::TestBackend::new(1, 1);
        let mut terminal = ratatui::Terminal::new(backend).expect("test backend should initialize");

        let draw_result = terminal.draw(|frame| {
            paint_selected_row(frame, Rect::new(0, 0, 0, 1), 0, 0);
            paint_selected_row(frame, Rect::new(0, 0, 1, 0), 0, 0);
        });

        assert!(draw_result.is_ok());
    }

    #[test]
    fn subtle_selected_row_uses_flat_background() {
        let backend = ratatui::backend::TestBackend::new(3, 1);
        let mut terminal = ratatui::Terminal::new(backend).expect("test backend should initialize");

        let draw_result = terminal.draw(|frame| {
            paint_subtle_selected_row(frame, Rect::new(0, 0, 3, 1), 0, 0);
        });

        assert!(draw_result.is_ok());
        assert_eq!(
            terminal.backend().buffer()[(0, 0)].bg,
            SUBTLE_SELECTED_ROW_BG
        );
        assert_eq!(
            terminal.backend().buffer()[(2, 0)].bg,
            SUBTLE_SELECTED_ROW_BG
        );
    }
}
