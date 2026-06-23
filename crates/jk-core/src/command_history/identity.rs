use std::ffi::OsString;
use std::path::PathBuf;

use super::redaction::{redact_argv, redact_text};
use crate::command::preview_argv;
use crate::{ExecutionMode, GlobalOptions, JjCommandSpec};

/// Command data captured from a typed command spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandIdentity {
    /// Exact process argv after global options are applied, excluding the `jj` binary.
    pub argv: Vec<OsString>,
    /// Display-only preview from the command spec.
    pub spec_preview: String,
    /// Broad command family for filtering and grouping.
    pub command_family: CommandFamily,
    /// Human-readable title from the command spec.
    pub title: String,
}

impl CommandIdentity {
    /// Captures command identity from a typed command spec.
    #[must_use]
    pub fn from_spec(spec: &JjCommandSpec) -> Self {
        Self {
            argv: redact_argv(spec.process_argv()),
            spec_preview: redact_text(&spec.preview()).0,
            command_family: CommandFamily::from_spec(spec),
            title: redact_text(spec.title()).0,
        }
    }

    /// Returns the exact redacted process command line captured for this command.
    #[must_use]
    pub fn process_preview(&self) -> String {
        preview_argv(&self.argv)
    }
}

/// Broad command family for history filtering.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CommandFamily {
    /// Bare `jj` or configured default command.
    JjDefault,
    /// `jj log`.
    JjLog,
    /// `jj diff`.
    JjDiff,
    /// `jj show`.
    JjShow,
    /// `jj status`.
    JjStatus,
    /// `jj describe`.
    JjDescribe,
    /// `jj evolog`.
    JjEvolog,
    /// `jj workspace ...`.
    JjWorkspace,
    /// Future `jj op ...`.
    JjOperation,
    /// Future user-entered `:` command.
    UserJjCommand,
    /// Future foreground external command.
    ExternalCommand,
    /// A command family not yet modeled.
    Other(String),
}

impl CommandFamily {
    fn from_spec(spec: &JjCommandSpec) -> Self {
        if spec.mode() == ExecutionMode::CommandMode {
            return Self::UserJjCommand;
        }

        let Some(command) = spec.argv().first() else {
            return Self::JjDefault;
        };
        match command.to_string_lossy().as_ref() {
            "log" => Self::JjLog,
            "diff" => Self::JjDiff,
            "show" => Self::JjShow,
            "status" => Self::JjStatus,
            "describe" => Self::JjDescribe,
            "evolog" => Self::JjEvolog,
            "workspace" => Self::JjWorkspace,
            "op" => Self::JjOperation,
            other => Self::Other(other.to_owned()),
        }
    }
}

/// The view and action that caused a command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSource {
    /// Source view that owned the command action.
    pub view: SourceView,
    /// Source action that triggered the command.
    pub action: SourceAction,
    /// Optional key binding or command key.
    pub key: Option<String>,
}

impl CommandSource {
    /// Creates a command source without a key binding.
    #[must_use]
    pub const fn new(view: SourceView, action: SourceAction) -> Self {
        Self {
            view,
            action,
            key: None,
        }
    }

    /// Adds the key binding that triggered this source action.
    #[must_use]
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}

/// App view that triggered a command.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SourceView {
    /// Log view.
    Log,
    /// Diff view.
    Diff,
    /// Show view.
    Show,
    /// Status view.
    Status,
    /// Evolog view.
    Evolog,
    /// Workspaces list.
    Workspaces,
    /// Selected workspace status view.
    WorkspaceStatus,
    /// Selected workspace diff view.
    WorkspaceDiff,
    /// Command history view.
    CommandHistory,
    /// Operation log view.
    OperationLog,
    /// Operation show view.
    OperationShow,
    /// Operation diff view.
    OperationDiff,
    /// A source view not yet modeled.
    Other(String),
}

/// App action that triggered a command.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SourceAction {
    /// Initial view load.
    InitialLoad,
    /// Explicit refresh.
    Refresh,
    /// Open a diff.
    OpenDiff,
    /// Open a show view.
    OpenShow,
    /// Open a status view.
    OpenStatus,
    /// Open an evolog view.
    OpenEvolog,
    /// Describe the selected revision.
    DescribeRevision,
    /// Abandon the selected revision.
    AbandonRevision,
    /// List workspaces.
    WorkspaceList,
    /// Show selected workspace status.
    WorkspaceStatus,
    /// Show selected workspace diff.
    WorkspaceDiff,
    /// Run selected workspace update-stale.
    WorkspaceUpdateStale,
    /// List repository operations.
    OperationLog,
    /// Show a selected operation.
    OperationShow,
    /// Diff a selected operation.
    OperationDiff,
    /// Undo the latest operation.
    Undo,
    /// Redo the latest undone operation.
    Redo,
    /// Run a user-entered `jj` command.
    UserJjCommand,
    /// A source action not yet modeled.
    Other(String),
}

/// Context captured when a command starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandExecutionContext {
    /// Process working directory, if set by the command spec.
    pub cwd: Option<PathBuf>,
    /// Repository path from global `jj` options, if set.
    pub repository: Option<PathBuf>,
    /// Redacted snapshot of global `jj` options.
    pub global_options: GlobalOptionsSnapshot,
}

impl CommandExecutionContext {
    /// Captures execution context from a typed command spec.
    #[must_use]
    pub fn from_spec(spec: &JjCommandSpec) -> Self {
        Self {
            cwd: spec.cwd().map(PathBuf::from),
            repository: spec.repository().map(PathBuf::from),
            global_options: GlobalOptionsSnapshot::from_global_options(spec.global_options()),
        }
    }
}

/// Redacted global `jj` options retained by command history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalOptionsSnapshot {
    /// Global argv rendered before the command family, excluding the `jj` binary.
    pub argv: Vec<OsString>,
}

impl GlobalOptionsSnapshot {
    /// Captures redacted global argv from typed global options.
    #[must_use]
    pub fn from_global_options(options: &GlobalOptions) -> Self {
        Self {
            argv: redact_argv(options.argv()),
        }
    }
}
