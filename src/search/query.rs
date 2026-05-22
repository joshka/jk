use std::ops::Range;

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
    pub(super) fn match_ranges(&self, text: &str) -> Vec<Range<usize>> {
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
