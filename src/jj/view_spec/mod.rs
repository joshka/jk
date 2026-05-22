//! View specifications for rendered `jj` views.
//!
//! This module owns the provenance carried with a `jj` invocation after it
//! leaves the command/process boundary: how views are constructed, which target
//! is safe to treat as an exact change id, how labels present navigated targets,
//! how the app-level diff format toggle rewrites args, and how direct startup
//! commands recover a navigation revset from argv.

use super::command::option_value;
use super::{JjCommand, LogViewMode};

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
mod tests;
