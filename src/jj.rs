//! Command construction and output loading for the `jj` CLI.
//!
//! `jk` intentionally treats `jj`'s rendered terminal output as the product
//! contract. Shelling out keeps user config, templates, graph symbols, colors,
//! and future jj formatting behavior aligned with the CLI instead of rebuilding
//! a parallel view from repository data.

use std::process::Command;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use ratatui::text::Line;
use serde_json::Value;

#[allow(unused_imports)]
pub use crate::jj_actions::JjOperationTargetKind;
pub use crate::jj_actions::{
    CommandOutput, JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind,
    JjBookmarkMutationPlan, JjBookmarkTarget, JjCommitPlan, JjDescribePlan, JjDescribeTarget,
    JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery, JjOperationRecoveryKind,
    JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSquashPlan,
    JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan,
};

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
const FETCH_ARGS: [&str; 2] = ["git", "fetch"];
const NEW_TRUNK_ARGS: [&str; 2] = ["new", "trunk()"];
const BOOKMARK_COMMAND_WORDS: [&str; 2] = ["bookmark", "list"];
const BOOKMARK_METADATA_TEMPLATE: &str = r#"name ++ "\t" ++ if(remote, remote, "") ++ "\t" ++ if(normal_target, normal_target.change_id(), "") ++ "\t" ++ if(normal_target, normal_target.commit_id(), "") ++ "\n""#;
const CHANGE_ID_TEMPLATE: &str = "change_id ++ \"\\n\"";
const OPERATION_ID_TEMPLATE: &str = "self.id() ++ \"\\n\"";
const OPERATION_LOG_LIMIT: &str = "100";
const RESOLVE_CONFLICT_TEMPLATE: &str = r#"self.conflicted_files().map(|entry| "{\"path\":" ++ json(entry.path()) ++ ",\"file_type\":" ++ json(entry.file_type()) ++ ",\"side_count\":" ++ json(entry.conflict_side_count()) ++ "}\n").join("")"#;

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
            | Self::OperationShow
            | Self::OperationDiff => &[],
        }
    }

    fn groups_log_items(self) -> bool {
        matches!(self, Self::Default | Self::Log)
    }
}

/// The diff presentation selected by `jk`'s view-format modal.
///
/// This tracks the app's own `--git` toggle. Other jj diff tools may still be
/// passed through as args, but they are not treated as this modal state unless
/// they render as the explicit `--git` format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffFormat {
    Default,
    Git,
}

impl DiffFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Git => "git",
        }
    }

    fn arg(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Git => Some("--git"),
        }
    }
}

/// A concrete `jj` invocation plus the navigation target it represents.
///
/// `args` preserve the command line passed to jj. `target` is set for views
/// opened from the graph, where navigation should use a jj change id rather
/// than the commit id printed beside it. `target_is_exact_change` records
/// whether the target came from an exact graph change id instead of
/// from parsing a direct startup revset such as `main` or `@`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewSpec {
    command: JjCommand,
    args: Vec<String>,
    target: Option<String>,
    target_is_exact_change: bool,
    path: Option<String>,
    diff_format: DiffFormat,
}

impl ViewSpec {
    pub fn new(command: JjCommand, args: Vec<String>) -> Self {
        let diff_format = parse_diff_format(&args);
        Self {
            command,
            args,
            target: None,
            target_is_exact_change: false,
            path: None,
            diff_format,
        }
    }

    pub fn bookmarks(args: Vec<String>) -> Self {
        Self {
            command: JjCommand::Bookmarks,
            args,
            target: None,
            target_is_exact_change: false,
            path: None,
            diff_format: DiffFormat::Default,
        }
    }

    pub fn show(revset: String, diff_format: DiffFormat) -> Self {
        Self {
            command: JjCommand::Show,
            args: diff_format_args(diff_format, [revset.clone()]),
            target: Some(revset),
            target_is_exact_change: true,
            path: None,
            diff_format,
        }
    }

    pub fn diff(revset: String, diff_format: DiffFormat) -> Self {
        Self {
            command: JjCommand::Diff,
            args: diff_format_args(diff_format, ["-r".to_owned(), revset.clone()]),
            target: Some(revset),
            target_is_exact_change: true,
            path: None,
            diff_format,
        }
    }

    pub fn resolve(revset: Option<String>) -> Self {
        let revset = revset.unwrap_or_else(|| "@".to_owned());
        let args = vec!["-r".to_owned(), revset.clone()];

        Self {
            command: JjCommand::Resolve,
            args,
            target: Some(revset),
            target_is_exact_change: false,
            path: None,
            diff_format: DiffFormat::Default,
        }
    }

    pub fn file_list(revset: Option<String>, selected_path: Option<String>) -> Self {
        let args = revset
            .as_ref()
            .map(|revset| vec!["-r".to_owned(), revset.clone()])
            .unwrap_or_default();

        Self {
            command: JjCommand::FileList,
            args,
            target: revset,
            target_is_exact_change: false,
            path: selected_path,
            diff_format: DiffFormat::Default,
        }
    }

    pub fn file_show(revset: Option<String>, path: String) -> Self {
        let args = revset
            .as_ref()
            .map(|revset| vec!["-r".to_owned(), revset.clone(), path.clone()])
            .unwrap_or_else(|| vec![path.clone()]);

        Self {
            command: JjCommand::FileShow,
            args,
            target: revset,
            target_is_exact_change: false,
            path: Some(path),
            diff_format: DiffFormat::Default,
        }
    }

