//! In-app command registry rendering.

use super::spec::{CommandSpec, TOP_LEVEL_SPECS};

/// Render command registry lines without filtering.
pub fn command_overview_lines() -> Vec<String> {
    command_overview_lines_with_query(None)
}

/// Render command registry lines filtered by command/alias query.
pub fn command_overview_lines_with_query(query: Option<&str>) -> Vec<String> {
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

    lines
}

/// Add grouped day-one help sections with compact two-column rendering.
fn push_grouped_help_sections(lines: &mut Vec<String>, filter: Option<&str>) {
    let mut matched_any = false;
    matched_any |= push_help_group(
        lines,
        "Navigation",
        &[
            ("j/k, Up/Down", "move by item", "native", "A"),
            ("PgUp/PgDn", "page by viewport", "native", "A"),
            ("Ctrl+u/d", "page up/down", "native", "A"),
            ("g/G, Home/End", "jump top/bottom", "native", "A"),
            ("Left/Right", "screen history", "native", "A"),
            ("Ctrl+o/i", "back/forward", "native", "A"),
            (":", "run exact command", "native", "A"),
        ],
        filter,
    );
    matched_any |= push_help_group(
        lines,
        "Views",
        &[
            ("l", "log history", "native", "A"),
            ("s", "status working copy", "native", "A"),
            ("Enter", "show selected revision", "native", "A"),
            ("d", "diff selected revision", "native", "A"),
            ("o", "operation log", "guided", "B"),
            ("L", "bookmark list", "guided", "B"),
            ("v", "resolve list", "guided", "B"),
            ("f", "file list", "guided", "B"),
            ("t", "tag list", "guided", "B"),
            ("w", "workspace root", "passthrough", "A"),
            ("?", "commands help", "native", "A"),
            ("K", "keys keymap", "native", "A"),
        ],
        filter,
    );
    matched_any |= push_help_group(
        lines,
        "Actions",
        &[
            ("n", "new change", "guided", "B"),
            ("c", "commit", "guided", "B"),
            ("D", "describe selected", "guided", "B"),
            ("b", "bookmark set", "guided", "C"),
            ("F / P", "git fetch / push", "guided", "B/C"),
            ("B/S/X", "rebase/squash/split", "guided", "C"),
            ("a", "abandon selected", "guided", "C"),
            ("u/U", "undo / redo", "guided", "C"),
            ("p", "toggle log patch", "native", "A"),
        ],
        filter,
    );
    matched_any |= push_help_group(
        lines,
        "Safety",
        &[
            ("y", "accept confirm prompt", "native", "C"),
            ("n / Esc", "reject or cancel", "native", "A"),
            ("Esc", "cancel command mode", "native", "A"),
            ("Tier C", "explicit confirmation", "native", "C"),
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
    entries: &[(&str, &str, &str, &str)],
    filter: Option<&str>,
) -> bool {
    let mut compact = Vec::new();
    for (key, flow, mode, tier) in entries {
        if !matches_filter(filter, &[title, key, flow, mode, tier]) {
            continue;
        }
        compact.push(format!("{:<12} {:<22} {:<10} {}", key, flow, mode, tier));
    }

    if compact.is_empty() {
        return false;
    }

    lines.push(format!("{title}:"));
    lines.push(format!(
        "{:<12} {:<22} {:<10} {}",
        "key", "flow", "mode", "tier"
    ));
    lines.push("-".repeat(50));
    lines.extend(compact_two_column(&compact, 50));
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
        let mode = spec.mode.as_str();
        let tier = spec.tier.as_str();
        if !matches_filter(filter, &[&display_name, mode, tier]) {
            continue;
        }

        entries.push(format!("{:<18} {:<10} {}", display_name, mode, tier));
    }

    for (name, mode, tier) in [
        ("aliases (local)", "native", "A"),
        ("keys (local)", "native", "A"),
        ("keymap (local)", "native", "A"),
    ] {
        if !matches_filter(filter, &[name, mode, tier]) {
            continue;
        }
        entries.push(format!("{:<18} {:<10} {}", name, mode, tier));
    }

    if entries.is_empty() {
        lines.push("(no matching commands)".to_string());
        return;
    }

    lines.push(format!("{:<18} {:<10} {}", "command", "mode", "tier"));
    lines.push("-".repeat(32));
    lines.extend(compact_two_column(&entries, 40));
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
fn compact_two_column(entries: &[String], width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    for pair in entries.chunks(2) {
        if pair.len() == 1 {
            lines.push(pair[0].clone());
            continue;
        }

        lines.push(format!("{:<width$}  {}", pair[0], pair[1], width = width));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::{command_overview_lines, command_overview_lines_with_query};

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
        insta::assert_snapshot!(command_overview_lines_with_query(Some("flow")).join("\n"));
    }
}
