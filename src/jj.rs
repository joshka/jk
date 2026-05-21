//! Command construction and output loading for the `jj` CLI.
//!
//! `jk` intentionally treats `jj`'s rendered terminal output as the product
//! contract. Shelling out keeps user config, templates, graph symbols, colors,
//! and future jj formatting behavior aligned with the CLI instead of rebuilding
//! a parallel view from repository data.

use std::process::Command;

use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::interactive_process::InteractiveCommand;
use crate::jj_actions::CommandOutput;

mod view_spec;

pub use view_spec::{DiffFormat, ViewSpec};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjCommand {
    Default,
    Log,
    Show,
    Diff,
    Status,
    Resolve,
    FileList,
    FileShow,
    Bookmarks,
    Workspaces,
    OperationLog,
    OperationShow,
    OperationDiff,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogViewMode {
    Default,
    Trunk,
    Recent,
    All,
    CustomRevset(String),
}

impl LogViewMode {
    pub fn label(&self) -> &str {
        match self {
            Self::Default => "default work",
            Self::Trunk => "trunk work",
            Self::Recent => "recent work",
            Self::All => "repo overview",
            Self::CustomRevset(_) => "custom revset",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Default => Self::Trunk,
            Self::Trunk => Self::Recent,
            Self::Recent => Self::All,
            Self::All | Self::CustomRevset(_) => Self::Default,
        }
    }

    pub fn from_spec(spec: &ViewSpec) -> Self {
        if spec.command == JjCommand::Default {
            return Self::Default;
        }

        revset_from_log_args(&spec.args)
            .map(Self::from_revset)
            .unwrap_or(Self::Default)
    }

    fn from_revset(revset: &str) -> Self {
        match revset {
            TRUNK_WORK_REVSET => Self::Trunk,
            RECENT_WORK_REVSET => Self::Recent,
            ALL_REPO_REVSET => Self::All,
            _ => Self::CustomRevset(revset.to_owned()),
        }
    }

    fn args(&self) -> Vec<String> {
        match self {
            Self::Default => Vec::new(),
            Self::Trunk => vec!["-r".to_owned(), TRUNK_WORK_REVSET.to_owned()],
            Self::Recent => vec!["-r".to_owned(), RECENT_WORK_REVSET.to_owned()],
            Self::All => vec!["-r".to_owned(), ALL_REPO_REVSET.to_owned()],
            Self::CustomRevset(revset) => vec!["-r".to_owned(), revset.clone()],
        }
    }
}

const TRUNK_WORK_REVSET: &str = "trunk().. | trunk()";
const RECENT_WORK_REVSET: &str = "latest(mutable(), 20) | @ | trunk()";
const ALL_REPO_REVSET: &str = "all()";
const JJ_GIT_REMOTE_ARGS: [&str; 3] = ["git", "remote", "list"];
const NEW_TRUNK_ARGS: [&str; 2] = ["new", "trunk()"];
const BOOKMARK_COMMAND_WORDS: [&str; 2] = ["bookmark", "list"];
const WORKSPACE_LIST_COMMAND_WORDS: [&str; 2] = ["workspace", "list"];
const CHANGE_ID_TEMPLATE: &str = "change_id ++ \"\\n\"";
const OPERATION_LOG_LIMIT: &str = "100";

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
fn parse_git_remotes(stdout: &str) -> Vec<String> {
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

impl JjCommand {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "jj",
            Self::Log => "jj log",
            Self::Show => "jj show",
            Self::Diff => "jj diff",
            Self::Status => "jj status",
            Self::Resolve => "jj resolve",
            Self::FileList => "jj file list",
            Self::FileShow => "jj file show",
            Self::Bookmarks => "jj bookmark list",
            Self::Workspaces => "jj workspace list",
            Self::OperationLog => "jj operation log",
            Self::OperationShow => "jj operation show",
            Self::OperationDiff => "jj operation diff",
        }
    }

    fn command_words(self) -> &'static [&'static str] {
        match self {
            Self::Default => &[],
            Self::Log => &["log"],
            Self::Show => &["show"],
            Self::Diff => &["diff"],
            Self::Status => &["status"],
            Self::Resolve => &["log"],
            Self::FileList => &["file", "list"],
            Self::FileShow => &["file", "show"],
            Self::Bookmarks => &BOOKMARK_COMMAND_WORDS,
            Self::Workspaces => &WORKSPACE_LIST_COMMAND_WORDS,
            Self::OperationLog => &["operation", "log"],
            Self::OperationShow => &["operation", "show"],
            Self::OperationDiff => &["operation", "diff"],
        }
    }

    fn prefix_args(self) -> &'static [&'static str] {
        match self {
            Self::OperationLog => &["--at-op=@", "--limit", OPERATION_LOG_LIMIT],
            Self::Default
            | Self::Log
            | Self::Show
            | Self::Diff
            | Self::Status
            | Self::Resolve
            | Self::FileList
            | Self::FileShow
            | Self::Bookmarks
            | Self::Workspaces
            | Self::OperationShow
            | Self::OperationDiff => &[],
        }
    }

    pub(crate) fn groups_log_items(self) -> bool {
        matches!(self, Self::Default | Self::Log)
    }
}

