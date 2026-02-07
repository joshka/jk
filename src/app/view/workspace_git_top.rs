use super::common::{capitalize_word, plural_suffix};
use super::strip_ansi;

pub(crate) fn render_workspace_list_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut workspace_lines = Vec::new();
    let mut workspace_count = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if line.contains(':') {
            workspace_count += 1;
        }
        workspace_lines.push(line);
    }

    let mut rendered = vec![
        "Workspace List".to_string(),
        "==============".to_string(),
        String::new(),
        format!(
            "Summary: {workspace_count} workspace{} listed",
            plural_suffix(workspace_count)
        ),
        String::new(),
    ];
    rendered.extend(workspace_lines);
    rendered.push(String::new());
    rendered
        .push("Tip: use `workspace add/forget/rename` flows from normal mode or `:`".to_string());
    rendered
}

pub(crate) fn render_git_fetch_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = git_fetch_summary(&detail_lines);

    let mut rendered = vec![
        "Git Fetch".to_string(),
        "=========".to_string(),
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
    rendered.push("Tip: run `log` or `status` to inspect fetched changes".to_string());
    rendered
}

pub(crate) fn render_git_push_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = git_push_summary(&detail_lines);

    let mut rendered = vec![
        "Git Push".to_string(),
        "========".to_string(),
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
        .push("Tip: push stays confirm-gated with a dry-run preview when available".to_string());
    rendered
}

