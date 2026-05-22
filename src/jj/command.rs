//! `jj` command vocabulary and argv construction for `ViewSpec`.
//!
//! Higher layers choose a `ViewSpec`; this module decides which `jj` subcommand words, revset
//! presets, and command-shape quirks that spec implies.

use crate::jj::ViewSpec;

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

/// Named log revset modes that cycle within the default/log surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogViewMode {
    /// Home log mode used by the default startup surface.
    Default,
    /// Work relative to `trunk()`.
    Trunk,
    /// Recent mutable work plus the working copy and trunk.
    Recent,
    /// Repository-wide overview.
    All,
    /// User-provided explicit revset carried through the log view state.
    CustomRevset(String),
}

impl LogViewMode {
    pub fn label(&self) -> &str {
        match self {
            Self::Default => "default work",
            Self::Trunk => "trunk work",
            Self::Recent => "recent work",
            Self::All => "repo overview",
            Self::CustomRevset(_) => "custom revset",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Default => Self::Trunk,
            Self::Trunk => Self::Recent,
            Self::Recent => Self::All,
            Self::All | Self::CustomRevset(_) => Self::Default,
        }
    }

    pub fn from_spec(spec: &ViewSpec) -> Self {
        if spec.command() == JjCommand::Default {
            return Self::Default;
        }

        revset_from_log_args(spec.args())
            .map(Self::from_revset)
            .unwrap_or(Self::Default)
    }

    fn from_revset(revset: &str) -> Self {
        match revset {
            TRUNK_WORK_REVSET => Self::Trunk,
            RECENT_WORK_REVSET => Self::Recent,
            ALL_REPO_REVSET => Self::All,
            _ => Self::CustomRevset(revset.to_owned()),
        }
    }

    pub fn args(&self) -> Vec<String> {
        match self {
            Self::Default => Vec::new(),
            Self::Trunk => vec!["-r".to_owned(), TRUNK_WORK_REVSET.to_owned()],
            Self::Recent => vec!["-r".to_owned(), RECENT_WORK_REVSET.to_owned()],
            Self::All => vec!["-r".to_owned(), ALL_REPO_REVSET.to_owned()],
            Self::CustomRevset(revset) => vec!["-r".to_owned(), revset.clone()],
        }
    }
}

pub const TRUNK_WORK_REVSET: &str = "trunk().. | trunk()";
pub const RECENT_WORK_REVSET: &str = "latest(mutable(), 20) | @ | trunk()";
pub const ALL_REPO_REVSET: &str = "all()";
pub const JJ_GIT_REMOTE_ARGS: [&str; 3] = ["git", "remote", "list"];
pub const NEW_TRUNK_ARGS: [&str; 2] = ["new", "trunk()"];
const BOOKMARK_COMMAND_WORDS: [&str; 2] = ["bookmark", "list"];
const WORKSPACE_LIST_COMMAND_WORDS: [&str; 2] = ["workspace", "list"];
pub const CHANGE_ID_TEMPLATE: &str = "change_id ++ \"\\n\"";
pub const OPERATION_LOG_LIMIT: &str = "100";

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

    fn prefix_args(self) -> &'static [&'static str] {
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

fn revset_from_log_args(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revisions"], &["--revisions="])
}

pub fn workspace_root_command_args() -> Vec<String> {
    vec!["root".to_owned()]
}

pub fn resolve_exact_change_id_command_argv(revset: &str) -> Vec<String> {
    vec![
        "log".to_owned(),
        "--no-graph".to_owned(),
        "-r".to_owned(),
        revset.to_owned(),
        "-T".to_owned(),
        CHANGE_ID_TEMPLATE.to_owned(),
    ]
}

/// Build the `jj` argv for one `ViewSpec`, optionally overriding the template or graph mode.
pub fn jj_command_args(spec: &ViewSpec, template: Option<&str>, no_graph: bool) -> Vec<String> {
    let mut args = command_words(spec)
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect::<Vec<_>>();
    args.extend(
        spec.command()
            .prefix_args()
            .iter()
            .map(|arg| (*arg).to_owned()),
    );
    if no_graph {
        args.push("--no-graph".to_owned());
    }
    if let Some(template) = template {
        args.push("-T".to_owned());
        args.push(template.to_owned());
    }
    args.extend(spec.args().iter().cloned());
    args
}

fn command_words(spec: &ViewSpec) -> &'static [&'static str] {
    spec.command().command_words()
}

/// Find the value associated with a flag that may use either `--flag value` or `--flag=value`.
pub fn option_value<'a>(
    args: &'a [String],
    value_options: &[&str],
    value_prefixes: &[&str],
) -> Option<&'a str> {
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        if value_options.contains(&arg.as_str()) {
            return args.next().map(String::as_str);
        }
        if let Some(value) = value_prefixes
            .iter()
            .find_map(|prefix| arg.strip_prefix(prefix))
        {
            return Some(value);
        }
    }

    None
}
