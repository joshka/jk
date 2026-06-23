//! Title and status chrome around the borderless log body.
//!
//! The log body should continue to look like `jj` output, so this module owns only the one-line
//! command title and one-line command affordance bar around the content area.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

const CHROME_STYLE: Style = Style::new().fg(Color::White).bg(Color::Black);
const CHROME_BADGE_STYLE: Style = Style::new().fg(Color::Black).bg(Color::White);

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
    let mut text_lines = Vec::new();
    if !command_discovery {
        text_lines.push(Line::from(Span::styled(
            display_title,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        text_lines.push(Line::from(""));
    }
    text_lines.extend(lines.iter().map(|line| overlay_line(line)));
    let text = Text::from(text_lines);
    let mut block = Block::bordered();
    if command_discovery {
        block = block.title(Span::styled(
            display_title,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    }
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::new().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, overlay);
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
        return lines.len().saturating_add(2);
    }

    lines.len().saturating_add(4)
}

fn overlay_line(line: &str) -> Line<'_> {
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
        return Line::from(Span::styled(
            line,
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
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

fn find_next_cell_prefix(line: &str, start: usize) -> Option<usize> {
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

fn find_cell_prefix(line: &str, start: usize) -> Option<usize> {
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

fn find_space_run(line: &str, start: usize, min_len: usize) -> Option<usize> {
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

fn skip_spaces(line: &str, start: usize) -> usize {
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
            Span::styled("jk", CHROME_BADGE_STYLE),
            Span::styled(" ", CHROME_STYLE),
            Span::styled(self.title, CHROME_STYLE.add_modifier(Modifier::BOLD)),
        ]))
        .style(CHROME_STYLE);
        frame.render_widget(title, areas.title);

        let status = Paragraph::new(Line::from(self.status)).style(CHROME_STYLE);
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
