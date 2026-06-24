//! Rendering adapter for inline details in `jj` log output.
//!
//! `jj` still owns the log body. This module inserts wrapped detail text into that body while
//! continuing the visible graph prefix, then converts the final ANSI string into Ratatui text for
//! display.

use ansi_to_tui::IntoText as _;
use ratatui::text::Text;
use textwrap::Options;

use crate::ansi_text::strip_ansi;

const DEFAULT_EXPANDED_PREFIX: &str = "│  ";

/// A rendered `jj` log body with optional inline details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderedLog<'a> {
    body: &'a str,
    expanded_details: Option<ExpandedDetails<'a>>,
}

/// Details to insert after a rendered log line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpandedDetails<'a> {
    after_line: usize,
    text: &'a str,
}

impl<'a> RenderedLog<'a> {
    /// Creates a render adapter for an opaque `jj` log body.
    pub const fn new(body: &'a str) -> Self {
        Self {
            body,
            expanded_details: None,
        }
    }

    /// Adds optional inline details to insert while rendering.
    pub const fn with_expanded_details(mut self, details: Option<ExpandedDetails<'a>>) -> Self {
        self.expanded_details = details;
        self
    }

    /// Renders the log body and wraps inline details to the content width.
    pub fn render_with_width(&self, width: usize) -> String {
        let mut rendered = String::new();
        self.write_to(&mut rendered, width);
        rendered
    }

    /// Maps an original body line to its final rendered line after inserted details.
    pub fn line_after_insertions(&self, line: usize, width: usize) -> usize {
        let Some(details) = self.expanded_details else {
            return line;
        };
        if details.after_line >= line {
            return line;
        }

        let lines = self.body.split_inclusive('\n').collect::<Vec<_>>();
        let inserted_lines = expanded_prefix_at_line(&lines, details.after_line)
            .map(|prefix| expanded_details_line_count(details.text, &prefix, width))
            .unwrap_or_default();
        line + inserted_lines
    }

    /// Writes the body, inserting details after the configured rendered line.
    fn write_to(&self, rendered: &mut String, width: usize) {
        let Some(details) = self.expanded_details else {
            rendered.push_str(self.body);
            return;
        };

        let lines = self.body.split_inclusive('\n').collect::<Vec<_>>();
        for (line_index, line) in lines.iter().enumerate() {
            rendered.push_str(line);
            if line_index == details.after_line {
                let prefix = expanded_prefix_at_line(&lines, line_index)
                    .unwrap_or_else(|| DEFAULT_EXPANDED_PREFIX.to_owned());
                if !line.ends_with('\n') {
                    rendered.push('\n');
                }
                write_expanded_details(rendered, details.text, &prefix, width);
            }
        }
    }
}

impl<'a> ExpandedDetails<'a> {
    /// Creates details to insert after `after_line`.
    pub const fn new(after_line: usize, text: &'a str) -> Self {
        Self { after_line, text }
    }
}

impl std::fmt::Display for RenderedLog<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.render_with_width(usize::MAX))
    }
}

/// Writes blank-padded, wrapped details using the continued graph prefix.
fn write_expanded_details(rendered: &mut String, details: &str, prefix: &str, width: usize) {
    write_expanded_line(rendered, prefix, "");

    for line in details.lines() {
        if line.is_empty() {
            write_expanded_line(rendered, prefix, "");
            continue;
        }

        for wrapped in textwrap::wrap(line, wrap_options(detail_width(width, prefix))) {
            write_expanded_line(rendered, prefix, wrapped.as_ref());
        }
    }

    write_expanded_line(rendered, prefix, "");
}

/// Counts the lines that [`write_expanded_details`] would insert.
fn expanded_details_line_count(details: &str, prefix: &str, width: usize) -> usize {
    let detail_lines = details.lines().map(|line| {
        if line.is_empty() {
            1
        } else {
            textwrap::wrap(line, wrap_options(detail_width(width, prefix))).len()
        }
    });

    1 + detail_lines.sum::<usize>() + 1
}

/// Writes one inserted detail line with the graph prefix.
fn write_expanded_line(rendered: &mut String, prefix: &str, line: &str) {
    rendered.push_str(prefix);
    rendered.push_str(line);
    rendered.push('\n');
}

