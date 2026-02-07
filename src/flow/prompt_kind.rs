use super::builders::{
    build_bookmark_names_command, build_bookmark_rename_command, build_bookmark_target_command,
    build_file_chmod_command, build_file_track_command, build_file_untrack_command,
    build_tag_delete_command, build_tag_set_command, build_track_command,
    build_workspace_add_command, build_workspace_forget_command,
};

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
    MetaeditMessage {
        revision: String,
    },
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
            Self::MetaeditMessage { revision } => {
                if input.is_empty() {
                    Err("metadata message is required".to_string())
                } else {
                    Ok(vec![
                        "metaedit".to_string(),
                        "-m".to_string(),
                        input.to_string(),
                        revision.to_string(),
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
