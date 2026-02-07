#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Native,
    Guided,
    Passthrough,
}

impl ExecutionMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Guided => "guided",
            Self::Passthrough => "passthrough",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyTier {
    A,
    B,
    C,
}

impl SafetyTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: &'static str,
    pub mode: ExecutionMode,
    pub tier: SafetyTier,
}

const TOP_LEVEL_SPECS: [CommandSpec; 44] = [
    CommandSpec {
        name: "abandon",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "absorb",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "bisect",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "bookmark",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "commit",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "config",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "describe",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "diff",
        mode: ExecutionMode::Native,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "diffedit",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "duplicate",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "edit",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "evolog",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "file",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "fix",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "gerrit",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "git",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "help",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "interdiff",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "log",
        mode: ExecutionMode::Native,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "metaedit",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "new",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "next",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "operation",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "parallelize",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "prev",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "rebase",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "redo",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "resolve",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "restore",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "revert",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "root",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "show",
        mode: ExecutionMode::Native,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "sign",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "simplify-parents",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "sparse",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "split",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "squash",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "status",
        mode: ExecutionMode::Native,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "tag",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "undo",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "unsign",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "util",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "version",
        mode: ExecutionMode::Passthrough,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "workspace",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
];

pub fn lookup_top_level(command: &str) -> Option<CommandSpec> {
    TOP_LEVEL_SPECS
        .iter()
        .copied()
        .find(|spec| spec.name == command)
}

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
            "high-frequency aliases: b, ci, desc, op, st, gf, gp, rbm, rbt, jjgf, jjgp, jjrbm, jjst, jjl"
                .to_string(),
        );
        lines.push("tips: use :aliases for mappings and :keys for active keybinds".to_string());
        lines.push(
            "group defaults: bookmark/file/tag/workspace -> list, resolve -> resolve -l, operation -> log".to_string(),
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

pub fn command_safety(tokens: &[String]) -> SafetyTier {
    let Some(first) = tokens.first().map(String::as_str) else {
        return SafetyTier::A;
    };

    match first {
        "git" => {
            if matches!(tokens.get(1).map(String::as_str), Some("push")) {
                SafetyTier::C
            } else {
                SafetyTier::B
            }
        }
        "operation" => match tokens.get(1).map(String::as_str) {
            Some("restore" | "revert") => SafetyTier::C,
            Some("log" | "show" | "diff") => SafetyTier::A,
            _ => SafetyTier::B,
        },
        "workspace" => match tokens.get(1).map(String::as_str) {
            Some("list" | "root") => SafetyTier::A,
            _ => SafetyTier::B,
        },
        "resolve" => {
            if tokens
                .iter()
                .any(|token| token == "-l" || token == "--list")
            {
                SafetyTier::A
            } else {
                SafetyTier::B
            }
        }
        "bookmark" => match tokens.get(1).map(String::as_str) {
            Some("set" | "move" | "delete" | "forget" | "rename") => SafetyTier::C,
            Some("create" | "track" | "untrack") => SafetyTier::B,
            _ => SafetyTier::A,
        },
        "file" => match tokens.get(1).map(String::as_str) {
            Some("annotate" | "list" | "search" | "show") => SafetyTier::A,
            _ => SafetyTier::B,
        },
        "tag" => match tokens.get(1).map(String::as_str) {
            Some("list") => SafetyTier::A,
            _ => SafetyTier::B,
        },
        value => lookup_top_level(value)
            .map(|spec| spec.tier)
            .unwrap_or(SafetyTier::B),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        ExecutionMode, SafetyTier, TOP_LEVEL_SPECS, command_overview_lines,
        command_overview_lines_with_query, command_safety, lookup_top_level,
    };

    fn to_vec(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn includes_all_jj_top_level_commands() {
        assert_eq!(TOP_LEVEL_SPECS.len(), 44);

        let names: HashSet<&str> = TOP_LEVEL_SPECS.iter().map(|spec| spec.name).collect();
        assert_eq!(names.len(), 44);
    }

    #[test]
    fn tracks_expected_execution_modes_for_core_commands() {
        assert_eq!(
            lookup_top_level("log").map(|spec| spec.mode),
            Some(ExecutionMode::Native)
        );
        assert_eq!(
            lookup_top_level("rebase").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("workspace").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("operation").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("restore").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("absorb").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("duplicate").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("parallelize").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
    }

    #[test]
    fn applies_subcommand_safety_overrides() {
        assert_eq!(command_safety(&to_vec(&["git", "push"])), SafetyTier::C);
        assert_eq!(command_safety(&to_vec(&["git", "fetch"])), SafetyTier::B);
        assert_eq!(
            command_safety(&to_vec(&["operation", "restore", "abc"])),
            SafetyTier::C
        );
        assert_eq!(
            command_safety(&to_vec(&["operation", "show"])),
            SafetyTier::A
        );
        assert_eq!(
            command_safety(&to_vec(&["workspace", "root"])),
            SafetyTier::A
        );
        assert_eq!(command_safety(&to_vec(&["resolve", "-l"])), SafetyTier::A);
        assert_eq!(command_safety(&to_vec(&["resolve"])), SafetyTier::B);
        assert_eq!(
            command_safety(&to_vec(&["bookmark", "set", "feature"])),
            SafetyTier::C
        );
        assert_eq!(
            command_safety(&to_vec(&["bookmark", "track", "feature"])),
            SafetyTier::B
        );
        assert_eq!(command_safety(&to_vec(&["file", "list"])), SafetyTier::A);
        assert_eq!(command_safety(&to_vec(&["file", "show"])), SafetyTier::A);
        assert_eq!(command_safety(&to_vec(&["file", "track"])), SafetyTier::B);
        assert_eq!(command_safety(&to_vec(&["tag", "list"])), SafetyTier::A);
        assert_eq!(command_safety(&to_vec(&["tag", "set"])), SafetyTier::B);
    }

    #[test]
    fn falls_back_to_top_level_safety() {
        assert_eq!(command_safety(&to_vec(&["log"])), SafetyTier::A);
        assert_eq!(command_safety(&to_vec(&["rebase"])), SafetyTier::C);
        assert_eq!(command_safety(&to_vec(&["new"])), SafetyTier::B);
    }

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