    pub fn operation_show(operation_id: String) -> Self {
        Self {
            command: JjCommand::OperationShow,
            args: vec![operation_id.clone()],
            target: Some(operation_id),
            target_is_exact_change: false,
            path: None,
            diff_format: DiffFormat::Default,
        }
    }

    pub fn operation_diff(operation_id: String) -> Self {
        Self {
            command: JjCommand::OperationDiff,
            args: vec!["--operation".to_owned(), operation_id.clone()],
            target: Some(operation_id),
            target_is_exact_change: false,
            path: None,
            diff_format: DiffFormat::Default,
        }
    }

    pub fn for_log_mode(home_command: JjCommand, mode: &LogViewMode) -> Self {
        match mode {
            LogViewMode::Default => Self::new(home_command, Vec::new()),
            _ => Self::new(JjCommand::Log, mode.args()),
        }
    }

    pub fn command(&self) -> JjCommand {
        self.command
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn label(&self) -> String {
        let command = self.label_prefix();
        if self.args.is_empty() {
            command.to_owned()
        } else {
            format!("{} {}", command, self.args.join(" "))
        }
    }

    pub fn app_label(&self) -> String {
        let command = self.app_label_prefix();

        let args = self.display_args();
        if args.is_empty() {
            command.to_owned()
        } else {
            format!("{} {}", command, args.join(" "))
        }
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn exact_change_target(&self) -> Option<&str> {
        if self.target_is_exact_change {
            self.target.as_deref()
        } else {
            None
        }
    }

    pub fn has_exact_change_target(&self) -> bool {
        self.exact_change_target().is_some()
    }

    pub fn with_exact_change_target(mut self) -> Self {
        self.target_is_exact_change = self.target.is_some();
        self
    }

    pub fn without_exact_change_target(mut self) -> Self {
        self.target_is_exact_change = false;
        self
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    fn label_prefix(&self) -> &'static str {
        self.command.label()
    }

    fn app_label_prefix(&self) -> &'static str {
        match self.command {
            JjCommand::Default => "jk",
            JjCommand::Log => "jk log",
            JjCommand::Show => "jk show",
            JjCommand::Diff => "jk diff",
            JjCommand::Status => "jk status",
            JjCommand::Resolve => "jk resolve",
            JjCommand::FileList => "jk file list",
            JjCommand::FileShow => "jk file show",
            JjCommand::Bookmarks => "jk bookmarks",
            JjCommand::OperationLog => "jk operation log",
            JjCommand::OperationShow => "jk operation show",
            JjCommand::OperationDiff => "jk operation diff",
        }
    }

    /// Returns the revset to use when opening another detail view from this one.
    ///
    /// Navigated views already know their change id target. Direct startup views
    /// such as `jk show main` do not, so this falls back to command-specific
    /// jj argument parsing. Diff views intentionally ignore filesets here; when
    /// jj diff receives only paths, the revision still defaults to `@`.
    pub fn navigation_revset(&self) -> Option<String> {
        self.target.clone().or_else(|| match self.command {
            JjCommand::Show => Some(show_revset_arg(&self.args).unwrap_or("@").to_owned()),
            JjCommand::Diff => Some(diff_revset_arg(&self.args).unwrap_or("@").to_owned()),
            JjCommand::Resolve => Some(revision_arg(&self.args).unwrap_or("@").to_owned()),
            JjCommand::FileList => Some(revision_arg(&self.args).unwrap_or("@").to_owned()),
            JjCommand::FileShow => Some(
                revision_arg(self.file_show_context_args())
                    .unwrap_or("@")
                    .to_owned(),
            ),
            JjCommand::Default
            | JjCommand::Log
            | JjCommand::Status
            | JjCommand::Bookmarks
            | JjCommand::OperationLog
            | JjCommand::OperationShow
            | JjCommand::OperationDiff => None,
        })
    }

    pub fn diff_format(&self) -> DiffFormat {
        self.diff_format
    }

    pub fn with_diff_format(&self, diff_format: DiffFormat) -> Self {
        if !matches!(self.command, JjCommand::Show | JjCommand::Diff) {
            return self.clone();
        }

        let mut spec = self.clone();
        spec.diff_format = diff_format;
        spec.args = diff_format_args(
            diff_format,
            spec.args
                .into_iter()
                .filter(|arg| arg != "--git")
                .collect::<Vec<_>>(),
        );
        spec
    }

    pub fn show_context_revset(&self) -> String {
        self.target
            .clone()
            .or_else(|| match self.command {
                JjCommand::Resolve => revision_arg(&self.args).map(str::to_owned),
                JjCommand::FileList => revision_arg(&self.args).map(str::to_owned),
                JjCommand::FileShow => {
                    revision_arg(self.file_show_context_args()).map(str::to_owned)
                }
                JjCommand::OperationShow | JjCommand::OperationDiff => None,
                _ => show_revset_arg(&self.args).map(str::to_owned),
            })
            .unwrap_or_else(|| "@".to_owned())
    }

    fn display_args(&self) -> Vec<String> {
        let Some(target) = &self.target else {
            return self.args.clone();
        };

        if matches!(self.command, JjCommand::FileShow)
            && self.path.is_some()
            && !self.args.is_empty()
        {
            let split = self.args.len() - 1;
            let mut display_args = self.args[..split]
                .iter()
                .map(|arg| {
                    if arg == target {
                        short_id(target).to_owned()
                    } else {
                        arg.to_owned()
                    }
                })
                .collect::<Vec<_>>();
            display_args.push(self.args[split].clone());
            display_args
        } else {
            self.args
                .iter()
                .map(|arg| {
                    if arg == target {
                        short_id(target).to_owned()
                    } else {
                        arg.to_owned()
                    }
                })
                .collect()
        }
    }

    fn file_show_context_args(&self) -> &[String] {
        if matches!(self.command, JjCommand::FileShow)
            && self.path.is_some()
            && !self.args.is_empty()
        {
            &self.args[..self.args.len() - 1]
        } else {
            &self.args
        }
    }
}

fn parse_diff_format(args: &[String]) -> DiffFormat {
    if args.iter().any(|arg| arg == "--git") {
        DiffFormat::Git
    } else {
        DiffFormat::Default
    }
}

fn diff_format_args(
    diff_format: DiffFormat,
    args: impl IntoIterator<Item = String>,
) -> Vec<String> {
    diff_format
        .arg()
        .into_iter()
        .map(str::to_owned)
        .chain(args)
        .collect()
}

fn revset_from_log_args(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revisions"], &["--revisions="])
}

pub fn git_fetch() -> Result<CommandOutput> {
    run_direct_command(&FETCH_ARGS, "jj git fetch", "fetched")
}

pub fn new_trunk() -> Result<CommandOutput> {
    run_direct_command(&NEW_TRUNK_ARGS, "jj new trunk()", "created new change")
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

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

/// One selectable item parsed from rendered graph output.
///
/// A visible graph item can span multiple terminal lines. When jj prints a real
/// revision, metadata stores the full change id and commit id used for
/// navigation and copying.
#[derive(Clone, Debug)]
pub struct LogItem {
    lines: Vec<Line<'static>>,
    change_id: Option<String>,
    commit_id: Option<String>,
}

impl LogItem {
    pub fn new(
        lines: Vec<Line<'static>>,
        change_id: Option<String>,
        commit_id: Option<String>,
    ) -> Self {
        Self {
            lines,
            change_id,
            commit_id,
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn action_id(&self) -> Option<&str> {
        self.change_id()
    }

    pub fn change_id(&self) -> Option<&str> {
        self.change_id.as_deref()
    }

    pub fn commit_id(&self) -> Option<&str> {
        self.commit_id.as_deref()
    }

    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// One selectable bookmark item parsed from rendered bookmark output.
#[derive(Clone, Debug)]
pub struct BookmarkItem {
    lines: Vec<Line<'static>>,
    name: String,
    target_change_id: Option<String>,
    target_commit_id: Option<String>,
    local: bool,
}

impl BookmarkItem {
    pub fn new(
        lines: Vec<Line<'static>>,
        name: String,
        target_change_id: Option<String>,
        target_commit_id: Option<String>,
    ) -> Self {
        Self {
            lines,
            name,
            target_change_id,
            target_commit_id,
            local: true,
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn bookmark_name(&self) -> &str {
        &self.name
    }

    pub fn target_change_id(&self) -> Option<&str> {
        self.target_change_id.as_deref()
    }

    pub fn target_commit_id(&self) -> Option<&str> {
        self.target_commit_id.as_deref()
    }

    pub fn is_local(&self) -> bool {
        self.local
    }

    #[cfg(test)]
    pub(crate) fn with_local(mut self, local: bool) -> Self {
        self.local = local;
        self
    }

    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// One selectable file item parsed from rendered file-list output.
#[derive(Clone, Debug)]
pub struct FileListItem {
    lines: Vec<Line<'static>>,
    path: String,
}

impl FileListItem {
    pub fn new(lines: Vec<Line<'static>>, path: String) -> Self {
        Self { lines, path }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// One conflicted path reported by the resolve template contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveEntry {
    path: Option<String>,
    file_type: Option<String>,
    side_count: Option<usize>,
    raw_line: Option<String>,
}

impl ResolveEntry {
    pub fn parsed(
        path: Option<String>,
        file_type: Option<String>,
        side_count: Option<usize>,
    ) -> Self {
        Self {
            path,
            file_type,
            side_count,
            raw_line: None,
        }
    }

    pub fn unparsed(raw_line: String) -> Self {
        Self {
            path: None,
            file_type: None,
            side_count: None,
            raw_line: Some(raw_line),
        }
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn file_type(&self) -> Option<&str> {
        self.file_type.as_deref()
    }

    pub fn side_count(&self) -> Option<usize> {
        self.side_count
    }

    pub fn raw_line(&self) -> Option<&str> {
        self.raw_line.as_deref()
    }
}

/// One selectable operation item parsed from rendered operation-log output.
#[derive(Clone, Debug)]
pub struct OperationLogItem {
    lines: Vec<Line<'static>>,
    operation_id: Option<String>,
}

impl OperationLogItem {
    pub fn new(lines: Vec<Line<'static>>, operation_id: Option<String>) -> Self {
        Self {
            lines,
            operation_id,
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn operation_id(&self) -> Option<&str> {
        self.operation_id.as_deref()
    }

    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn load_entries(spec: &ViewSpec) -> Result<Vec<LogItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    if spec.command.groups_log_items() {
        let metadata = run_jj_with_template(spec, r#"change_id ++ " " ++ commit_id ++ "\n""#)?;
        Ok(group_lines(lines, metadata))
    } else {
        Ok(lines
            .into_iter()
            .map(|line| LogItem::new(vec![line], spec.target.clone(), None))
            .collect())
    }
}

pub fn load_bookmark_entries(spec: &ViewSpec) -> Result<Vec<BookmarkItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;
    let metadata = run_jj_bookmark_metadata(spec)?;
    Ok(pair_bookmark_lines(lines, metadata))
}

pub fn load_resolve_entries(spec: &ViewSpec) -> Result<Vec<ResolveEntry>> {
    Ok(
        run_jj_template_lines(spec, RESOLVE_CONFLICT_TEMPLATE, true)?
            .into_iter()
            .map(|line| parse_resolve_entry_line(&line))
            .collect(),
    )
}

pub fn load_file_list_entries(spec: &ViewSpec) -> Result<Vec<FileListItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    Ok(lines
        .into_iter()
        .filter_map(|line| {
            let path = parse_file_list_path(&line_text(&line))?;
            Some(FileListItem::new(vec![line], path))
        })
        .collect())
}

pub fn load_operation_log_entries(spec: &ViewSpec) -> Result<Vec<OperationLogItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;
    let operation_ids = run_operation_log_ids(spec)?;
    Ok(group_operation_log_lines(lines, operation_ids))
}

pub fn load_compact_log_context(revset: &str) -> Result<Vec<Line<'static>>> {
    let spec = ViewSpec::new(JjCommand::Log, vec!["-r".to_owned(), revset.to_owned()]);
    let output = run_jj(&spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    Ok(group_lines(lines, Vec::new())
        .into_iter()
        .next()
        .map(|item| item.lines().into_iter().take(2).collect())
        .unwrap_or_default())
}

pub fn document_plain_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

fn run_jj(spec: &ViewSpec, color: ColorMode) -> Result<std::process::Output> {
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

fn run_jj_with_template(spec: &ViewSpec, template: &str) -> Result<Vec<RevisionMetadata>> {
    Ok(run_jj_template_lines(spec, template, false)?
        .into_iter()
        .filter_map(|line| parse_metadata_line(&line))
        .collect())
}

fn run_jj_bookmark_metadata(spec: &ViewSpec) -> Result<Vec<BookmarkMetadata>> {
    Ok(
        run_jj_template_lines(spec, BOOKMARK_METADATA_TEMPLATE, false)?
            .into_iter()
            .filter_map(|line| parse_bookmark_metadata_line(&line))
            .collect(),
    )
}

fn run_jj_template_lines(spec: &ViewSpec, template: &str, no_graph: bool) -> Result<Vec<String>> {
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

fn run_operation_log_ids(spec: &ViewSpec) -> Result<Vec<Option<String>>> {
    let mut jj = base_command(ColorMode::Never);
    jj.args(jj_command_args(spec, Some(OPERATION_ID_TEMPLATE), true));

    let output = jj.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("{} metadata failed: {}", spec.label(), stderr.trim()));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().map(parse_operation_id_line).collect())
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

fn group_lines(lines: Vec<Line<'static>>, metadata: Vec<RevisionMetadata>) -> Vec<LogItem> {
    let mut items = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_metadata: Option<RevisionMetadata> = None;
    let mut metadata = metadata.into_iter();

    for line in lines {
        let starts_item = starts_log_item(&line);
        let standalone_graph_line = is_standalone_graph_line(&line);

        if (starts_item || standalone_graph_line) && !current_lines.is_empty() {
            items.push(LogItem::new(
                current_lines,
                current_metadata
                    .as_ref()
                    .map(|metadata| metadata.change_id.clone()),
                current_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.commit_id.clone()),
            ));
            current_lines = Vec::new();
            current_metadata = None;
        }

        if starts_item {
            current_metadata = metadata.next();
        }
        current_lines.push(line);
    }

    if !current_lines.is_empty() {
        items.push(LogItem::new(
            current_lines,
            current_metadata
                .as_ref()
                .map(|metadata| metadata.change_id.clone()),
            current_metadata.and_then(|metadata| metadata.commit_id),
        ));
    }

    items
}

fn pair_bookmark_lines(
    lines: Vec<Line<'static>>,
    metadata: Vec<BookmarkMetadata>,
) -> Vec<BookmarkItem> {
    let mut items = Vec::new();
    let mut metadata = metadata.into_iter();

    for line in lines {
        let text = line_text(&line);
        let metadata = metadata.next();
        let bookmark_name = metadata
            .as_ref()
            .map(|metadata| metadata.name.clone())
            .unwrap_or_else(|| bookmark_name_from_rendered_row(&text));
        let mut item = BookmarkItem::new(
            vec![line],
            bookmark_name,
            metadata
                .as_ref()
                .and_then(|metadata| metadata.target_change_id.clone()),
            metadata
                .as_ref()
                .and_then(|metadata| metadata.target_commit_id.clone()),
        );
        item.local = metadata.as_ref().is_some_and(BookmarkMetadata::is_local);
        items.push(item);
    }

    items
}

fn group_operation_log_lines(
    lines: Vec<Line<'static>>,
    operation_ids: Vec<Option<String>>,
) -> Vec<OperationLogItem> {
    let mut items = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_operation_id = None;
    let mut operation_ids = operation_ids.into_iter();

    for line in lines {
        let starts_item = starts_operation_log_item(&line);
        let standalone_graph_line = is_standalone_graph_line(&line);

        if (starts_item || standalone_graph_line) && !current_lines.is_empty() {
            items.push(OperationLogItem::new(current_lines, current_operation_id));
            current_lines = Vec::new();
            current_operation_id = None;
        }

        if starts_item {
            current_operation_id = operation_ids.next().flatten();
        }
        current_lines.push(line);
    }

    if !current_lines.is_empty() {
        items.push(OperationLogItem::new(current_lines, current_operation_id));
    }

    items
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RevisionMetadata {
    change_id: String,
    commit_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BookmarkMetadata {
    name: String,
    remote: Option<String>,
    target_change_id: Option<String>,
    target_commit_id: Option<String>,
}

impl BookmarkMetadata {
    fn is_local(&self) -> bool {
        self.remote.is_none()
    }
}

fn parse_metadata_line(line: &str) -> Option<RevisionMetadata> {
    let mut change_id = None;
    let mut commit_id = None;

    for token in line.split_whitespace() {
        if change_id.is_none() && is_full_change_id(token) {
            change_id = Some(token.to_owned());
        } else if commit_id.is_none() && is_full_commit_id(token) {
            commit_id = Some(token.to_owned());
        }
    }

    change_id.map(|change_id| RevisionMetadata {
        change_id,
        commit_id,
    })
}

fn parse_bookmark_metadata_line(line: &str) -> Option<BookmarkMetadata> {
    if line.is_empty() {
        return None;
    }

    let mut fields = line.split('\t');
    let name = fields.next()?;
    if name.is_empty() {
        return None;
    }
    let remote = fields
        .next()
        .filter(|field| !field.is_empty())
        .map(str::to_owned);

    Some(BookmarkMetadata {
        name: name.to_owned(),
        remote,
        target_change_id: fields
            .next()
            .filter(|field| !field.is_empty())
            .map(str::to_owned),
        target_commit_id: fields
            .next()
            .filter(|field| !field.is_empty())
            .map(str::to_owned),
    })
}

fn parse_operation_id_line(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|token| is_operation_id(token))
        .map(str::to_owned)
}

fn parse_resolve_entry_line(line: &str) -> ResolveEntry {
    let raw_line = line.to_owned();
    let Ok(Value::Object(fields)) = serde_json::from_str::<Value>(line) else {
        return ResolveEntry::unparsed(raw_line);
    };

    ResolveEntry::parsed(
        string_field(&fields, "path"),
        string_field(&fields, "file_type"),
        integer_field(&fields, "side_count"),
    )
}

fn parse_file_list_path(line: &str) -> Option<String> {
    (!line.is_empty()).then(|| line.to_owned())
}

fn string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    fields.get(name).and_then(Value::as_str).map(str::to_owned)
}

fn integer_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<usize> {
    fields
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
}

fn show_revset_arg(args: &[String]) -> Option<&str> {
    let mut skip_next = false;

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if show_option_takes_value(arg) {
            skip_next = !arg.contains('=');
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        return Some(arg);
    }

    None
}

fn show_option_takes_value(arg: &str) -> bool {
    matches!(
        arg,
        "-T" | "--template"
            | "--tool"
            | "--context"
            | "-R"
            | "--repository"
            | "--at-operation"
            | "--at-op"
            | "--config"
            | "--config-file"
    ) || [
        "--template=",
        "--tool=",
        "--context=",
        "--repository=",
        "--at-operation=",
        "--at-op=",
        "--config=",
        "--config-file=",
    ]
    .iter()
    .any(|prefix| arg.starts_with(prefix))
}

fn diff_revset_arg(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revisions"], &["--revisions="])
        .or_else(|| option_value(args, &["-t", "--to"], &["--to="]))
}

fn revision_arg(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revision"], &["--revision="])
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

fn is_full_commit_id(token: &str) -> bool {
    token.len() == 40 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_full_change_id(token: &str) -> bool {
    token.len() == 32 && token.bytes().all(|byte| byte.is_ascii_lowercase())
}

fn is_operation_id(token: &str) -> bool {
    token.len() == 128 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn starts_log_item(line: &Line<'_>) -> bool {
    starts_log_item_text(&line_text(line))
}

fn starts_log_item_text(text: &str) -> bool {
    first_content_char(text).is_some_and(|character| matches!(character, '@' | '○' | '◆'))
}

fn starts_operation_log_item(line: &Line<'_>) -> bool {
    first_content_char(&line_text(line)).is_some_and(|character| matches!(character, '@' | '○'))
}

fn is_standalone_graph_line(line: &Line<'_>) -> bool {
    let text = line_text(line);
    first_content_char(&text).is_none_or(|character| character == '~')
}

fn first_content_char(text: &str) -> Option<char> {
    text.chars()
        .find(|character| !matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮'))
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn bookmark_name_from_rendered_row(text: &str) -> String {
    text.split_once(':')
        .map(|(name, _)| name.trim())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| text.trim())
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_ansi_output_to_selectable_items() {
        let text =
            b"\x1b[1m@\x1b[0m  change\n\xE2\x94\x82  description\n\xE2\x97\x8B  parent\n".to_vec();
        let lines = text.into_text().unwrap().lines;
        let metadata = vec![metadata("abc", "123"), metadata("def", "456")];
        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].lines.len(), 2);
        assert_eq!(items[0].change_id(), Some("abc"));
        assert_eq!(items[0].commit_id(), Some("123"));
        assert_eq!(items[0].lines[0].spans[0].content, "@");
    }

    #[test]
    fn does_not_split_on_description_mentions() {
        let lines = b"@  change\n\xE2\x94\x82  email me@example.com\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![metadata("abc", "123")];

        assert_eq!(group_lines(lines, metadata).len(), 1);
    }

    #[test]
    fn pairs_one_metadata_line_with_multi_line_display_items() {
        let lines = b"@  current\n\xE2\x94\x82  current description\n\xE2\x97\x8B  parent\n\xE2\x94\x82  parent description\n\xE2\x97\x86  root\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![
            metadata("current", "111"),
            metadata("parent", "222"),
            metadata("root", "333"),
        ];
        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].lines.len(), 2);
        assert_eq!(items[0].change_id(), Some("current"));
        assert_eq!(items[1].lines.len(), 2);
        assert_eq!(items[1].change_id(), Some("parent"));
        assert_eq!(items[2].lines.len(), 1);
        assert_eq!(items[2].change_id(), Some("root"));
    }

    #[test]
    fn keeps_elided_graph_rows_separate() {
        let lines = b"@  change\n\xE2\x94\x82  desc\n\xE2\x94\x82 ~  (elided revisions)\n\xE2\x94\x9C\xE2\x94\x80\xE2\x95\xAF\n\xE2\x97\x8B  parent\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![metadata("abc", "123"), metadata("def", "456")];
        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 4);
        assert_eq!(items[0].change_id(), Some("abc"));
        assert_eq!(items[1].change_id(), None);
        assert_eq!(items[2].change_id(), None);
        assert_eq!(items[3].change_id(), Some("def"));
    }

    #[test]
    fn groups_operation_log_rows_and_preserves_styles() {
        let text =
            b"\x1b[1m@\x1b[0m  current\n\xE2\x94\x82  args: jj describe\n\xE2\x97\x8B  previous\n"
                .to_vec();
        let lines = text.into_text().unwrap().lines;
        let operation_ids = vec![Some(operation_id('a')), Some(operation_id('b'))];

        let items = group_operation_log_lines(lines, operation_ids);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].line_count(), 2);
        assert_eq!(items[0].operation_id(), Some(operation_id('a').as_str()));
        assert_eq!(items[0].lines[0].spans[0].content, "@");
        assert_eq!(items[1].operation_id(), Some(operation_id('b').as_str()));
    }

    #[test]
    fn operation_log_rows_allow_missing_metadata() {
        let lines = b"@  current\n\xE2\x94\x82  args: jj describe\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;

        let items = group_operation_log_lines(lines, vec![None]);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].operation_id(), None);
    }

    #[test]
    fn parses_revision_metadata_lines() {
        assert_eq!(
            parse_metadata_line(
                "@  tvykuurwpnwzzqulzrvwvmxxotnlywqw 64d399917e441072c228d7811743550753c9f6cf"
            ),
            Some(RevisionMetadata {
                change_id: "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
                commit_id: Some("64d399917e441072c228d7811743550753c9f6cf".to_owned()),
            })
        );
        assert_eq!(
            parse_metadata_line("@  tvykuurwpnwzzqulzrvwvmxxotnlywqw"),
            Some(RevisionMetadata {
                change_id: "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
                commit_id: None,
            })
        );
        assert_eq!(parse_metadata_line("│ ~  (elided revisions)"), None);
    }

    #[test]
    fn parses_operation_id_lines() {
        let operation_id = operation_id('a');

        assert_eq!(
            parse_operation_id_line(&("@  ".to_owned() + &operation_id + "\n")),
            Some(operation_id)
        );
        assert_eq!(parse_operation_id_line("not-an-operation"), None);
    }

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
    fn file_show_context_revset_defaults_to_current_revision() {
        let spec = ViewSpec::file_show(None, "src/main.rs".to_owned());

        assert_eq!(spec.show_context_revset(), "@");
        assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
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
    fn resolve_entry_parser_keeps_exact_fields() {
        let entry = parse_resolve_entry_line(
            r#"{"path":"dir/space file.txt","file_type":"file","side_count":3}"#,
        );

        assert_eq!(
            entry,
            ResolveEntry::parsed(
                Some("dir/space file.txt".to_owned()),
                Some("file".to_owned()),
                Some(3),
            )
        );
    }

    #[test]
    fn resolve_entry_parser_degrades_invalid_json_to_unparsed_row() {
        let entry = parse_resolve_entry_line("{not json");

        assert_eq!(entry, ResolveEntry::unparsed("{not json".to_owned()));
    }

    #[test]
    fn resolve_entry_parser_allows_missing_exact_path() {
        let entry =
            parse_resolve_entry_line(r#"{"path":null,"file_type":"symlink","side_count":2}"#);

        assert_eq!(
            entry,
            ResolveEntry::parsed(None, Some("symlink".to_owned()), Some(2))
        );
    }

    #[test]
    fn file_list_path_parser_preserves_exact_text() {
        assert_eq!(
            parse_file_list_path("src/path with spaces"),
            Some("src/path with spaces".to_owned())
        );
        assert_eq!(parse_file_list_path(""), None);
    }

    #[test]
    fn file_list_item_preserves_row_lines_and_path() {
        let lines = b"src/path with spaces\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let item = FileListItem::new(lines, "src/path with spaces".to_owned());

        assert_eq!(item.line_count(), 1);
        assert_eq!(item.path(), "src/path with spaces");
        assert_eq!(item.row_text(), "src/path with spaces");
    }

    #[test]
    fn parses_bookmark_metadata_lines() {
        assert_eq!(
            parse_bookmark_metadata_line(
                "main\t\twuqolszplkmommqzmxpmmwtwrpuuwkmo\t2f81d8af4234fef19b84d1495383a55999bb37fa"
            ),
            Some(BookmarkMetadata {
                name: "main".to_owned(),
                remote: None,
                target_change_id: Some("wuqolszplkmommqzmxpmmwtwrpuuwkmo".to_owned()),
                target_commit_id: Some("2f81d8af4234fef19b84d1495383a55999bb37fa".to_owned()),
            })
        );
        assert_eq!(
            parse_bookmark_metadata_line("main\torigin\t\t"),
            Some(BookmarkMetadata {
                name: "main".to_owned(),
                remote: Some("origin".to_owned()),
                target_change_id: None,
                target_commit_id: None,
            })
        );
        assert_eq!(
            parse_bookmark_metadata_line("main\t\t\t"),
            Some(BookmarkMetadata {
                name: "main".to_owned(),
                remote: None,
                target_change_id: None,
                target_commit_id: None,
            })
        );
        assert_eq!(parse_bookmark_metadata_line(""), None);
    }

    #[test]
    fn pairs_bookmark_rows_in_render_order() {
        let lines = b"main: okrnpmzv d10e26b6 Update agent repository guidance\nprototype: nqvrkyps f65c4354 docs: add explicit unsupported warning\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![
            bookmark_metadata(
                "main",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            bookmark_metadata(
                "prototype",
                Some("nqvrkypsnqvrkypsnqvrkypsnqvrkyps"),
                Some("f65c4354f65c4354f65c4354f65c4354f65c4354"),
            ),
        ];

        let items = pair_bookmark_lines(lines, metadata);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].line_count(), 1);
        assert_eq!(items[0].bookmark_name(), "main");
        assert_eq!(
            items[0].target_change_id(),
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv")
        );
        assert_eq!(
            items[0].target_commit_id(),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6")
        );
        assert_eq!(items[1].bookmark_name(), "prototype");
    }

    #[test]
    fn remote_bookmark_rows_do_not_advance_local_metadata() {
        let lines = b"main: okrnpmzv d10e26b6 Update agent repository guidance\nmain@origin: okrnpmzv d10e26b6 Update agent repository guidance\nprototype: nqvrkyps f65c4354 docs: add explicit unsupported warning\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![
            bookmark_metadata(
                "main",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            remote_bookmark_metadata(
                "main",
                "origin",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            bookmark_metadata(
                "prototype",
                Some("nqvrkypsnqvrkypsnqvrkypsnqvrkyps"),
                Some("f65c4354f65c4354f65c4354f65c4354f65c4354"),
            ),
        ];

        let items = pair_bookmark_lines(lines, metadata);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].bookmark_name(), "main");
        assert!(items[0].is_local());
        assert_eq!(items[1].bookmark_name(), "main");
        assert!(!items[1].is_local());
        assert_eq!(
            items[1].target_change_id(),
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv")
        );
        assert_eq!(
            items[1].target_commit_id(),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6")
        );
        assert_eq!(items[2].bookmark_name(), "prototype");
        assert!(items[2].is_local());
    }

    #[test]
    fn bookmark_rows_without_metadata_are_not_marked_local() {
        let lines = b"remote-looking-but-not-trusted: okrnpmzv d10e26b6\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;

        let items = pair_bookmark_lines(lines, Vec::new());

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].bookmark_name(), "remote-looking-but-not-trusted");
        assert!(!items[0].is_local());
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

    #[test]
    fn file_views_ignore_diff_format_toggle() {
        let show_spec = ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned());
        let list_spec = ViewSpec::file_list(Some("main".to_owned()), None);

        assert_eq!(show_spec.with_diff_format(DiffFormat::Git), show_spec);
        assert_eq!(list_spec.with_diff_format(DiffFormat::Git), list_spec);
    }

    fn metadata(change_id: &str, commit_id: &str) -> RevisionMetadata {
        RevisionMetadata {
            change_id: change_id.to_owned(),
            commit_id: Some(commit_id.to_owned()),
        }
    }

    fn bookmark_metadata(
        name: &str,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkMetadata {
        bookmark_metadata_with_remote(name, None, target_change_id, target_commit_id)
    }

    fn remote_bookmark_metadata(
        name: &str,
        remote: &str,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkMetadata {
        bookmark_metadata_with_remote(name, Some(remote), target_change_id, target_commit_id)
    }

    fn bookmark_metadata_with_remote(
        name: &str,
        remote: Option<&str>,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkMetadata {
        BookmarkMetadata {
            name: name.to_owned(),
            remote: remote.map(str::to_owned),
            target_change_id: target_change_id.map(str::to_owned),
            target_commit_id: target_commit_id.map(str::to_owned),
        }
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }

    #[test]
    fn app_label_shortens_navigated_targets() {
        let spec = ViewSpec::show(
            "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
            DiffFormat::Default,
        );

        assert_eq!(
            spec.exact_change_target(),
            Some("tvykuurwpnwzzqulzrvwvmxxotnlywqw")
        );
        assert_eq!(spec.app_label(), "jk show tvykuurw");

        let spec = ViewSpec::diff(
            "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
            DiffFormat::Default,
        );

        assert_eq!(
            spec.exact_change_target(),
            Some("tvykuurwpnwzzqulzrvwvmxxotnlywqw")
        );
        assert_eq!(spec.app_label(), "jk diff -r tvykuurw");
    }

    #[test]
    fn exact_change_target_provenance_is_explicit() {
        let direct = ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]);
        assert_eq!(direct.navigation_revset().as_deref(), Some("main"));
        assert_eq!(direct.exact_change_target(), None);

        let file_list =
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target();
        assert_eq!(file_list.exact_change_target(), Some("change-a"));

        let file_show = ViewSpec::file_show(Some("change-a".to_owned()), "src/main.rs".to_owned())
            .with_exact_change_target();
        assert_eq!(file_show.exact_change_target(), Some("change-a"));
    }

    #[test]
    fn show_context_revset_prefers_navigation_target() {
        let spec = ViewSpec::show("abc".to_owned(), DiffFormat::Default);

        assert_eq!(spec.show_context_revset(), "abc");
    }

    #[test]
    fn show_context_revset_uses_direct_revset() {
        let spec = ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]);

        assert_eq!(spec.show_context_revset(), "main");
    }

