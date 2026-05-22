use ratatui::text::Line;

/// Flatten rendered Ratatui lines to plain text for search and copy helpers.
///
/// Style is intentionally discarded at this boundary; callers that render content should keep the
/// original `Line` values.
pub fn document_plain_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

/// Concatenate rendered spans into plain text when callers intentionally discard style.
pub(crate) fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}
