//! File and content mutation command plans.
//!
//! These plans own argv, preview wording, and direct execution for `jj restore`, `jj revert`, and
//! `jj file` mutations after a view or menu has already selected the action target. Availability
//! policy stays with the feature surface that offers the action.

use super::CommandOutput;
use crate::jj::{exact_change_id_revset, root_file_fileset};
use crate::jj::{run_direct_args, run_direct_args_stdout};
use color_eyre::Result;

/// Preview-first plan for `jj restore`.
///
/// Previews show the forward diff that jj will remove. `jk` does not simulate the resulting graph
/// or file content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRestorePlan {
    target: JjRestoreTarget,
    path: Option<String>,
}

/// File mutation subcommand represented by a preview-first plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileMutationKind {
    Track,
    Untrack,
    Chmod(JjFileChmodMode),
}

/// File executable-bit mode accepted by `jj file chmod`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileChmodMode {
    Executable,
    Normal,
}

/// Exact target scope for a file mutation.
///
/// Paths are stored as repository-root file paths and converted to exact root-file filesets during
/// argv construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjFileMutationTarget {
    WorkingCopy { path: String },
    ExactRevision { revision: String, path: String },
}

/// Preview-first plan for `jj file track`, `untrack`, and `chmod`.
///
/// The plan owns argv construction for exact filesets. It does not decide whether a file action is
/// available; that policy belongs to the selected view/action-menu owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjFileMutationPlan {
    kind: JjFileMutationKind,
    target: JjFileMutationTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum JjRestoreTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

/// Preview-first plan for reverse-applying one exact revision into `@`.
///
/// Previews show the selected revision's forward diff because that is the source jj will
/// reverse-apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRevertPlan {
    revision: String,
}

impl JjRestorePlan {
    /// Restore the entire forward diff from one exact rendered revision.
    pub fn for_revision(revision: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: None,
        }
    }

    /// Restore one repository-root path from one exact rendered revision.
    pub fn for_path(revision: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: Some(path.into()),
        }
    }

    /// Restore one repository-root path from the current working-copy diff.
    pub fn for_working_copy_path(path: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::CurrentWorkingCopy,
            path: Some(path.into()),
        }
    }

    pub fn revision(&self) -> &str {
        self.target.label()
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = match &self.target {
            JjRestoreTarget::ExactChange(revision) => vec![
                "restore".to_owned(),
                "--changes-in".to_owned(),
                exact_change_id_revset(revision),
            ],
            JjRestoreTarget::CurrentWorkingCopy => vec!["restore".to_owned()],
        };
        if let Some(path) = &self.path {
            argv.push(root_file_fileset(path));
        }
        argv
    }

    pub fn preview_diff_label(&self) -> String {
        let label_args = self.preview_diff_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn preview_diff_argv(&self) -> Vec<String> {
        let mut argv = match &self.target {
            JjRestoreTarget::ExactChange(revision) => vec![
                "diff".to_owned(),
                "-r".to_owned(),
                exact_change_id_revset(revision),
            ],
            JjRestoreTarget::CurrentWorkingCopy => vec!["diff".to_owned()],
        };
        if let Some(path) = &self.path {
            argv.push(root_file_fileset(path));
        }
        argv
    }

    /// Load jj's forward diff and wrap it with the restore preview contract.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    /// Execute `jj restore`; confirmation and result transitions happen outside the plan.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "restored")
    }

    /// Summarize the restore using jj's forward diff rather than a simulated result.
    pub fn preview_summary(&self, forward_diff: &str) -> String {
        let mut lines = vec![
            format!("target revision: {}", self.revision()),
            format!("command: {}", self.command_label()),
        ];

        match &self.path {
            Some(path) => {
                lines.push(format!("selected path: {path}"));
                lines.push(format!("exact fileset: {}", root_file_fileset(path)));
                lines.push(self.target.path_restore_effect());
            }
            None => lines.push(
                "effect: restore removes the selected revision's forward diff from that exact revision"
                    .to_owned(),
            ),
        }

        lines.extend([
            format!("preview source: {}", self.preview_diff_label()),
            "honesty: the output below is the forward diff that jj restore removes; jk is not simulating the final graph".to_owned(),
            "confirmation: press Enter to run jj restore".to_owned(),
            "undo path: jj undo".to_owned(),
            String::new(),
            "forward diff:".to_owned(),
            forward_diff.trim_end().to_owned(),
        ]);

        lines.join("\n")
    }
}

impl JjRestoreTarget {
    fn label(&self) -> &str {
        match self {
            Self::ExactChange(revision) => revision,
            Self::CurrentWorkingCopy => "@",
        }
    }

    fn path_restore_effect(&self) -> String {
        match self {
            Self::ExactChange(_) => {
                "effect: restore removes the selected path's forward diff from that exact revision"
                    .to_owned()
            }
            Self::CurrentWorkingCopy => {
                "effect: restore removes the selected path's working-copy diff from @".to_owned()
            }
        }
    }
}

impl JjFileChmodMode {
    pub fn command_arg(self) -> &'static str {
        match self {
            Self::Executable => "x",
            Self::Normal => "n",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::Normal => "normal",
        }
    }
}

impl JjFileMutationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Untrack => "untrack",
            Self::Chmod(JjFileChmodMode::Executable) => "chmod x",
            Self::Chmod(JjFileChmodMode::Normal) => "chmod n",
        }
    }

    fn success_fallback(self) -> &'static str {
        match self {
            Self::Track => "tracked file",
            Self::Untrack => "untracked file",
            Self::Chmod(JjFileChmodMode::Executable) => "set file executable",
            Self::Chmod(JjFileChmodMode::Normal) => "set file normal",
        }
    }
}

