use ratatui::text::Line;

use crate::documents::{DocumentLines, FileAnchor, PinnedDocument};

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

/// Return the last file anchor at or above the current scroll offset.
pub fn active_file(anchors: &[FileAnchor], scroll_offset: usize) -> Option<&FileAnchor> {
    anchors
        .iter()
        .take_while(|anchor| anchor.line_index() <= scroll_offset)
        .last()
}

pub(super) fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
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
