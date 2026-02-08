//! Wrapper views for operation-related commands.

use super::common::{capitalize_word, plural_suffix};
use super::strip_ansi;

/// Render `operation show` output with summary and section spacing.
pub(crate) fn render_operation_show_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut detail_lines: Vec<String> = Vec::new();
    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if line.ends_with(':')
            && matches!(detail_lines.last(), Some(previous) if !previous.is_empty())
        {
            detail_lines.push(String::new());
        }

        detail_lines.push(line);
    }

    while matches!(detail_lines.last(), Some(previous) if previous.is_empty()) {
        detail_lines.pop();
    }

    let operation_id = detail_lines
        .first()
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or("@");

    let mut rendered = vec![
        "Operation Details".to_string(),
        "=================".to_string(),
        String::new(),
        format!("Summary: operation {operation_id}"),
        String::new(),
    ];
    rendered.extend(detail_lines);
    rendered.push(String::new());
    rendered.push("Tip: operation restore/revert remain confirm-gated with previews".to_string());
    rendered
}

/// Render mutation wrappers for `operation restore`/`operation revert`.
pub(crate) fn render_operation_mutation_view(subcommand: &str, lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = operation_mutation_summary(subcommand, &detail_lines);

    let mut rendered = vec![
        format!("Operation {}", capitalize_word(subcommand)),
        "=================".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `operation log` and `status` to validate repository state".to_string());
    rendered
}

/// Build a concise mutation summary using known signal lines when available.
pub(crate) fn operation_mutation_summary(subcommand: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: operation {subcommand} completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_operation_mutation_signal(subcommand, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

/// Return whether line matches a high-signal mutation confirmation.
pub(crate) fn is_operation_mutation_signal(subcommand: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match subcommand {
        "restore" => trimmed.starts_with("Restored to operation "),
        "revert" => trimmed.starts_with("Reverted operation "),
        _ => {
            trimmed.starts_with("Restored to operation ")
                || trimmed.starts_with("Reverted operation ")
        }
    }
}

/// Render `operation diff` output with changed-entry summary.
pub(crate) fn render_operation_diff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut detail_lines: Vec<String> = Vec::new();
    let mut commit_count = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if is_operation_entry_header(&line) {
            commit_count += 1;
        }

        if line.ends_with(':')
            && matches!(detail_lines.last(), Some(previous) if !previous.is_empty())
        {
            detail_lines.push(String::new());
        }

        detail_lines.push(line);
    }

    while matches!(detail_lines.last(), Some(previous) if previous.is_empty()) {
        detail_lines.pop();
    }

    let summary = if commit_count == 0 {
        "Summary: operation delta shown".to_string()
    } else {
        format!(
            "Summary: {commit_count} changed commit entr{} shown",
            if commit_count == 1 { "y" } else { "ies" }
        )
    };

    let mut rendered = vec![
        "Operation Diff".to_string(),
        "==============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(detail_lines);
    rendered.push(String::new());
    rendered
        .push("Tip: use operation show/restore/revert for deeper operation workflows".to_string());
    rendered
}

/// Render `operation log` output with operation counting and spacing.
pub(crate) fn render_operation_log_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut operation_lines: Vec<String> = Vec::new();
    let mut operation_count = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if is_operation_entry_header(&line) {
            if matches!(operation_lines.last(), Some(previous) if !previous.is_empty()) {
                operation_lines.push(String::new());
            }
            operation_count += 1;
        }

        operation_lines.push(line);
    }

    while matches!(operation_lines.last(), Some(previous) if previous.is_empty()) {
        operation_lines.pop();
    }

    let mut rendered = vec![
        "Operation Log".to_string(),
        "=============".to_string(),
        String::new(),
        format!(
            "Summary: {operation_count} operation entr{} shown",
            if operation_count == 1 { "y" } else { "ies" }
        ),
        String::new(),
    ];
    rendered.extend(operation_lines);
    rendered.push(String::new());
    rendered.push("Tip: restore/revert operations stay confirm-gated with previews".to_string());
    rendered
}

/// Return whether a line starts a new operation entry.
pub(crate) fn is_operation_entry_header(line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim_start();
    trimmed.starts_with('@') || trimmed.starts_with('○')
}
