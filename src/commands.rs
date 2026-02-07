#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Native,
    Guided,
    Passthrough,
    Defer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyTier {
    A,
    B,
    C,
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
        mode: ExecutionMode::Defer,
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
        mode: ExecutionMode::Defer,
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
        mode: ExecutionMode::Passthrough,
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
        mode: ExecutionMode::Defer,
        tier: SafetyTier::B,
    },
    CommandSpec {
        name: "parallelize",
        mode: ExecutionMode::Defer,
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
        mode: ExecutionMode::Defer,
        tier: SafetyTier::C,
    },
    CommandSpec {
        name: "revert",
        mode: ExecutionMode::Defer,
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
        mode: ExecutionMode::Passthrough,
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
        mode: ExecutionMode::Defer,
        tier: SafetyTier::B,
    },
];

pub fn lookup_top_level(command: &str) -> Option<CommandSpec> {
    TOP_LEVEL_SPECS
        .iter()
        .copied()
        .find(|spec| spec.name == command)
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
        "bookmark" => match tokens.get(1).map(String::as_str) {
            Some("set" | "move" | "delete" | "forget" | "rename") => SafetyTier::C,
            Some("create" | "track" | "untrack") => SafetyTier::B,
            _ => SafetyTier::A,
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
            Some(ExecutionMode::Defer)
        );
    }

    #[test]
    fn applies_subcommand_safety_overrides() {
        assert_eq!(command_safety(&to_vec(&["git", "push"])), SafetyTier::C);
        assert_eq!(command_safety(&to_vec(&["git", "fetch"])), SafetyTier::B);
        assert_eq!(
            command_safety(&to_vec(&["bookmark", "set", "feature"])),
            SafetyTier::C
        );
        assert_eq!(
            command_safety(&to_vec(&["bookmark", "track", "feature"])),
            SafetyTier::B
        );
    }

    #[test]
    fn falls_back_to_top_level_safety() {
        assert_eq!(command_safety(&to_vec(&["log"])), SafetyTier::A);
        assert_eq!(command_safety(&to_vec(&["rebase"])), SafetyTier::C);
        assert_eq!(command_safety(&to_vec(&["new"])), SafetyTier::B);
    }
}
