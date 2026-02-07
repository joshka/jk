use crate::alias::{alias_overview_lines, alias_overview_lines_with_query, normalize_alias};
use crate::commands::{command_overview_lines, command_overview_lines_with_query};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowAction {
    Execute(Vec<String>),
    Render { lines: Vec<String>, status: String },
    Prompt(PromptRequest),
    Status(String),
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptRequest {
    pub label: String,
    pub allow_empty: bool,
    pub kind: PromptKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptKind {
    NewMessage,
    DescribeMessage {
        revision: String,
    },
    CommitMessage,
    GitFetchRemote,
    GitPushBookmark,
    BookmarkCreate {
        target_revision: String,
    },
    BookmarkSet {
        target_revision: String,
    },
    BookmarkMove {
        target_revision: String,
    },
    BookmarkDelete,
    BookmarkForget,
    BookmarkRename,
    BookmarkTrack,
    BookmarkUntrack,
    TagSet {
        default_revision: String,
    },
    TagDelete,
    RebaseDestination {
        source_revision: String,
    },
    SquashInto {
        from_revision: String,
    },
    SplitFileset {
        revision: String,
    },
    RestoreFrom {
        target_revision: String,
    },
    RevertRevisions {
        default_revisions: String,
        onto_revision: String,
    },
    OperationRestore,
    OperationRevert {
        default_operation: String,
    },
    WorkspaceAdd,
    WorkspaceForget,
    WorkspaceRename,
    FileTrack,
    FileUntrack,
    FileChmod {
        default_revision: String,
    },
}

impl PromptKind {
    pub fn to_tokens(&self, input: &str) -> Result<Vec<String>, String> {
        match self {
            Self::NewMessage => {
                if input.is_empty() {
                    Ok(vec!["new".to_string()])
                } else {
                    Ok(vec!["new".to_string(), "-m".to_string(), input.to_string()])
                }
            }
            Self::DescribeMessage { revision } => {
                if input.is_empty() {
                    return Err("description is required".to_string());
                }

                let mut tokens = vec!["describe".to_string(), "-m".to_string(), input.to_string()];
                if revision != "@" {
                    tokens.push(revision.clone());
                }
                Ok(tokens)
            }
            Self::CommitMessage => {
                if input.is_empty() {
                    Ok(vec!["commit".to_string()])
                } else {
                    Ok(vec![
                        "commit".to_string(),
                        "-m".to_string(),
                        input.to_string(),
                    ])
                }
            }
            Self::GitFetchRemote => {
                if input.is_empty() {
                    Ok(vec!["git".to_string(), "fetch".to_string()])
                } else {
                    Ok(vec![
                        "git".to_string(),
                        "fetch".to_string(),
                        "--remote".to_string(),
                        input.to_string(),
                    ])
                }
            }
            Self::GitPushBookmark => {
                if input.is_empty() {
                    Ok(vec!["git".to_string(), "push".to_string()])
                } else {
                    Ok(vec![
                        "git".to_string(),
                        "push".to_string(),
                        "--bookmark".to_string(),
                        input.to_string(),
                    ])
                }
            }
            Self::BookmarkCreate { target_revision } => build_bookmark_target_command(
                "create",
                input,
                target_revision,
                "bookmark name required",
                "-r",
            ),
            Self::BookmarkSet { target_revision } => build_bookmark_target_command(
                "set",
                input,
                target_revision,
                "bookmark name required",
                "-r",
            ),
            Self::BookmarkMove { target_revision } => build_bookmark_target_command(
                "move",
                input,
                target_revision,
                "bookmark name required",
                "--to",
            ),
            Self::BookmarkDelete => build_bookmark_names_command("delete", input),
            Self::BookmarkForget => build_bookmark_names_command("forget", input),
            Self::BookmarkRename => build_bookmark_rename_command(input),
            Self::BookmarkTrack => build_track_command("track", input),
            Self::BookmarkUntrack => build_track_command("untrack", input),
            Self::TagSet { default_revision } => build_tag_set_command(input, default_revision),
            Self::TagDelete => build_tag_delete_command(input),
            Self::RebaseDestination { source_revision } => {
                if input.is_empty() {
                    Err("destination revset is required".to_string())
                } else {
                    Ok(vec![
                        "rebase".to_string(),
                        "-r".to_string(),
                        source_revision.to_string(),
                        "-d".to_string(),
                        input.to_string(),
                    ])
                }
            }
            Self::SquashInto { from_revision } => {
                let into = if input.is_empty() { "@-" } else { input };
                Ok(vec![
                    "squash".to_string(),
                    "--from".to_string(),
                    from_revision.to_string(),
                    "--into".to_string(),
                    into.to_string(),
                ])
            }
            Self::SplitFileset { revision } => {
                if input.is_empty() {
                    Err("split fileset is required (for example: src/main.rs)".to_string())
                } else {
                    Ok(vec![
                        "split".to_string(),
                        "-r".to_string(),
                        revision.to_string(),
                        input.to_string(),
                    ])
                }
            }
            Self::RestoreFrom { target_revision } => {
                let from = if input.is_empty() { "@-" } else { input };
                Ok(vec![
                    "restore".to_string(),
                    "--from".to_string(),
                    from.to_string(),
                    "--to".to_string(),
                    target_revision.to_string(),
                ])
            }
            Self::RevertRevisions {
                default_revisions,
                onto_revision,
            } => {
                let revisions = if input.is_empty() {
                    default_revisions
                } else {
                    input
                };
                Ok(vec![
                    "revert".to_string(),
                    "-r".to_string(),
                    revisions.to_string(),
                    "-o".to_string(),
                    onto_revision.to_string(),
                ])
            }
            Self::OperationRestore => {
                if input.is_empty() {
                    Err("operation id is required".to_string())
                } else {
                    Ok(vec![
                        "operation".to_string(),
                        "restore".to_string(),
                        input.to_string(),
                    ])
                }
            }
            Self::OperationRevert { default_operation } => {
                let operation = if input.is_empty() {
                    default_operation
                } else {
                    input
                };

                if operation == "@" {
                    Ok(vec!["operation".to_string(), "revert".to_string()])
                } else {
                    Ok(vec![
                        "operation".to_string(),
                        "revert".to_string(),
                        operation.to_string(),
                    ])
                }
            }
            Self::WorkspaceAdd => build_workspace_add_command(input),
            Self::WorkspaceForget => build_workspace_forget_command(input),
            Self::WorkspaceRename => {
                if input.is_empty() {
                    Err("workspace name is required".to_string())
                } else {
                    Ok(vec![
                        "workspace".to_string(),
                        "rename".to_string(),
                        input.to_string(),
                    ])
                }
            }
            Self::FileTrack => build_file_track_command(input),
            Self::FileUntrack => build_file_untrack_command(input),
            Self::FileChmod { default_revision } => {
                build_file_chmod_command(input, default_revision)
            }
        }
    }
}

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
        [command] if command == "next" => FlowAction::Execute(vec!["next".to_string()]),
        [command] if command == "prev" => FlowAction::Execute(vec!["prev".to_string()]),
        [command] if command == "edit" => FlowAction::Execute(vec!["edit".to_string(), selected]),
        [command] if command == "show" => FlowAction::Execute(vec!["show".to_string(), selected]),
        [command] if command == "diff" => {
            FlowAction::Execute(vec!["diff".to_string(), "-r".to_string(), selected])
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

fn canonical_tag_subcommand(command: &str) -> String {
    match command {
        "d" => "delete".to_string(),
        "l" => "list".to_string(),
        "s" => "set".to_string(),
        value => value.to_string(),
    }
}

fn build_bookmark_target_command(
    subcommand: &str,
    input: &str,
    target_revision: &str,
    empty_message: &str,
    target_flag: &str,
) -> Result<Vec<String>, String> {
    if input.is_empty() {
        return Err(empty_message.to_string());
    }

    Ok(vec![
        "bookmark".to_string(),
        subcommand.to_string(),
        input.to_string(),
        target_flag.to_string(),
        target_revision.to_string(),
    ])
}

fn build_track_command(subcommand: &str, input: &str) -> Result<Vec<String>, String> {
    if input.is_empty() {
        return Err("bookmark name is required".to_string());
    }

    let segments: Vec<&str> = input.split_whitespace().collect();
    if segments.len() > 2 {
        return Err("use format: <bookmark> [remote]".to_string());
    }

    let mut tokens = vec![
        "bookmark".to_string(),
        subcommand.to_string(),
        segments[0].to_string(),
    ];

    if let Some(remote) = segments.get(1) {
        tokens.push("--remote".to_string());
        tokens.push((*remote).to_string());
    }

    Ok(tokens)
}

fn build_bookmark_names_command(subcommand: &str, input: &str) -> Result<Vec<String>, String> {
    let names: Vec<&str> = input.split_whitespace().collect();
    if names.is_empty() {
        return Err("at least one bookmark name is required".to_string());
    }

    let mut tokens = vec!["bookmark".to_string(), subcommand.to_string()];
    tokens.extend(names.into_iter().map(ToString::to_string));
    Ok(tokens)
}

fn build_bookmark_rename_command(input: &str) -> Result<Vec<String>, String> {
    let names: Vec<&str> = input.split_whitespace().collect();
    if names.len() != 2 {
        return Err("use format: <old> <new>".to_string());
    }

    Ok(vec![
        "bookmark".to_string(),
        "rename".to_string(),
        names[0].to_string(),
        names[1].to_string(),
    ])
}

fn has_revision_flag(tokens: &[String]) -> bool {
    tokens.iter().any(|token| {
        matches!(token.as_str(), "-r" | "--revision" | "--to")
            || token.starts_with("-r=")
            || token.starts_with("--revision=")
            || token.starts_with("--to=")
    })
}

fn build_tag_set_command(input: &str, default_revision: &str) -> Result<Vec<String>, String> {
    let segments: Vec<String> = input.split_whitespace().map(ToString::to_string).collect();
    if segments.is_empty() {
        return Err("at least one tag name is required".to_string());
    }

    let mut tokens = vec!["tag".to_string(), "set".to_string()];
    if segments.len() >= 2 && !segments[1].starts_with('-') && !has_revision_flag(&segments) {
        tokens.push(segments[0].clone());
        tokens.push("--revision".to_string());
        tokens.push(segments[1].clone());
        tokens.extend(segments.into_iter().skip(2));
        return Ok(tokens);
    }

    tokens.extend(segments.clone());
    if !has_revision_flag(&segments) {
        tokens.push("--revision".to_string());
        tokens.push(default_revision.to_string());
    }
    Ok(tokens)
}

fn build_tag_delete_command(input: &str) -> Result<Vec<String>, String> {
    let names: Vec<&str> = input.split_whitespace().collect();
    if names.is_empty() {
        return Err("at least one tag name is required".to_string());
    }

    let mut tokens = vec!["tag".to_string(), "delete".to_string()];
    tokens.extend(names.into_iter().map(ToString::to_string));
    Ok(tokens)
}

fn build_workspace_add_command(input: &str) -> Result<Vec<String>, String> {
    let segments: Vec<&str> = input.split_whitespace().collect();
    match segments.as_slice() {
        [destination] => Ok(vec![
            "workspace".to_string(),
            "add".to_string(),
            (*destination).to_string(),
        ]),
        [destination, name] => Ok(vec![
            "workspace".to_string(),
            "add".to_string(),
            "--name".to_string(),
            (*name).to_string(),
            (*destination).to_string(),
        ]),
        _ => Err("use format: <destination> [name]".to_string()),
    }
}

fn build_workspace_forget_command(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = vec!["workspace".to_string(), "forget".to_string()];
    tokens.extend(
        input
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    );
    Ok(tokens)
}

fn build_file_track_command(input: &str) -> Result<Vec<String>, String> {
    let paths: Vec<&str> = input.split_whitespace().collect();
    if paths.is_empty() {
        return Err("at least one file/fileset is required".to_string());
    }

    let mut tokens = vec!["file".to_string(), "track".to_string()];
    tokens.extend(paths.into_iter().map(ToString::to_string));
    Ok(tokens)
}

fn build_file_untrack_command(input: &str) -> Result<Vec<String>, String> {
    let paths: Vec<&str> = input.split_whitespace().collect();
    if paths.is_empty() {
        return Err("at least one file/fileset is required".to_string());
    }

    let mut tokens = vec!["file".to_string(), "untrack".to_string()];
    tokens.extend(paths.into_iter().map(ToString::to_string));
    Ok(tokens)
}

fn build_file_chmod_command(input: &str, default_revision: &str) -> Result<Vec<String>, String> {
    let parts: Vec<String> = input.split_whitespace().map(ToString::to_string).collect();
    if parts.len() < 2 {
        return Err("use format: <mode> <path...> [--revision REVSET]".to_string());
    }

    let mut tokens = vec!["file".to_string(), "chmod".to_string()];
    tokens.extend(parts.clone());
    if !has_revision_flag(&parts) {
        tokens.push("--revision".to_string());
        tokens.push(default_revision.to_string());
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{FlowAction, PromptKind, plan_command};

    fn selected() -> Option<String> {
        Some("abc12345".to_string())
    }

    fn assert_prompt_kind(action: FlowAction, expected: PromptKind) {
        match action {
            FlowAction::Prompt(request) => assert_eq!(request.kind, expected),
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn plans_show_diff_edit_and_abandon_with_selected_revision() {
        assert_eq!(
            plan_command("show", selected()),
            FlowAction::Execute(vec!["show".to_string(), "abc12345".to_string()])
        );
        assert_eq!(
            plan_command("diff", selected()),
            FlowAction::Execute(vec![
                "diff".to_string(),
                "-r".to_string(),
                "abc12345".to_string()
            ])
        );
        assert_eq!(
            plan_command("edit", selected()),
            FlowAction::Execute(vec!["edit".to_string(), "abc12345".to_string()])
        );
        assert_eq!(
            plan_command("abandon", selected()),
            FlowAction::Execute(vec!["abandon".to_string(), "abc12345".to_string()])
        );
    }

    #[test]
    fn plans_prompt_for_describe_commit_and_rewrite_flows() {
        let describe = plan_command("describe", selected());
        match describe {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::DescribeMessage {
                        revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        let commit = plan_command("commit", selected());
        match commit {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::CommitMessage);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        let rebase = plan_command("rebase", selected());
        match rebase {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::RebaseDestination {
                        source_revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        let squash = plan_command("jjsq", selected());
        match squash {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::SquashInto {
                        from_revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        let split = plan_command("split", selected());
        match split {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::SplitFileset {
                        revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn maps_alias_and_prompts_fetch_push() {
        let fetch = plan_command("gf", selected());
        match fetch {
            FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::GitFetchRemote),
            other => panic!("expected prompt, got {other:?}"),
        }

        let push = plan_command("jjgp", selected());
        match push {
            FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::GitPushBookmark),
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn maps_fetch_push_variant_aliases() {
        assert_eq!(
            plan_command("jjgfa", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "fetch".to_string(),
                "--all-remotes".to_string()
            ])
        );
        assert_eq!(
            plan_command("jjgpt", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "push".to_string(),
                "--tracked".to_string()
            ])
        );
        assert_eq!(
            plan_command("jjgpa", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "push".to_string(),
                "--all".to_string()
            ])
        );
        assert_eq!(
            plan_command("jjgpd", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "push".to_string(),
                "--deleted".to_string()
            ])
        );
    }

    #[test]
    fn maps_log_and_status_aliases() {
        assert_eq!(
            plan_command("jjst", selected()),
            FlowAction::Execute(vec!["status".to_string()])
        );
        assert_eq!(
            plan_command("jjl", selected()),
            FlowAction::Execute(vec!["log".to_string()])
        );
    }

    #[test]
    fn maps_core_jj_default_aliases() {
        assert_eq!(
            plan_command("st", selected()),
            FlowAction::Execute(vec!["status".to_string()])
        );
        assert_prompt_kind(
            plan_command("desc", selected()),
            PromptKind::DescribeMessage {
                revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(plan_command("ci", selected()), PromptKind::CommitMessage);
        assert_eq!(
            plan_command("b", selected()),
            FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
        );
        assert_eq!(
            plan_command("op", selected()),
            FlowAction::Execute(vec!["operation".to_string(), "log".to_string()])
        );
    }

    #[test]
    fn covers_oh_my_zsh_gold_alias_flow_contract() {
        assert_prompt_kind(plan_command("jjgf", selected()), PromptKind::GitFetchRemote);
        assert_prompt_kind(
            plan_command("jjgp", selected()),
            PromptKind::GitPushBookmark,
        );
        assert_eq!(
            plan_command("jjgfa", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "fetch".to_string(),
                "--all-remotes".to_string()
            ])
        );
        assert_eq!(
            plan_command("jjgpt", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "push".to_string(),
                "--tracked".to_string()
            ])
        );
        assert_eq!(
            plan_command("jjgpa", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "push".to_string(),
                "--all".to_string()
            ])
        );
        assert_eq!(
            plan_command("jjgpd", selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "push".to_string(),
                "--deleted".to_string()
            ])
        );

        assert_prompt_kind(
            plan_command("jjrb", selected()),
            PromptKind::RebaseDestination {
                source_revision: "abc12345".to_string(),
            },
        );
        assert_eq!(
            plan_command("jjrbm", selected()),
            FlowAction::Execute(vec![
                "rebase".to_string(),
                "-d".to_string(),
                "trunk()".to_string()
            ])
        );
        assert_eq!(
            plan_command("jjst", selected()),
            FlowAction::Execute(vec!["status".to_string()])
        );
        assert_eq!(
            plan_command("jjl", selected()),
            FlowAction::Execute(vec!["log".to_string()])
        );
        assert_eq!(
            plan_command("jjd", selected()),
            FlowAction::Execute(vec![
                "diff".to_string(),
                "-r".to_string(),
                "abc12345".to_string()
            ])
        );
        assert_prompt_kind(plan_command("jjc", selected()), PromptKind::CommitMessage);
        assert_prompt_kind(
            plan_command("jjds", selected()),
            PromptKind::DescribeMessage {
                revision: "abc12345".to_string(),
            },
        );
        assert_eq!(
            plan_command("jje", selected()),
            FlowAction::Execute(vec!["edit".to_string(), "abc12345".to_string()])
        );
        assert_prompt_kind(plan_command("jjn", selected()), PromptKind::NewMessage);
        assert_eq!(
            plan_command("jjnt", selected()),
            FlowAction::Execute(vec!["new".to_string(), "trunk()".to_string()])
        );
        assert_prompt_kind(
            plan_command("jjsp", selected()),
            PromptKind::SplitFileset {
                revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("jjsq", selected()),
            PromptKind::SquashInto {
                from_revision: "abc12345".to_string(),
            },
        );
        assert_eq!(
            plan_command("jjb", selected()),
            FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
        );
        assert_eq!(
            plan_command("jjbl", selected()),
            FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
        );
        assert_prompt_kind(
            plan_command("jjbs", selected()),
            PromptKind::BookmarkSet {
                target_revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("jjbm", selected()),
            PromptKind::BookmarkMove {
                target_revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(plan_command("jjbt", selected()), PromptKind::BookmarkTrack);
        assert_prompt_kind(
            plan_command("jjbu", selected()),
            PromptKind::BookmarkUntrack,
        );
        assert_prompt_kind(
            plan_command("jjrs", selected()),
            PromptKind::RestoreFrom {
                target_revision: "abc12345".to_string(),
            },
        );
        assert_eq!(
            plan_command("jja", selected()),
            FlowAction::Execute(vec!["abandon".to_string(), "abc12345".to_string()])
        );
        assert_eq!(
            plan_command("jjrt", selected()),
            FlowAction::Execute(vec!["root".to_string()])
        );
    }

    #[test]
    fn maps_rebase_aliases_to_expected_destinations() {
        assert_eq!(
            plan_command("rbm", selected()),
            FlowAction::Execute(vec![
                "rebase".to_string(),
                "-d".to_string(),
                "main".to_string()
            ])
        );

        assert_eq!(
            plan_command("rbt", selected()),
            FlowAction::Execute(vec![
                "rebase".to_string(),
                "-d".to_string(),
                "trunk()".to_string()
            ])
        );

        assert_eq!(
            plan_command("rbm release", selected()),
            FlowAction::Execute(vec![
                "rebase".to_string(),
                "-d".to_string(),
                "release".to_string()
            ])
        );
    }

    #[test]
    fn covers_gold_command_flow_contract() {
        assert_eq!(
            plan_command("log", selected()),
            FlowAction::Execute(vec!["log".to_string()])
        );
        assert_eq!(
            plan_command("status", selected()),
            FlowAction::Execute(vec!["status".to_string()])
        );
        assert_eq!(
            plan_command("show", selected()),
            FlowAction::Execute(vec!["show".to_string(), "abc12345".to_string()])
        );
        assert_eq!(
            plan_command("diff", selected()),
            FlowAction::Execute(vec![
                "diff".to_string(),
                "-r".to_string(),
                "abc12345".to_string()
            ])
        );

        assert_prompt_kind(plan_command("new", selected()), PromptKind::NewMessage);
        assert_prompt_kind(
            plan_command("describe", selected()),
            PromptKind::DescribeMessage {
                revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("commit", selected()),
            PromptKind::CommitMessage,
        );
        assert_eq!(
            plan_command("next", selected()),
            FlowAction::Execute(vec!["next".to_string()])
        );
        assert_eq!(
            plan_command("prev", selected()),
            FlowAction::Execute(vec!["prev".to_string()])
        );
        assert_eq!(
            plan_command("edit", selected()),
            FlowAction::Execute(vec!["edit".to_string(), "abc12345".to_string()])
        );

        assert_prompt_kind(
            plan_command("rebase", selected()),
            PromptKind::RebaseDestination {
                source_revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("squash", selected()),
            PromptKind::SquashInto {
                from_revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("split", selected()),
            PromptKind::SplitFileset {
                revision: "abc12345".to_string(),
            },
        );
        assert_eq!(
            plan_command("abandon", selected()),
            FlowAction::Execute(vec!["abandon".to_string(), "abc12345".to_string()])
        );
        assert_eq!(
            plan_command("undo", selected()),
            FlowAction::Execute(vec!["undo".to_string()])
        );
        assert_eq!(
            plan_command("redo", selected()),
            FlowAction::Execute(vec!["redo".to_string()])
        );

        assert_eq!(
            plan_command("bookmark", selected()),
            FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
        );
        assert_prompt_kind(
            plan_command("bookmark create", selected()),
            PromptKind::BookmarkCreate {
                target_revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("bookmark set", selected()),
            PromptKind::BookmarkSet {
                target_revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("bookmark move", selected()),
            PromptKind::BookmarkMove {
                target_revision: "abc12345".to_string(),
            },
        );
        assert_prompt_kind(
            plan_command("bookmark track", selected()),
            PromptKind::BookmarkTrack,
        );
        assert_prompt_kind(
            plan_command("bookmark untrack", selected()),
            PromptKind::BookmarkUntrack,
        );

        assert_prompt_kind(
            plan_command("git fetch", selected()),
            PromptKind::GitFetchRemote,
        );
        assert_prompt_kind(
            plan_command("git push", selected()),
            PromptKind::GitPushBookmark,
        );
    }

    #[test]
    fn bookmark_without_subcommand_lists() {
        assert_eq!(
            plan_command("bookmark", selected()),
            FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
        );
    }

    #[test]
    fn bookmark_short_subcommand_is_canonicalized() {
        match plan_command("bookmark s", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::BookmarkSet {
                        target_revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn adds_guided_restore_and_revert_flows() {
        match plan_command("restore", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::RestoreFrom {
                        target_revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("revert", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::RevertRevisions {
                        default_revisions: "abc12345".to_string(),
                        onto_revision: "@".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn adds_guided_delete_forget_and_rename_flows() {
        match plan_command("bookmark delete", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::BookmarkDelete);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("bookmark forget", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::BookmarkForget);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("bookmark rename", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::BookmarkRename);
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn adds_guided_operation_subcommand_flows() {
        match plan_command("operation show", selected()) {
            FlowAction::Execute(tokens) => {
                assert_eq!(tokens, vec!["operation".to_string(), "show".to_string()]);
            }
            other => panic!("expected execute, got {other:?}"),
        }

        match plan_command("operation diff", selected()) {
            FlowAction::Execute(tokens) => {
                assert_eq!(tokens, vec!["operation".to_string(), "diff".to_string()]);
            }
            other => panic!("expected execute, got {other:?}"),
        }

        match plan_command("operation restore", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::OperationRestore);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("operation revert", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::OperationRevert {
                        default_operation: "@".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn adds_guided_workspace_subcommand_flows() {
        match plan_command("workspace rename", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::WorkspaceRename);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("workspace add", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::WorkspaceAdd);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("workspace root", selected()) {
            FlowAction::Execute(tokens) => {
                assert_eq!(tokens, vec!["workspace".to_string(), "root".to_string()])
            }
            other => panic!("expected execute, got {other:?}"),
        }
    }

    #[test]
    fn prompt_kind_builds_tokens() {
        let bookmark = PromptKind::BookmarkSet {
            target_revision: "abc12345".to_string(),
        };
        assert_eq!(
            bookmark.to_tokens("feature"),
            Ok(vec![
                "bookmark".to_string(),
                "set".to_string(),
                "feature".to_string(),
                "-r".to_string(),
                "abc12345".to_string()
            ])
        );

        let track = PromptKind::BookmarkTrack;
        assert_eq!(
            track.to_tokens("topic origin"),
            Ok(vec![
                "bookmark".to_string(),
                "track".to_string(),
                "topic".to_string(),
                "--remote".to_string(),
                "origin".to_string()
            ])
        );

        let rebase = PromptKind::RebaseDestination {
            source_revision: "abc12345".to_string(),
        };
        assert_eq!(
            rebase.to_tokens("main"),
            Ok(vec![
                "rebase".to_string(),
                "-r".to_string(),
                "abc12345".to_string(),
                "-d".to_string(),
                "main".to_string()
            ])
        );

        let squash = PromptKind::SquashInto {
            from_revision: "abc12345".to_string(),
        };
        assert_eq!(
            squash.to_tokens(""),
            Ok(vec![
                "squash".to_string(),
                "--from".to_string(),
                "abc12345".to_string(),
                "--into".to_string(),
                "@-".to_string()
            ])
        );

        let bookmark_delete = PromptKind::BookmarkDelete;
        assert_eq!(
            bookmark_delete.to_tokens("feature hotfix"),
            Ok(vec![
                "bookmark".to_string(),
                "delete".to_string(),
                "feature".to_string(),
                "hotfix".to_string()
            ])
        );

        let bookmark_rename = PromptKind::BookmarkRename;
        assert_eq!(
            bookmark_rename.to_tokens("old-name new-name"),
            Ok(vec![
                "bookmark".to_string(),
                "rename".to_string(),
                "old-name".to_string(),
                "new-name".to_string()
            ])
        );

        let restore = PromptKind::RestoreFrom {
            target_revision: "@".to_string(),
        };
        assert_eq!(
            restore.to_tokens(""),
            Ok(vec![
                "restore".to_string(),
                "--from".to_string(),
                "@-".to_string(),
                "--to".to_string(),
                "@".to_string()
            ])
        );

        let revert = PromptKind::RevertRevisions {
            default_revisions: "abc12345".to_string(),
            onto_revision: "@".to_string(),
        };
        assert_eq!(
            revert.to_tokens(""),
            Ok(vec![
                "revert".to_string(),
                "-r".to_string(),
                "abc12345".to_string(),
                "-o".to_string(),
                "@".to_string()
            ])
        );

        let operation_restore = PromptKind::OperationRestore;
        assert_eq!(
            operation_restore.to_tokens("abc123"),
            Ok(vec![
                "operation".to_string(),
                "restore".to_string(),
                "abc123".to_string()
            ])
        );

        let operation_revert = PromptKind::OperationRevert {
            default_operation: "@".to_string(),
        };
        assert_eq!(
            operation_revert.to_tokens(""),
            Ok(vec!["operation".to_string(), "revert".to_string()])
        );

        let workspace_add = PromptKind::WorkspaceAdd;
        assert_eq!(
            workspace_add.to_tokens("tmp/ws demo"),
            Ok(vec![
                "workspace".to_string(),
                "add".to_string(),
                "--name".to_string(),
                "demo".to_string(),
                "tmp/ws".to_string()
            ])
        );

        let tag_set = PromptKind::TagSet {
            default_revision: "abc12345".to_string(),
        };
        assert_eq!(
            tag_set.to_tokens("v0.2.0"),
            Ok(vec![
                "tag".to_string(),
                "set".to_string(),
                "v0.2.0".to_string(),
                "--revision".to_string(),
                "abc12345".to_string()
            ])
        );
        assert_eq!(
            tag_set.to_tokens("v0.2.0 main"),
            Ok(vec![
                "tag".to_string(),
                "set".to_string(),
                "v0.2.0".to_string(),
                "--revision".to_string(),
                "main".to_string()
            ])
        );
        assert_eq!(
            tag_set.to_tokens("v0.2.0 --revision release"),
            Ok(vec![
                "tag".to_string(),
                "set".to_string(),
                "v0.2.0".to_string(),
                "--revision".to_string(),
                "release".to_string()
            ])
        );

        let tag_delete = PromptKind::TagDelete;
        assert_eq!(
            tag_delete.to_tokens("v0.1.0 v0.2.0"),
            Ok(vec![
                "tag".to_string(),
                "delete".to_string(),
                "v0.1.0".to_string(),
                "v0.2.0".to_string()
            ])
        );

        let file_track = PromptKind::FileTrack;
        assert_eq!(
            file_track.to_tokens("src/app.rs src/flows.rs"),
            Ok(vec![
                "file".to_string(),
                "track".to_string(),
                "src/app.rs".to_string(),
                "src/flows.rs".to_string()
            ])
        );

        let file_untrack = PromptKind::FileUntrack;
        assert_eq!(
            file_untrack.to_tokens("target/generated.txt"),
            Ok(vec![
                "file".to_string(),
                "untrack".to_string(),
                "target/generated.txt".to_string()
            ])
        );

        let file_chmod = PromptKind::FileChmod {
            default_revision: "abc12345".to_string(),
        };
        assert_eq!(
            file_chmod.to_tokens("x scripts/deploy.sh"),
            Ok(vec![
                "file".to_string(),
                "chmod".to_string(),
                "x".to_string(),
                "scripts/deploy.sh".to_string(),
                "--revision".to_string(),
                "abc12345".to_string()
            ])
        );
        assert_eq!(
            file_chmod.to_tokens("n scripts/deploy.sh --revision @-"),
            Ok(vec![
                "file".to_string(),
                "chmod".to_string(),
                "n".to_string(),
                "scripts/deploy.sh".to_string(),
                "--revision".to_string(),
                "@-".to_string()
            ])
        );

        let workspace_forget = PromptKind::WorkspaceForget;
        assert_eq!(
            workspace_forget.to_tokens(""),
            Ok(vec!["workspace".to_string(), "forget".to_string()])
        );

        let workspace_rename = PromptKind::WorkspaceRename;
        assert_eq!(
            workspace_rename.to_tokens("main"),
            Ok(vec![
                "workspace".to_string(),
                "rename".to_string(),
                "main".to_string()
            ])
        );
    }

    #[test]
    fn undo_and_redo_execute_directly() {
        assert_eq!(
            plan_command("undo", selected()),
            FlowAction::Execute(vec!["undo".to_string()])
        );
        assert_eq!(
            plan_command("redo", selected()),
            FlowAction::Execute(vec!["redo".to_string()])
        );
    }

    #[test]
    fn next_and_prev_execute_directly() {
        assert_eq!(
            plan_command("next", selected()),
            FlowAction::Execute(vec!["next".to_string()])
        );
        assert_eq!(
            plan_command("prev", selected()),
            FlowAction::Execute(vec!["prev".to_string()])
        );
    }

    #[test]
    fn renders_command_registry_in_app() {
        match plan_command("commands", selected()) {
            FlowAction::Render { lines, status } => {
                assert_eq!(status, "Showing command registry".to_string());
                assert!(
                    lines
                        .iter()
                        .any(|line| line.contains("jj top-level coverage"))
                );
            }
            other => panic!("expected render action, got {other:?}"),
        }
    }

    #[test]
    fn defaults_operation_workspace_resolve_file_and_tag_to_list_views() {
        assert_eq!(
            plan_command("op", selected()),
            FlowAction::Execute(vec!["operation".to_string(), "log".to_string()])
        );
        assert_eq!(
            plan_command("workspace", selected()),
            FlowAction::Execute(vec!["workspace".to_string(), "list".to_string()])
        );
        assert_eq!(
            plan_command("resolve", selected()),
            FlowAction::Execute(vec!["resolve".to_string(), "-l".to_string()])
        );
        assert_eq!(
            plan_command("file", selected()),
            FlowAction::Execute(vec!["file".to_string(), "list".to_string()])
        );
        assert_eq!(
            plan_command("tag", selected()),
            FlowAction::Execute(vec!["tag".to_string(), "list".to_string()])
        );
    }

    #[test]
    fn canonicalizes_tag_short_subcommands() {
        assert_eq!(
            plan_command("tag l", selected()),
            FlowAction::Execute(vec!["tag".to_string(), "list".to_string()])
        );
        assert_eq!(
            plan_command("tag s v0.1.0", selected()),
            FlowAction::Execute(vec![
                "tag".to_string(),
                "set".to_string(),
                "v0.1.0".to_string()
            ])
        );
        assert_eq!(
            plan_command("tag d v0.1.0", selected()),
            FlowAction::Execute(vec![
                "tag".to_string(),
                "delete".to_string(),
                "v0.1.0".to_string()
            ])
        );
    }

    #[test]
    fn adds_guided_tag_set_and_delete_flows() {
        match plan_command("tag set", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::TagSet {
                        default_revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("tag delete", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::TagDelete);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("tag s", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::TagSet {
                        default_revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("tag d", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::TagDelete);
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn adds_guided_file_track_and_untrack_flows() {
        match plan_command("file track", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::FileTrack);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("file untrack", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::FileUntrack);
            }
            other => panic!("expected prompt, got {other:?}"),
        }

        match plan_command("file chmod", selected()) {
            FlowAction::Prompt(request) => {
                assert_eq!(
                    request.kind,
                    PromptKind::FileChmod {
                        default_revision: "abc12345".to_string()
                    }
                );
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn filters_command_registry_with_query() {
        match plan_command("commands work", selected()) {
            FlowAction::Render { lines, status } => {
                assert_eq!(status, "Showing command registry for `work`".to_string());
                assert!(lines.iter().any(|line| line.starts_with("workspace")));
                assert!(!lines.iter().any(|line| line.starts_with("rebase")));
            }
            other => panic!("expected render action, got {other:?}"),
        }
    }

    #[test]
    fn renders_alias_catalog_in_app() {
        match plan_command("aliases", selected()) {
            FlowAction::Render { lines, status } => {
                assert_eq!(status, "Showing alias catalog".to_string());
                assert!(lines.iter().any(|line| line.contains("jjrbm")));
            }
            other => panic!("expected render action, got {other:?}"),
        }
    }

    #[test]
    fn filters_alias_catalog_with_query() {
        match plan_command("aliases push", selected()) {
            FlowAction::Render { lines, status } => {
                assert_eq!(status, "Showing alias catalog for `push`".to_string());
                assert!(lines.iter().any(|line| line.contains("jjgp")));
                assert!(!lines.iter().any(|line| line.contains("jjrbm")));
            }
            other => panic!("expected render action, got {other:?}"),
        }
    }

    #[test]
    fn parses_quoted_arguments_in_command_mode() {
        assert_eq!(
            plan_command(r#"git push --bookmark "team feature""#, selected()),
            FlowAction::Execute(vec![
                "git".to_string(),
                "push".to_string(),
                "--bookmark".to_string(),
                "team feature".to_string()
            ])
        );
    }

    #[test]
    fn reports_invalid_quoted_command_input() {
        assert_eq!(
            plan_command(r#"git push --bookmark "unterminated"#, selected()),
            FlowAction::Status("Invalid command quoting".to_string())
        );
    }
}
