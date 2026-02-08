//! Command planner from command-line text to runtime actions.
//!
//! Planning prioritizes local render views and guided prompts before falling back to direct command
//! execution.

use crate::alias::{alias_overview_lines, alias_overview_lines_with_query, normalize_alias};
use crate::commands::{command_overview_lines, command_overview_lines_with_query};

use super::{FlowAction, PromptKind, PromptRequest};

/// Plan a command-mode line into an executable runtime action.
///
/// Decision order is:
/// 1. empty/quit fast-path handling,
/// 1. shell-like tokenization and alias normalization,
/// 1. local render actions (`commands`, `help`, `aliases`),
/// 1. guided prompt entrypoints for mutation flows,
/// 1. default direct execution.
///
/// Selected revision defaults to `@` when no row mapping is available, so selection-driven flows
/// remain executable even outside `log`-like views.
///
/// # Examples
///
/// ```text
/// input: "rebase", selected_revision: Some("qpvuntsm")
/// output: prompt for destination, source revision = "qpvuntsm"
/// ```
///
/// ```text
/// input: "operation", selected_revision: None
/// output: execute ["operation", "log"]
/// ```
pub fn plan_command(raw_command: &str, selected_revision: Option<String>) -> FlowAction {
    let trimmed = raw_command.trim();
    if trimmed.is_empty() {
        return FlowAction::Status("Ready".to_string());
    }

    if trimmed == "q" || trimmed == "quit" {
        return FlowAction::Quit;
    }

    let Some(raw_tokens) = shlex::split(trimmed) else {
        return FlowAction::Status("Invalid command quoting".to_string());
    };
    let mut tokens = normalize_alias(&raw_tokens);
    tokens = canonicalize_tokens(tokens);

    if tokens.is_empty() {
        return FlowAction::Execute(vec!["log".to_string()]);
    }

    let selected = selected_revision.unwrap_or_else(|| "@".to_string());

    match tokens.as_slice() {
        [command, tail @ ..]
            if (command == "commands" || command == "help") && !tail.is_empty() =>
        {
            let query = tail.join(" ");
            FlowAction::Render {
                lines: command_overview_lines_with_query(Some(&query)),
                status: format!("Showing command registry for `{query}`"),
            }
        }
        [command, tail @ ..] if command == "aliases" && !tail.is_empty() => {
            let query = tail.join(" ");
            FlowAction::Render {
                lines: alias_overview_lines_with_query(Some(&query)),
                status: format!("Showing alias catalog for `{query}`"),
            }
        }
        [command] if command == "commands" || command == "?" => FlowAction::Render {
            lines: command_overview_lines(),
            status: "Showing command registry".to_string(),
        },
        [command] if command == "help" => FlowAction::Render {
            lines: command_overview_lines(),
            status: "Showing command registry".to_string(),
        },
        [command] if command == "aliases" => FlowAction::Render {
            lines: alias_overview_lines(),
            status: "Showing alias catalog".to_string(),
        },
        [command] if command == "operation" => {
            FlowAction::Execute(vec!["operation".to_string(), "log".to_string()])
        }
        [command, subcommand] if command == "operation" && subcommand == "show" => {
            FlowAction::Execute(vec!["operation".to_string(), "show".to_string()])
        }
        [command, subcommand] if command == "operation" && subcommand == "diff" => {
            FlowAction::Execute(vec!["operation".to_string(), "diff".to_string()])
        }
        [command, subcommand] if command == "operation" && subcommand == "restore" => {
            FlowAction::Prompt(PromptRequest {
                label: "operation id to restore (required)".to_string(),
                allow_empty: false,
                kind: PromptKind::OperationRestore,
            })
        }
        [command, subcommand] if command == "operation" && subcommand == "revert" => {
            FlowAction::Prompt(PromptRequest {
                label: "operation id to revert (blank = @)".to_string(),
                allow_empty: true,
                kind: PromptKind::OperationRevert {
                    default_operation: "@".to_string(),
                },
            })
        }
        [command] if command == "workspace" => {
            FlowAction::Execute(vec!["workspace".to_string(), "list".to_string()])
        }
        [command] if command == "resolve" => {
            FlowAction::Execute(vec!["resolve".to_string(), "-l".to_string()])
        }
        [command] if command == "file" => {
            FlowAction::Execute(vec!["file".to_string(), "list".to_string()])
        }
        [command, subcommand] if command == "file" && subcommand == "track" => {
            FlowAction::Prompt(PromptRequest {
                label: "file paths/filesets to track".to_string(),
                allow_empty: false,
                kind: PromptKind::FileTrack,
            })
        }
        [command, subcommand] if command == "file" && subcommand == "untrack" => {
            FlowAction::Prompt(PromptRequest {
                label: "file paths/filesets to untrack".to_string(),
                allow_empty: false,
                kind: PromptKind::FileUntrack,
            })
        }
        [command, subcommand] if command == "file" && subcommand == "chmod" => {
            FlowAction::Prompt(PromptRequest {
                label: format!(
                    "file chmod: <mode> <path...> [--revision REVSET] (defaults to {selected})"
                ),
                allow_empty: false,
                kind: PromptKind::FileChmod {
                    default_revision: selected,
                },
            })
        }
        [command] if command == "tag" => {
            FlowAction::Execute(vec!["tag".to_string(), "list".to_string()])
        }
        [command, subcommand] if command == "tag" && subcommand == "set" => {
            FlowAction::Prompt(PromptRequest {
                label: format!(
                    "tag set: <name...> [revision] (blank revision defaults to {selected})"
                ),
                allow_empty: false,
                kind: PromptKind::TagSet {
                    default_revision: selected,
                },
            })
        }
        [command, subcommand] if command == "tag" && subcommand == "delete" => {
            FlowAction::Prompt(PromptRequest {
                label: "tag names to delete (space-separated)".to_string(),
                allow_empty: false,
                kind: PromptKind::TagDelete,
            })
        }
        [command, subcommand] if command == "workspace" && subcommand == "root" => {
            FlowAction::Execute(vec!["workspace".to_string(), "root".to_string()])
        }
        [command, subcommand] if command == "workspace" && subcommand == "update-stale" => {
            FlowAction::Execute(vec!["workspace".to_string(), "update-stale".to_string()])
        }
        [command, subcommand] if command == "workspace" && subcommand == "rename" => {
            FlowAction::Prompt(PromptRequest {
                label: "new workspace name".to_string(),
                allow_empty: false,
                kind: PromptKind::WorkspaceRename,
            })
        }
        [command, subcommand] if command == "workspace" && subcommand == "forget" => {
            FlowAction::Prompt(PromptRequest {
                label: "workspace names to forget (blank = current)".to_string(),
                allow_empty: true,
                kind: PromptKind::WorkspaceForget,
            })
        }
        [command, subcommand] if command == "workspace" && subcommand == "add" => {
            FlowAction::Prompt(PromptRequest {
                label: "workspace add: <destination> [name]".to_string(),
                allow_empty: false,
                kind: PromptKind::WorkspaceAdd,
            })
        }
        [command] if command == "new" => FlowAction::Prompt(PromptRequest {
            label: "new message (blank for none)".to_string(),
            allow_empty: true,
            kind: PromptKind::NewMessage,
        }),
        [command] if command == "describe" => FlowAction::Prompt(PromptRequest {
            label: format!("describe message for {selected}"),
            allow_empty: false,
            kind: PromptKind::DescribeMessage { revision: selected },
        }),
        [command] if command == "commit" => FlowAction::Prompt(PromptRequest {
            label: "commit message (blank for default)".to_string(),
            allow_empty: true,
            kind: PromptKind::CommitMessage,
        }),
        [command] if command == "metaedit" => FlowAction::Prompt(PromptRequest {
            label: format!("metaedit message for {selected}"),
            allow_empty: false,
            kind: PromptKind::MetaeditMessage { revision: selected },
        }),
        [command] if command == "next" => FlowAction::Execute(vec!["next".to_string()]),
        [command] if command == "prev" => FlowAction::Execute(vec!["prev".to_string()]),
        [command] if command == "edit" => FlowAction::Execute(vec!["edit".to_string(), selected]),
        [command] if command == "show" => FlowAction::Execute(vec!["show".to_string(), selected]),
        [command] if command == "diff" => {
            FlowAction::Execute(vec!["diff".to_string(), "-r".to_string(), selected])
        }
        [command] if command == "evolog" => {
            FlowAction::Execute(vec!["evolog".to_string(), "-r".to_string(), selected])
        }
        [command] if command == "interdiff" => FlowAction::Execute(vec![
            "interdiff".to_string(),
            "--from".to_string(),
            "@-".to_string(),
            "--to".to_string(),
            selected,
        ]),
        [command] if command == "diffedit" => {
            FlowAction::Execute(vec!["diffedit".to_string(), "-r".to_string(), selected])
        }
        [command] if command == "fix" => {
            FlowAction::Execute(vec!["fix".to_string(), "-s".to_string(), selected])
        }
        [command] if command == "abandon" => {
            FlowAction::Execute(vec!["abandon".to_string(), selected])
        }
        [command] if command == "undo" => FlowAction::Execute(vec!["undo".to_string()]),
        [command] if command == "redo" => FlowAction::Execute(vec!["redo".to_string()]),
        [command] if command == "restore" => FlowAction::Prompt(PromptRequest {
            label: format!("restore from revset into {selected} (blank = @-)"),
            allow_empty: true,
            kind: PromptKind::RestoreFrom {
                target_revision: selected,
            },
        }),
        [command] if command == "revert" => FlowAction::Prompt(PromptRequest {
            label: format!("revert revset (blank = {selected})"),
            allow_empty: true,
            kind: PromptKind::RevertRevisions {
                default_revisions: selected,
                onto_revision: "@".to_string(),
            },
        }),
        [command] if command == "absorb" => {
            FlowAction::Execute(vec!["absorb".to_string(), "--from".to_string(), selected])
        }
        [command] if command == "duplicate" => {
            FlowAction::Execute(vec!["duplicate".to_string(), selected])
        }
        [command] if command == "parallelize" => {
            FlowAction::Execute(vec!["parallelize".to_string(), selected])
        }
        [command] if command == "simplify-parents" => {
            FlowAction::Execute(vec!["simplify-parents".to_string(), selected])
        }
        [command] if command == "bookmark" => {
            FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
        }
        [command, subcommand] if command == "bookmark" && subcommand == "create" => {
            FlowAction::Prompt(PromptRequest {
                label: format!("bookmark name for revision {selected}"),
                allow_empty: false,
                kind: PromptKind::BookmarkCreate {
                    target_revision: selected,
                },
            })
        }
        [command, subcommand] if command == "bookmark" && subcommand == "set" => {
            FlowAction::Prompt(PromptRequest {
                label: format!("bookmark name for revision {selected}"),
                allow_empty: false,
                kind: PromptKind::BookmarkSet {
                    target_revision: selected,
                },
            })
        }
        [command, subcommand] if command == "bookmark" && subcommand == "move" => {
            FlowAction::Prompt(PromptRequest {
                label: format!("bookmark name to move to {selected}"),
                allow_empty: false,
                kind: PromptKind::BookmarkMove {
                    target_revision: selected,
                },
            })
        }
        [command, subcommand] if command == "bookmark" && subcommand == "delete" => {
            FlowAction::Prompt(PromptRequest {
                label: "delete bookmarks (space-separated names)".to_string(),
                allow_empty: false,
                kind: PromptKind::BookmarkDelete,
            })
        }
        [command, subcommand] if command == "bookmark" && subcommand == "forget" => {
            FlowAction::Prompt(PromptRequest {
                label: "forget bookmarks (space-separated names)".to_string(),
                allow_empty: false,
                kind: PromptKind::BookmarkForget,
            })
        }
        [command, subcommand] if command == "bookmark" && subcommand == "rename" => {
            FlowAction::Prompt(PromptRequest {
                label: "rename bookmark: <old> <new>".to_string(),
                allow_empty: false,
                kind: PromptKind::BookmarkRename,
            })
        }
        [command, subcommand] if command == "bookmark" && subcommand == "track" => {
            FlowAction::Prompt(PromptRequest {
                label: "track: <name> [remote]".to_string(),
                allow_empty: false,
                kind: PromptKind::BookmarkTrack,
            })
        }
        [command, subcommand] if command == "bookmark" && subcommand == "untrack" => {
            FlowAction::Prompt(PromptRequest {
                label: "untrack: <name> [remote]".to_string(),
                allow_empty: false,
                kind: PromptKind::BookmarkUntrack,
            })
        }
        [command, subcommand] if command == "git" && subcommand == "fetch" => {
            FlowAction::Prompt(PromptRequest {
                label: "fetch remote (blank for default)".to_string(),
                allow_empty: true,
                kind: PromptKind::GitFetchRemote,
            })
        }
        [command, subcommand] if command == "git" && subcommand == "push" => {
            FlowAction::Prompt(PromptRequest {
                label: "push bookmark (blank for default tracked push)".to_string(),
                allow_empty: true,
                kind: PromptKind::GitPushBookmark,
            })
        }
        [command] if command == "rebase" => FlowAction::Prompt(PromptRequest {
            label: format!("rebase destination revset for {selected}"),
            allow_empty: false,
            kind: PromptKind::RebaseDestination {
                source_revision: selected,
            },
        }),
        [command] if command == "squash" => FlowAction::Prompt(PromptRequest {
            label: format!("squash into revset (blank = @-) from {selected}"),
            allow_empty: true,
            kind: PromptKind::SquashInto {
                from_revision: selected,
            },
        }),
        [command] if command == "split" => FlowAction::Prompt(PromptRequest {
            label: "split fileset (required for non-interactive mode)".to_string(),
            allow_empty: false,
            kind: PromptKind::SplitFileset { revision: selected },
        }),
        _ => FlowAction::Execute(tokens),
    }
}

