//! Rendered `jj` row loading and narrow metadata pairing.
//!
//! This module owns conversion from rendered jj terminal output and narrow
//! machine-oriented metadata templates into the selectable row models consumed
//! by the app's list views. Command identity, navigation provenance, and the
//! process boundary stay in `jj.rs`.

mod revisions;
use ratatui::text::Line;
use serde_json::Value;

pub use self::revisions::{LogItem, load_compact_log_context, load_entries};

/// Flatten rendered Ratatui lines to plain text for search and copy helpers.
///
/// Style is intentionally discarded at this boundary; callers that render content should keep the
/// original `Line` values.
pub fn document_plain_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

// Metadata loaders fail closed when their side-channel row count no longer
// matches rendered jj rows; selection should stay usable without guessed ids.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RowMetadata<T> {
    Valid(Vec<T>),
    Drifted,
}

impl<T> RowMetadata<T> {
    pub(crate) fn into_rows_matching(self, rendered_row_count: usize) -> Option<Vec<T>> {
        match self {
            Self::Valid(rows) if rows.len() == rendered_row_count => Some(rows),
            Self::Valid(_) | Self::Drifted => None,
        }
    }
}

pub(crate) fn string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    fields.get(name).and_then(Value::as_str).map(str::to_owned)
}

pub(crate) fn non_empty_string_field(
    fields: &serde_json::Map<String, Value>,
    name: &str,
) -> Option<String> {
    string_field(fields, name).filter(|field| !field.is_empty())
}

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

pub(crate) fn boolean_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<bool> {
    fields.get(name).and_then(Value::as_bool)
}

pub(crate) fn is_standalone_graph_line(line: &Line<'_>) -> bool {
    let text = line_text(line);
    first_content_char(&text).is_none_or(|character| character == '~')
}

pub(crate) fn first_content_char(text: &str) -> Option<char> {
    text.chars()
        .find(|character| !matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮'))
}

pub(crate) fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}
