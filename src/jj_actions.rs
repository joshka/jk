//! Preview-first action and mutation plans for `jj` commands.
//!
//! These value types own argv construction, labels, preview summaries, and
//! direct execution for user-confirmed mutation flows. Syntax quoting helpers
//! stay in [`crate::jj_syntax`]; rendered row loading stays in
//! [`crate::jj_rows`] and view-spec command construction stays in
//! [`crate::jj`].

use crate::jj::{
    ColorMode, base_command, run_direct_args, run_direct_args_stdout, summarize_output,
};
use crate::jj_syntax::{exact_change_id_revset, root_file_fileset};
use color_eyre::Result;
use color_eyre::eyre::eyre;

mod bookmarks;
mod git_sync;
mod rewrite;
mod working_copy;

pub use bookmarks::{
    JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkMutationPlan, JjBookmarkTarget,
    JjBookmarkTrackingTarget, validate_bookmark_rename_new_name,
};
pub use git_sync::{JjGitFetch, JjGitPush, JjGitPushTarget};
pub use rewrite::{JjAbsorbPlan, JjRebasePlan, JjSquashPlan};
pub use working_copy::{
    JjDuplicatePlan, JjNewPlan, JjSplitPlan, JjSplitTarget, JjWorkingCopyNavigationKind,
    JjWorkingCopyNavigationPlan,
};

const DESCRIPTION_FIRST_LINE_TEMPLATE: &str = "description.first_line() ++ \"\\n\"";

// Shared result envelope for preview and confirmed command output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    message: String,
}

impl CommandOutput {
    pub(crate) fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

// Operation recovery plans keep global undo/redo separate from selected
// operation restore/revert, which target one exact operation id.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjOperationRecoveryKind {
    Undo,
    Redo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjOperationTargetKind {
    Restore,
    Revert,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjOperationRecovery {
    kind: JjOperationRecoveryKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjOperationTarget {
    kind: JjOperationTargetKind,
    operation_id: String,
}

impl JjOperationRecovery {
    pub fn new(kind: JjOperationRecoveryKind) -> Self {
        Self { kind }
    }

    #[cfg(test)]
    pub fn kind(&self) -> JjOperationRecoveryKind {
        self.kind
    }

    pub fn command_label(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "jj undo",
            JjOperationRecoveryKind::Redo => "jj redo",
        }
    }

    pub fn command_argv(&self) -> Vec<String> {
        match self.kind {
            JjOperationRecoveryKind::Undo => vec!["undo".to_owned()],
            JjOperationRecoveryKind::Redo => vec!["redo".to_owned()],
        }
    }

    pub fn preview_text(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => concat!(
                "effect: globally undo the last operation in the current repository\n",
                "selection: the selected operation-log row is not an argument\n",
                "redo path: jj redo\n",
                "confirmation: press Enter to run jj undo",
            ),
            JjOperationRecoveryKind::Redo => concat!(
                "effect: globally redo the most recently undone operation in the current ",
                "repository\n",
                "selection: the selected operation-log row is not an argument\n",
                "failure path: jj may report that no redo is available\n",
                "confirmation: press Enter to run jj redo",
            ),
        }
    }

    pub fn success_hint(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "jj redo",
            JjOperationRecoveryKind::Redo => "jj undo",
        }
    }

    pub fn status_action(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "undo",
            JjOperationRecoveryKind::Redo => "redo",
        }
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            self.command_label(),
            self.status_action(),
        )
    }
}

impl JjOperationTargetKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Restore => "restore",
            Self::Revert => "revert",
        }
    }

    fn success_fallback(self) -> &'static str {
        match self {
            Self::Restore => "restored operation",
            Self::Revert => "reverted operation",
        }
    }
}

impl JjOperationTarget {
    pub fn restore(operation_id: impl Into<String>) -> Self {
        Self {
            kind: JjOperationTargetKind::Restore,
            operation_id: operation_id.into(),
        }
    }

    pub fn revert(operation_id: impl Into<String>) -> Self {
        Self {
            kind: JjOperationTargetKind::Revert,
            operation_id: operation_id.into(),
        }
    }