    #[test]
    fn show_context_revset_skips_option_values() {
        let spec = ViewSpec::new(
            JjCommand::Show,
            vec![
                "--template".to_owned(),
                "description".to_owned(),
                "--summary".to_owned(),
            ],
        );

        assert_eq!(spec.show_context_revset(), "@");
    }

    #[test]
    fn show_context_revset_finds_revset_after_options() {
        let spec = ViewSpec::new(
            JjCommand::Show,
            vec![
                "--template=description".to_owned(),
                "--summary".to_owned(),
                "main".to_owned(),
            ],
        );

        assert_eq!(spec.show_context_revset(), "main");
    }

    #[test]
    fn show_context_revset_defaults_to_current_revision() {
        let spec = ViewSpec::new(JjCommand::Show, Vec::new());

        assert_eq!(spec.show_context_revset(), "@");
    }

    #[test]
    fn git_diff_format_adds_git_argument_to_navigated_views() {
        let spec = ViewSpec::show("abc".to_owned(), DiffFormat::Git);

        assert_eq!(spec.args(), ["--git", "abc"]);
        assert_eq!(spec.app_label(), "jk show --git abc");

        let spec = ViewSpec::diff("abc".to_owned(), DiffFormat::Git);

        assert_eq!(spec.args(), ["--git", "-r", "abc"]);
        assert_eq!(spec.app_label(), "jk diff --git -r abc");
    }

