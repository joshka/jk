//! Search query parsing, matching, and highlight styling.

use std::ops::Range;

use ratatui::text::{Line, Span};

/// A smart-case search query.
///
/// Lowercase queries match case-insensitively. Queries containing uppercase
/// letters become case-sensitive, following the common Vim-style convention.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    /// Original user-entered query text.
    text: String,
    /// Whether matching should preserve case based on the smart-case rule.
    case_sensitive: bool,
}

impl SearchQuery {
    /// Build a smart-case query, returning `None` for an empty prompt submission.
    pub fn new(text: String) -> Option<Self> {
        (!text.is_empty()).then(|| {
            let case_sensitive = text.chars().any(char::is_uppercase);
            Self {
                text,
                case_sensitive,
            }
        })
    }

    /// Return the original query text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return whether any match exists in the provided text.
    pub fn matches(&self, text: &str) -> bool {
        self.find_in(text).is_some()
    }

    /// Return every non-overlapping match range in the provided text.
    fn match_ranges(&self, text: &str) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        let mut search_start = 0;

        while search_start < text.len() {
            let Some(range) = self.find_in(&text[search_start..]) else {
                break;
            };
            let start = search_start + range.start;
            let end = search_start + range.end;
            ranges.push(start..end);
            search_start = end;
        }

        ranges
    }

    /// Return the first match range in the provided text, respecting smart-case behavior.
    fn find_in(&self, text: &str) -> Option<Range<usize>> {
        if self.case_sensitive {
            text.find(&self.text)
                .map(|start| start..start + self.text.len())
        } else {
            find_case_insensitive(text, &self.text)
        }
    }
}

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

/// Find the first case-insensitive match range using character boundaries.
fn find_case_insensitive(text: &str, needle: &str) -> Option<Range<usize>> {
    let needle_len = needle.chars().count();
    let needle = needle.to_lowercase();

    for (start, _) in text.char_indices() {
        let end = text[start..]
            .char_indices()
            .nth(needle_len)
            .map(|(index, _)| start + index)
            .unwrap_or(text.len());
        let candidate = &text[start..end];
        if candidate.to_lowercase() == needle {
            return Some(start..end);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use ratatui::style::Modifier;

    use super::*;

    #[test]
    fn search_query_uses_smart_case() {
        let query = SearchQuery::new("cargo".to_owned()).unwrap();

        assert!(query.matches("Cargo.toml"));

        let query = SearchQuery::new("Cargo".to_owned()).unwrap();

        assert!(query.matches("Cargo.toml"));
        assert!(!query.matches("cargo.toml"));
    }

    #[test]
    fn highlight_line_reverses_only_matching_text() {
        let query = SearchQuery::new("arg".to_owned()).unwrap();
        let line = Line::from("Cargo.toml");

        let highlighted = highlight_line(line, Some(&query));

        assert_eq!(highlighted.spans.len(), 3);
        assert_eq!(highlighted.spans[0].content.as_ref(), "C");
        assert!(
            !highlighted.spans[0]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
        assert_eq!(highlighted.spans[1].content.as_ref(), "arg");
        assert!(
            highlighted.spans[1]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
        assert_eq!(highlighted.spans[2].content.as_ref(), "o.toml");
        assert!(
            !highlighted.spans[2]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
    }

    #[test]
    fn highlight_line_preserves_existing_style_around_match() {
        let query = SearchQuery::new("cargo".to_owned()).unwrap();
        let style = ratatui::style::Style::default().fg(ratatui::style::Color::Green);
        let line = Line::from(Span::styled("Cargo.toml", style));

        let highlighted = highlight_line(line, Some(&query));

        assert_eq!(highlighted.spans[0].content.as_ref(), "Cargo");
        assert_eq!(
            highlighted.spans[0].style.fg,
            Some(ratatui::style::Color::Green)
        );
        assert!(
            highlighted.spans[0]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
        assert_eq!(highlighted.spans[1].content.as_ref(), ".toml");
        assert_eq!(
            highlighted.spans[1].style.fg,
            Some(ratatui::style::Color::Green)
        );
        assert!(
            !highlighted.spans[1]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
    }

    #[test]
    fn highlight_line_handles_matches_split_across_spans() {
        let query = SearchQuery::new("cargo".to_owned()).unwrap();
        let line = Line::from(vec![Span::raw("Car"), Span::raw("go.toml")]);

        let highlighted = highlight_line(line, Some(&query));

        assert_eq!(highlighted.spans.len(), 3);
        assert_eq!(highlighted.spans[0].content.as_ref(), "Car");
        assert!(
            highlighted.spans[0]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
        assert_eq!(highlighted.spans[1].content.as_ref(), "go");
        assert!(
            highlighted.spans[1]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
        assert_eq!(highlighted.spans[2].content.as_ref(), ".toml");
        assert!(
            !highlighted.spans[2]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
    }
}
