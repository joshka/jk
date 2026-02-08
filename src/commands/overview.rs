//! In-app command registry rendering.

use super::spec::{CommandSpec, TOP_LEVEL_SPECS};

/// Render command registry lines without filtering.
pub fn command_overview_lines() -> Vec<String> {
    command_overview_lines_with_query_and_recent(None, &[])
}

/// Render command registry lines including recent intents.
pub fn command_overview_lines_with_recent(recent_intents: &[String]) -> Vec<String> {
    command_overview_lines_with_query_and_recent(None, recent_intents)
}

/// Render command registry lines filtered by command/alias query.
pub fn command_overview_lines_with_query(query: Option<&str>) -> Vec<String> {
    command_overview_lines_with_query_and_recent(query, &[])
}

/// Render command registry lines filtered by query and augmented with recent intents.
pub fn command_overview_lines_with_query_and_recent(
    query: Option<&str>,
    recent_intents: &[String],
) -> Vec<String> {
    let filter = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    let mut lines = vec![
        "jk command registry".to_string(),
        String::new(),
        "common flows (grouped, condensed)".to_string(),
        String::new(),
    ];

    push_grouped_help_sections(&mut lines, filter.as_deref());
    lines.push(String::new());
    push_registry_section(&mut lines, filter.as_deref());

    if filter.is_none() {
        lines.push(String::new());
        lines.push("aliases: b ci desc op st gf gp rbm rbt jjgf jjgp jjrbm jjst jjl".to_string());
        lines.push("tips: :aliases shows mappings, :keys shows active keybinds".to_string());
        lines.push(
            "defaults: bookmark/file/tag/workspace -> list, resolve -> resolve -l, operation -> log"
                .to_string(),
        );
    }
    if filter.is_none() && !recent_intents.is_empty() {
        lines.push(String::new());
        lines.push(format!("recent intents: {}", recent_intents.join(", ")));
    }

    lines
}

/// Render workflow-scoped help lines for high-frequency intent families.
pub fn command_workflow_lines(workflow: &str, recent_intents: &[String]) -> Option<Vec<String>> {
    let workflow = workflow.trim().to_ascii_lowercase();
    let (title, narrative, entries): (&str, &str, Vec<(&str, &str, &str)>) = match workflow.as_str()
    {
        "inspect" => (
            "inspect",
            "Scan history, inspect details, and return to context quickly.",
            vec![
                ("l", "open log home", "start"),
                ("j/k", "move by revision item", "select"),
                ("Enter", "show selected revision", "inspect"),
                ("d", "diff selected revision", "inspect"),
                ("Left", "return to previous screen", "continue"),
            ],
        ),
        "rewrite" => (
            "rewrite",
            "Apply safe rewrite operations with explicit confirmation where needed.",
            vec![
                ("D", "describe selected revision", "edit metadata"),
                ("B/S/X", "rebase, squash, split", "rewrite"),
                ("a", "abandon selected revision", "drop"),
                ("y / n", "accept or reject confirm", "safety"),
                ("o", "inspect operation log", "audit"),
            ],
        ),
        "sync" => (
            "sync",
            "Verify state first, then fetch/push with guarded flows.",
            vec![
                ("s", "check working copy status", "verify"),
                ("F", "run fetch prompt flow", "sync"),
                ("P", "run push prompt flow", "sync"),
                ("d", "review selected diff before push", "verify"),
                ("Left", "return to prior context", "continue"),
            ],
        ),
        "recover" => (
            "recover",
            "Use operation history and undo/redo loop to recover confidently.",
            vec![
                ("o", "open operation log", "inspect"),
                ("Enter", "show operation details", "inspect"),
                ("u / U", "undo or redo latest op", "recover"),
                ("status", "verify working copy state", "verify"),
                ("log", "confirm resulting history", "verify"),
            ],
        ),
        _ => return None,
    };

    let mut lines = vec![
        format!("jk workflow help: {title}"),
        String::new(),
        format!("story: {narrative}"),
        String::new(),
        format!("{:<10} {:<36} {}", "key", "action", "why"),
        "-".repeat(60),
    ];

    let compact: Vec<String> = entries
        .iter()
        .map(|(key, action, why)| format!("{:<10} {:<36} {}", key, action, why))
        .collect();
    lines.extend(compact_two_column(&compact, 60));
    lines.push(String::new());
    lines.push("docs: docs/workflows.md".to_string());
    if !recent_intents.is_empty() {
        lines.push(format!("recent intents: {}", recent_intents.join(", ")));
    }

    Some(lines)
}

