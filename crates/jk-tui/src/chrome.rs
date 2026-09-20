//! Title and status chrome around the borderless log body.
//!
//! The log body should continue to look like `jj` output, so this module owns only the one-line
//! command title and one-line command affordance bar around the content area.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

const SURFACE_BACKGROUND: Color = Color::Rgb(30, 35, 47);
const REGION_BACKGROUND: Color = Color::Rgb(58, 72, 90);
const DANGER_BACKGROUND: Color = Color::Rgb(120, 45, 50);
const ACCENT_BACKGROUND: Color = Color::LightCyan;
const CHROME_STYLE: Style = Style::new().fg(Color::White).bg(SURFACE_BACKGROUND);
const CHROME_BADGE_STYLE: Style = Style::new().fg(Color::Black).bg(ACCENT_BACKGROUND);

/// Semantic color treatment for the borderless status region.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StatusTone {
    /// Normal navigation and command hints.
    #[default]
    Neutral,
    /// Work is in progress but the current content remains interactive.
    Loading,
    /// The requested work failed and the last usable content remains visible.
    Failure,
}

impl StatusTone {
    const fn style(self) -> Style {
        match self {
            Self::Neutral => Style::new().fg(Color::White).bg(REGION_BACKGROUND),
            Self::Loading => Style::new().fg(Color::Black).bg(ACCENT_BACKGROUND),
            Self::Failure => Style::new().fg(Color::White).bg(DANGER_BACKGROUND),
        }
    }
}

const OVERLAY_BACKGROUND: Color = SURFACE_BACKGROUND;
const OVERLAY_HEADER: Color = REGION_BACKGROUND;

/// Renders a small mode-specific help overlay centered in the content area.
pub fn render_help_overlay(frame: &mut Frame<'_>, area: Rect, title: &str, lines: &[String]) {
    if area.is_empty() {
        return;
    }

    let overlay = centered_rect(
        area,
        overlay_width(title, lines, area.width),
        overlay_height(title, lines),
    );
    frame.render_widget(Clear, overlay);

    let command_discovery = title == "Command discovery";
    let display_title = if command_discovery { "Help" } else { title };
    frame.render_widget(
        Block::default().style(Style::new().fg(Color::White).bg(OVERLAY_BACKGROUND)),
        overlay,
    );
    let header = Rect::new(overlay.x, overlay.y, overlay.width, overlay.height.min(1));
    frame.render_widget(
        Paragraph::new(format!("  {display_title}"))
            .style(Style::new().fg(Color::White).bg(OVERLAY_HEADER).bold()),
        header,
    );

    let body = Rect::new(
        overlay.x.saturating_add(2),
        overlay.y.saturating_add(2),
        overlay.width.saturating_sub(4),
        overlay.height.saturating_sub(2),
    );
    let text_lines = lines
        .iter()
        .map(|line| overlay_line(line, usize::from(body.width)))
        .collect::<Vec<_>>();
    let text = Text::from(text_lines);
    let paragraph = Paragraph::new(text)
        .style(Style::new().fg(Color::White).bg(OVERLAY_BACKGROUND))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, body);
}

fn overlay_width(title: &str, lines: &[String], area_width: u16) -> usize {
    const COMMAND_DISCOVERY_MAX_WIDTH: usize = 132;

    if title == "Command discovery" {
        let content_width = lines
            .iter()
            .map(|line| visible_width(line))
            .chain(std::iter::once(visible_width("Help")))
            .max()
            .unwrap_or(0);
        let area_width = usize::from(area_width);
        return content_width
            .saturating_add(4)
            .clamp(56, COMMAND_DISCOVERY_MAX_WIDTH)
            .min(area_width);
    }

    let content_width = lines
        .iter()
        .map(|line| visible_width(line))
        .chain(std::iter::once(visible_width(title)))
        .max()
        .unwrap_or(0);
    content_width.saturating_add(4).clamp(56, 96)
}

fn overlay_height(title: &str, lines: &[String]) -> usize {
    if title == "Command discovery" {
        return lines.len().saturating_add(3);
    }

    lines.len().saturating_add(4)
}

