//! `jj` process integration for `jk`.
//!
//! [`JjLog`] and [`JjDiff`] expose rendered terminal output alongside structured records for
//! navigation. Log rendering retains the user's configured template, graph, revset, and colors;
//! semantic records let the TUI move by change and recover selection after refresh.
//!
//! Command modules provide queries for inspection, mutations, workspaces, bookmarks, and remotes.
//! Their command specifications carry arguments and execution policies into [`JjCommandRunner`].
//! The system runners launch `jj`; optional [`RecordingJjCommandRunner`] wrappers retain command
//! history. External programs use the separate [`ExternalCommandRunner`] interface.
//!
//! Parsers expose the metadata needed by each workflow. Bookmark lists and abandon's embedded
//! Git-format patch have their own formats; they do not inherit every configured jj presentation
//! option.

mod command;

pub mod abandon;
pub mod bookmarks;
pub mod describe;
pub mod diff;
pub mod edit;
pub mod evolog;
pub mod git_remote;
pub mod log;
pub mod new;
pub mod operation;
pub mod rebase;
pub mod recovery;
pub mod restore;
pub mod show;
pub mod squash;
pub mod status;
pub mod workspaces;

pub use abandon::{
    AbandonDetails, AbandonDetailsError, AbandonFile, AbandonProbeError, AbandonQuery, JjAbandon,
};
pub use bookmarks::{
    BookmarkMutation, BookmarkParseError, BookmarkRef, BookmarkSnapshot, JjBookmarks,
    JjBookmarksError, parse_bookmark_list,
};
pub use command::{
    CancellableSystemJjCommandRunner, CancellationToken, ExternalCommandRunner, JjCommandRunner,
    RecordingExternalCommandRunner, RecordingJjCommandRunner, SystemExternalCommandRunner,
    SystemJjCommandRunner,
};
pub use describe::{DescribeQuery, JjDescribe};
pub use diff::{DiffFormat, DiffQuery, JjDiff, JjDiffError};
pub use edit::{EditQuery, JjEdit};
pub use evolog::{EvologQuery, JjEvolog, JjEvologError};
pub use git_remote::{
    GitRemote, GitRemoteParseError, JjGitRemote, JjGitRemoteError, parse_remote_list,
};
pub use log::{JjLog, JjLogCommand, JjLogError, LogTemplateSelection};
pub use new::{JjNew, NewQuery};
pub use operation::{JjOperation, JjOperationError, OperationQuery};
pub use rebase::{
    JjRebase, RebaseDestinationRole, RebaseQuery, RebaseQueryError, RebaseSourceRole,
};
pub use recovery::{JjRecovery, RecoveryCommand};
pub use restore::{JjRestore, RestoreQuery};
pub use show::{JjShow, JjShowError, ShowQuery};
pub use squash::{JjSquash, SquashQuery};
pub use status::{JjStatus, JjStatusError, StatusQuery};
pub use workspaces::{
    JjWorkspaces, JjWorkspacesError, WorkspaceInspectionQuery, WorkspaceListParseError,
    WorkspaceListSnapshot, WorkspaceSummary,
};