    #[test]
    fn diff_format_can_be_replaced_without_duplicating_git_flag() {
        let spec = ViewSpec::new(
            JjCommand::Diff,
            vec!["--git".to_owned(), "-r".to_owned(), "abc".to_owned()],
        );

        assert_eq!(spec.diff_format(), DiffFormat::Git);
        assert_eq!(
            spec.with_diff_format(DiffFormat::Git).args(),
            ["--git", "-r", "abc"]
        );
        assert_eq!(
            spec.with_diff_format(DiffFormat::Default).args(),
            ["-r", "abc"]
        );
    }

    #[test]
    fn navigation_revset_uses_direct_show_startup_revset() {
        let spec = ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]);

        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    }

    #[test]
    fn navigation_revset_defaults_direct_show_to_current_revision() {
        let spec = ViewSpec::new(JjCommand::Show, Vec::new());

        assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
    }

    #[test]
    fn navigation_revset_uses_direct_diff_startup_revset() {
        let spec = ViewSpec::new(
            JjCommand::Diff,
            vec!["--git".to_owned(), "-r".to_owned(), "main".to_owned()],
        );

        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    }

    #[test]
    fn navigation_revset_ignores_direct_diff_filesets() {
        let spec = ViewSpec::new(JjCommand::Diff, vec!["src/main.rs".to_owned()]);

        assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
    }

    #[test]
    fn navigation_revset_uses_direct_diff_to_revision() {
        let spec = ViewSpec::new(
            JjCommand::Diff,
            vec!["--from".to_owned(), "main".to_owned(), "--to=@".to_owned()],
        );

        assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
    }

    #[test]
    fn navigation_revset_defaults_direct_diff_from_revision_to_current_revision() {
        let spec = ViewSpec::new(
            JjCommand::Diff,
            vec!["--from".to_owned(), "main".to_owned()],
        );

        assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
    }

    #[test]
    fn navigation_revset_uses_long_direct_diff_revision_option() {
        let spec = ViewSpec::new(
            JjCommand::Diff,
            vec!["--revisions=main".to_owned(), "src/main.rs".to_owned()],
        );

        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    }

    #[test]
    fn tool_git_is_passthrough_not_view_format_state() {
        let spec = ViewSpec::new(JjCommand::Diff, vec!["--tool=:git".to_owned()]);

        assert_eq!(spec.diff_format(), DiffFormat::Default);
        assert_eq!(spec.args(), ["--tool=:git"]);
    }

    #[test]
    fn log_view_mode_uses_plain_default_command() {
        let spec = ViewSpec::for_log_mode(JjCommand::Default, &LogViewMode::Default);

        assert_eq!(spec.command(), JjCommand::Default);
        assert!(spec.args().is_empty());
    }

    #[test]
    fn log_view_mode_uses_explicit_revset_for_named_modes() {
        let spec = ViewSpec::for_log_mode(JjCommand::Default, &LogViewMode::Trunk);

        assert_eq!(spec.command(), JjCommand::Log);
        assert_eq!(spec.args(), ["-r", TRUNK_WORK_REVSET]);
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
        assert_eq!(FETCH_ARGS, ["git", "fetch"]);
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
