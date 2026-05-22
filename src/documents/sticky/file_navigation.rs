use crate::documents::{DocumentLines, FileAnchor};

pub fn next_file_offset(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
) -> Option<usize> {
    let current = current_file_index(document, anchors, scroll_offset);
    let next_index = current.map_or(0, |index| index.saturating_add(1));
    anchors
        .get(next_index)
        .map(|anchor| file_activation_offset(document, anchor))
}

pub fn previous_file_offset(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
) -> Option<usize> {
    let current = current_file_index(document, anchors, scroll_offset)?;
    current
        .checked_sub(1)
        .and_then(|index| anchors.get(index))
        .map(|anchor| file_activation_offset(document, anchor))
}

pub fn current_file_label<'a>(
    document: &DocumentLines,
    anchors: &'a [FileAnchor],
    scroll_offset: usize,
) -> Option<&'a str> {
    current_file_index(document, anchors, scroll_offset)
        .and_then(|index| anchors.get(index))
        .map(FileAnchor::label)
}

fn current_file_index(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
) -> Option<usize> {
    anchors
        .iter()
        .enumerate()
        .take_while(|(_, anchor)| file_activation_offset(document, anchor) <= scroll_offset)
        .last()
        .map(|(index, _)| index)
}

fn file_activation_offset(document: &DocumentLines, anchor: &FileAnchor) -> usize {
    let previous_line = anchor.line_index().saturating_sub(1);
    if anchor.line_index() > 0 && document.line_is_blank(previous_line) {
        previous_line
    } else {
        anchor.line_index()
    }
}
