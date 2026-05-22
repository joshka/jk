//! Operation-log row loading and operation-id metadata pairing.
//!
//! This module owns the operation-specific rendered row contract: rendered
//! `jj operation log` rows are preserved as styled lines, while operation ids
//! come from a narrow metadata template and are paired only when row counts
//! still match. Shared graph-line classification and row-metadata drift policy
//! stay in `rows` because revision rows use the same mechanics.

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;

use crate::jj::{ColorMode, ViewSpec, run_jj, run_jj_template_lines};
use crate::rendered_rows::{RowMetadata, first_content_char, is_standalone_graph_line, line_text};

pub const OPERATION_ID_TEMPLATE: &str = "self.id() ++ \"\\n\"";

/// One selectable operation item parsed from rendered operation-log output.
#[derive(Clone, Debug)]
pub struct OperationLogItem {
    /// Preserved rendered lines for one selectable operation-log row.
    lines: Vec<Line<'static>>,
    /// Exact operation id paired from metadata when row counts still match.
    operation_id: Option<String>,
}

impl OperationLogItem {
    /// Builds one rendered operation-log item and its paired exact operation id.
    pub fn new(lines: Vec<Line<'static>>, operation_id: Option<String>) -> Self {
        Self {
            lines,
            operation_id,
        }
    }

    /// Returns the preserved rendered lines for this operation row.
    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    /// Returns the number of rendered lines in this operation row.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the exact operation id paired with this row, if metadata was valid.
    pub fn operation_id(&self) -> Option<&str> {
        self.operation_id.as_deref()
    }

    /// Returns plain row text for copy and search surfaces.
    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Loads rendered operation-log rows and pairs them with exact metadata ids.
pub fn load_operation_log_entries(spec: &ViewSpec) -> Result<Vec<OperationLogItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;
    let operation_ids = run_operation_log_ids(spec)?;
    Ok(group_operation_log_lines(lines, operation_ids))
}

/// Loads exact operation ids through a narrow metadata template.
fn run_operation_log_ids(spec: &ViewSpec) -> Result<RowMetadata<String>> {
    Ok(parse_operation_id_lines(run_jj_template_lines(
        spec,
        OPERATION_ID_TEMPLATE,
        true,
    )?))
}

/// Groups rendered lines into selectable rows and attaches ids only when metadata still matches.
fn group_operation_log_lines(
    lines: Vec<Line<'static>>,
    operation_ids: RowMetadata<String>,
) -> Vec<OperationLogItem> {
    let operation_row_count = lines
        .iter()
        .filter(|line| starts_operation_log_item(line))
        .count();
    let mut operation_ids = operation_ids
        .into_rows_matching(operation_row_count)
        .map(Vec::into_iter);

    let mut items = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_operation_id = None;

    for line in lines {
        let starts_item = starts_operation_log_item(&line);
        let standalone_graph_line = is_standalone_graph_line(&line);

        if (starts_item || standalone_graph_line) && !current_lines.is_empty() {
            items.push(OperationLogItem::new(current_lines, current_operation_id));
            current_lines = Vec::new();
            current_operation_id = None;
        }

        if starts_item {
            current_operation_id = operation_ids.as_mut().and_then(Iterator::next);
        }
        current_lines.push(line);
    }

    if !current_lines.is_empty() {
        items.push(OperationLogItem::new(current_lines, current_operation_id));
    }

    items
}

/// Parses one metadata line into an exact operation id only when the shape is exact.
fn parse_operation_id_line(line: &str) -> Option<String> {
    let mut tokens = line.split_whitespace();
    let operation_id = tokens.next()?;

    if tokens.next().is_some() || line != operation_id || !is_operation_id(operation_id) {
        return None;
    }

    Some(operation_id.to_owned())
}

/// Parses metadata lines and fails closed on the first malformed row.
fn parse_operation_id_lines(lines: Vec<String>) -> RowMetadata<String> {
    let mut operation_ids = Vec::new();
    for line in lines {
        let Some(operation_id) = parse_operation_id_line(&line) else {
            return RowMetadata::Drifted;
        };
        operation_ids.push(operation_id);
    }
    RowMetadata::Valid(operation_ids)
}

/// Returns whether the token is one full hexadecimal operation id.
fn is_operation_id(token: &str) -> bool {
    token.len() == 128 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Returns whether a rendered line starts a new operation-log item.
fn starts_operation_log_item(line: &Line<'_>) -> bool {
    first_content_char(&line_text(line)).is_some_and(|character| matches!(character, '@' | '○'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_operation_log_rows_and_preserves_styles() {
        let text =
            b"\x1b[1m@\x1b[0m  current\n\xE2\x94\x82  args: jj describe\n\xE2\x97\x8B  previous\n"
                .to_vec();
        let lines = text.into_text().unwrap().lines;
        let operation_ids = operation_id_rows(vec![operation_id('a'), operation_id('b')]);

        let items = group_operation_log_lines(lines, operation_ids);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].line_count(), 2);
        assert_eq!(items[0].operation_id(), Some(operation_id('a').as_str()));
        assert_eq!(items[0].lines[0].spans[0].content, "@");
        assert_eq!(items[1].operation_id(), Some(operation_id('b').as_str()));
    }

    #[test]
    fn operation_log_rows_fail_closed_on_malformed_id_metadata_line() {
        let lines = b"@  current\n\xE2\x97\x8B  previous\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let operation_ids =
            parse_operation_id_lines(vec![operation_id('a'), "not-an-operation".to_owned()]);

        let items = group_operation_log_lines(lines, operation_ids);

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.operation_id().is_none()));
        assert_eq!(items[0].row_text(), "@  current");
        assert_eq!(items[1].row_text(), "○  previous");
    }

    #[test]
    fn operation_log_rows_fail_closed_when_metadata_line_is_missing_id() {
        let lines = b"@  current\n\xE2\x97\x8B  previous\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let operation_ids = parse_operation_id_lines(vec![operation_id('a'), String::new()]);

        let items = group_operation_log_lines(lines, operation_ids);

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.operation_id().is_none()));
        assert_eq!(items[0].row_text(), "@  current");
        assert_eq!(items[1].row_text(), "○  previous");
    }

    #[test]
    fn operation_log_rows_fail_closed_on_extra_metadata() {
        let lines = b"@  current\n\xE2\x94\x82  args: jj describe\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let operation_ids = operation_id_rows(vec![operation_id('a'), operation_id('b')]);

        let items = group_operation_log_lines(lines, operation_ids);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].operation_id(), None);
        assert_eq!(items[0].row_text(), "@  current\n│  args: jj describe");
    }

    #[test]
    fn operation_log_rows_fail_closed_on_row_count_mismatch() {
        let lines = b"@  current\n\xE2\x97\x8B  previous\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let operation_ids = operation_id_rows(vec![operation_id('a')]);

        let items = group_operation_log_lines(lines, operation_ids);

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.operation_id().is_none()));
        assert_eq!(items[0].row_text(), "@  current");
        assert_eq!(items[1].row_text(), "○  previous");
    }

    #[test]
    fn parses_operation_id_lines_exact_shape() {
        let operation_id = operation_id('a');

        assert_eq!(
            parse_operation_id_line(&operation_id),
            Some(operation_id.clone())
        );
        assert_eq!(
            parse_operation_id_line(&format!("junk {operation_id}")),
            None
        );
        assert_eq!(parse_operation_id_line("not-an-operation"), None);
    }

    fn operation_id_rows(rows: Vec<String>) -> RowMetadata<String> {
        RowMetadata::Valid(rows)
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }
}
