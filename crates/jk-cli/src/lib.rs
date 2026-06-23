//! `jj` process integration for `jk`.
//!
//! The current MVP needs two views of the same `jj` command:
//!
//! - rendered terminal output that keeps the user's configured template, graph, revset, and colors;
//! - semantic records that let the TUI move by change, preserve selection, and expand the selected
//!   description.
//!
//! `JjLog` provides that bridge by running `jj` as a child process. `JjDiff` follows the same
//! rendered-output-first boundary for selected-change inspection. This is a temporary integration
//! boundary until `jj-cli` / `jj-lib` can provide both pieces without parsing command output.

mod command;

pub mod describe;
pub mod diff;
pub mod evolog;
pub mod log;
pub mod show;
pub mod status;
pub mod workspaces;

pub use command::{JjCommandRunner, RecordingJjCommandRunner, SystemJjCommandRunner};
pub use describe::{DescribeQuery, JjDescribe};
pub use diff::{DiffFormat, DiffQuery, JjDiff, JjDiffError};
pub use evolog::{EvologQuery, JjEvolog, JjEvologError};
pub use log::{JjLog, JjLogCommand, JjLogError, LogTemplateSelection};
pub use show::{JjShow, JjShowError, ShowQuery};
pub use status::{JjStatus, JjStatusError, StatusQuery};
pub use workspaces::{
    JjWorkspaces, JjWorkspacesError, WorkspaceInspectionQuery, WorkspaceListParseError,
    WorkspaceListSnapshot, WorkspaceSummary,
};
