//! Lightweight structure over rendered `jj` output.
//!
//! This module does not try to model repository data. It recognizes just enough
//! of jj's default and git diff text to pin file context while preserving the
//! original spans/styles produced by the CLI.

use ratatui::text::{Line, Span};

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
            .is_some_and(|line| line_text(line).trim().is_empty())
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

/// Project a rendered document into sticky fixed lines plus a scrollable active-file body.
pub fn project_with_active_file(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
    prefix: impl IntoIterator<Item = Line<'static>>,
) -> PinnedDocument {
    // Show views pass compact log context as a prefix; diff views pass no
    // prefix. When context exists, keep a blank line before the file heading so
    // the sticky header looks like jj output rather than a fused paragraph.
    let Some(anchor) = active_file(anchors, scroll_offset)
        .or_else(|| file_after_separator(document, anchors, scroll_offset))
    else {
        return PinnedDocument {
            fixed_lines: Vec::new(),
            body_lines: document.lines().to_vec(),
            body_scroll_offset: scroll_offset,
        };
    };

    let fixed_lines = fixed_lines(prefix, anchor);
    PinnedDocument {
        fixed_lines,
        body_lines: lines_from_active_file(document, anchor.line_index()),
        body_scroll_offset: scroll_offset.saturating_sub(anchor.line_index().saturating_add(1)),
    }
}

/// Build sticky fixed lines from an optional prefix and the active file heading.
fn fixed_lines(
    prefix: impl IntoIterator<Item = Line<'static>>,
    anchor: &FileAnchor,
) -> Vec<Line<'static>> {
    let mut lines = prefix.into_iter().collect::<Vec<_>>();
    if !lines.is_empty() {
        lines.push(Line::default());
    }
    lines.push(anchor.heading());
    lines
}

/// Return the last file anchor at or above the current scroll offset.
pub fn active_file(anchors: &[FileAnchor], scroll_offset: usize) -> Option<&FileAnchor> {
    anchors
        .iter()
        .take_while(|anchor| anchor.line_index() <= scroll_offset)
        .last()
}

/// Activate the first file after a separating blank line to avoid dead scroll presses.
fn file_after_separator<'a>(
    document: &DocumentLines,
    anchors: &'a [FileAnchor],
    scroll_offset: usize,
) -> Option<&'a FileAnchor> {
    // jj show commonly separates commit metadata from the first file with a
    // blank line. Activating the file on that separator avoids a dead scroll
    // press where only hidden state changes.
    anchors.first().filter(|anchor| {
        anchor.line_index() == scroll_offset.saturating_add(1)
            && document
                .lines()
                .get(scroll_offset)
                .is_some_and(|line| line_text(line).trim().is_empty())
    })
}

/// Return all lines after the active file heading as the scrollable body.
fn lines_from_active_file(
    document: &DocumentLines,
    file_heading_index: usize,
) -> Vec<Line<'static>> {
    document
        .lines()
        .iter()
        .skip(file_heading_index.saturating_add(1))
        .cloned()
        .collect()
}

struct FileHeading {
    heading: Line<'static>,
    label: String,
}

fn file_anchor(line_index: usize, line: &Line<'static>) -> Option<FileAnchor> {
    let heading = file_heading(line)?;
    Some(FileAnchor {
        line_index,
        heading: heading.heading,
        label: heading.label,
    })
}

fn file_heading(line: &Line<'static>) -> Option<FileHeading> {
    let text = line_text(line);
    git_file_heading(line, &text).or_else(|| default_file_heading(line, &text))
}

fn default_file_heading(line: &Line<'static>, text: &str) -> Option<FileHeading> {
    let trimmed = text.trim_end();
    let label = default_file_label(trimmed)?;
    Some(FileHeading {
        heading: styled_subline(line, 0, trimmed.len()),
        label,
    })
}

fn default_file_label(heading: &str) -> Option<String> {
    let body = heading.strip_suffix(':')?;
    let file = [
        "Added ",
        "Modified ",
        "Removed ",
        "Deleted ",
        "Renamed ",
        "Copied ",
    ]
    .into_iter()
    .find_map(|prefix| body.strip_prefix(prefix))?;
    let file = file
        .strip_prefix("regular file ")
        .or_else(|| file.strip_prefix("executable file "))
        .or_else(|| file.strip_prefix("symlink "))
        .unwrap_or(file);
    Some(
        file.rsplit_once(" => ")
            .map(|(_, destination)| destination)
            .unwrap_or(file)
            .to_owned(),
    )
}

fn git_file_heading(line: &Line<'static>, text: &str) -> Option<FileHeading> {
    let prefix = "diff --git ";
    let rest = text.strip_prefix(prefix)?;
    let space_index = rest.find(' ')?;
    let b_path_start = prefix.len() + space_index + 1;
    let b_path = &text[b_path_start..];
    let (label, start, end) = clean_git_path_range(b_path, b_path_start)?;
    Some(FileHeading {
        heading: styled_subline(line, start, end),
        label: label.to_owned(),
    })
}

fn clean_git_path_range(path: &str, path_start: usize) -> Option<(&str, usize, usize)> {
    let trimmed = path.trim();
    let trim_start = path.find(trimmed).unwrap_or(0);
    let clean = trimmed
        .strip_prefix("b/")
        .or_else(|| trimmed.strip_prefix("a/"))?;
    let clean_start = path_start + trim_start + trimmed.len().saturating_sub(clean.len());
    Some((clean, clean_start, clean_start + clean.len()))
}

fn styled_subline(line: &Line<'static>, start: usize, end: usize) -> Line<'static> {
    let mut spans = Vec::new();
    let mut span_start = 0;

    for source_span in &line.spans {
        let content = source_span.content.as_ref();
        let span_end = span_start + content.len();
        let overlap_start = start.max(span_start);
        let overlap_end = end.min(span_end);
        if overlap_start < overlap_end
            && let Some(content) =
                content.get((overlap_start - span_start)..(overlap_end - span_start))
        {
            // Sticky file headers should look like the original jj line,
            // including colors from default jj output or `--git` output.
            let mut span = Span::from(content.to_owned());
            span.style = source_span.style;
            spans.push(span);
        }
        span_start = span_end;
    }

    Line::from(spans)
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[cfg(test)]
mod tests;
