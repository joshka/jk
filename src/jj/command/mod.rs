//! `jj` command vocabulary and argv construction for `ViewSpec`.
//!
//! Higher layers choose a `ViewSpec`; this root keeps the shipped command
//! families visible while child modules own log-mode revset policy and argv
//! assembly quirks for rendered views or direct repository queries.

use crate::jj::ViewSpec;

mod argv;
mod log_mode;

#[cfg(test)]
pub use self::argv::CHANGE_ID_TEMPLATE;
pub use self::argv::{
    ALL_REPO_REVSET, JJ_GIT_REMOTE_ARGS, NEW_TRUNK_ARGS, OPERATION_LOG_LIMIT, RECENT_WORK_REVSET,
    TRUNK_WORK_REVSET, jj_command_args, option_value, resolve_exact_change_id_command_argv,
    workspace_root_command_args,
};
pub use self::log_mode::LogViewMode;

/// The shipped `jj` command families that can back a `jk` view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjCommand {
    /// Default log surface using jk's home revset behavior.
    Default,
    /// Explicit `jj log` startup or navigation surface.
    Log,
    /// `jj show` detail view.
    Show,
    /// `jj diff` detail view.
    Diff,
    /// `jj status` working-copy summary.
    Status,
    /// `jj resolve` conflict surface.
    Resolve,
    /// `jj file list` file-oriented listing surface.
    FileList,
    /// `jj file show` file detail surface.
    FileShow,
    /// `jj bookmark list` bookmark management surface.
    Bookmarks,
    /// `jj workspace list` workspace management surface.
    Workspaces,
    /// `jj operation log` history surface.
    OperationLog,
    /// `jj operation show` detail surface for one operation.
    OperationShow,
    /// `jj operation diff` detail surface for one operation diff.
    OperationDiff,
}

const BOOKMARK_COMMAND_WORDS: [&str; 2] = ["bookmark", "list"];
const WORKSPACE_LIST_COMMAND_WORDS: [&str; 2] = ["workspace", "list"];

impl JjCommand {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "jj",
            Self::Log => "jj log",
            Self::Show => "jj show",
            Self::Diff => "jj diff",
            Self::Status => "jj status",
            Self::Resolve => "jj resolve",
            Self::FileList => "jj file list",
            Self::FileShow => "jj file show",
            Self::Bookmarks => "jj bookmark list",
            Self::Workspaces => "jj workspace list",
            Self::OperationLog => "jj operation log",
            Self::OperationShow => "jj operation show",
            Self::OperationDiff => "jj operation diff",
        }
    }

    pub fn command_words(self) -> &'static [&'static str] {
        match self {
            Self::Default => &[],
            Self::Log => &["log"],
            Self::Show => &["show"],
            Self::Diff => &["diff"],
            Self::Status => &["status"],
            Self::Resolve => &["log"],
            Self::FileList => &["file", "list"],
            Self::FileShow => &["file", "show"],
            Self::Bookmarks => &BOOKMARK_COMMAND_WORDS,
            Self::Workspaces => &WORKSPACE_LIST_COMMAND_WORDS,
            Self::OperationLog => &["operation", "log"],
            Self::OperationShow => &["operation", "show"],
            Self::OperationDiff => &["operation", "diff"],
        }
    }

    pub fn prefix_args(self) -> &'static [&'static str] {
        match self {
            Self::OperationLog => &["--at-op=@", "--limit", OPERATION_LOG_LIMIT],
            Self::Default
            | Self::Log
            | Self::Show
            | Self::Diff
            | Self::Status
            | Self::Resolve
            | Self::FileList
            | Self::FileShow
            | Self::Bookmarks
            | Self::Workspaces
            | Self::OperationShow
            | Self::OperationDiff => &[],
        }
    }

    pub fn groups_log_items(self) -> bool {
        matches!(self, Self::Default | Self::Log)
    }
}

fn command_words(spec: &ViewSpec) -> &'static [&'static str] {
    spec.command().command_words()
}
