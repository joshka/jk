//! `jj` subprocess execution helpers.
//!
//! All command execution goes through this module so `--no-pager` and color policy remain
//! consistent across runtime, previews, and metadata lookups.

use std::process::Command;

use crate::error::JkError;

/// Captured subprocess output and success metadata for one `jj` invocation.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Tokens passed after `jj --no-pager`.
    pub command: Vec<String>,
    /// Renderable output lines from combined stdout/stderr.
    pub output: Vec<String>,
    /// True when the process exits with status code 0.
    pub success: bool,
}

/// Run `jj` with color-enabled output for terminal rendering flows.
pub fn run(tokens: &[String]) -> Result<CommandResult, JkError> {
    run_with_color(tokens, "always")
}

/// Run `jj` with color disabled for parser-friendly output processing.
pub fn run_plain(tokens: &[String]) -> Result<CommandResult, JkError> {
    run_with_color(tokens, "never")
}

/// Execute `jj` and normalize output into a line-oriented result.
///
/// If either stream is empty, this returns the non-empty stream; otherwise both streams are joined
/// so users can see warnings/errors next to regular output.
fn run_with_color(tokens: &[String], color: &str) -> Result<CommandResult, JkError> {
    let mut command = Command::new("jj");
    command.arg("--no-pager");
    command.arg("--color");
    command.arg(color);
    command.args(tokens);
    // Keep color output deterministic in non-interactive capture contexts (for example VHS).
    command.env_remove("NO_COLOR");
    command.env("CLICOLOR_FORCE", "1");
    command.env("COLORTERM", "truecolor");
    command.env("TERM", "xterm-256color");

    let output = command
        .output()
        .map_err(|source| JkError::JjCommand { source })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let body = if stdout.trim().is_empty() {
        stderr.to_string()
    } else if stderr.trim().is_empty() {
        stdout.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };

    let lines = if body.trim().is_empty() {
        vec!["(no output)".to_string()]
    } else {
        body.lines().map(ToString::to_string).collect()
    };

    Ok(CommandResult {
        command: tokens.to_vec(),
        output: lines,
        success: output.status.success(),
    })
}
