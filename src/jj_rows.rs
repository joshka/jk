//! Rendered `jj` row loading and narrow metadata pairing.
//!
//! This module owns conversion from rendered jj terminal output and narrow
//! machine-oriented metadata templates into the selectable row models consumed
//! by the app's list views. Command identity, navigation provenance, and the
//! process boundary stay in `jj.rs`.

mod revisions;
use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;
use serde_json::Value;

use crate::jj::{ColorMode, ViewSpec, run_jj};

pub use self::revisions::{LogItem, load_compact_log_context, load_entries};

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

fn parse_file_list_path(line: &str) -> Option<String> {
    (!line.is_empty()).then(|| line.to_owned())
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

#[cfg(test)]
mod tests {
    use super::*;

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
