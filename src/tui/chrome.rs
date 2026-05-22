use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui_macros::{line, span, vertical};

use crate::status_line::{StatusKind, StatusLine};

use super::status_hints::status_hint_spans;

/// Fixed chrome layout for one terminal frame.
///
/// View renderers receive only `main`; this keeps title and status ownership in shared chrome
/// instead of leaking frame geometry into feature views.
#[derive(Clone, Copy, Debug)]
pub struct Areas {
    pub title: Rect,
    pub main: Rect,
    pub status: Rect,
}

/// Split the available terminal area into title, main content, and status rows.
///
/// The layout is deliberately stable at one row of chrome above and below the view so small
/// terminals degrade by shrinking the view, not by changing view-local rendering contracts.
pub fn areas(area: Rect) -> Areas {
    let [title, main, status] = vertical![==1, >=1, ==1].areas(area);
    Areas {
        title,
        main,
        status,
    }
}

/// Draw shared title and status chrome without touching the view's main content area.
pub fn render_chrome(frame: &mut Frame<'_>, areas: Areas, status: &StatusLine) {
    frame.render_widget(title_bar(status), areas.title);
    frame.render_widget(status_line(status, areas.status.width), areas.status);
}

fn title_bar(status: &StatusLine) -> Paragraph<'_> {
    Paragraph::new(line![
        span!(Style::default().fg(Color::DarkGray); "{title}", title = status.title())
    ])
}

fn status_line(status: &StatusLine, width: u16) -> Paragraph<'_> {
    Paragraph::new(status_line_text(status, width))
}

fn status_line_text(status: &StatusLine, width: u16) -> Line<'_> {
    let mut spans = line![span!(
        status_style(status);
        "{message}",
        message = status.message()
    )]
    .spans;
    let message_width = line_width(status.message());
    let available_hint_width = usize::from(width)
        .saturating_sub(message_width)
        .saturating_sub(2) as u16;
    // Hints are optional chrome. They fit only in the terminal width left after the primary status
    // message, so a narrow terminal keeps the state message visible instead of forcing view content
    // or status text to wrap.
    let hint_spans = status_hint_spans(status.hints(), available_hint_width).spans;

    if !hint_spans.is_empty() {
        spans.extend(line!["  "].spans);
        spans.extend(hint_spans);
    }

    Line::from(spans)
}

fn line_width(line: &str) -> usize {
    line.chars().count()
}

fn status_style(status: &StatusLine) -> Style {
    match status.kind() {
        StatusKind::Ready => Style::default().fg(Color::DarkGray),
        StatusKind::Error => Style::default().fg(Color::Red),
    }
}
