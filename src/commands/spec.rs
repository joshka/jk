//! Command execution and safety metadata.
//!
//! The planner/runtime consult this module to decide wrapper style and whether confirmation gating
//! is required for a command.

/// Rendering/execution strategy used for a top-level command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Native,
    Guided,
    Passthrough,
}

/// Safety tier used for confirmation policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyTier {
    A,
    B,
    C,
}

/// Static metadata for one top-level `jj` command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    /// Command token as typed after `jj`.
    pub name: &'static str,
    /// Execution/rendering mode selected for this command.
    pub mode: ExecutionMode,
    /// Default safety tier for this command.
    pub tier: SafetyTier,
}

pub(super) const TOP_LEVEL_SPECS: [CommandSpec; 44] = [
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
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
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
        mode: ExecutionMode::Guided,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "file",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "fix",
        mode: ExecutionMode::Guided,
        tier: SafetyTier::C,
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
        mode: ExecutionMode::Guided,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "log",
        mode: ExecutionMode::Native,
        tier: SafetyTier::A,
    },
    CommandSpec {
        name: "metaedit",
        mode: ExecutionMode::Guided,
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
        mode: ExecutionMode::Guided,
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
        mode: ExecutionMode::Guided,
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

/// Return metadata for a known top-level command name.
pub fn lookup_top_level(command: &str) -> Option<CommandSpec> {
    TOP_LEVEL_SPECS
        .iter()
        .copied()
        .find(|spec| spec.name == command)
}

/// Classify command safety with subcommand-aware overrides.
///
/// Defaults are optimistic for empty input (`A`) and conservative for unknown commands (`B`).
/// Subcommand overrides intentionally escalate potentially destructive operations (for example
/// `git push` and `operation restore`) to tier `C`.
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

    use super::{ExecutionMode, SafetyTier, TOP_LEVEL_SPECS, command_safety, lookup_top_level};

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
        assert_eq!(
            lookup_top_level("interdiff").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("evolog").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("metaedit").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("simplify-parents").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("fix").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("resolve").map(|spec| spec.mode),
            Some(ExecutionMode::Guided)
        );
        assert_eq!(
            lookup_top_level("diffedit").map(|spec| spec.mode),
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
        assert_eq!(command_safety(&to_vec(&["fix"])), SafetyTier::C);
        assert_eq!(command_safety(&to_vec(&["diffedit"])), SafetyTier::C);
    }
}
