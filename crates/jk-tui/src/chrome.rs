//! Title and status chrome around the borderless log body.
//!
//! The log body should continue to look like `jj` output, so this module owns only the one-line
//! command title and one-line command affordance bar around the content area.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

/// Default log-view status text shown when there is no transient error.
pub const LOG_STATUS: &str =
    "? help  H home  L log  d diff  r refresh  j/k move  space page  q quit";

/// Default diff-view status text shown when there is no transient error.
pub const DIFF_STATUS: &str = "? help  r refresh  j/k line  space page  q quit";

/// Renders a small mode-specific help overlay centered in the content area.
pub fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, title: &str, lines: &[&str]) {
    if area.is_empty() {
        return;
    }

    let overlay = centered_rect(area, 72, lines.len().saturating_add(4));
    frame.render_widget(Clear, overlay);

    let text = Text::from(
        std::iter::once(Line::from(Span::styled(
            title,
            Style::new().add_modifier(Modifier::BOLD),
        )))
        .chain(std::iter::once(Line::from("")))
        .chain(lines.iter().copied().map(Line::from))
        .collect::<Vec<_>>(),
    );
    let paragraph = Paragraph::new(text)
        .block(Block::bordered())
        .style(Style::new().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, overlay);
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
            Span::styled("jk", Style::new().fg(Color::Black).bg(Color::White)),
            Span::raw(" "),
            Span::styled(self.title, Style::new().add_modifier(Modifier::BOLD)),
        ]));
        frame.render_widget(title, areas.title);

        let status = Paragraph::new(Line::from(self.status))
            .style(Style::new().fg(Color::Black).bg(Color::White));
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