fn overlay_line(line: &str, content_width: usize) -> Line<'_> {
    if line.ends_with(':') {
        return Line::from(Span::styled(
            line,
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(line) = command_row_line(line) {
        return line;
    }

    if line.starts_with("Type ")
        || line == "actions, and aliases."
        || line.starts_with("Examples:")
        || line.contains(" closes")
        || line.starts_with("showing ")
        || line.trim_start().starts_with("key ")
    {
        return Line::from(Span::styled(line, Style::new().fg(Color::Gray)));
    }

    if line.starts_with('>') {
        let padding = content_width.saturating_sub(Span::raw(line).width());
        return Line::from(Span::styled(
            format!("{line}{}", " ".repeat(padding)),
            Style::new()
                .fg(Color::Black)
                .bg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ));
    }

    Line::from(line)
}

fn command_row_line(line: &str) -> Option<Line<'_>> {
    if !line.starts_with("  ") || line.trim_start().starts_with("no matching") {
        return None;
    }

    let mut spans = Vec::new();
    let mut cursor = 0;
    while let Some(prefix_start) = find_cell_prefix(line, cursor) {
        if prefix_start > cursor {
            spans.push(Span::raw(&line[cursor..prefix_start]));
        }

        let key_start = skip_spaces(line, prefix_start);
        let separator_start = find_space_run(line, key_start, 2)?;
        let separator_end = skip_spaces(line, separator_start);
        let key = &line[key_start..separator_start];
        if key.trim().is_empty() {
            return None;
        }

        spans.push(Span::raw(&line[prefix_start..key_start]));
        spans.push(Span::styled(
            key,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(&line[separator_start..separator_end]));

        let next_prefix = find_next_cell_prefix(line, separator_end);
        let action_end = next_prefix.unwrap_or(line.len());
        spans.push(Span::styled(
            &line[separator_end..action_end],
            Style::new().fg(Color::White),
        ));

        cursor = action_end;
        if next_prefix.is_none() {
            break;
        }
    }

    if cursor < line.len() {
        spans.push(Span::raw(&line[cursor..]));
    }

    Some(Line::from(spans))
}

const fn find_next_cell_prefix(line: &str, start: usize) -> Option<usize> {
    let mut cursor = start;
    while let Some(prefix_start) = find_cell_prefix(line, cursor) {
        let key_start = skip_spaces(line, prefix_start);
        if find_space_run(line, key_start, 2).is_some() {
            return Some(prefix_start);
        }
        cursor = key_start.saturating_add(1);
    }
    None
}

const fn find_cell_prefix(line: &str, start: usize) -> Option<usize> {
    let mut cursor = start;
    while let Some(space_start) = find_space_run(line, cursor, 2) {
        let key_start = skip_spaces(line, space_start);
        if key_start < line.len() {
            return Some(space_start);
        }
        cursor = key_start;
    }
    None
}

const fn find_space_run(line: &str, start: usize, min_len: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() {
        if bytes[cursor] != b' ' {
            cursor += 1;
            continue;
        }

        let run_start = cursor;
        while cursor < bytes.len() && bytes[cursor] == b' ' {
            cursor += 1;
        }
        if cursor.saturating_sub(run_start) >= min_len {
            return Some(run_start);
        }
    }
    None
}

const fn skip_spaces(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor] == b' ' {
        cursor += 1;
    }
    cursor
}

fn visible_width(text: &str) -> usize {
    text.chars().count()
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
    status_tone: StatusTone,
}

impl<'a> ViewChrome<'a> {
    /// Creates chrome for a command title and status message.
    pub const fn new(title: &'a str, status: &'a str) -> Self {
        Self {
            title,
            status,
            status_tone: StatusTone::Neutral,
        }
    }

    /// Colors the full status row for its current semantic state.
    pub const fn with_status_tone(mut self, status_tone: StatusTone) -> Self {
        self.status_tone = status_tone;
        self
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
            Span::styled("jk", CHROME_BADGE_STYLE),
            Span::styled(" ", CHROME_STYLE),
            Span::styled(self.title, CHROME_STYLE.add_modifier(Modifier::BOLD)),
        ]))
        .style(CHROME_STYLE);
        frame.render_widget(title, areas.title);

        let status = Paragraph::new(Line::from(self.status)).style(self.status_tone.style());
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

    use super::*;

    #[test]
    fn selector_overlay_uses_colored_regions_without_a_border() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
        terminal
            .draw(|frame| {
                render_help_overlay(
                    frame,
                    frame.area(),
                    "Diff files",
                    &["> src/main.rs".into(), "  src/lib.rs".into()],
                );
            })
            .expect("draw selector overlay");

        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        let overlay = centered_rect(Rect::new(0, 0, 80, 20), 56, 6);
        assert!(
            !['┌', '┐', '└', '┘']
                .into_iter()
                .any(|glyph| rendered.contains(glyph))
        );
        assert_eq!(buffer[(overlay.x, overlay.y)].bg, OVERLAY_HEADER);
        assert_eq!(
            buffer[(overlay.x, overlay.y.saturating_add(1))].bg,
            OVERLAY_BACKGROUND
        );
        let selected_y = overlay.y.saturating_add(2);
        for x in overlay.x.saturating_add(2)..overlay.right().saturating_sub(2) {
            assert_eq!(buffer[(x, selected_y)].bg, Color::LightCyan);
        }
    }
}
