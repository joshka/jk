//! Title and status chrome around the borderless log body.
//!
//! The log body should continue to look like `jj` output, so this module owns only the one-line
//! command title and one-line command affordance bar around the content area.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};

use crate::styles::{SUPPORTING, SURFACE, TITLE};

/// Renders a content-sized borderless overlay centered in the content area.
pub fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, title: &str, lines: &[String]) {
    if area.is_empty() {
        return;
    }
    let title = if title == "Command discovery" {
        "Help"
    } else {
        title
    };
    let text = overlay_text(title, lines);
    let width = u16::try_from(text.width().saturating_add(4))
        .unwrap_or(u16::MAX)
        .min(area.width);
    let paragraph = Paragraph::new(text)
        .style(SURFACE)
        .wrap(Wrap { trim: false });
    let height = paragraph
        .line_count(width.saturating_sub(4).max(1))
        .saturating_add(2);
    let overlay = centered_rect(area, usize::from(width), height);
    frame.render_widget(Clear, overlay);
    frame.render_widget(
        paragraph.block(Block::default().padding(Padding::new(2, 2, 1, 1))),
        overlay,
    );
}

fn overlay_text<'a>(title: &'a str, lines: &'a [String]) -> Text<'a> {
    let mut text = vec![Line::from(Span::styled(title, TITLE)), Line::default()];
    text.extend(lines.iter().map(|line| Line::from(line.as_str())));
    Text::from(text)
}

fn centered_rect(area: Rect, preferred_width: usize, preferred_height: usize) -> Rect {
    let width = u16::try_from(preferred_width)
        .unwrap_or(u16::MAX)
        .min(area.width);
    let height = u16::try_from(preferred_height)
        .unwrap_or(u16::MAX)
        .min(area.height);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

const DEFAULT_TITLE: &str = "jj log";

/// Returns the command title shown in the title bar.
pub fn title_or_default(title: String) -> String {
    if title.is_empty() {
        DEFAULT_TITLE.to_owned()
    } else {
        title
    }
}

/// Borderless title/status chrome for a view.
#[derive(Clone, Copy, Debug)]
pub struct ViewChrome<'a> {
    title: &'a str,
    status: &'a str,
}

impl<'a> ViewChrome<'a> {
    /// Creates chrome for a command title and status message.
    pub const fn new(title: &'a str, status: &'a str) -> Self {
        Self { title, status }
    }

    /// Splits the terminal into title, content, and status rows.
    pub fn layout(area: Rect) -> ChromeAreas {
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        ChromeAreas {
            title: chunks[0],
            content: chunks[1],
            status: chunks[2],
        }
    }

    /// Renders the title and status rows without touching the content area.
    pub fn render(&self, frame: &mut Frame<'_>, areas: ChromeAreas) {
        let title = Paragraph::new(Line::from(vec![
            Span::styled("jk ", SUPPORTING),
            Span::styled(self.title, TITLE),
        ]))
        .style(SURFACE);
        frame.render_widget(title, areas.title);

        let status = Paragraph::new(Line::from(self.status)).style(SURFACE);
        frame.render_widget(status, areas.status);
    }
}

/// Screen regions reserved by [`ViewChrome`].
#[derive(Clone, Copy, Debug)]
pub struct ChromeAreas {
    /// The area available to the rendered log body.
    pub content: Rect,
    title: Rect,
    status: Rect,
}

impl ChromeAreas {
    /// Returns the width available to the one-line status row.
    pub const fn status_width(self) -> u16 {
        self.status.width
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::prelude::Color;

    use super::*;

    #[test]
    fn overlay_clears_underlying_text_and_uses_terminal_palette() {
        let mut terminal = Terminal::new(TestBackend::new(32, 12)).expect("test terminal");
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new("underlying".repeat(50)), frame.area());
                render_help_overlay(
                    frame,
                    frame.area(),
                    "Actions",
                    &["n  new change".to_owned()],
                );
            })
            .expect("draw help");
        let buffer = terminal.backend().buffer();
        let blank = &buffer[(8, 3)];
        assert_eq!(blank.symbol(), " ");
        assert_eq!(blank.fg, Color::Reset);
        assert_eq!(blank.bg, Color::Reset);
        assert!(
            !buffer
                .content
                .iter()
                .any(|cell| matches!(cell.symbol(), "┌" | "┐" | "│"))
        );
    }

    #[test]
    fn overlay_measures_wide_and_combining_characters_in_cells() {
        let lines = vec!["界界 e\u{301}".to_owned()];
        let text = overlay_text("Help", &lines);
        assert_eq!(text.width(), 6);
        let mut terminal = Terminal::new(TestBackend::new(12, 8)).expect("test terminal");
        terminal
            .draw(|frame| render_help_overlay(frame, frame.area(), "Help", &lines))
            .expect("draw");
        assert_eq!(terminal.backend().buffer()[(3, 4)].symbol(), "界");
        assert_eq!(terminal.backend().buffer()[(8, 4)].symbol(), "e\u{301}");
    }

    #[test]
    fn tiny_help_surfaces_do_not_write_outside_the_viewport() {
        for width in 0..12 {
            for height in 0..6 {
                let mut terminal =
                    Terminal::new(TestBackend::new(width, height)).expect("terminal");
                terminal
                    .draw(|frame| {
                        render_help_overlay(frame, frame.area(), "Help", &["j next".to_owned()]);
                    })
                    .expect("draw");
            }
        }
    }
}