    #[cfg(test)]
    pub fn kind(&self) -> JjOperationTargetKind {
        self.kind
    }

    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub fn status_action(&self) -> &'static str {
        self.kind.label()
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        let action = match self.kind {
            JjOperationTargetKind::Restore => "restore",
            JjOperationTargetKind::Revert => "revert",
        };
        vec![
            "operation".to_owned(),
            action.to_owned(),
            self.operation_id.clone(),
        ]
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            self.kind.success_fallback(),
        )
    }

    pub fn preview_summary(&self) -> String {
        let effect = match self.kind {
            JjOperationTargetKind::Restore => {
                "effect: restore the repository state to the selected operation by creating a new operation"
            }
            JjOperationTargetKind::Revert => {
                "effect: revert exactly the selected operation by applying its inverse"
            }
        };

        [
            format!("command: {}", self.command_label()),
            String::new(),
            format!("operation id: {}", self.operation_id),
            effect.to_owned(),
            "selection: the selected operation-log row supplies this exact operation id".to_owned(),
            format!("confirmation: press Enter to run {}", self.command_label()),
            "recovery: jj undo".to_owned(),
        ]
        .join("\n")
    }
}

// Description finalization plans update messages without opening an editor;
// commit always targets @ and describe may target @ or an exact change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjDescribeTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjDescribePlan {
    target: JjDescribeTarget,
    message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjCommitPlan {
    message: String,
}

// Content mutation plans preview the diff jj will remove or reverse-apply, and
// file mutation plans restrict paths through exact root-file filesets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRestorePlan {
    target: JjRestoreTarget,
    path: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileMutationKind {
    Track,
    Untrack,
    Chmod(JjFileChmodMode),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileChmodMode {
    Executable,
    Normal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjFileMutationTarget {
    WorkingCopy { path: String },
    ExactRevision { revision: String, path: String },
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRevertPlan {
    revision: String,
}

impl JjDescribeTarget {
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    pub fn exact_change_id(&self) -> Option<&str> {
        match self {
            Self::ExactChange(change_id) => Some(change_id),
            Self::CurrentWorkingCopy => None,
        }
    }

    fn command_arg(&self) -> String {
        match self {
            Self::ExactChange(change_id) => exact_change_id_revset(change_id),
            Self::CurrentWorkingCopy => "@".to_owned(),
        }
    }

    fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => format!("exact selected revision {change_id}"),
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }
}

impl JjDescribePlan {
    pub fn new(target: JjDescribeTarget, message: impl Into<String>) -> Self {
        Self {
            target,
            message: message.into(),
        }
    }

    pub fn target(&self) -> &JjDescribeTarget {
        &self.target
    }

    pub fn command_label(&self) -> String {
        format!(
            "jj describe {} --message {}",
            self.target.label(),
            self.message
        )
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "describe".to_owned(),
            self.target.command_arg(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "described")
    }

    pub fn preview_summary(&self) -> String {
        format!(
            "command: {}\n\ntarget: {}\nmessage: {}\n\neffect: updates only the target change description without opening an editor\nconfirmation: press Enter to run jj describe\nundo path: jj undo",
            self.command_label(),
            self.target.preview_target(),
            self.message,
        )
    }
}

impl JjCommitPlan {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn command_label(&self) -> String {
        format!("jj commit --message {}", self.message)
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "commit".to_owned(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            "committed current working-copy change",
        )
    }

    pub fn preview_summary(&self) -> String {
        format!(
            "command: {}\n\ntarget: current working-copy change (@)\nmessage: {}\n\neffect: updates @ with the message and creates a new working-copy change on top\nselection: selected graph rows are not arguments to jj commit\nconfirmation: press Enter to run jj commit\nundo path: jj undo",
            self.command_label(),
            self.message,
        )
    }
}

// Content/file mutation plans share exact change/path targeting and preview
// honesty about showing jj's source diff rather than simulating the result.
impl JjRestorePlan {
    pub fn for_revision(revision: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: None,
        }
    }

    pub fn for_path(revision: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: Some(path.into()),
        }
    }

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

    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "restored")
    }

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
    pub fn track(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Track,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    pub fn untrack(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Untrack,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    pub fn chmod_working_copy(path: impl Into<String>, mode: JjFileChmodMode) -> Self {
        Self {
            kind: JjFileMutationKind::Chmod(mode),
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

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

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            self.kind.success_fallback(),
        )
    }

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

    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "reverted")
    }

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