/// Canonicalize command and selected subcommands after alias expansion.
fn canonicalize_tokens(mut tokens: Vec<String>) -> Vec<String> {
    if tokens.is_empty() {
        return tokens;
    }

    tokens[0] = canonical_command_name(&tokens[0]);

    if tokens[0] == "bookmark" && tokens.len() > 1 {
        tokens[1] = canonical_bookmark_subcommand(&tokens[1]);
    }
    if tokens[0] == "tag" && tokens.len() > 1 {
        tokens[1] = canonical_tag_subcommand(&tokens[1]);
    }

    tokens
}

/// Canonicalize top-level shorthand command aliases.
fn canonical_command_name(command: &str) -> String {
    match command {
        "desc" => "describe".to_string(),
        "st" => "status".to_string(),
        "ci" => "commit".to_string(),
        "b" => "bookmark".to_string(),
        "op" => "operation".to_string(),
        value => value.to_string(),
    }
}

/// Canonicalize bookmark subcommand shorthands.
fn canonical_bookmark_subcommand(command: &str) -> String {
    match command {
        "c" => "create".to_string(),
        "d" => "delete".to_string(),
        "f" => "forget".to_string(),
        "l" => "list".to_string(),
        "m" => "move".to_string(),
        "r" => "rename".to_string(),
        "s" => "set".to_string(),
        "t" => "track".to_string(),
        "u" => "untrack".to_string(),
        value => value.to_string(),
    }
}

/// Canonicalize tag subcommand shorthands.
fn canonical_tag_subcommand(command: &str) -> String {
    match command {
        "d" => "delete".to_string(),
        "l" => "list".to_string(),
        "s" => "set".to_string(),
        value => value.to_string(),
    }
}
