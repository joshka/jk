//! View specifications for rendered `jj` views.
//!
//! This module owns the provenance carried with a `jj` invocation after it
//! leaves the command/process boundary: how views are constructed, which target
//! is safe to treat as an exact change id, how labels present navigated targets,
//! how the app-level diff format toggle rewrites args, and how direct startup
//! commands recover a navigation revset from argv.

use super::{JjCommand, LogViewMode, option_value};

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
    pub(super) command: JjCommand,
    pub(super) args: Vec<String>,
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

    pub fn workspaces(args: Vec<String>) -> Self {
        Self {
            command: JjCommand::Workspaces,
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
            JjCommand::Workspaces => "jk workspaces",
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
            | JjCommand::Workspaces
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

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use super::super::{ALL_REPO_REVSET, RECENT_WORK_REVSET, TRUNK_WORK_REVSET};
    use super::*;

    #[test]
    fn bookmark_list_spec_uses_bookmark_labels() {
        let spec = ViewSpec::bookmarks(vec!["--revision".to_owned(), "main".to_owned()]);

        assert_eq!(spec.command(), JjCommand::Bookmarks);
        assert_eq!(spec.args(), ["--revision", "main"]);
        assert_eq!(spec.label(), "jj bookmark list --revision main");
        assert_eq!(spec.app_label(), "jk bookmarks --revision main");
    }

    #[test]
    fn workspace_spec_uses_workspace_labels() {
        let spec = ViewSpec::workspaces(Vec::new());

        assert_eq!(spec.command(), JjCommand::Workspaces);
        assert!(spec.args().is_empty());
        assert_eq!(spec.label(), "jj workspace list");
        assert_eq!(spec.app_label(), "jk workspaces");
    }

    #[test]
    fn file_list_spec_keeps_selected_path_out_of_args() {
        let spec = ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned()));

        assert_eq!(spec.command(), JjCommand::FileList);
        assert_eq!(spec.args(), ["-r", "main"]);
        assert_eq!(spec.path(), Some("src/main.rs"));
        assert_eq!(spec.exact_change_target(), None);
        assert_eq!(spec.label(), "jj file list -r main");
        assert_eq!(spec.app_label(), "jk file list -r main");
        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
        assert_eq!(spec.show_context_revset(), "main");
    }

    #[test]
    fn file_show_spec_keeps_exact_path_identity() {
        let spec = ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned());

        assert_eq!(spec.command(), JjCommand::FileShow);
        assert_eq!(spec.args(), ["-r", "main", "src/main.rs"]);
        assert_eq!(spec.path(), Some("src/main.rs"));
        assert_eq!(spec.exact_change_target(), None);
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
    fn resolve_spec_defaults_to_current_revision() {
        let spec = ViewSpec::resolve(None);

        assert_eq!(spec.command(), JjCommand::Resolve);
        assert_eq!(spec.args(), ["-r", "@"]);
        assert_eq!(spec.label(), "jj resolve -r @");
        assert_eq!(spec.app_label(), "jk resolve -r @");
        assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
        assert_eq!(spec.show_context_revset(), "@");
    }

    #[test]
    fn resolve_spec_records_direct_revset_without_exact_target() {
        let spec = ViewSpec::resolve(Some("main".to_owned()));

        assert_eq!(spec.command(), JjCommand::Resolve);
        assert_eq!(spec.args(), ["-r", "main"]);
        assert_eq!(spec.exact_change_target(), None);
        assert_eq!(spec.label(), "jj resolve -r main");
        assert_eq!(spec.app_label(), "jk resolve -r main");
        assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
        assert_eq!(spec.show_context_revset(), "main");
    }

    #[test]
    fn operation_show_spec_uses_positional_operation_id() {
        let spec = ViewSpec::operation_show(operation_id('a'));

        assert_eq!(spec.command(), JjCommand::OperationShow);
        assert_eq!(spec.args(), [operation_id('a')]);
        assert_eq!(spec.app_label(), "jk operation show aaaaaaaa");
    }

    #[test]
    fn operation_diff_spec_uses_operation_option() {
        let spec = ViewSpec::operation_diff(operation_id('b'));

        assert_eq!(spec.command(), JjCommand::OperationDiff);
        assert_eq!(spec.args(), ["--operation", operation_id('b').as_str()]);
        assert_eq!(spec.app_label(), "jk operation diff --operation bbbbbbbb");
    }

    #[test]
    fn file_views_ignore_diff_format_toggle() {
        let show_spec = ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned());
        let list_spec = ViewSpec::file_list(Some("main".to_owned()), None);

        assert_eq!(show_spec.with_diff_format(DiffFormat::Git), show_spec);
        assert_eq!(list_spec.with_diff_format(DiffFormat::Git), list_spec);
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
    fn log_view_mode_uses_recent_revset_for_recent_mode() {
        let spec = ViewSpec::for_log_mode(JjCommand::Default, &LogViewMode::Recent);

        assert_eq!(spec.command(), JjCommand::Log);
        assert_eq!(spec.args(), ["-r", RECENT_WORK_REVSET]);
    }

    #[test]
    fn log_view_mode_uses_all_revset_for_all_mode() {
        let spec = ViewSpec::for_log_mode(JjCommand::Default, &LogViewMode::All);

        assert_eq!(spec.command(), JjCommand::Log);
        assert_eq!(spec.args(), ["-r", ALL_REPO_REVSET]);
    }
}