/// Add grouped day-one help sections with compact two-column rendering.
fn push_grouped_help_sections(lines: &mut Vec<String>, filter: Option<&str>) {
    let mut matched_any = false;
    matched_any |= push_help_group(
        lines,
        "Navigation",
        &[
            ("j/k, Up/Down", "move by item", "selection"),
            ("PgUp/PgDn", "page by viewport", "navigation"),
            ("Ctrl+u/d", "page up/down", "navigation"),
            ("g/G, Home/End", "jump top/bottom", "navigation"),
            ("Left/Right", "screen history", "back/forward"),
            ("Ctrl+o/i", "back/forward", "history"),
            (":", "run exact command", "jj-style"),
        ],
        filter,
    );
    matched_any |= push_help_group(
        lines,
        "Views",
        &[
            ("l", "log history", "home"),
            ("s", "status working copy", "triage"),
            ("Enter", "show selected revision", "inspect"),
            ("d", "diff selected revision", "inspect"),
            ("o", "operation log", "jj op log"),
            ("L", "bookmark list", "jj bookmark list"),
            ("v", "resolve list", "jj resolve -l"),
            ("f", "file list", "jj file list"),
            ("t", "tag list", "jj tag list"),
            ("w", "workspace root", "jj root"),
            ("?", "commands help", "discover"),
            ("K", "keys keymap", "discover"),
        ],
        filter,
    );
    matched_any |= push_help_group(
        lines,
        "Actions",
        &[
            ("n", "new change", "opens prompt"),
            ("c", "commit", "opens prompt"),
            ("D", "describe selected", "opens prompt"),
            ("b", "bookmark set", "prompt + confirm"),
            ("F / P", "git fetch / push", "prompt; push confirms"),
            ("B/S/X", "rebase/squash/split", "confirm"),
            ("a", "abandon selected", "confirm"),
            ("u/U", "undo / redo", "confirm"),
            ("p", "toggle log patch", "view detail"),
        ],
        filter,
    );
    matched_any |= push_help_group(
        lines,
        "Safety",
        &[
            ("y", "accept confirmation", "run command"),
            ("n / Esc", "reject or cancel", "safe exit"),
            ("Esc", "cancel command mode", "safe exit"),
            (
                "high-risk actions",
                "always ask confirmation",
                "before mutate",
            ),
        ],
        filter,
    );

    if !matched_any {
        lines.push("(no matching flows)".to_string());
    }
}

/// Add one help section and return whether any entries matched.
fn push_help_group(
    lines: &mut Vec<String>,
    title: &str,
    entries: &[(&str, &str, &str)],
    filter: Option<&str>,
) -> bool {
    let mut compact = Vec::new();
    for (key, flow, note) in entries {
        if !matches_filter(filter, &[title, key, flow, note]) {
            continue;
        }
        compact.push(format!("{:<12} {:<24} {}", key, flow, note));
    }

    if compact.is_empty() {
        return false;
    }

    lines.push(format!("{title}:"));
    lines.push(format!("{:<12} {:<24} {}", "key", "action", "notes"));
    lines.push("-".repeat(48));
    lines.extend(compact_two_column(&compact, 48));
    lines.push(String::new());
    true
}

/// Add full top-level command registry with common commands grouped first.
fn push_registry_section(lines: &mut Vec<String>, filter: Option<&str>) {
    lines.push("all top-level commands (common first, condensed)".to_string());

    let mut specs: Vec<CommandSpec> = TOP_LEVEL_SPECS.to_vec();
    specs.sort_by_key(|spec| (command_priority(spec.name), spec.name));

    let mut entries = Vec::new();
    for spec in specs {
        let display_name = command_display_name(spec);
        let behavior = command_behavior_label(spec);
        if !matches_filter(filter, &[&display_name, behavior]) {
            continue;
        }

        entries.push(format!("{:<18} {}", display_name, behavior));
    }

    for (name, behavior) in [
        ("aliases (local)", "local view"),
        ("keys (local)", "local view"),
        ("keymap (local)", "local view"),
    ] {
        if !matches_filter(filter, &[name, behavior]) {
            continue;
        }
        entries.push(format!("{:<18} {}", name, behavior));
    }

    if entries.is_empty() {
        lines.push("(no matching commands)".to_string());
        return;
    }

    lines.push(format!("{:<18} {}", "command", "behavior"));
    lines.push("-".repeat(52));
    lines.extend(compact_two_column(&entries, 52));
}

/// Convert internal execution/safety metadata to plain user-facing behavior labels.
fn command_behavior_label(spec: CommandSpec) -> &'static str {
    use super::spec::{ExecutionMode, SafetyTier};

    match (spec.mode, spec.tier) {
        (ExecutionMode::Guided, SafetyTier::C) => "opens prompt + asks confirmation",
        (ExecutionMode::Guided, _) => "opens prompt",
        (ExecutionMode::Passthrough, SafetyTier::C) => "runs as jj + asks confirmation",
        (ExecutionMode::Passthrough, _) => "runs as jj",
        (ExecutionMode::Native, SafetyTier::C) => "runs now + asks confirmation",
        (ExecutionMode::Native, _) => "runs now",
    }
}

/// Return whether any candidate text matches optional filter.
fn matches_filter(filter: Option<&str>, values: &[&str]) -> bool {
    match filter {
        Some(filter) => values
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(filter)),
        None => true,
    }
}