fn revset_from_log_args(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revisions"], &["--revisions="])
}

pub fn new_trunk() -> Result<CommandOutput> {
    run_direct_command(&NEW_TRUNK_ARGS, "jj new trunk()", "created new change")
}

pub(crate) fn load_workspace_root() -> Result<String> {
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

fn workspace_root_command_args() -> Vec<String> {
    vec!["root".to_owned()]
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

fn resolve_exact_change_id_command_argv(revset: &str) -> Vec<String> {
    vec![
        "log".to_owned(),
        "--no-graph".to_owned(),
        "-r".to_owned(),
        revset.to_owned(),
        "-T".to_owned(),
        CHANGE_ID_TEMPLATE.to_owned(),
    ]
}

pub(crate) fn run_jj(spec: &ViewSpec, color: ColorMode) -> Result<std::process::Output> {
    let mut jj = base_command(color);
    jj.args(jj_command_args(spec, None, false));

    let output = jj.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("{} failed: {}", spec.label(), stderr.trim()));
    }
    Ok(output)
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
pub(crate) fn run_direct_args(
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

pub(crate) fn run_direct_args_stdout(args: Vec<String>, label: &str) -> Result<String> {
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
pub(crate) fn interactive_jj_command(args: Vec<String>, label: &str) -> InteractiveCommand {
    let mut command = InteractiveCommand::new("jj", label);
    command.arg("--no-pager").args(args);
    command
}

pub(crate) fn run_jj_template_lines(
    spec: &ViewSpec,
    template: &str,
    no_graph: bool,
) -> Result<Vec<String>> {
    let mut jj = base_command(ColorMode::Never);
    jj.args(jj_command_args(spec, Some(template), no_graph));

    let output = jj.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let metadata_label = if matches!(spec.command(), JjCommand::Resolve) {
            "jj log resolve metadata".to_owned()
        } else {
            spec.label().to_owned()
        };

        return Err(eyre!(
            "{} metadata failed: {}",
            metadata_label,
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(str::to_owned).collect())
}

fn jj_command_args(spec: &ViewSpec, template: Option<&str>, no_graph: bool) -> Vec<String> {
    let mut args = command_words(spec)
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect::<Vec<_>>();
    args.extend(
        spec.command
            .prefix_args()
            .iter()
            .map(|arg| (*arg).to_owned()),
    );
    if no_graph {
        args.push("--no-graph".to_owned());
    }
    if let Some(template) = template {
        args.push("-T".to_owned());
        args.push(template.to_owned());
    }
    args.extend(spec.args.iter().cloned());
    args
}

fn command_words(spec: &ViewSpec) -> &'static [&'static str] {
    spec.command.command_words()
}

pub(crate) fn base_command(color: ColorMode) -> Command {
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

pub(crate) fn summarize_output(stdout: &[u8], stderr: &[u8], fallback: &str) -> String {
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

fn parse_exact_change_id(output: &str) -> Result<String> {
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
pub(crate) enum ColorMode {
    Always,
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

fn option_value<'a>(
    args: &'a [String],
    value_options: &[&str],
    value_prefixes: &[&str],
) -> Option<&'a str> {
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        if value_options.contains(&arg.as_str()) {
            return args.next().map(String::as_str);
        }
        if let Some(value) = value_prefixes
            .iter()
            .find_map(|prefix| arg.strip_prefix(prefix))
        {
            return Some(value);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;
    use crate::jj_actions::JjGitFetch;
    use crate::operation_log::OPERATION_ID_TEMPLATE;
    use crate::resolve::RESOLVE_CONFLICT_TEMPLATE;
    use crate::workspaces::WORKSPACE_METADATA_TEMPLATE;

    #[test]
    fn bookmark_list_command_uses_bookmark_words_and_labels() {
        let spec = ViewSpec::bookmarks(vec!["--revision".to_owned(), "main".to_owned()]);

        assert_eq!(spec.command(), JjCommand::Bookmarks);
        assert_eq!(
            jj_command_args(&spec, None, false),
            vec!["bookmark", "list", "--revision", "main"]
        );
        assert_eq!(spec.label(), "jj bookmark list --revision main");
        assert_eq!(spec.app_label(), "jk bookmarks --revision main");
    }

    #[test]
    fn workspace_commands_use_read_only_root_list_and_metadata_template() {
        let spec = ViewSpec::workspaces(Vec::new());

        assert_eq!(workspace_root_command_args(), vec!["root"]);
        assert_eq!(spec.command(), JjCommand::Workspaces);
        assert_eq!(
            jj_command_args(&spec, None, false),
            vec!["workspace", "list"]
        );
        assert_eq!(
            jj_command_args(&spec, Some(WORKSPACE_METADATA_TEMPLATE), false),
            vec!["workspace", "list", "-T", WORKSPACE_METADATA_TEMPLATE,]
        );
        assert!(!WORKSPACE_METADATA_TEMPLATE.contains("root"));
        assert_eq!(spec.label(), "jj workspace list");
        assert_eq!(spec.app_label(), "jk workspaces");
    }

    #[test]
    fn file_list_command_uses_file_words_and_keeps_selected_path_out_of_args() {
        let spec = ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned()));

        assert_eq!(spec.command(), JjCommand::FileList);
        assert_eq!(spec.args(), ["-r", "main"]);
        assert_eq!(spec.path(), Some("src/main.rs"));
        assert_eq!(spec.exact_change_target(), None);
        assert_eq!(
            jj_command_args(&spec, None, false),
            vec!["file", "list", "-r", "main"]
        );
        assert_eq!(spec.label(), "jj file list -r main");
        assert_eq!(spec.app_label(), "jk file list -r main");
        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
        assert_eq!(spec.show_context_revset(), "main");
    }

    #[test]
    fn file_show_command_keeps_exact_path_identity() {
        let spec = ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned());

        assert_eq!(spec.command(), JjCommand::FileShow);
        assert_eq!(spec.args(), ["-r", "main", "src/main.rs"]);
        assert_eq!(spec.path(), Some("src/main.rs"));
        assert_eq!(spec.exact_change_target(), None);
        assert_eq!(
            jj_command_args(&spec, None, false),
            vec!["file", "show", "-r", "main", "src/main.rs"]
        );
        assert_eq!(spec.label(), "jj file show -r main src/main.rs");
        assert_eq!(spec.app_label(), "jk file show -r main src/main.rs");
        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
        assert_eq!(spec.show_context_revset(), "main");
    }

    #[test]
    fn resolve_command_defaults_to_current_revision() {
        let spec = ViewSpec::resolve(None);

        assert_eq!(spec.command(), JjCommand::Resolve);
        assert_eq!(spec.args(), ["-r", "@"]);
        assert_eq!(
            jj_command_args(&spec, Some(RESOLVE_CONFLICT_TEMPLATE), true),
            vec![
                "log",
                "--no-graph",
                "-T",
                RESOLVE_CONFLICT_TEMPLATE,
                "-r",
                "@",
            ]
        );
        assert_eq!(spec.label(), "jj resolve -r @");
        assert_eq!(spec.app_label(), "jk resolve -r @");
        assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
        assert_eq!(spec.show_context_revset(), "@");
    }

    #[test]
    fn resolve_command_uses_log_template_contract_without_graph() {
        let spec = ViewSpec::resolve(Some("main".to_owned()));

        assert_eq!(spec.command(), JjCommand::Resolve);
        assert_eq!(spec.args(), ["-r", "main"]);
        assert_eq!(spec.exact_change_target(), None);
        assert_eq!(
            jj_command_args(&spec, Some(RESOLVE_CONFLICT_TEMPLATE), true),
            vec![
                "log",
                "--no-graph",
                "-T",
                RESOLVE_CONFLICT_TEMPLATE,
                "-r",
                "main",
            ]
        );
        assert_eq!(spec.label(), "jj resolve -r main");
        assert_eq!(spec.app_label(), "jk resolve -r main");
        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
        assert_eq!(spec.show_context_revset(), "main");
    }

    #[test]
    fn operation_log_command_uses_at_op_prefix() {
        assert_eq!(
            jj_command_args(
                &ViewSpec::new(JjCommand::OperationLog, Vec::new()),
                None,
                false
            ),
            vec![
                "operation",
                "log",
                "--at-op=@",
                "--limit",
                OPERATION_LOG_LIMIT
            ]
        );
    }

    #[test]
    fn operation_log_id_command_disables_graph_for_template_output() {
        assert_eq!(
            jj_command_args(
                &ViewSpec::new(JjCommand::OperationLog, Vec::new()),
                Some(OPERATION_ID_TEMPLATE),
                true
            ),
            vec![
                "operation",
                "log",
                "--at-op=@",
                "--limit",
                OPERATION_LOG_LIMIT,
                "--no-graph",
                "-T",
                OPERATION_ID_TEMPLATE,
            ]
        );
    }

    #[test]
    fn operation_show_command_uses_positional_operation_id() {
        let spec = ViewSpec::operation_show(operation_id('a'));

        assert_eq!(spec.command(), JjCommand::OperationShow);
        assert_eq!(spec.args(), [operation_id('a')]);
        assert_eq!(
            jj_command_args(&spec, None, false),
            vec!["operation", "show", operation_id('a').as_str()]
        );
        assert_eq!(spec.app_label(), "jk operation show aaaaaaaa");
    }

    #[test]
    fn operation_diff_command_uses_operation_option() {
        let spec = ViewSpec::operation_diff(operation_id('b'));

        assert_eq!(spec.command(), JjCommand::OperationDiff);
        assert_eq!(spec.args(), ["--operation", operation_id('b').as_str()]);
        assert_eq!(
            jj_command_args(&spec, None, false),
            vec![
                "operation",
                "diff",
                "--operation",
                operation_id('b').as_str()
            ]
        );
        assert_eq!(spec.app_label(), "jk operation diff --operation bbbbbbbb");
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }

    #[test]
    fn log_view_mode_parses_custom_revset_from_log_spec() {
        let spec = ViewSpec::new(JjCommand::Log, vec!["-r".to_owned(), "::".to_owned()]);

        assert_eq!(
            LogViewMode::from_spec(&spec),
            LogViewMode::CustomRevset("::".to_owned())
        );
    }

    #[test]
    fn log_view_mode_recognizes_named_recent_revset() {
        let spec = ViewSpec::new(
            JjCommand::Log,
            vec!["-r".to_owned(), RECENT_WORK_REVSET.to_owned()],
        );

        assert_eq!(LogViewMode::from_spec(&spec), LogViewMode::Recent);
    }

    #[test]
    fn fetch_command_args_are_stable() {
        assert_eq!(
            JjGitFetch::default_remotes().command_argv(),
            vec!["git", "fetch"]
        );
        assert_eq!(
            JjGitFetch::for_remote("origin").command_argv(),
            vec!["git", "fetch", "--remote", "exact:\"origin\""]
        );
    }

    #[test]
    fn interactive_jj_command_inherits_stdio_and_keeps_no_pager() {
        let command = interactive_jj_command(
            vec![
                "split".to_owned(),
                "--revision".to_owned(),
                "exactly(change_id(\"abc\"), 1)".to_owned(),
            ],
            "jj split",
        );

        assert_eq!(command.program(), OsStr::new("jj"));
        assert_eq!(
            command.argv(),
            vec![
                OsStr::new("--no-pager"),
                OsStr::new("split"),
                OsStr::new("--revision"),
                OsStr::new("exactly(change_id(\"abc\"), 1)"),
            ]
        );
        assert_eq!(
            command.stdio_intent(),
            crate::interactive_process::StdioIntent::Inherit
        );
    }

    #[test]
    fn new_trunk_command_args_are_stable() {
        assert_eq!(NEW_TRUNK_ARGS, ["new", "trunk()"]);
    }

    #[test]
    fn parses_git_remotes_from_jj_remote_list_output() {
        let stdout = "origin https://example.com/repo.git\nupstream git@github.com:org/repo.git\n";
        assert_eq!(parse_git_remotes(stdout), vec!["origin", "upstream"]);
    }

    #[test]
    fn summarize_output_prefers_real_output_over_fallback() {
        assert_eq!(
            summarize_output(b"fetched origin\n", b"", "fetched"),
            "fetched origin"
        );
        assert_eq!(summarize_output(b"", b"warning\n", "fetched"), "warning");
        assert_eq!(summarize_output(b"", b"", "fetched"), "fetched");
    }

    #[test]
    fn parse_exact_change_id_requires_exactly_one_result() {
        assert_eq!(parse_exact_change_id("abc\n").unwrap(), "abc");
        assert!(parse_exact_change_id("").is_err());
        assert!(parse_exact_change_id("abc\ndef\n").is_err());
    }

    #[test]
    fn parse_exact_change_id_rejects_graph_like_output() {
        assert!(parse_exact_change_id("@ abcdefghijkl\n│  some graph suffix").is_err());
    }

    #[test]
    fn resolve_exact_change_id_command_uses_no_graph_contract() {
        assert_eq!(
            resolve_exact_change_id_command_argv("main"),
            vec!["log", "--no-graph", "-r", "main", "-T", CHANGE_ID_TEMPLATE,]
        );
    }
}