// Abandon safety owns the preflight probes and strong-confirm preview; keep it
// separate from rewrite plans because it classifies destructive risk first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPlan {
    revision: String,
}

impl JjAbandonPlan {
    pub fn new(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
        }
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn command_label(&self) -> String {
        format!("jj abandon {}", self.revision)
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec!["abandon".to_owned(), self.exact_change_id_revset()]
    }

    pub fn diff_summary_label(&self) -> String {
        format!("jj diff -r {} --summary", self.revision)
    }

    pub fn diff_summary_argv(&self) -> Vec<String> {
        vec![
            "diff".to_owned(),
            "-r".to_owned(),
            self.exact_change_id_revset(),
            "--summary".to_owned(),
        ]
    }

    pub fn run_preview(&self) -> Result<JjAbandonPreview> {
        let summary = run_direct_args_stdout(self.diff_summary_argv(), &self.diff_summary_label())?;
        let title = self.load_title().ok().flatten();

        Ok(JjAbandonPreview::new(self.revision.clone(), title, summary))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "abandoned")
    }

    fn load_title(&self) -> Result<Option<String>> {
        let mut jj = base_command(ColorMode::Never);
        jj.args(self.title_argv());

        let output = jj.output()?;
        if !output.status.success() {
            return Err(eyre!(
                "{} title failed: {}",
                self.revision,
                summarize_output(&output.stdout, &output.stderr, "could not read title")
            ));
        }

        let title = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok((!title.is_empty()).then_some(title))
    }

    fn title_argv(&self) -> Vec<String> {
        vec![
            "log".to_owned(),
            "-r".to_owned(),
            self.exact_change_id_revset(),
            "--no-graph".to_owned(),
            "-T".to_owned(),
            DESCRIPTION_FIRST_LINE_TEMPLATE.to_owned(),
        ]
    }

    fn exact_change_id_revset(&self) -> String {
        exact_change_id_revset(&self.revision)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPreview {
    revision: String,
    title: Option<String>,
    summary: String,
    change_state: AbandonChangeState,
}

impl JjAbandonPreview {
    pub fn new(revision: String, title: Option<String>, summary: String) -> Self {
        let change_state = if summary.trim().is_empty() {
            AbandonChangeState::Empty
        } else {
            AbandonChangeState::NonEmpty
        };

        Self {
            revision,
            title,
            summary,
            change_state,
        }
    }

    #[cfg(test)]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn is_empty_change(&self) -> bool {
        self.change_state == AbandonChangeState::Empty
    }

    pub fn preview_text(&self) -> String {
        let title = self.title.as_deref().unwrap_or("<no description>");
        let summary = if self.summary.trim().is_empty() {
            "empty change".to_owned()
        } else {
            self.summary.trim().to_owned()
        };
        let confirmation = if self.is_empty_change() {
            "press Enter to abandon this empty change".to_owned()
        } else {
            format!(
                "type exact revision '{}' before abandon runs",
                self.revision
            )
        };

        format!(
            "change: {}\ntitle: {}\ndiff summary:\n{}\n\neffect: abandon removes the selected change from the visible history; recovery stays available through jj undo\nconfirmation: {}\nundo path: jj undo",
            self.revision, title, summary, confirmation
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AbandonChangeState {
    Empty,
    NonEmpty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_plan_targets_exact_change_before_message() {
        let plan = JjDescribePlan::new(
            JjDescribeTarget::exact_change("abcdefghijklmnopqrstuvwxzyabcdef"),
            "New description",
        );

        assert_eq!(
            plan.command_argv(),
            vec![
                "describe",
                "exactly(change_id(\"abcdefghijklmnopqrstuvwxzyabcdef\"), 1)",
                "--message",
                "New description"
            ]
        );
        assert_eq!(
            plan.command_label(),
            "jj describe abcdefghijklmnopqrstuvwxzyabcdef --message New description"
        );
        assert!(
            plan.preview_summary()
                .contains("target: exact selected revision")
        );
        assert!(plan.preview_summary().contains("without opening an editor"));
    }

    #[test]
    fn describe_plan_can_target_current_working_copy() {
        let plan = JjDescribePlan::new(JjDescribeTarget::current_working_copy(), "Describe @");

        assert_eq!(
            plan.command_argv(),
            vec!["describe", "@", "--message", "Describe @"]
        );
        assert_eq!(plan.command_label(), "jj describe @ --message Describe @");
        assert!(
            plan.preview_summary()
                .contains("current working-copy change (@)")
        );
    }

    #[test]
    fn commit_plan_uses_message_without_revision_argument() {
        let plan = JjCommitPlan::new("Commit working copy");

        assert_eq!(
            plan.command_argv(),
            vec!["commit", "--message", "Commit working copy"]
        );
        assert_eq!(
            plan.command_label(),
            "jj commit --message Commit working copy"
        );
        assert!(
            plan.preview_summary()
                .contains("target: current working-copy change (@)")
        );
        assert!(
            plan.preview_summary()
                .contains("selected graph rows are not arguments")
        );
    }

    #[test]
    fn restore_plan_uses_exact_change_revset_for_revision_restore() {
        let restore = JjRestorePlan::for_revision("change-a");

        assert_eq!(
            restore.command_argv(),
            vec![
                "restore",
                "--changes-in",
                "exactly(change_id(\"change-a\"), 1)"
            ]
        );
        assert_eq!(
            restore.preview_diff_argv(),
            vec!["diff", "-r", "exactly(change_id(\"change-a\"), 1)"]
        );
        assert_eq!(
            restore.command_label(),
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1)"
        );

        let preview = restore.preview_summary("M src/main.rs\n");
        assert!(preview.contains("target revision: change-a"));
        assert!(preview.contains("effect: restore removes the selected revision's forward diff"));
        assert!(preview.contains("preview source: jj diff -r exactly(change_id(\"change-a\"), 1)"));
        assert!(preview.contains("jk is not simulating the final graph"));
        assert!(preview.contains("confirmation: press Enter to run jj restore"));
        assert!(preview.contains("undo path: jj undo"));
        assert!(preview.contains("forward diff:\nM src/main.rs"));
    }

    #[test]
    fn restore_plan_uses_root_file_fileset_for_exact_paths() {
        let restore = JjRestorePlan::for_path("change-a", "dir/with spaces/quo\"te\\[glob]?*");

        assert_eq!(
            restore.command_argv(),
            vec![
                "restore",
                "--changes-in",
                "exactly(change_id(\"change-a\"), 1)",
                "root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\""
            ]
        );
        assert_eq!(
            restore.preview_diff_argv(),
            vec![
                "diff",
                "-r",
                "exactly(change_id(\"change-a\"), 1)",
                "root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\""
            ]
        );

        let preview = restore.preview_summary("A dir/with spaces/quo\"te\\[glob]?*\n");
        assert!(preview.contains("selected path: dir/with spaces/quo\"te\\[glob]?*"));
        assert!(
            preview.contains("exact fileset: root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\"")
        );
        assert!(preview.contains("effect: restore removes the selected path's forward diff"));
    }

    #[test]
    fn restore_plan_uses_default_working_copy_restore_for_status_paths() {
        let restore = JjRestorePlan::for_working_copy_path("src/status.rs");

        assert_eq!(
            restore.command_argv(),
            vec!["restore", "root-file:\"src/status.rs\""]
        );
        assert_eq!(
            restore.preview_diff_argv(),
            vec!["diff", "root-file:\"src/status.rs\""]
        );
        assert_eq!(
            restore.command_label(),
            "jj restore root-file:\"src/status.rs\""
        );

        let preview = restore.preview_summary("M src/status.rs\n");
        assert!(preview.contains("target revision: @"));
        assert!(preview.contains("selected path: src/status.rs"));
        assert!(preview.contains("working-copy diff from @"));
        assert!(preview.contains("preview source: jj diff root-file:\"src/status.rs\""));
    }

    #[test]
    fn revert_plan_uses_exact_change_revset_and_working_copy_destination() {
        let revert = JjRevertPlan::new("change-a");

        assert_eq!(
            revert.command_argv(),
            vec![
                "revert",
                "-r",
                "exactly(change_id(\"change-a\"), 1)",
                "-o",
                "@"
            ]
        );
        assert_eq!(
            revert.preview_diff_argv(),
            vec!["diff", "-r", "exactly(change_id(\"change-a\"), 1)"]
        );
        assert_eq!(
            revert.command_label(),
            "jj revert -r exactly(change_id(\"change-a\"), 1) -o @"
        );

        let preview = revert.preview_summary("M src/main.rs\n");
        assert!(preview.contains("target revision: change-a"));
        assert!(preview.contains("reverse-applies the selected revision's forward diff into @"));
        assert!(preview.contains("preview source: jj diff -r exactly(change_id(\"change-a\"), 1)"));
        assert!(preview.contains("jk is not simulating the final graph"));
        assert!(preview.contains("confirmation: press Enter to run jj revert"));
        assert!(preview.contains("undo path: jj undo"));
        assert!(preview.contains("forward diff:\nM src/main.rs"));
    }

    #[test]
    fn operation_undo_command_has_no_operation_id_argument() {
        let recovery = JjOperationRecovery::new(JjOperationRecoveryKind::Undo);
        let selected_operation_id = operation_id('c');

        assert_eq!(recovery.command_label(), "jj undo");
        assert_eq!(recovery.command_argv(), ["undo"]);
        assert!(!recovery.command_argv().contains(&selected_operation_id));
        assert!(
            recovery
                .preview_text()
                .contains("selected operation-log row is not an argument")
        );
    }

    #[test]
    fn operation_redo_command_has_no_operation_id_argument() {
        let recovery = JjOperationRecovery::new(JjOperationRecoveryKind::Redo);
        let selected_operation_id = operation_id('d');

        assert_eq!(recovery.command_label(), "jj redo");
        assert_eq!(recovery.command_argv(), ["redo"]);
        assert!(!recovery.command_argv().contains(&selected_operation_id));
        assert!(
            recovery
                .preview_text()
                .contains("selected operation-log row is not an argument")
        );
    }

    #[test]
    fn operation_restore_command_targets_exact_operation_id() {
        let operation_id = operation_id('e');
        let target = JjOperationTarget::restore(operation_id.clone());

        assert_eq!(target.kind(), JjOperationTargetKind::Restore);
        assert_eq!(target.operation_id(), operation_id.as_str());
        assert_eq!(
            target.command_argv(),
            ["operation", "restore", operation_id.as_str()]
        );
        assert_eq!(
            target.command_label(),
            format!("jj operation restore {operation_id}")
        );
        assert!(
            target
                .preview_summary()
                .contains(&format!("operation id: {operation_id}"))
        );
        assert!(target.preview_summary().contains(
            "restore the repository state to the selected operation by creating a new operation"
        ));
        assert!(target.preview_summary().contains(&format!(
            "confirmation: press Enter to run jj operation restore {operation_id}"
        )));
    }

    #[test]
    fn operation_revert_command_targets_exact_operation_id() {
        let operation_id = operation_id('f');
        let target = JjOperationTarget::revert(operation_id.clone());

        assert_eq!(target.kind(), JjOperationTargetKind::Revert);
        assert_eq!(target.operation_id(), operation_id.as_str());
        assert_eq!(
            target.command_argv(),
            ["operation", "revert", operation_id.as_str()]
        );
        assert_eq!(
            target.command_label(),
            format!("jj operation revert {operation_id}")
        );
        assert!(
            target
                .preview_summary()
                .contains(&format!("operation id: {operation_id}"))
        );
        assert!(
            target
                .preview_summary()
                .contains("revert exactly the selected operation by applying its inverse")
        );
        assert!(target.preview_summary().contains(&format!(
            "confirmation: press Enter to run jj operation revert {operation_id}"
        )));
    }

    #[test]
    fn abandon_plan_uses_exact_revision_command_shape() {
        let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            abandon.command_argv(),
            vec![
                "abandon",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
            ]
        );
        assert_eq!(
            abandon.command_label(),
            "jj abandon tvykuurwpnwzzqulzrvwvmxxotnlywqw"
        );
    }

    #[test]
    fn abandon_diff_summary_probe_uses_revision_summary() {
        let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            abandon.diff_summary_argv(),
            vec![
                "diff",
                "-r",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)",
                "--summary"
            ]
        );
        assert_eq!(
            abandon.diff_summary_label(),
            "jj diff -r tvykuurwpnwzzqulzrvwvmxxotnlywqw --summary"
        );
    }

    #[test]
    fn abandon_title_probe_uses_exact_change_id_revset() {
        let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            abandon.title_argv(),
            vec![
                "log",
                "-r",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)",
                "--no-graph",
                "-T",
                DESCRIPTION_FIRST_LINE_TEMPLATE,
            ]
        );
    }

    #[test]
    fn file_track_uses_root_file_fileset_after_double_dash() {
        let plan = JjFileMutationPlan::track("-leading dir/quo\"te\\[glob]?*.rs");

        assert_eq!(
            plan.command_argv(),
            vec![
                "file",
                "track",
                "--",
                "root-file:\"-leading dir/quo\\\"te\\\\[glob]?*.rs\""
            ]
        );
        let preview = plan.preview_summary();
        assert!(preview.contains("selected path: -leading dir/quo\"te\\[glob]?*.rs"));
        assert!(
            preview.contains("exact fileset: root-file:\"-leading dir/quo\\\"te\\\\[glob]?*.rs\"")
        );
        assert!(preview.contains("effect: starts tracking this exact untracked working-copy path"));
        assert!(preview.contains("output: jj stdout and stderr are preserved"));
        assert!(preview.contains("recovery: jj undo"));
        assert!(preview.contains("review: jj status; jj op show -p"));
    }

    #[test]
    fn file_untrack_uses_root_file_fileset_and_mentions_ignore_requirement() {
        let plan = JjFileMutationPlan::untrack("dir/file.rs");

        assert_eq!(
            plan.command_argv(),
            vec!["file", "untrack", "--", "root-file:\"dir/file.rs\""]
        );
        assert!(
            plan.preview_summary()
                .contains("jj requires the path to already be ignored")
        );
    }

    #[test]
    fn file_chmod_modes_use_installed_jj_mode_args() {
        let executable =
            JjFileMutationPlan::chmod_working_copy("src/main.rs", JjFileChmodMode::Executable);
        let normal = JjFileMutationPlan::chmod_working_copy("src/main.rs", JjFileChmodMode::Normal);

        assert_eq!(
            executable.command_argv(),
            vec!["file", "chmod", "x", "--", "root-file:\"src/main.rs\""]
        );
        assert_eq!(
            normal.command_argv(),
            vec!["file", "chmod", "n", "--", "root-file:\"src/main.rs\""]
        );
        assert!(
            executable
                .preview_summary()
                .contains("chmod mode: x (executable)")
        );
        assert!(normal.preview_summary().contains("chmod mode: n (normal)"));
    }

    #[test]
    fn exact_revision_file_chmod_uses_exact_change_revset_before_mode_and_fileset() {
        let plan = JjFileMutationPlan::chmod_exact_revision(
            "change-a",
            "dir/space file.rs",
            JjFileChmodMode::Executable,
        );

        assert_eq!(
            plan.command_argv(),
            vec![
                "file",
                "chmod",
                "-r",
                "exactly(change_id(\"change-a\"), 1)",
                "x",
                "--",
                "root-file:\"dir/space file.rs\""
            ]
        );
        assert_eq!(
            plan.command_label(),
            "jj file chmod -r exactly(change_id(\"change-a\"), 1) x -- root-file:\"dir/space file.rs\""
        );
        assert!(
            plan.preview_summary()
                .contains("target scope: exact selected revision change-a")
        );
    }

    #[test]
    fn abandon_preview_classifies_empty_summary_as_empty_change() {
        let preview = JjAbandonPreview::new(
            "change-id".to_owned(),
            Some("Start feature".to_owned()),
            "\n".to_owned(),
        );

        assert!(preview.is_empty_change());
        assert_eq!(preview.revision(), "change-id");
        assert!(preview.preview_text().contains("title: Start feature"));
        assert!(
            preview
                .preview_text()
                .contains("diff summary:\nempty change")
        );
        assert!(
            preview
                .preview_text()
                .contains("press Enter to abandon this empty change")
        );
        assert!(preview.preview_text().contains("undo path: jj undo"));
    }

    #[test]
    fn abandon_preview_classifies_non_empty_summary_as_strong_confirm() {
        let preview = JjAbandonPreview::new(
            "change-id".to_owned(),
            Some("Edit files".to_owned()),
            "M src/main.rs\nA README.md\n".to_owned(),
        );

        assert!(!preview.is_empty_change());
        let text = preview.preview_text();
        assert!(text.contains("change: change-id"));
        assert!(text.contains("title: Edit files"));
        assert!(text.contains("M src/main.rs\nA README.md"));
        assert!(text.contains("type exact revision 'change-id' before abandon runs"));
        assert!(text.contains("undo path: jj undo"));
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }
}
