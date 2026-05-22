//! Lightweight structure over rendered `jj` output.
//!
//! This root does not try to model repository data. It recognizes just enough
//! of jj's default and git diff text to pin file context while preserving the
//! original spans/styles produced by the CLI.

use ratatui::text::Line;

mod anchors;
mod projection;

#[cfg(test)]
use self::anchors::default_file_label;
use self::anchors::file_anchor;
#[cfg(test)]
pub use self::projection::active_file;
#[cfg(test)]
use self::projection::line_text;
pub use self::projection::project_with_active_file;

/// Lines emitted by `jj`, with their terminal styling preserved.
#[derive(Clone, Debug)]
pub struct DocumentLines {
    /// Rendered lines loaded from `jj`, preserving styles and wording.
    lines: Vec<Line<'static>>,
}

impl DocumentLines {
    /// Build a rendered document from already-styled `jj` lines.
    pub fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }

    /// Return the rendered lines as loaded from `jj`.
    pub fn lines(&self) -> &[Line<'static>] {
        &self.lines
    }

    /// Return the total number of rendered lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Detect file anchors from rendered lines using lightweight heading recognition.
    pub fn file_anchors(&self) -> Vec<FileAnchor> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(line_index, line)| file_anchor(line_index, line))
            .collect()
    }

    /// Return whether the indexed rendered line is blank after trimming.
    pub fn line_is_blank(&self, line_index: usize) -> bool {
        self.lines
            .get(line_index)
            .is_some_and(|line| projection::line_text(line).trim().is_empty())
    }
}

/// A file heading detected in rendered jj output.
///
/// `heading` is the styled text shown in the sticky header. `label` is the plain
/// file name used for copy actions and file navigation state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileAnchor {
    /// Line index where the file heading appears in the rendered document.
    line_index: usize,
    /// Styled heading text reused by sticky headers.
    heading: Line<'static>,
    /// Plain file label used for navigation and copy surfaces.
    label: String,
}

impl FileAnchor {
    /// Return the rendered line index of this file heading.
    pub fn line_index(&self) -> usize {
        self.line_index
    }

    /// Return the styled heading reused in sticky headers.
    pub fn heading(&self) -> Line<'static> {
        self.heading.clone()
    }

    /// Return the plain file label for navigation and copy flows.
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// A document split into fixed context and a scrollable body.
///
/// The fixed lines are derived from rendered jj output rather than regenerated,
/// so colors and wording stay aligned with user config and jj defaults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedDocument {
    /// Fixed sticky lines shown above the scrollable body.
    fixed_lines: Vec<Line<'static>>,
    /// Scrollable body lines beneath the sticky header.
    body_lines: Vec<Line<'static>>,
    /// Scroll offset into the scrollable body after removing pinned lines.
    body_scroll_offset: usize,
}

impl PinnedDocument {
    /// Return the sticky fixed lines shown above the body.
    pub fn fixed_lines(&self) -> &[Line<'static>] {
        &self.fixed_lines
    }

    /// Return the scrollable body lines beneath the sticky header.
    pub fn body_lines(&self) -> &[Line<'static>] {
        &self.body_lines
    }

    /// Return the body-local scroll offset after sticky projection.
    pub fn body_scroll_offset(&self) -> usize {
        self.body_scroll_offset
    }

    /// Return the height consumed by the sticky header in terminal rows.
    pub fn sticky_height(&self) -> u16 {
        self.fixed_lines.len().min(u16::MAX as usize) as u16
    }
}

#[cfg(test)]
mod tests;