pub(crate) fn git_fetch_summary(detail_lines: &[String]) -> String {
    if detail_lines
        .iter()
        .any(|line| strip_ansi(line).contains("Nothing changed"))
    {
        return "Summary: no remote updates fetched".to_string();
    }
    if detail_lines.is_empty() {
        return "Summary: fetch completed with no output".to_string();
    }
    if let Some(signal) = detail_lines.iter().find(|line| is_git_fetch_signal(line)) {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_git_fetch_signal(line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    trimmed.starts_with("Fetched ")
        || trimmed.starts_with("From ")
        || trimmed.starts_with("Updated bookmark ")
}

pub(crate) fn git_push_summary(detail_lines: &[String]) -> String {
    if detail_lines
        .iter()
        .any(|line| strip_ansi(line).contains("Nothing changed"))
    {
        return "Summary: no bookmark updates pushed".to_string();
    }
    if detail_lines.is_empty() {
        return "Summary: push completed with no output".to_string();
    }
    if let Some(signal) = detail_lines.iter().find(|line| is_git_push_signal(line)) {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_git_push_signal(line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    trimmed.starts_with("Pushed bookmark ") || trimmed.starts_with("Pushed ")
}

pub(crate) fn render_top_level_mutation_view(
    command_name: &str,
    lines: Vec<String>,
) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = top_level_mutation_summary(command_name, &detail_lines);
    let title = format!("{} Result", capitalize_word(command_name));
    let mut rendered = vec![
        title.clone(),
        "=".repeat(title.len()),
        String::new(),
        summary,
    ];
    rendered.push(String::new());

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push(top_level_mutation_tip(command_name).to_string());
    rendered
}

pub(crate) fn top_level_mutation_summary(command_name: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: `{command_name}` completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_top_level_mutation_signal(command_name, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_top_level_mutation_signal(command_name: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match command_name {
        "new" | "describe" | "commit" | "metaedit" | "edit" | "next" | "prev" => {
            trimmed.starts_with("Working copy now at:")
                || trimmed.starts_with("Working copy  (@) :")
                || trimmed.starts_with("Rebased ")
        }
        "undo" => trimmed.starts_with("Undid operation"),
        "redo" => trimmed.starts_with("Redid operation"),
        "rebase" | "squash" | "split" | "simplify-parents" | "diffedit" => {
            trimmed.starts_with("Rebased ")
        }
        "fix" => trimmed.starts_with("Fixed ") || trimmed.starts_with("Rebased "),
        "abandon" => trimmed.starts_with("Abandoned "),
        "restore" => trimmed.starts_with("Restored "),
        "revert" => trimmed.starts_with("Reverted "),
        "absorb" => trimmed.starts_with("Absorbed "),
        "duplicate" => trimmed.starts_with("Duplicated "),
        "parallelize" => trimmed.starts_with("Parallelized "),
        _ => false,
    }
}

pub(crate) fn top_level_mutation_tip(command_name: &str) -> &'static str {
    match command_name {
        "new" | "describe" | "commit" | "metaedit" => {
            "Tip: run `show` or `log` to inspect the updated revision"
        }
        "edit" | "next" | "prev" => "Tip: run `show` or `diff` to inspect the selected revision",
        "undo" | "redo" => "Tip: run `operation log` to inspect operation history",
        "rebase" | "squash" | "split" | "simplify-parents" | "diffedit" | "fix" | "abandon"
        | "restore" | "revert" | "absorb" | "duplicate" | "parallelize" => {
            "Tip: run `log`, `status`, or `diff` to verify the resulting history"
        }
        _ => "Tip: run `log` and `status` to verify repository state",
    }
}

pub(crate) fn render_bookmark_mutation_view(
    subcommand: Option<&str>,
    lines: Vec<String>,
) -> Vec<String> {
    let subcommand = subcommand.unwrap_or("update");
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = bookmark_mutation_summary(subcommand, &detail_lines);

    let mut rendered = vec![
        format!("Bookmark {}", capitalize_word(subcommand)),
        "================".to_string(),
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
    rendered.push("Tip: run `bookmark list` to verify bookmark state".to_string());
    rendered
}

pub(crate) fn bookmark_mutation_summary(subcommand: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: bookmark {subcommand} completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_bookmark_mutation_signal(subcommand, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_bookmark_mutation_signal(subcommand: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match subcommand {
        "create" => trimmed.starts_with("Created bookmark "),
        "set" | "move" => trimmed.starts_with("Moved bookmark "),
        "track" => trimmed.starts_with("Started tracking bookmark "),
        "untrack" => trimmed.starts_with("Stopped tracking bookmark "),
        "delete" => trimmed.starts_with("Deleted bookmark "),
        "forget" => trimmed.starts_with("Forgot bookmark "),
        "rename" => trimmed.starts_with("Renamed bookmark "),
        _ => {
            trimmed.starts_with("Created bookmark ")
                || trimmed.starts_with("Moved bookmark ")
                || trimmed.starts_with("Started tracking bookmark ")
                || trimmed.starts_with("Stopped tracking bookmark ")
                || trimmed.starts_with("Deleted bookmark ")
                || trimmed.starts_with("Forgot bookmark ")
                || trimmed.starts_with("Renamed bookmark ")
        }
    }
}

pub(crate) fn render_bookmark_list_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Bookmark List".to_string(),
        "=============".to_string(),
        String::new(),
    ];
    rendered.extend(lines);
    rendered.push(String::new());
    rendered.push("Tip: use `bookmark set/move/track` flows from normal mode or `:`".to_string());
    rendered
}

pub(crate) fn render_workspace_mutation_view(
    subcommand: Option<&str>,
    lines: Vec<String>,
) -> Vec<String> {
    let subcommand = subcommand.unwrap_or("update");
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = workspace_mutation_summary(subcommand, &detail_lines);

    let mut rendered = vec![
        format!("Workspace {}", capitalize_word(subcommand)),
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
    rendered.push("Tip: run `workspace list` to inspect workspace state".to_string());
    rendered
}

pub(crate) fn workspace_mutation_summary(subcommand: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: workspace {subcommand} completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_workspace_mutation_signal(subcommand, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_workspace_mutation_signal(subcommand: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match subcommand {
        "add" => trimmed.starts_with("Created workspace "),
        "forget" => trimmed.starts_with("Forgot workspace "),
        "rename" => trimmed.starts_with("Renamed workspace "),
        "update-stale" => trimmed.starts_with("Updated ") && trimmed.contains("stale workspace"),
        _ => {
            trimmed.starts_with("Created workspace ")
                || trimmed.starts_with("Forgot workspace ")
                || trimmed.starts_with("Renamed workspace ")
                || (trimmed.starts_with("Updated ") && trimmed.contains("stale workspace"))
        }
    }
}
