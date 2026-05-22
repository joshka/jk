use serde_json::Value;

/// Row metadata that must stay aligned with rendered `jj` rows.
///
/// Feature roots decide how to use the metadata; this helper only reports whether the side channel
/// still matches the rendered row count. A mismatch means the caller should discard the metadata
/// instead of guessing alignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RowMetadata<T> {
    /// Metadata rows still match the rendered row count and may be paired safely.
    Valid(Vec<T>),
    /// Metadata drifted from the rendered rows and must be ignored by the caller.
    Drifted,
}

impl<T> RowMetadata<T> {
    /// Return rows only when the rendered row count still matches.
    pub fn into_rows_matching(self, rendered_row_count: usize) -> Option<Vec<T>> {
        match self {
            Self::Valid(rows) if rows.len() == rendered_row_count => Some(rows),
            Self::Valid(_) | Self::Drifted => None,
        }
    }
}

/// Extract a required string field from feature-owned metadata templates.
///
/// Callers own field names and schema meaning; this helper only reads one string-valued field
/// without inventing fallbacks.
pub fn string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    fields.get(name).and_then(Value::as_str).map(str::to_owned)
}

/// Extract a required non-empty string field, rejecting empty metadata values.
pub fn non_empty_string_field(
    fields: &serde_json::Map<String, Value>,
    name: &str,
) -> Option<String> {
    string_field(fields, name).filter(|field| !field.is_empty())
}

/// Extract an optional string field while treating absent, null, and empty values as `None`.
pub fn optional_string_field(
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
///
/// Callers own the policy for missing or malformed fields; this helper only reports whether the
/// JSON value was a boolean.
pub fn boolean_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<bool> {
    fields.get(name).and_then(Value::as_bool)
}
