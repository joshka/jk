//! Selected-row highlighting for rendered log output.
//!
//! The highlight is painted directly into the Ratatui buffer after the log body renders. This keeps
//! the `jj` text intact while making selection visible with a short cyan gradient that fades back
//! to the terminal background.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Color;

const SELECTED_ROW_START: RgbColor = RgbColor::new(82, 196, 192);
const SELECTED_ROW_END: RgbColor = RgbColor::new(12, 32, 38);
const SELECTED_GRAPH_FG: RgbColor = RgbColor::new(15, 20, 31);
const SELECTED_ROW_FADE_COLUMNS: usize = 24;
const SELECTED_ROW_TOTAL_COLUMNS: usize = 36;

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
        let column = usize::from(x - area.left());
        let width = usize::from(area.width);
        frame.buffer_mut()[(x, y)].set_bg(selected_row_bg(column, width));
    }

    frame.buffer_mut()[(area.left(), y)].set_fg(SELECTED_GRAPH_FG.into_color());
}

/// Returns the background color for one column of the selection highlight.
fn selected_row_bg(column: usize, width: usize) -> Color {
    if column >= SELECTED_ROW_TOTAL_COLUMNS {
        return Color::Reset;
    }

    selected_row_gradient(column, width).into_color()
}

/// Interpolates the visible gradient before the tail resets to the terminal.
fn selected_row_gradient(column: usize, width: usize) -> RgbColor {
    let fade_width = width
        .min(SELECTED_ROW_TOTAL_COLUMNS)
        .min(SELECTED_ROW_FADE_COLUMNS);
    let steps = fade_width.saturating_sub(1).max(1);
    SELECTED_ROW_START.interpolate(SELECTED_ROW_END, column.min(steps), steps)
}

/// Small RGB value used to keep gradient math independent from Ratatui.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl RgbColor {
    /// Creates an RGB color from channel values.
    const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Interpolates between two RGB colors using integer channel math.
    fn interpolate(self, end: Self, step: usize, steps: usize) -> Self {
        Self {
            red: interpolate_channel(self.red, end.red, step, steps),
            green: interpolate_channel(self.green, end.green, step, steps),
            blue: interpolate_channel(self.blue, end.blue, step, steps),
        }
    }

    /// Converts this color into Ratatui's RGB color type.
    const fn into_color(self) -> Color {
        Color::Rgb(self.red, self.green, self.blue)
    }
}

/// Interpolates one color channel without using float rounding.
fn interpolate_channel(start: u8, end: u8, step: usize, steps: usize) -> u8 {
    let start = usize::from(start);
    let end = usize::from(end);
    if start <= end {
        channel_from_usize(start + ((end - start) * step / steps))
    } else {
        channel_from_usize(start - ((start - end) * step / steps))
    }
}

/// Converts a calculated channel value into a saturated byte.
fn channel_from_usize(value: usize) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_row_background_darkens_from_cool_blue_then_resets() {
        assert_eq!(
            selected_row_bg(0, 5),
            Color::Rgb(
                SELECTED_ROW_START.red,
                SELECTED_ROW_START.green,
                SELECTED_ROW_START.blue
            )
        );
        assert_eq!(
            selected_row_bg(23, 80),
            Color::Rgb(
                SELECTED_ROW_END.red,
                SELECTED_ROW_END.green,
                SELECTED_ROW_END.blue
            )
        );
        assert_eq!(selected_row_bg(35, 80), selected_row_bg(23, 80));
        assert_eq!(selected_row_bg(36, 80), Color::Reset);
        assert_eq!(selected_row_bg(12, 80), Color::Rgb(46, 111, 112));
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
}
