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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjCommand {
    Default,
    Log,
    Show,
    Diff,
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
const FETCH_ARGS: [&str; 2] = ["git", "fetch"];
const NEW_TRUNK_ARGS: [&str; 2] = ["new", "trunk()"];
const CHANGE_ID_TEMPLATE: &str = "change_id ++ \"\\n\"";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    message: String,
}

impl CommandOutput {
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl JjCommand {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "jj",
            Self::Log => "jj log",
            Self::Show => "jj show",
            Self::Diff => "jj diff",
        }
    }

    fn subcommand(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Log => Some("log"),
            Self::Show => Some("show"),
            Self::Diff => Some("diff"),
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
/// than the commit id printed beside it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewSpec {
    command: JjCommand,
    args: Vec<String>,
    target: Option<String>,
    diff_format: DiffFormat,
}

impl ViewSpec {
    pub fn new(command: JjCommand, args: Vec<String>) -> Self {
        let diff_format = parse_diff_format(&args);
        Self {
            command,
            args,
            target: None,
            diff_format,
        }
    }

    pub fn show(revset: String, diff_format: DiffFormat) -> Self {
        Self {
            command: JjCommand::Show,
            args: diff_format_args(diff_format, [revset.clone()]),
            target: Some(revset),
            diff_format,
        }
    }

    pub fn diff(revset: String, diff_format: DiffFormat) -> Self {
        Self {
            command: JjCommand::Diff,
            args: diff_format_args(diff_format, ["-r".to_owned(), revset.clone()]),
            target: Some(revset),
            diff_format,
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
        if self.args.is_empty() {
            self.command.label().to_owned()
        } else {
            format!("{} {}", self.command.label(), self.args.join(" "))
        }
    }

    pub fn app_label(&self) -> String {
        let command = match self.command {
            JjCommand::Default => "jk",
            JjCommand::Log => "jk log",
            JjCommand::Show => "jk show",
            JjCommand::Diff => "jk diff",
        };

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
            JjCommand::Default | JjCommand::Log => None,
        })
    }

    pub fn diff_format(&self) -> DiffFormat {
        self.diff_format
    }

    pub fn with_diff_format(&self, diff_format: DiffFormat) -> Self {
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
            .or_else(|| show_revset_arg(&self.args).map(str::to_owned))
            .unwrap_or_else(|| "@".to_owned())
    }

    fn display_args(&self) -> Vec<String> {
        let Some(target) = &self.target else {
            return self.args.clone();
        };

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
    jj.args(["log", "-r", revset, "-T", CHANGE_ID_TEMPLATE]);

    let output = jj.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("{} failed: {}", revset, stderr.trim()));
    }

    parse_exact_change_id(&String::from_utf8(output.stdout)?)
        .map_err(|error| eyre!("{} {}", revset, error))
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

fn run_jj(spec: &ViewSpec, color: ColorMode) -> Result<std::process::Output> {
    let mut jj = base_command(color);

    if let Some(subcommand) = spec.command.subcommand() {
        jj.arg(subcommand);
    }
    jj.args(&spec.args);

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

    Ok(CommandOutput {
        message: summarize_output(&output.stdout, &output.stderr, success_fallback),
    })
}

fn run_jj_with_template(spec: &ViewSpec, template: &str) -> Result<Vec<RevisionMetadata>> {
    let mut jj = base_command(ColorMode::Never);

    if let Some(subcommand) = spec.command.subcommand() {
        jj.arg(subcommand);
    }
    jj.args(["-T", template]);
    jj.args(&spec.args);

    let output = jj.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!("{} metadata failed: {}", spec.label(), stderr.trim()));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.lines().filter_map(parse_metadata_line).collect())
}

fn base_command(color: ColorMode) -> Command {
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

fn summarize_output(stdout: &[u8], stderr: &[u8], fallback: &str) -> String {
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
enum ColorMode {
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct RevisionMetadata {
    change_id: String,
    commit_id: Option<String>,
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

fn starts_log_item(line: &Line<'_>) -> bool {
    starts_log_item_text(&line_text(line))
}

fn starts_log_item_text(text: &str) -> bool {
    first_content_char(text).is_some_and(|character| matches!(character, '@' | '○' | '◆'))
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

    fn metadata(change_id: &str, commit_id: &str) -> RevisionMetadata {
        RevisionMetadata {
            change_id: change_id.to_owned(),
            commit_id: Some(commit_id.to_owned()),
        }
    }

    #[test]
    fn app_label_shortens_navigated_targets() {
        let spec = ViewSpec::show(
            "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
            DiffFormat::Default,
        );

        assert_eq!(spec.app_label(), "jk show tvykuurw");

        let spec = ViewSpec::diff(
            "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
            DiffFormat::Default,
        );

        assert_eq!(spec.app_label(), "jk diff -r tvykuurw");
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
}
