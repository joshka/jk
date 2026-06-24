use jk_tui::operation_log_view::{OperationLogRow, OperationLogSnapshot};

pub fn operation_log_snapshot(title: &str, rendered: &str) -> OperationLogSnapshot {
    let rendered_lines = rendered.lines().map(str::to_owned).collect::<Vec<_>>();
    let rows = operation_log_rows(&rendered_lines);
    OperationLogSnapshot::from_rendered(rows, rendered_lines).with_title(title)
}

fn operation_log_rows(rendered_lines: &[String]) -> Vec<OperationLogRow> {
    let mut rows = Vec::new();
    for (rendered_line, line) in rendered_lines.iter().enumerate() {
        let plain_line = strip_ansi(line);
        if let Some(row) = parse_operation_row(&plain_line, rendered_line) {
            rows.push(row);
        } else if let Some(title) = parse_operation_title_line(&plain_line)
            && let Some(row) = rows.last_mut()
            && row.title.trim().is_empty()
        {
            row.title = title.to_owned();
        }
    }
    rows
}

fn parse_operation_row(line: &str, rendered_line: usize) -> Option<OperationLogRow> {
    let mut fields = line.split_whitespace();
    let marker = fields.next()?;
    let operation_id = fields.next()?;
    if !is_operation_marker(marker) || !looks_like_operation_id(operation_id) {
        return None;
    }
    Some(
        OperationLogRow::new(
            operation_id,
            operation_id.chars().take(12).collect::<String>(),
            String::new(),
            marker == "@",
        )
        .with_rendered_line(rendered_line),
    )
}

fn parse_operation_title_line(line: &str) -> Option<&str> {
    line.strip_prefix("│  ")
        .map(str::trim)
        .filter(|line| !line.is_empty())
}

fn is_operation_marker(marker: &str) -> bool {
    matches!(marker, "@" | "○" | "◆" | "×" | "◉")
}

fn looks_like_operation_id(value: &str) -> bool {
    value.len() >= 8 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn strip_ansi(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(character);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_extracts_operation_rows_and_titles() {
        let rendered = "\
@ abcdef1234567890 user@example.test now
│  latest operation
○ 0123456789abcdef user@example.test earlier
│  previous operation
";

        let snapshot = operation_log_snapshot("jj op log", rendered);

        assert_eq!(snapshot.title(), "jj op log");
        assert_eq!(
            snapshot.rendered_lines(),
            [
                "@ abcdef1234567890 user@example.test now",
                "│  latest operation",
                "○ 0123456789abcdef user@example.test earlier",
                "│  previous operation",
            ]
        );
        assert_eq!(snapshot.rows().len(), 2);
        assert_eq!(snapshot.rows()[0].operation_id, "abcdef1234567890");
        assert_eq!(snapshot.rows()[0].display_id, "abcdef123456");
        assert_eq!(snapshot.rows()[0].title, "latest operation");
        assert!(snapshot.rows()[0].current);
        assert_eq!(snapshot.rows()[0].rendered_line, 0);
        assert_eq!(snapshot.rows()[1].operation_id, "0123456789abcdef");
        assert_eq!(snapshot.rows()[1].title, "previous operation");
        assert_eq!(snapshot.rows()[1].rendered_line, 2);
        assert!(!snapshot.rows()[1].current);
    }

    #[test]
    fn snapshot_ignores_ansi_sequences_before_parsing() {
        let rendered = "\u{1b}[32m@ abcdef1234567890 user@example.test now\u{1b}[0m\n\
│  colored current operation\n";

        let snapshot = operation_log_snapshot("jj op log", rendered);

        assert_eq!(snapshot.rows().len(), 1);
        assert_eq!(
            snapshot.rendered_lines(),
            [
                "\u{1b}[32m@ abcdef1234567890 user@example.test now\u{1b}[0m",
                "│  colored current operation",
            ]
        );
        assert_eq!(snapshot.rows()[0].operation_id, "abcdef1234567890");
        assert_eq!(snapshot.rows()[0].title, "colored current operation");
        assert!(snapshot.rows()[0].current);
    }
}
