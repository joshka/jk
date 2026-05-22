//! Shared rendered `jj` row helpers.
//!
//! Feature-specific row models should live with the feature that owns the user-visible behavior.
//! This module keeps domain-neutral helpers used by those row models: plain-text flattening,
//! metadata drift handling, JSON field extraction, graph-line detection, and rendered line text
//! extraction.

use ratatui::text::Line;
use serde_json::Value;

/// Flatten rendered Ratatui lines to plain text for search and copy helpers.
///
/// Style is intentionally discarded at this boundary; callers that render content should keep the
/// original `Line` values.
pub fn document_plain_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

/// Row metadata that must stay aligned with rendered `jj` rows.
///
/// Feature roots decide how to use the metadata; this helper only reports
/// whether the side channel still matches the rendered row count. A mismatch
/// means the caller should discard the metadata instead of guessing alignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RowMetadata<T> {
    Valid(Vec<T>),
    Drifted,
}

impl<T> RowMetadata<T> {
    /// Return rows only when the rendered row count still matches.
    pub(crate) fn into_rows_matching(self, rendered_row_count: usize) -> Option<Vec<T>> {
        match self {
            Self::Valid(rows) if rows.len() == rendered_row_count => Some(rows),
            Self::Valid(_) | Self::Drifted => None,
        }
    }
}

/// Extract a required string field from feature-owned metadata templates.
pub(crate) fn string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    fields.get(name).and_then(Value::as_str).map(str::to_owned)
}

/// Extract a required non-empty string field, rejecting empty metadata values.
pub(crate) fn non_empty_string_field(
    fields: &serde_json::Map<String, Value>,
    name: &str,
) -> Option<String> {
    string_field(fields, name).filter(|field| !field.is_empty())
}

/// Extract an optional string field while treating absent, null, and empty values as `None`.
pub(crate) fn optional_string_field(
    fields: &serde_json::Map<String, Value>,
    name: &str,
) -> Option<Option<String>> {
    match fields.get(name) {
        Some(Value::Null) | None => Some(None),
        Some(Value::String(value)) if value.is_empty() => Some(None),
        Some(Value::String(value)) => Some(Some(value.clone())),
        _ => None,
    }
}

/// Extract a required boolean field from feature-owned metadata templates.
pub(crate) fn boolean_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<bool> {
    fields.get(name).and_then(Value::as_bool)
}

/// Detect log rows that contain only structural glyphs or `~`.
///
/// The helper stays intentionally narrow so graph-row detection does not drift
/// into general rendered-text parsing.
pub(crate) fn is_standalone_graph_line(line: &Line<'_>) -> bool {
    let text = line_text(line);
    first_content_char(&text).is_none_or(|character| character == '~')
}

/// Return the first non-graph character in a rendered line.
pub(crate) fn first_content_char(text: &str) -> Option<char> {
    text.chars()
        .find(|character| !matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮'))
}

/// Concatenate rendered spans into plain text when callers intentionally discard style.
pub(crate) fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}
