use ratatui::text::Line;

use super::text::line_text;

/// Detect log rows that contain only structural glyphs or `~`.
///
/// The helper stays intentionally narrow so graph-row detection does not drift into general
/// rendered-text parsing.
pub(crate) fn is_standalone_graph_line(line: &Line<'_>) -> bool {
    let text = line_text(line);
    first_content_char(&text).is_none_or(|character| character == '~')
}

/// Return the first non-graph character in a rendered line.
///
/// Shared row parsers use this to distinguish graph-only prefixes from the first content glyph
/// without committing to feature-specific row formats.
pub(crate) fn first_content_char(text: &str) -> Option<char> {
    text.chars()
        .find(|character| !matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮'))
}
