//! Wrapper views for root/version/resolve command outputs.

use super::common::plural_suffix;
use super::strip_ansi;

/// Render workspace root path output.
pub(crate) fn render_root_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Workspace Root".to_string(),
        "==============".to_string(),
        String::new(),
    ];

    for line in lines {
        let display_line = line.trim_end().to_string();
        let stripped = strip_ansi(&display_line);
        let trimmed = stripped.trim();
        let display_trimmed = display_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        rendered.push(display_trimmed.to_string());
    }

    rendered.push(String::new());
    rendered.push("Tip: use jjrt/jk root to inspect current workspace path".to_string());
    rendered
}

/// Render version output with first-line summary.
pub(crate) fn render_version_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = if let Some(line) = detail_lines.first() {
        format!("Summary: {}", line.trim())
    } else {
        "Summary: version command completed with no output".to_string()
    };

    let mut rendered = vec![
        "Version".to_string(),
        "=======".to_string(),
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
    rendered.push("Tip: run `version` after upgrades to confirm toolchain state".to_string());
    rendered
}

/// Render resolve list output with conflict-count summary.
pub(crate) fn render_resolve_list_view(lines: Vec<String>) -> Vec<String> {
    let mut body_lines = Vec::new();
    let mut conflict_count = 0usize;
    let mut saw_no_conflicts = false;

    for line in lines {
        let display_line = line.trim_end().to_string();
        let stripped = strip_ansi(&display_line);
        let trimmed = stripped.trim();
        let display_trimmed = display_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "(no output)" {
            continue;
        }

        if trimmed.contains("No conflicts found") {
            saw_no_conflicts = true;
        } else if !trimmed.starts_with("Error:") && !trimmed.starts_with("Hint:") {
            conflict_count += 1;
        }

        body_lines.push(display_trimmed.to_string());
    }

    let summary = if saw_no_conflicts || conflict_count == 0 {
        "Summary: no conflicts listed".to_string()
    } else {
        format!(
            "Summary: {conflict_count} conflicted path{} listed",
            plural_suffix(conflict_count)
        )
    };

    let mut rendered = vec![
        "Resolve List".to_string(),
        "============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if body_lines.is_empty() {
        rendered.push("(no conflicts found)".to_string());
    } else {
        rendered.extend(body_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: run `resolve <path>` to open a merge tool for specific conflicts".to_string());
    rendered
}

/// Render resolve action output with signal-line summary when available.
pub(crate) fn render_resolve_action_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let detail_count = detail_lines.len();

    let summary = if let Some(signal) = detail_lines.iter().find(|line| {
        let stripped = strip_ansi(line);
        stripped.trim().starts_with("Resolved ")
    }) {
        format!("Summary: {}", strip_ansi(signal).trim())
    } else if detail_count == 0 {
        "Summary: resolve command completed with no output".to_string()
    } else {
        format!(
            "Summary: {detail_count} output line{}",
            plural_suffix(detail_count)
        )
    };

    let mut rendered = vec![
        "Resolve".to_string(),
        "=======".to_string(),
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
    rendered
        .push("Tip: use `resolve -l` to verify remaining conflicts after resolution".to_string());
    rendered
}

/// Return whether command tokens request resolve list mode.
pub(crate) fn has_resolve_list_flag(command: &[String]) -> bool {
    command
        .iter()
        .any(|token| token == "-l" || token == "--list")
}
