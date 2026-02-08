//! CLI argument surface for launching `jk`.

use clap::Parser;

/// Raw startup command tokens passed to the interactive runtime.
///
/// The parser keeps trailing arguments untouched so `jk <command> -- <args...>` can be forwarded
/// to planner/execution without shell re-parsing.
#[derive(Debug, Parser)]
#[command(name = "jk", version, about = "Log-first jj TUI")]
pub struct Cli {
    #[arg(value_name = "COMMAND")]
    pub command: Option<String>,
    #[arg(value_name = "ARGS", trailing_var_arg = true)]
    pub args: Vec<String>,
}