/// Calculates wrapping width after reserving columns for the graph prefix.
fn detail_width(width: usize, prefix: &str) -> usize {
    width.saturating_sub(prefix.chars().count()).max(1)
}

/// Creates deterministic wrapping options for inline details.
fn wrap_options(width: usize) -> Options<'static> {
    Options::new(width).break_words(true)
}

/// Chooses the graph prefix that should continue into inserted detail lines.
fn expanded_prefix_at_line(lines: &[&str], line_index: usize) -> Option<String> {
    let line = lines.get(line_index)?;
    let next_graph_line = lines
        .iter()
        .skip(line_index + 1)
        .copied()
        .find(|line| !strip_ansi(line).trim().is_empty());
    Some(expanded_prefix(line, next_graph_line))
}

/// Chooses the graph prefix that should continue into inserted detail lines.
fn expanded_prefix(line: &str, next_line: Option<&str>) -> String {
    let prefix = continuation_prefix(line);
    let prefix = next_line.map(continuation_prefix).map_or_else(
        || prefix.clone(),
        |next_prefix| merge_continuation_prefixes(&prefix, &next_prefix),
    );
    let prefix = normalize_continuation_prefix(&prefix);
    if prefix.is_empty() {
        DEFAULT_EXPANDED_PREFIX.to_owned()
    } else {
        prefix
    }
}

/// Builds the graph continuation prefix implied by one rendered line.
fn continuation_prefix(line: &str) -> String {
    let line = strip_ansi(line);
    commit_continuation_prefix(&line).map_or_else(
        || body_continuation_prefix(&line),
        |prefix| format!("{prefix}  "),
    )
}

/// Combines continuation prefixes from surrounding graph lines.
fn merge_continuation_prefixes(previous: &str, next: &str) -> String {
    let width = previous.chars().count().max(next.chars().count());
    let mut merged = String::new();
    let previous = previous.chars().chain(std::iter::repeat(' ')).take(width);
    let next = next.chars().chain(std::iter::repeat(' ')).take(width);
    for (previous, next) in previous.zip(next) {
        merged.push(if previous == '│' || next == '│' {
            '│'
        } else {
            ' '
        });
    }
    merged
}

/// Keeps only the continuation lanes plus the usual spacing before detail text.
fn normalize_continuation_prefix(prefix: &str) -> String {
    let Some(last_lane) = prefix
        .chars()
        .enumerate()
        .filter_map(|(index, character)| (character == '│').then_some(index))
        .last()
    else {
        return String::new();
    };
    prefix
        .chars()
        .take(last_lane + 1)
        .chain([' ', ' '])
        .collect()
}

/// Builds a continuation prefix from a commit row.
fn commit_continuation_prefix(line: &str) -> Option<String> {
    let mut prefix = String::new();
    let mut seen_graph = false;

    for character in line.trim_end_matches('\n').chars() {
        if is_commit_marker(character) {
            prefix.push('│');
            return Some(prefix);
        }

        if is_graph_character(character) {
            prefix.push(continuation_for_graph_character(character));
            seen_graph = true;
            continue;
        }

        if character == ' ' && seen_graph {
            prefix.push(' ');
            continue;
        }

        break;
    }

    None
}

/// Builds a continuation prefix from a non-commit graph body line.
fn body_continuation_prefix(line: &str) -> String {
    let mut prefix = String::new();
    let mut seen_graph = false;

    for character in line.trim_end_matches('\n').chars() {
        if is_graph_character(character) {
            prefix.push(continuation_for_graph_character(character));
            seen_graph = true;
            continue;
        }

        if character == ' ' && seen_graph {
            prefix.push(' ');
            continue;
        }

        break;
    }

    prefix
}

/// Returns whether a graph item character represents a commit row.
const fn is_commit_marker(character: char) -> bool {
    matches!(character, '@' | '○' | '◆' | '×' | '+')
}

/// Returns whether a character is part of jj's rendered graph.
const fn is_graph_character(character: char) -> bool {
    matches!(
        character,
        '│' | '─' | '├' | '╭' | '╮' | '╯' | '╰' | '╲' | '╱'
    )
}

/// Returns the vertical continuation character for an existing graph segment.
const fn continuation_for_graph_character(character: char) -> char {
    match character {
        '│' => '│',
        _ => ' ',
    }
}

