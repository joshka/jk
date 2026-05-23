use std::process::Command;

use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::jj::ViewSpec;
use crate::jj::command::{
    JjCommand, jj_command_args, jj_command_args_with_template,
    jj_command_args_with_template_no_graph,
};

/// Preserve jj color escapes so the TUI can render the same styled output.
#[derive(Clone, Copy, Debug)]
pub enum ColorMode {
    Always,
    /// Disable color when jk is parsing semantic helper output rather than rendering it directly.
    Never,
}

#[derive(Clone, Copy, Debug)]
enum TemplateGraphStyle {
    Include,
    Omit,
}

impl ColorMode {
    fn as_arg(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
        }
    }
}

/// Run one rendered `jj` view command and return the raw process output.
///
/// Callers use this when the rendered stdout is itself the product surface, such as startup log
/// rows or detail-document loading.
pub fn run_jj(spec: &ViewSpec, color: ColorMode) -> Result<std::process::Output> {
    let label = spec.label();
    run_view_command(spec, &label, color, ViewCommandArgs::Default)
}

pub fn run_jj_template_lines(spec: &ViewSpec, template: &str) -> Result<Vec<String>> {
    run_jj_template_lines_with_style(spec, template, TemplateGraphStyle::Include)
}

pub fn run_jj_template_lines_no_graph(spec: &ViewSpec, template: &str) -> Result<Vec<String>> {
    run_jj_template_lines_with_style(spec, template, TemplateGraphStyle::Omit)
}

fn run_jj_template_lines_with_style(
    spec: &ViewSpec,
    template: &str,
    graph_style: TemplateGraphStyle,
) -> Result<Vec<String>> {
    let output = run_view_command(
        spec,
        &format!("{} metadata", metadata_label(spec)),
        ColorMode::Never,
        ViewCommandArgs::Template {
            template,
            graph_style,
        },
    )?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(str::to_owned).collect())
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

/// Run one `ViewSpec` through `jj` with optional template customization.
///
/// This keeps the `ViewSpec`-to-process boundary in one place so callers differ only in how they
/// interpret successful stdout.
enum ViewCommandArgs<'a> {
    Default,
    Template {
        template: &'a str,
        graph_style: TemplateGraphStyle,
    },
}

fn run_view_command(
    spec: &ViewSpec,
    label: &str,
    color: ColorMode,
    args: ViewCommandArgs<'_>,
) -> Result<std::process::Output> {
    let mut jj = base_command(color);
    match args {
        ViewCommandArgs::Default => jj.args(jj_command_args(spec)),
        ViewCommandArgs::Template {
            template,
            graph_style: TemplateGraphStyle::Include,
        } => jj.args(jj_command_args_with_template(spec, template)),
        ViewCommandArgs::Template {
            template,
            graph_style: TemplateGraphStyle::Omit,
        } => jj.args(jj_command_args_with_template_no_graph(spec, template)),
    };

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
