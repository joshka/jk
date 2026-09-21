//! Title and status chrome around the borderless log body.
//!
//! The log body should continue to look like `jj` output, so this module owns only the one-line
//! command title and one-line command affordance bar around the content area.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Line, Span, Style, Text};
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};

use crate::styles::{
    CANVAS, FOCUS, SUPPORTING, TITLE, WARNING, dialog, dialog_accent, dialog_error,
};

/// Semantic emphasis for the borderless status region.
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
    fn style(self) -> Style {
        match self {
            Self::Neutral => CANVAS,
            Self::Loading => CANVAS.patch(FOCUS),
            Self::Failure => CANVAS.patch(WARNING),
        }
    }
}

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
        .style(dialog())
        .wrap(Wrap { trim: false });
    let height = paragraph
        .line_count(width.saturating_sub(4).max(1))
        .saturating_add(2);
    let overlay = centered_rect(area, usize::from(width), height);
    frame.render_widget(Clear, overlay);
    frame.render_widget(Block::default().style(dialog()), overlay);
    frame.render_widget(
        paragraph.block(Block::default().padding(Padding::new(2, 2, 1, 1))),
        overlay,
    );
}

fn overlay_text<'a>(title: &'a str, lines: &'a [String]) -> Text<'a> {
    let mut text = vec![
        Line::from(Span::styled(title, dialog_accent())),
        Line::default(),
    ];
    text.extend(lines.iter().map(|line| overlay_line(line)));
    Text::from(text)
}

/// Styles one jk-owned overlay row using the shared heading, key, cursor, and error roles.
///
/// Rows use the existing help/menu text convention: two spaces separate key and action columns; a
/// leading `>` marks the current candidate. Never apply this formatter to jj-owned output.
#[must_use]
pub fn overlay_line(line: &str) -> Line<'_> {
    if line.ends_with(':') {
        return Line::from(Span::styled(line, TITLE));
    }
    if line.starts_with("error:") {
        return Line::from(Span::styled(line, dialog_error()));
    }
    if line.starts_with('>') || line.starts_with("! ") || line.starts_with(": ") {
        return Line::from(vec![
            Span::styled(&line[..1], dialog_accent()),
            Span::raw(&line[1..]),
        ]);
    }
    command_row_line(line).unwrap_or_else(|| Line::from(Span::styled(line, SUPPORTING)))
}

fn command_row_line(line: &str) -> Option<Line<'_>> {
    if line.trim_start().starts_with("no matching") || find_space_run(line, 0, 2).is_none() {
        return None;
    }

    let mut spans = Vec::new();
    let mut cursor = 0;
    let mut prefix = if line.starts_with(' ') {
        find_cell_prefix(line, 0)
    } else {
        Some(0)
    };
    while let Some(prefix_start) = prefix {
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
        spans.push(Span::styled(key, dialog_accent()));
        spans.push(Span::raw(&line[separator_start..separator_end]));

        let next_prefix = find_next_cell_prefix(line, separator_end);
        let action_end = next_prefix.unwrap_or(line.len());
        spans.push(Span::styled(&line[separator_end..action_end], SUPPORTING));

        cursor = action_end;
        prefix = next_prefix;
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

    /// Emphasizes the status row according to its current semantic state.
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
            Span::styled("jk ", SUPPORTING),
            Span::styled(self.title, TITLE),
        ]))
        .style(CANVAS);
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
    use ratatui::prelude::Color;

    use super::*;

    #[test]
    fn overlay_clears_panel_text_and_preserves_surrounding_canvas() {
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
        assert_eq!(Some(blank.fg), dialog().fg);
        assert_eq!(Some(blank.bg), dialog().bg);
        assert_eq!(buffer[(0, 0)].symbol(), "u");
        assert_eq!(buffer[(0, 0)].bg, Color::Reset);
        assert!(
            !buffer
                .content
                .iter()
                .any(|cell| matches!(cell.symbol(), "┌" | "┐" | "│"))
        );
    }

    #[test]
    fn all_overlay_paths_style_key_columns_selection_and_errors() {
        let rows = vec![
            "Enter          inspect selected target".to_owned(),
            "  d  open diff".to_owned(),
            "> chosen revision".to_owned(),
            "error: unavailable revision".to_owned(),
        ];
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
        terminal
            .draw(|frame| render_help_overlay(frame, frame.area(), "Choices", &rows))
            .expect("overlay");
        let buffer = terminal.backend().buffer();
        for (needle, foreground) in [
            ("Enter", dialog_accent().fg),
            ("d  open", dialog_accent().fg),
            ("> chosen", dialog_accent().fg),
            ("error:", dialog_error().fg),
        ] {
            let position = (0..20)
                .find_map(|y| {
                    let line = (0..80).map(|x| buffer[(x, y)].symbol()).collect::<String>();
                    line.find(needle)
                        .and_then(|x| u16::try_from(x).ok().map(|x| (x, y)))
                })
                .unwrap_or_else(|| panic!("missing {needle}"));
            assert_eq!(Some(buffer[position].fg), foreground);
            assert_eq!(Some(buffer[position].bg), dialog().bg);
        }
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
    fn wide_selected_path_and_following_row_remain_on_separate_lines() {
        let selected = format!("> {}", "界".repeat(29));
        let lines = vec![selected, "  next.rs".to_owned()];
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("terminal");
        terminal
            .draw(|frame| {
                render_help_overlay(frame, frame.area(), "Diff files", &lines);
            })
            .expect("draw wide selector");

        let buffer = terminal.backend().buffer();
        let overlay = centered_rect(Rect::new(0, 0, 80, 24), 64, 6);
        assert_eq!(buffer[(overlay.x + 2, overlay.y + 3)].symbol(), ">");
        assert_eq!(buffer[(overlay.right() - 4, overlay.y + 3)].symbol(), "界");
        assert_eq!(buffer[(overlay.x + 4, overlay.y + 4)].symbol(), "n");
        for y in overlay.top()..overlay.bottom() {
            for x in overlay.left()..overlay.right() {
                // Ratatui resets the placeholder after a wide glyph; the glyph's leading cell
                // supplies the style for both terminal columns.
                if x > overlay.left() && Span::raw(buffer[(x - 1, y)].symbol()).width() > 1 {
                    continue;
                }
                assert_eq!(Some(buffer[(x, y)].bg), dialog().bg, "panel cell ({x},{y})");
            }
        }
        assert_eq!(buffer[(0, 0)].bg, Color::Reset);
    }

    #[test]
    fn wide_and_combining_selector_rows_use_content_width_in_terminal_cells() {
        let wide = vec![format!("> {}", "界".repeat(29))];
        assert_eq!(overlay_text("Diff files", &wide).width(), 60);

        let combining = vec!["e\u{301}".repeat(55)];
        assert_eq!(overlay_text("Diff files", &combining).width(), 55);
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
