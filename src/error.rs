//! Error types shared across configuration loading, subprocess execution, and terminal runtime.

use std::io;

use thiserror::Error;

/// Top-level error type for `jk`.
///
/// Variants preserve enough context to render user-facing failure messages with the originating
/// path or subsystem.
#[derive(Debug, Error)]
pub enum JkError {
    #[error("failed to read configuration file `{path}`: {source}")]
    ConfigRead { path: String, source: io::Error },
    #[error("invalid keybinding configuration: {0}")]
    ConfigParse(String),
    #[error("failed to run `jj` command: {source}")]
    JjCommand { source: io::Error },
    #[error("terminal error: {0}")]
    Terminal(#[from] io::Error),
}
