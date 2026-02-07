use std::process::Command;

use crate::error::JkError;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: Vec<String>,
    pub output: Vec<String>,
    pub success: bool,
}

pub fn run(tokens: &[String]) -> Result<CommandResult, JkError> {
    let mut command = Command::new("jj");
    command.arg("--no-pager");
    command.args(tokens);

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