/// Converts rendered terminal text into Ratatui text, falling back to plain visible text if ANSI
/// parsing fails.
pub fn rendered_text(rendered: &str) -> Text<'static> {
    rendered
        .into_text()
        .unwrap_or_else(|_| Text::from(strip_ansi(rendered)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expanded_details_render_inline_after_selected_row() {
        let rendered = RenderedLog::new("@  aaa\n│  first\n○  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(1, "body")))
            .render_with_width(80);

        assert_eq!(rendered, "@  aaa\n│  first\n│  \n│  body\n│  \n○  bbb\n");
    }

    #[test]
    fn expanded_details_continue_prefixed_graph_lines() {
        let rendered = RenderedLog::new("│ ○  aaa\n│ │  first\n│ ○  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(1, "body")))
            .render_with_width(80);

        assert_eq!(
            rendered,
            "│ ○  aaa\n│ │  first\n│ │  \n│ │  body\n│ │  \n│ ○  bbb\n"
        );
    }

    #[test]
    fn expanded_details_link_next_graph_item_after_connector_line() {
        let rendered = RenderedLog::new("│ │ ○  aaa\n│ ├─╯  first\n│ ◆  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(1, "body")))
            .render_with_width(80);

        assert_eq!(
            rendered,
            "│ │ ○  aaa\n│ ├─╯  first\n│ │  \n│ │  body\n│ │  \n│ ◆  bbb\n"
        );
    }

    #[test]
    fn expanded_details_link_parallel_graph_item_after_connector_line() {
        let rendered = RenderedLog::new("│ ◆  aaa\n├─╯  first\n│ ○  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(1, "body")))
            .render_with_width(80);

        assert_eq!(
            rendered,
            "│ ◆  aaa\n├─╯  first\n│ │  \n│ │  body\n│ │  \n│ ○  bbb\n"
        );
    }

    #[test]
    fn expanded_details_keep_connector_separator_after_details() {
        let rendered = RenderedLog::new("│ ◆  aaa\n├─╯  first\n│ ○  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(0, "body")))
            .render_with_width(80);

        assert_eq!(
            rendered,
            "│ ◆  aaa\n│ │  \n│ │  body\n│ │  \n├─╯  first\n│ ○  bbb\n"
        );
    }

    #[test]
    fn expanded_details_keep_elision_separator_after_details() {
        let rendered = RenderedLog::new("│ ◆  aaa\n│ │  first\n~\n│ ○  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(1, "body")))
            .render_with_width(80);

        assert_eq!(
            rendered,
            "│ ◆  aaa\n│ │  first\n│ │  \n│ │  body\n│ │  \n~\n│ ○  bbb\n"
        );
    }

    #[test]
    fn expanded_details_keep_parallel_elision_separator_after_details() {
        let rendered =
            RenderedLog::new("│ ◆  aaa\n│ │  first\n│ ~  (elided revisions)\n│ │ ○  bbb\n")
                .with_expanded_details(Some(ExpandedDetails::new(1, "body")))
                .render_with_width(80);

        assert_eq!(
            rendered,
            "│ ◆  aaa\n│ │  first\n│ │  \n│ │  body\n│ │  \n│ ~  (elided revisions)\n│ │ ○  bbb\n"
        );
    }

    #[test]
    fn expanded_details_keep_elision_separator_and_blank_after_details() {
        let rendered = RenderedLog::new("◆  aaa\n│  first\n~\n\n○  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(1, "body")))
            .render_with_width(80);

        assert_eq!(
            rendered,
            "◆  aaa\n│  first\n│  \n│  body\n│  \n~\n\n○  bbb\n"
        );
    }

    #[test]
    fn expanded_details_wrap_to_content_width() {
        let rendered = RenderedLog::new("│ ○  aaa\n")
            .with_expanded_details(Some(ExpandedDetails::new(0, "one two three four five")))
            .render_with_width(13);

        assert_eq!(
            rendered,
            "│ ○  aaa\n│ │  \n│ │  one two\n│ │  three\n│ │  four\n│ │  five\n│ │  \n"
        );
    }

    #[test]
    fn maps_body_lines_after_inserted_details() {
        let log = RenderedLog::new("@  aaa\n○  bbb\n")
            .with_expanded_details(Some(ExpandedDetails::new(0, "one two three four five")));

        assert_eq!(log.line_after_insertions(0, 16), 0);
        assert_eq!(log.line_after_insertions(1, 16), 5);
    }
}
