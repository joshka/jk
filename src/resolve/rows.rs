//! Resolve conflict row loading and template parsing.
//!
//! The resolve list uses a narrow `jj log` template to preserve exact conflict paths for
//! path-first navigation while degrading malformed rows into visible unparsed entries.

use color_eyre::Result;
use serde_json::Value;

use crate::jj::{ViewSpec, run_jj_template_lines};
use crate::rendered_rows::string_field;

pub(crate) const RESOLVE_CONFLICT_TEMPLATE: &str = r#"self.conflicted_files().map(|entry| "{\"path\":" ++ json(entry.path()) ++ ",\"file_type\":" ++ json(entry.file_type()) ++ ",\"side_count\":" ++ json(entry.conflict_side_count()) ++ "}\n").join("")"#;

/// One conflicted path reported by the resolve template contract.
///
/// Invalid or drifted template rows are preserved as raw text so the resolve view can show a useful
/// row instead of silently dropping a conflicted file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveEntry {
    /// Exact conflicted path when the template row parsed successfully.
    path: Option<String>,
    /// File-type label from the conflict template when parsing succeeded.
    file_type: Option<String>,
    /// Conflict side count from the template when parsing succeeded.
    side_count: Option<usize>,
    /// Raw template line preserved when parsing drifted or returned invalid JSON.
    raw_line: Option<String>,
}

impl ResolveEntry {
    /// Builds one parsed resolve entry from template fields.
    pub fn parsed(
        path: Option<String>,
        file_type: Option<String>,
        side_count: Option<usize>,
    ) -> Self {
        Self {
            path,
            file_type,
            side_count,
            raw_line: None,
        }
    }

    /// Builds one degraded resolve entry that preserves the raw unparsed template row.
    pub fn unparsed(raw_line: String) -> Self {
        Self {
            path: None,
            file_type: None,
            side_count: None,
            raw_line: Some(raw_line),
        }
    }

    /// Returns the exact conflicted path when the template still proves it.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Returns the file-type label from the parsed template row.
    pub fn file_type(&self) -> Option<&str> {
        self.file_type.as_deref()
    }

    /// Returns the parsed conflict side count.
    pub fn side_count(&self) -> Option<usize> {
        self.side_count
    }

    /// Returns the preserved raw line when parsing degraded.
    pub fn raw_line(&self) -> Option<&str> {
        self.raw_line.as_deref()
    }
}

/// Load conflicted paths using jj's structured conflict template.
///
/// Process and template errors are returned to the caller. Per-row JSON drift is represented as
/// `ResolveEntry::unparsed` so the view can degrade row-by-row.
pub fn load_resolve_entries(spec: &ViewSpec) -> Result<Vec<ResolveEntry>> {
    Ok(
        run_jj_template_lines(spec, RESOLVE_CONFLICT_TEMPLATE, true)?
            .into_iter()
            .map(|line| parse_resolve_entry_line(&line))
            .collect(),
    )
}

/// Parses one resolve template line and degrades malformed JSON into a visible raw row.
fn parse_resolve_entry_line(line: &str) -> ResolveEntry {
    let raw_line = line.to_owned();
    let Ok(Value::Object(fields)) = serde_json::from_str::<Value>(line) else {
        return ResolveEntry::unparsed(raw_line);
    };

    ResolveEntry::parsed(
        string_field(&fields, "path"),
        string_field(&fields, "file_type"),
        integer_field(&fields, "side_count"),
    )
}

/// Extracts one unsigned integer field and converts it to the platform `usize`.
fn integer_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<usize> {
    fields
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_entry_parser_keeps_exact_fields() {
        let entry = parse_resolve_entry_line(
            r#"{"path":"dir/space file.txt","file_type":"file","side_count":3}"#,
        );

        assert_eq!(
            entry,
            ResolveEntry::parsed(
                Some("dir/space file.txt".to_owned()),
                Some("file".to_owned()),
                Some(3),
            )
        );
    }

    #[test]
    fn resolve_entry_parser_degrades_invalid_json_to_unparsed_row() {
        let entry = parse_resolve_entry_line("{not json");

        assert_eq!(entry, ResolveEntry::unparsed("{not json".to_owned()));
    }

    #[test]
    fn resolve_entry_parser_allows_missing_exact_path() {
        let entry =
            parse_resolve_entry_line(r#"{"path":null,"file_type":"symlink","side_count":2}"#);

        assert_eq!(
            entry,
            ResolveEntry::parsed(None, Some("symlink".to_owned()), Some(2))
        );
    }
}
