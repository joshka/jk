use std::collections::HashSet;

use ratatui::text::Line;

use crate::rendered_rows::line_text;

use super::metadata::{BookmarkMetadata, BookmarkMetadataCoverage};
use super::{BookmarkItem, BookmarkRowState};

/// Pairs rendered bookmark rows with metadata and degrades safely when counts drift.
pub fn pair_bookmark_lines(
    lines: Vec<Line<'static>>,
    metadata: Vec<BookmarkMetadata>,
    coverage: BookmarkMetadataCoverage,
) -> Vec<BookmarkItem> {
    if lines.len() != metadata.len() {
        return lines
            .into_iter()
            .map(|line| {
                let text = line_text(&line);
                BookmarkItem {
                    lines: vec![line],
                    name: bookmark_name_from_rendered_row(&text),
                    target_change_id: None,
                    target_commit_id: None,
                    state: BookmarkRowState::Unknown,
                }
            })
            .collect();
    }

    let local_names = metadata
        .iter()
        .filter(|metadata| metadata.remote.is_none())
        .map(|metadata| metadata.name.clone())
        .collect::<HashSet<_>>();
    let tracked_remote_names = metadata
        .iter()
        .filter(|metadata| metadata.remote.is_some() && metadata.tracked)
        .map(|metadata| metadata.name.clone())
        .collect::<HashSet<_>>();
    let untracked_remote_names = metadata
        .iter()
        .filter(|metadata| metadata.remote.is_some() && !metadata.tracked)
        .map(|metadata| metadata.name.clone())
        .collect::<HashSet<_>>();

    let mut items = Vec::new();
    let mut metadata = metadata.into_iter();

    for line in lines {
        let text = line_text(&line);
        let metadata = metadata.next();
        let bookmark_name = metadata
            .as_ref()
            .map(|metadata| metadata.name.clone())
            .unwrap_or_else(|| bookmark_name_from_rendered_row(&text));
        let mut item = BookmarkItem::new(
            vec![line],
            bookmark_name,
            metadata
                .as_ref()
                .and_then(|metadata| metadata.target_change_id.clone()),
            metadata
                .as_ref()
                .and_then(|metadata| metadata.target_commit_id.clone()),
        );
        item.state = metadata
            .as_ref()
            .map_or(BookmarkRowState::Unknown, |metadata| {
                metadata.row_state(
                    coverage,
                    &local_names,
                    &tracked_remote_names,
                    &untracked_remote_names,
                )
            });
        items.push(item);
    }

    items
}

pub fn bookmark_name_from_rendered_row(text: &str) -> String {
    text.split_once(':')
        .map(|(name, _)| name.trim())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| text.trim())
        .to_owned()
}
