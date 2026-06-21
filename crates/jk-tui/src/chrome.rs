//! Title and status chrome around the borderless log body.
//!
//! The log body should continue to look like `jj` output, so this module owns only the one-line
//! command title and one-line command affordance bar around the content area.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::Paragraph;

/// Default log-view status text shown when there is no transient error.
pub const LOG_STATUS: &str =
    "H home  L log  d diff  r refresh  j/k move  space page  enter/right expand  q quit";

/// Default diff-view status text shown when there is no transient error.
pub const DIFF_STATUS: &str =
    "H/L log  r refresh  j/k line  space page  [/]: file  </> hscroll  h/left fold  l/right unfold";

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
