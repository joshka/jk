use std::io;

use thiserror::Error;

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
