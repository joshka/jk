use std::ops::Range;

use ratatui::text::{Line, Span};

use crate::search::SearchQuery;

/// Flatten a styled line into plain text for search matching.
pub fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

/// Return whether any part of the line matches the query.
pub fn line_matches(line: &Line<'_>, query: &SearchQuery) -> bool {
    query.matches(&line_text(line))
}

/// Return whether any rendered line in an entry matches the query.
pub fn entry_matches(lines: &[Line<'static>], query: &SearchQuery) -> bool {
    lines.iter().any(|line| line_matches(line, query))
}

/// Highlight every match in the provided rendered line.
pub fn highlight_line(line: Line<'static>, query: Option<&SearchQuery>) -> Line<'static> {
    let Some(query) = query else {
        return line;
    };
    let ranges = query.match_ranges(&line_text(&line));
    if ranges.is_empty() {
        return line;
    }

    Line::from(highlight_spans(line.spans, &ranges))
}

/// Rebuild a rendered line with reversed style applied to all matching ranges.
fn highlight_spans(spans: Vec<Span<'static>>, ranges: &[Range<usize>]) -> Vec<Span<'static>> {
    let mut highlighted = Vec::new();
    let mut line_offset = 0;
    for span in spans {
        let content = span.content.into_owned();
        let span_start = line_offset;
        let span_end = span_start + content.len();
        highlighted.extend(highlight_span_parts(
            content, span.style, span_start, span_end, ranges,
        ));
        line_offset = span_end;
    }
    highlighted
}

/// Split one styled span into matched and unmatched segments for highlighting.
fn highlight_span_parts(
    content: String,
    style: ratatui::style::Style,
    span_start: usize,
    span_end: usize,
    ranges: &[Range<usize>],
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut cursor = 0;

    for range in ranges {
        let start = range.start.max(span_start);
        let end = range.end.min(span_end);
        if start >= end {
            continue;
        }

        let local_start = start - span_start;
        let local_end = end - span_start;
        if cursor < local_start {
            spans.push(Span::styled(content[cursor..local_start].to_owned(), style));
        }
        spans.push(Span::styled(
            content[local_start..local_end].to_owned(),
            style.patch(ratatui::style::Style::default().reversed()),
        ));
        cursor = local_end;
    }
    if cursor < content.len() {
        spans.push(Span::styled(content[cursor..].to_owned(), style));
    }

    spans
}
