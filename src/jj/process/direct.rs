use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::actions::CommandOutput;
use crate::jj::command::{
    JJ_GIT_REMOTE_ARGS, NEW_TRUNK_ARGS, resolve_exact_change_id_command_argv,
    workspace_root_command_args,
};
use crate::terminal_process::InteractiveCommand;

use super::run::{ColorMode, base_command, summarize_output};

#[allow(dead_code)]
pub fn git_remotes() -> Result<Vec<String>> {
    let mut jj = std::process::Command::new("jj");
    jj.args(&JJ_GIT_REMOTE_ARGS[..]);

    let output = jj.output()?;
    if !output.status.success() {
        return Err(eyre!(
            "jj git remote list failed: {}",
            summarize_output(&output.stdout, &output.stderr, "could not list git remotes")
        ));
    }

    Ok(parse_git_remotes(std::str::from_utf8(&output.stdout)?))
}

#[allow(dead_code)]
pub fn parse_git_remotes(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| !name.is_empty())
        .fold(Vec::new(), |mut acc, name| {
            if !acc.iter().any(|existing| existing == name) {
                acc.push(name.to_owned());
            }
            acc
        })
}

pub fn new_trunk() -> Result<CommandOutput> {
    run_direct_command(&NEW_TRUNK_ARGS, "jj new trunk()", "created new change")
}

pub fn load_workspace_root() -> Result<String> {
    let mut jj = base_command(ColorMode::Never);
    jj.args(workspace_root_command_args());

    let output = jj.output()?;
    if !output.status.success() {
        return Err(eyre!(
            "jj root failed: {}",
            summarize_output(
                &output.stdout,
                &output.stderr,
                "could not find workspace root"
            )
        ));
    }

    let root = String::from_utf8(output.stdout)?.trim().to_owned();
    if root.is_empty() {
        return Err(eyre!("jj root returned an empty path"));
    }
    Ok(root)
}

pub fn resolve_exact_change_id(revset: &str) -> Result<String> {
    let mut jj = base_command(ColorMode::Never);
    jj.args(resolve_exact_change_id_command_argv(revset));

    let output = jj.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("{} failed: {}", revset, stderr.trim()));
    }

    super::run::parse_exact_change_id(&String::from_utf8(output.stdout)?)
        .map_err(|error| eyre!("{} {}", revset, error))
}

#[allow(dead_code)]
pub fn run_direct_args(
    args: Vec<String>,
    label: &str,
    success_fallback: &str,
) -> Result<CommandOutput> {
    let mut jj = base_command(ColorMode::Never);
    jj.args(args);

    let output = jj.output()?;
    if !output.status.success() {
        return Err(eyre!(
            "{} failed: {}",
            label,
            summarize_output(&output.stdout, &output.stderr, "command failed")
        ));
    }

    Ok(CommandOutput::new(summarize_output(
        &output.stdout,
        &output.stderr,
        success_fallback,
    )))
}

pub fn run_direct_args_stdout(args: Vec<String>, label: &str) -> Result<String> {
    let mut jj = base_command(ColorMode::Never);
    jj.args(args);

    let output = jj.output()?;
    if !output.status.success() {
        return Err(eyre!(
            "{} failed: {}",
            label,
            summarize_output(&output.stdout, &output.stderr, "command failed")
        ));
    }

    Ok(String::from_utf8(output.stdout)?)
}

#[allow(dead_code)]
pub fn interactive_jj_command(args: Vec<String>, label: &str) -> InteractiveCommand {
    let mut command = InteractiveCommand::new("jj", label);
    command.arg("--no-pager").args(args);
    command
}

fn run_direct_command(args: &[&str], label: &str, success_fallback: &str) -> Result<CommandOutput> {
    let mut jj = base_command(ColorMode::Never);
    jj.args(args);

    let output = jj.output()?;
    if !output.status.success() {
        return Err(eyre!(
            "{} failed: {}",
            label,
            summarize_output(&output.stdout, &output.stderr, "command failed")
        ));
    }

    Ok(CommandOutput::new(summarize_output(
        &output.stdout,
        &output.stderr,
        success_fallback,
    )))
}