impl JjFileMutationTarget {
    fn path(&self) -> &str {
        match self {
            Self::WorkingCopy { path } | Self::ExactRevision { path, .. } => path,
        }
    }

    fn revision(&self) -> Option<&str> {
        match self {
            Self::WorkingCopy { .. } => None,
            Self::ExactRevision { revision, .. } => Some(revision),
        }
    }

    fn scope_label(&self) -> String {
        match self {
            Self::WorkingCopy { .. } => "working-copy change (@)".to_owned(),
            Self::ExactRevision { revision, .. } => {
                format!("exact selected revision {revision}")
            }
        }
    }
}

impl JjFileMutationPlan {
    /// Track an exact repository-root working-copy path.
    pub fn track(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Track,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    /// Untrack an exact repository-root working-copy path.
    pub fn untrack(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Untrack,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    /// Change executable mode for an exact repository-root path in `@`.
    pub fn chmod_working_copy(path: impl Into<String>, mode: JjFileChmodMode) -> Self {
        Self {
            kind: JjFileMutationKind::Chmod(mode),
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    /// Change executable mode for an exact repository-root path in a rendered revision.
    pub fn chmod_exact_revision(
        revision: impl Into<String>,
        path: impl Into<String>,
        mode: JjFileChmodMode,
    ) -> Self {
        Self {
            kind: JjFileMutationKind::Chmod(mode),
            target: JjFileMutationTarget::ExactRevision {
                revision: revision.into(),
                path: path.into(),
            },
        }
    }

    pub fn kind(&self) -> JjFileMutationKind {
        self.kind
    }

    pub fn path(&self) -> &str {
        self.target.path()
    }

    pub fn revision(&self) -> Option<&str> {
        self.target.revision()
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        let fileset = root_file_fileset(self.path());
        match self.kind {
            JjFileMutationKind::Track => {
                vec![
                    "file".to_owned(),
                    "track".to_owned(),
                    "--".to_owned(),
                    fileset,
                ]
            }
            JjFileMutationKind::Untrack => {
                vec![
                    "file".to_owned(),
                    "untrack".to_owned(),
                    "--".to_owned(),
                    fileset,
                ]
            }
            JjFileMutationKind::Chmod(mode) => {
                let mut argv = vec!["file".to_owned(), "chmod".to_owned()];
                if let Some(revision) = self.revision() {
                    argv.push("-r".to_owned());
                    argv.push(exact_change_id_revset(revision));
                }
                argv.push(mode.command_arg().to_owned());
                argv.push("--".to_owned());
                argv.push(fileset);
                argv
            }
        }
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Execute the file mutation; availability and confirmation are external policy.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            self.kind.success_fallback(),
        )
    }

    /// Summarize the exact target scope without pretending to know jj's final status output.
    pub fn preview_summary(&self) -> String {
        let mut lines = vec![
            format!("command: {}", self.command_label()),
            String::new(),
            format!("selected path: {}", self.path()),
            format!("exact fileset: {}", root_file_fileset(self.path())),
            format!("target scope: {}", self.target.scope_label()),
        ];

        if let JjFileMutationKind::Chmod(mode) = self.kind {
            lines.push(format!(
                "chmod mode: {} ({})",
                mode.command_arg(),
                mode.label()
            ));
        }

        lines.extend([
            self.effect_summary(),
            "confirmation: press Enter to run the command above".to_owned(),
            "output: jj stdout and stderr are preserved in this result pane".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj status; jj op show -p".to_owned(),
        ]);
        lines.join("\n")
    }

    fn effect_summary(&self) -> String {
        match self.kind {
            JjFileMutationKind::Track => {
                "effect: starts tracking this exact untracked working-copy path".to_owned()
            }
            JjFileMutationKind::Untrack => {
                "effect: stops tracking this exact working-copy path; jj requires the path to already be ignored".to_owned()
            }
            JjFileMutationKind::Chmod(JjFileChmodMode::Executable) => {
                "effect: marks this exact path executable in the target scope".to_owned()
            }
            JjFileMutationKind::Chmod(JjFileChmodMode::Normal) => {
                "effect: marks this exact path non-executable in the target scope".to_owned()
            }
        }
    }
}

impl JjRevertPlan {
    /// Build a revert plan for one exact rendered revision into `@`.
    pub fn new(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
        }
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "revert".to_owned(),
            "-r".to_owned(),
            exact_change_id_revset(&self.revision),
            "-o".to_owned(),
            "@".to_owned(),
        ]
    }

    pub fn preview_diff_label(&self) -> String {
        let label_args = self.preview_diff_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn preview_diff_argv(&self) -> Vec<String> {
        vec![
            "diff".to_owned(),
            "-r".to_owned(),
            exact_change_id_revset(&self.revision),
        ]
    }

    /// Load jj's forward diff and wrap it with the revert preview contract.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    /// Execute `jj revert`; refresh and result-screen transitions happen outside the plan.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "reverted")
    }

    /// Summarize revert using jj's forward diff rather than a simulated result.
    pub fn preview_summary(&self, forward_diff: &str) -> String {
        [
            format!("target revision: {}", self.revision),
            format!("command: {}", self.command_label()),
            "effect: revert reverse-applies the selected revision's forward diff into @".to_owned(),
            format!("preview source: {}", self.preview_diff_label()),
            "honesty: the output below is the forward diff that jj revert reverse-applies into @; jk is not simulating the final graph".to_owned(),
            "confirmation: press Enter to run jj revert".to_owned(),
            "undo path: jj undo".to_owned(),
            String::new(),
            "forward diff:".to_owned(),
            forward_diff.trim_end().to_owned(),
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests;
