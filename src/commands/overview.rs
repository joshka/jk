use super::spec::{CommandSpec, TOP_LEVEL_SPECS};

pub fn command_overview_lines() -> Vec<String> {
    command_overview_lines_with_query(None)
}

pub fn command_overview_lines_with_query(query: Option<&str>) -> Vec<String> {
    let filter = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    let mut lines = Vec::with_capacity(TOP_LEVEL_SPECS.len() + 3);
    lines.push("jk command registry (jj top-level coverage)".to_string());
    lines.push(format!("{:<18} {:<12} {}", "command", "mode", "tier"));
    lines.push("-".repeat(40));

    for spec in TOP_LEVEL_SPECS {
        let display_name = command_display_name(spec);
        if let Some(filter) = &filter
            && !display_name.to_ascii_lowercase().contains(filter)
        {
            continue;
        }

        lines.push(format!(
            "{:<18} {:<12} {}",
            display_name,
            spec.mode.as_str(),
            spec.tier.as_str()
        ));
    }

    for (name, mode, tier) in [
        ("aliases (local)", "native", "A"),
        ("keys (local)", "native", "A"),
        ("keymap (local)", "native", "A"),
    ] {
        if let Some(filter) = &filter
            && !name.to_ascii_lowercase().contains(filter)
        {
            continue;
        }

        lines.push(format!("{:<18} {:<12} {}", name, mode, tier));
    }

    if filter.is_none() {
        lines.push(String::new());
        lines.push(
            "high-frequency aliases: b, ci, desc, op, st, gf, gp, rbm, rbt, jjgf, jjgp, jjrbm, \
             jjst, jjl"
                .to_string(),
        );
        lines.push("tips: use :aliases for mappings and :keys for active keybinds".to_string());
        lines.push(
            "group defaults: bookmark/file/tag/workspace -> list, resolve -> resolve -l, \
             operation -> log"
                .to_string(),
        );
    }

    lines
}

fn command_display_name(spec: CommandSpec) -> String {
    match top_level_default_alias(spec.name) {
        Some(alias) => format!("{} ({alias})", spec.name),
        None => spec.name.to_string(),
    }
}

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

#[cfg(test)]
mod tests {
    use super::{command_overview_lines, command_overview_lines_with_query};

    #[test]
    fn renders_overview_lines_with_headers() {
        let lines = command_overview_lines();
        assert_eq!(
            lines.first(),
            Some(&"jk command registry (jj top-level coverage)".to_string())
        );
        assert!(lines.iter().any(|line| line.starts_with("log")));
        assert!(lines.iter().any(|line| line.starts_with("bookmark (b)")));
        assert!(lines.iter().any(|line| line.starts_with("status (st)")));
        assert!(lines.iter().any(|line| line.starts_with("workspace")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("high-frequency aliases"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains(":aliases for mappings"))
        );
        assert!(lines.iter().any(|line| line.starts_with("aliases (local)")));
        assert!(lines.iter().any(|line| line.contains("group defaults:")));
    }

    #[test]
    fn filters_overview_lines_by_query() {
        let lines = command_overview_lines_with_query(Some("work"));
        assert!(lines.iter().any(|line| line.starts_with("workspace")));
        assert!(!lines.iter().any(|line| line.starts_with("rebase")));
    }

    #[test]
    fn filters_overview_lines_by_default_alias() {
        let lines = command_overview_lines_with_query(Some("ci"));
        assert!(lines.iter().any(|line| line.starts_with("commit (ci)")));
        assert!(!lines.iter().any(|line| line.starts_with("workspace")));
    }

    #[test]
    fn filters_overview_lines_for_local_views() {
        let lines = command_overview_lines_with_query(Some("keys"));
        assert!(lines.iter().any(|line| line.starts_with("keys (local)")));
        assert!(!lines.iter().any(|line| line.starts_with("workspace")));
    }
}
