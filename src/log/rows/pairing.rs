use ratatui::text::Line;

use super::LogItem;
use super::metadata::RevisionMetadata;
use crate::rendered_rows::{RowMetadata, is_standalone_graph_line, line_text};

/// Group rendered terminal lines into selectable log items and pair optional metadata by row.
pub fn group_lines(
    lines: Vec<Line<'static>>,
    metadata: RowMetadata<RevisionMetadata>,
) -> Vec<LogItem> {
    let revision_row_count = lines.iter().filter(|line| starts_log_item(line)).count();
    let mut metadata = metadata
        .into_rows_matching(revision_row_count)
        .map(Vec::into_iter);

    let mut items = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_metadata: Option<RevisionMetadata> = None;

    for line in lines {
        let starts_item = starts_log_item(&line);
        let standalone_graph_line = is_standalone_graph_line(&line);

        if (starts_item || standalone_graph_line) && !current_lines.is_empty() {
            items.push(LogItem::new(
                current_lines,
                current_metadata
                    .as_ref()
                    .map(|metadata| metadata.change_id.clone()),
                current_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.commit_id.clone()),
            ));
            current_lines = Vec::new();
            current_metadata = None;
        }

        if starts_item {
            current_metadata = metadata.as_mut().and_then(Iterator::next);
        }
        current_lines.push(line);
    }

    if !current_lines.is_empty() {
        items.push(LogItem::new(
            current_lines,
            current_metadata
                .as_ref()
                .map(|metadata| metadata.change_id.clone()),
            current_metadata.and_then(|metadata| metadata.commit_id),
        ));
    }

    items
}

fn starts_log_item(line: &Line<'_>) -> bool {
    starts_log_item_text(&line_text(line))
}

pub fn starts_log_item_text(text: &str) -> bool {
    crate::rendered_rows::first_content_char(text)
        .is_some_and(|character| matches!(character, '@' | '○' | '◆'))
}
