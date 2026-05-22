//! Process execution helpers for the `jj` CLI boundary.
//!
//! Higher layers decide which `ViewSpec` or direct argv to run. This module owns how those
//! commands inherit pager/color settings, execute `jj`, and turn exit status plus stdio into
//! app-usable errors or output values.

use std::process::Command;

use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::actions::CommandOutput;
use crate::jj::ViewSpec;
use crate::jj::command::{
    JJ_GIT_REMOTE_ARGS, JjCommand, NEW_TRUNK_ARGS, jj_command_args,
    resolve_exact_change_id_command_argv, workspace_root_command_args,
};
use crate::terminal_process::InteractiveCommand;

#[allow(dead_code)]
pub fn git_remotes() -> Result<Vec<String>> {
    let mut jj = Command::new("jj");
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

    parse_exact_change_id(&String::from_utf8(output.stdout)?)
        .map_err(|error| eyre!("{} {}", revset, error))
}

/// Run one rendered `jj` view command and return the raw process output.
///
/// Callers use this when the rendered stdout is itself the product surface, such as startup log
/// rows or detail-document loading.
pub fn run_jj(spec: &ViewSpec, color: ColorMode) -> Result<std::process::Output> {
    let label = spec.label();
    run_view_command(spec, &label, color, None, false)
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

pub fn run_jj_template_lines(
    spec: &ViewSpec,
    template: &str,
    no_graph: bool,
) -> Result<Vec<String>> {
    let output = run_view_command(
        spec,
        &format!("{} metadata", metadata_label(spec)),
        ColorMode::Never,
        Some(template),
        no_graph,
    )?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(str::to_owned).collect())
}

/// Run one `ViewSpec` through `jj` with optional template customization.
///
/// This keeps the `ViewSpec`-to-process boundary in one place so callers differ only in how they
/// interpret successful stdout.
fn run_view_command(
    spec: &ViewSpec,
    label: &str,
    color: ColorMode,
    template: Option<&str>,
    no_graph: bool,
) -> Result<std::process::Output> {
    let mut jj = base_command(color);
    jj.args(jj_command_args(spec, template, no_graph));

    let output = jj.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("{label} failed: {}", stderr.trim()));
    }
    Ok(output)
}

/// Label metadata-only loads that reuse the rendered-view command shape.
fn metadata_label(spec: &ViewSpec) -> String {
    if matches!(spec.command(), JjCommand::Resolve) {
        "jj log resolve metadata".to_owned()
    } else {
        spec.label().to_owned()
    }
}

pub fn base_command(color: ColorMode) -> Command {
    let mut jj = Command::new("jj");
    // Codex and users may set pager/color environment differently. The TUI
    // needs raw colored jj output so ratatui can render the same colors and
    // graph symbols the CLI would have produced.
    jj.arg("--no-pager")
        .args(["--color", color.as_arg()])
        .env_remove("NO_COLOR")
        .env_remove("PAGER");
    jj
}

pub fn summarize_output(stdout: &[u8], stderr: &[u8], fallback: &str) -> String {
    let mut parts = Vec::new();
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);

    if !stdout.trim().is_empty() {
        parts.push(stdout.trim().to_owned());
    }
    if !stderr.trim().is_empty() {
        parts.push(stderr.trim().to_owned());
    }

    if parts.is_empty() {
        fallback.to_owned()
    } else {
        parts.join(" | ")
    }
}

pub fn parse_exact_change_id(output: &str) -> Result<String> {
    let mut ids = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned);

    let Some(change_id) = ids.next() else {
        return Err(eyre!("did not resolve to any revisions"));
    };
    if ids.next().is_some() {
        return Err(eyre!("resolved to multiple revisions"));
    }

    Ok(change_id)
}

#[derive(Clone, Copy, Debug)]
pub enum ColorMode {
    /// Preserve jj color escapes so the TUI can render the same styled output.
    Always,
    /// Disable color when jk is parsing semantic helper output rather than rendering it directly.
    Never,
}

impl ColorMode {
    fn as_arg(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
        }
    }
}
