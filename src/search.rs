//! Search query parsing, matching, and highlight styling.

use ratatui::text::{Line, Span};

/// A smart-case search query.
///
/// Lowercase queries match case-insensitively. Queries containing uppercase
/// letters become case-sensitive, following the common Vim-style convention.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    text: String,
    case_sensitive: bool,
}

impl SearchQuery {
    pub fn new(text: String) -> Option<Self> {
        (!text.is_empty()).then(|| {
            let case_sensitive = text.chars().any(char::is_uppercase);
            Self {
                text,
                case_sensitive,
            }
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn matches(&self, text: &str) -> bool {
        if self.case_sensitive {
            text.contains(&self.text)
        } else {
            text.to_lowercase().contains(&self.text.to_lowercase())
        }
    }
}

pub fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

pub fn line_matches(line: &Line<'_>, query: &SearchQuery) -> bool {
    query.matches(&line_text(line))
}

pub fn entry_matches(lines: &[Line<'static>], query: &SearchQuery) -> bool {
    lines.iter().any(|line| line_matches(line, query))
}

pub fn highlight_line(line: Line<'static>, query: Option<&SearchQuery>) -> Line<'static> {
    let Some(query) = query else {
        return line;
    };
    if !line_matches(&line, query) {
        return line;
    }

    Line::from(
        line.spans
            .into_iter()
            .map(|span| highlight_span(span, query))
            .collect::<Vec<_>>(),
    )
}

fn highlight_span(span: Span<'static>, query: &SearchQuery) -> Span<'static> {
    if query.matches(span.content.as_ref()) {
        span.patch_style(ratatui::style::Style::default().reversed())
    } else {
        span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_uses_smart_case() {
        let query = SearchQuery::new("cargo".to_owned()).unwrap();

        assert!(query.matches("Cargo.toml"));

        let query = SearchQuery::new("Cargo".to_owned()).unwrap();

        assert!(query.matches("Cargo.toml"));
        assert!(!query.matches("cargo.toml"));
    }
}