/// Rank commonly used commands ahead of long-tail commands.
fn command_priority(name: &str) -> u8 {
    match name {
        "log" | "status" | "show" | "diff" | "operation" | "bookmark" | "file" | "resolve"
        | "tag" | "root" | "git" | "help" | "new" | "commit" | "describe" | "rebase" | "squash"
        | "split" | "abandon" | "undo" | "redo" => 0,
        _ => 1,
    }
}

/// Build display name with optional top-level alias annotation.
fn command_display_name(spec: CommandSpec) -> String {
    match top_level_default_alias(spec.name) {
        Some(alias) => format!("{} ({alias})", spec.name),
        None => spec.name.to_string(),
    }
}

/// Return top-level `jj` aliases that should be surfaced in registry output.
fn top_level_default_alias(name: &str) -> Option<&'static str> {
    match name {
        "bookmark" => Some("b"),
        "commit" => Some("ci"),
        "describe" => Some("desc"),
        "operation" => Some("op"),
        "status" => Some("st"),
        _ => None,
    }
}

/// Pack one-column entry lines into compact two-column output.
fn compact_two_column(entries: &[String], min_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let first_column_width = entries
        .iter()
        .step_by(2)
        .map(|entry| entry.len())
        .max()
        .unwrap_or(min_width)
        .max(min_width);

    for pair in entries.chunks(2) {
        if pair.len() == 1 {
            lines.push(pair[0].clone());
            continue;
        }

        lines.push(format!(
            "{:<width$}  {}",
            pair[0],
            pair[1],
            width = first_column_width
        ));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::{
        command_overview_lines, command_overview_lines_with_query,
        command_overview_lines_with_recent, command_workflow_lines,
    };

    #[test]
    fn renders_overview_lines_with_headers() {
        let lines = command_overview_lines();
        assert_eq!(lines.first(), Some(&"jk command registry".to_string()));
        assert!(lines.iter().any(|line| line.contains("common flows")));
        assert!(lines.iter().any(|line| line.contains("Navigation:")));
        assert!(lines.iter().any(|line| line.contains("Safety:")));
        assert!(lines.iter().any(|line| line.contains("log history")));
        assert!(lines.iter().any(|line| line.contains("status (st)")));
        assert!(lines.iter().any(|line| line.contains("bookmark (b)")));
        assert!(lines.iter().any(|line| line.contains("workspace")));
        assert!(lines.iter().any(|line| line.contains("aliases: b ci desc")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains(":aliases shows mappings"))
        );
        assert!(lines.iter().any(|line| line.contains("aliases (local)")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("defaults: bookmark/file/tag"))
        );
    }

    #[test]
    fn filters_overview_lines_by_query() {
        let lines = command_overview_lines_with_query(Some("work"));
        assert!(lines.iter().any(|line| line.contains("workspace")));
        assert!(!lines.iter().any(|line| line.contains("rebase")));
    }

    #[test]
    fn filters_overview_lines_by_default_alias() {
        let lines = command_overview_lines_with_query(Some("ci"));
        assert!(lines.iter().any(|line| line.contains("commit (ci)")));
        assert!(!lines.iter().any(|line| line.contains("workspace")));
    }

    #[test]
    fn filters_overview_lines_for_local_views() {
        let lines = command_overview_lines_with_query(Some("keys"));
        assert!(lines.iter().any(|line| line.contains("keys (local)")));
        assert!(!lines.iter().any(|line| line.contains("workspace")));
    }

    #[test]
    fn snapshot_renders_default_overview_spacing() {
        insta::assert_snapshot!(command_overview_lines().join("\n"));
    }

    #[test]
    fn snapshot_renders_filtered_overview_spacing() {
        insta::assert_snapshot!(command_overview_lines_with_query(Some("prompt")).join("\n"));
    }

    #[test]
    fn renders_recent_intents_in_unfiltered_overview() {
        let lines = command_overview_lines_with_recent(&[
            ":status".to_string(),
            ":log".to_string(),
            ":git push".to_string(),
        ]);
        assert!(
            lines
                .iter()
                .any(|line| line.contains("recent intents: :status, :log, :git push"))
        );
    }

    #[test]
    fn renders_workflow_help_presets() {
        let inspect = command_workflow_lines("inspect", &[":status".to_string()])
            .expect("inspect workflow should render");
        assert_eq!(
            inspect.first(),
            Some(&"jk workflow help: inspect".to_string())
        );
        assert!(
            inspect
                .iter()
                .any(|line| line.contains("recent intents: :status"))
        );
    }

    #[test]
    fn snapshot_renders_inspect_workflow_help() {
        let lines = command_workflow_lines("inspect", &[":status".to_string(), ":log".to_string()])
            .expect("inspect workflow should render");
        insta::assert_snapshot!(lines.join("\n"));
    }

    #[test]
    fn snapshot_renders_rewrite_workflow_help() {
        let lines = command_workflow_lines("rewrite", &[":rebase".to_string()])
            .expect("rewrite workflow should render");
        insta::assert_snapshot!(lines.join("\n"));
    }
}
