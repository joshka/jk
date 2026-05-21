//! Rendered `jj` row loading and narrow metadata pairing.
//!
//! This module owns conversion from rendered jj terminal output and narrow
//! machine-oriented metadata templates into the selectable row models consumed
//! by the app's list views. Command identity, navigation provenance, and the
//! process boundary stay in `jj.rs`.

mod bookmarks;
mod revisions;
mod workspaces;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;
use serde_json::Value;

use crate::jj::{ColorMode, ViewSpec, run_jj, run_jj_template_lines};

pub use self::bookmarks::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState, load_bookmark_entries,
};
pub use self::revisions::{LogItem, load_compact_log_context, load_entries};
#[cfg(test)]
pub(crate) use self::workspaces::WORKSPACE_METADATA_TEMPLATE;
pub use self::workspaces::{WorkspaceContext, WorkspaceItem, load_workspace_context};
pub(crate) const RESOLVE_CONFLICT_TEMPLATE: &str = r#"self.conflicted_files().map(|entry| "{\"path\":" ++ json(entry.path()) ++ ",\"file_type\":" ++ json(entry.file_type()) ++ ",\"side_count\":" ++ json(entry.conflict_side_count()) ++ "}\n").join("")"#;

/// One selectable file item parsed from rendered file-list output.
///
/// The rendered line is kept as the presentation source, and `path` is only the exact file-list
/// text used by follow-up navigation or file actions.
#[derive(Clone, Debug)]
pub struct FileListItem {
    lines: Vec<Line<'static>>,
    path: String,
}

impl FileListItem {
    pub fn new(lines: Vec<Line<'static>>, path: String) -> Self {
        Self { lines, path }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    #[cfg(test)]
    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// One conflicted path reported by the resolve template contract.
///
/// Invalid or drifted template rows are preserved as raw text so the resolve view can show a useful
/// row instead of silently dropping a conflicted file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveEntry {
    path: Option<String>,
    file_type: Option<String>,
    side_count: Option<usize>,
    raw_line: Option<String>,
}

impl ResolveEntry {
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

    pub fn unparsed(raw_line: String) -> Self {
        Self {
            path: None,
            file_type: None,
            side_count: None,
            raw_line: Some(raw_line),
        }
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn file_type(&self) -> Option<&str> {
        self.file_type.as_deref()
    }

    pub fn side_count(&self) -> Option<usize> {
        self.side_count
    }

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

/// Load a rendered file-list view and pair each visible row with its exact path text.
///
/// This preserves jj's colorized output and filters only empty rows. The loader does not infer file
/// status or ownership beyond the rendered path string.
pub fn load_file_list_entries(spec: &ViewSpec) -> Result<Vec<FileListItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    Ok(lines
        .into_iter()
        .filter_map(|line| {
            let path = parse_file_list_path(&line_text(&line))?;
            Some(FileListItem::new(vec![line], path))
        })
        .collect())
}

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

fn parse_file_list_path(line: &str) -> Option<String> {
    (!line.is_empty()).then(|| line.to_owned())
}

fn string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    fields.get(name).and_then(Value::as_str).map(str::to_owned)
}

fn non_empty_string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    string_field(fields, name).filter(|field| !field.is_empty())
}

fn optional_string_field(
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

fn boolean_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<bool> {
    fields.get(name).and_then(Value::as_bool)
}

fn integer_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<usize> {
    fields
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
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

    #[test]
    fn file_list_path_parser_preserves_exact_text() {
        assert_eq!(
            parse_file_list_path("src/path with spaces"),
            Some("src/path with spaces".to_owned())
        );
        assert_eq!(parse_file_list_path(""), None);
    }

    #[test]
    fn file_list_item_preserves_row_lines_and_path() {
        let lines = b"src/path with spaces\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let item = FileListItem::new(lines, "src/path with spaces".to_owned());

        assert_eq!(item.line_count(), 1);
        assert_eq!(item.path(), "src/path with spaces");
        assert_eq!(item.row_text(), "src/path with spaces");
    }
}
