//! `jj` process integration for `jk`.
//!
//! The log-first MVP needs two views of the same `jj` command:
//!
//! - rendered terminal output that keeps the user's configured template, graph, revset, and colors;
//! - semantic records that let the TUI move by change, preserve selection, and expand the selected
//!   description.
//!
//! `JjLog` provides that bridge by running `jj` as a child process. `JjDiff` follows the same
//! rendered-output-first boundary for selected-change inspection. This is a temporary integration
//! boundary until `jj-cli` / `jj-lib` can provide both pieces without parsing command output.

pub mod diff;
pub mod log;

pub use diff::{JjDiff, JjDiffError};
pub use log::{JjLog, JjLogCommand, JjLogError};
