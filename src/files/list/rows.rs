//! Rendered `jj file list` row loading and exact path identity.
//!
//! The rendered line remains the presentation source. The parsed path is only
//! the exact non-empty row text used by refresh, copy, drill-down, and file
//! action targets.

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;

use crate::jj::{ColorMode, ViewSpec, run_jj};
use crate::rendered_rows::line_text;

/// One selectable file item parsed from rendered file-list output.
///
/// The rendered line is kept as the presentation source, and `path` is only the exact file-list
/// text used by follow-up navigation or file actions.
#[derive(Clone, Debug)]
pub(crate) struct FileListItem {
    /// Rendered row lines preserved for display and search.
    lines: Vec<Line<'static>>,
    /// Exact file path text selected from `jj file list`.
    path: String,
}

impl FileListItem {
    /// Build one rendered file-list item with its exact path identity.
    pub(crate) fn new(lines: Vec<Line<'static>>, path: String) -> Self {
        Self { lines, path }
    }

    /// Return the rendered lines shown for this item.
    pub(crate) fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    /// Return the number of rendered lines owned by this item.
    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Return the exact file path selected from the rendered row text.
    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    #[cfg(test)]
    pub(crate) fn row_text(&self) -> String {
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
pub(crate) fn load_file_list_entries(spec: &ViewSpec) -> Result<Vec<FileListItem>> {
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

/// Parse one rendered file-list row into its exact path text, ignoring empty rows.
fn parse_file_list_path(line: &str) -> Option<String> {
    (!line.is_empty()).then(|| line.to_owned())
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
