//! Root preview-first action and mutation plans for `jj` commands.
//!
//! Root plans own argv labels, argv construction, preview summaries, and direct execution envelopes
//! for user-confirmed mutation flows. They preserve preview honesty by showing the exact `jj`
//! command that will run, exact revsets/filesets where a target comes from rendered metadata, and
//! `jj`'s own preview output where available instead of simulating final graph or file results.
//!
//! Family modules and feature-owned action modules own their narrower command areas:
//!
//! - [`git_sync`] owns Git fetch and push plans.
//! - [`operation`] owns operation recovery plans.
//! - [`rewrite`] owns rewrite plans such as absorb, rebase, and squash.
//! - [`working_copy`] owns working-copy creation, duplication, splitting, and navigation plans.
//!
//! Feature views and action menus own availability decisions and target selection. The app
//! lifecycle owns prompt flow, confirmation strength, refresh/reveal policy, and result-screen
//! transitions after a plan runs. Syntax quoting helpers stay in [`crate::jj_syntax`]; rendered row
//! loading stays in [`crate::jj_rows`] and view-spec command construction stays in [`crate::jj`].

use crate::jj::{
    ColorMode, base_command, run_direct_args, run_direct_args_stdout, summarize_output,
};
use crate::jj_syntax::{exact_change_id_revset, root_file_fileset};
use color_eyre::Result;
use color_eyre::eyre::eyre;

mod git_sync;
mod operation;
mod rewrite;
mod working_copy;

// Re-export plan types as the boundary consumed by views, menus, and the app lifecycle. The
// owning modules keep family-specific policy local while this root module keeps the top-level
// action vocabulary discoverable from one import path.
pub use crate::bookmarks::actions::{
    JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkMutationPlan, JjBookmarkTarget,
    validate_bookmark_rename_new_name,
};
pub use git_sync::{JjGitFetch, JjGitPush, JjGitPushTarget};
pub use operation::{JjOperationRecovery, JjOperationRecoveryKind, JjOperationTarget};
pub use rewrite::{JjAbsorbPlan, JjRebasePlan, JjSquashPlan};
pub use working_copy::{
    JjDuplicatePlan, JjNewPlan, JjSplitPlan, JjSplitTarget, JjWorkingCopyNavigationKind,
    JjWorkingCopyNavigationPlan,
};

const DESCRIPTION_FIRST_LINE_TEMPLATE: &str = "description.first_line() ++ \"\\n\"";

/// Shared result envelope for preview and confirmed command output.
///
/// `CommandOutput` deliberately carries only presentation-ready text. Preview plans put their
/// honest preview summary here; confirmed execution plans put preserved `jj` stdout/stderr or a
/// narrow fallback message here. Callers display the message in the action-output pane instead of
/// reparsing command output, reconstructing jj wording, or inferring follow-up state transitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    message: String,
}

impl CommandOutput {
    /// Wrap presentation-ready output from a preview or confirmed execution path.
    pub(crate) fn new(message: String) -> Self {
        Self { message }
    }

    /// Return output exactly as the result pane should present it.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Target for non-interactive `jj describe --message` finalization.
///
/// Exact changes come from rendered row metadata and are quoted before argv construction; the
/// working-copy target stays as jj's `@` revset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjDescribeTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

/// Preview-first plan for updating a change description without opening an editor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjDescribePlan {
    target: JjDescribeTarget,
    message: String,
}

/// Preview-first plan for committing the current working-copy change.
///
/// This plan never consumes graph selection; `jj commit` always acts on `@` and creates the next
/// working-copy change according to jj's normal behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjCommitPlan {
    message: String,
}

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

impl JjDescribeTarget {
    /// Target an exact rendered change id, quoted during argv construction.
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    /// Target jj's current working-copy revset (`@`) without exact-change quoting.
    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    /// Expose the exact change id only when the target came from rendered row metadata.
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
    /// Build a non-interactive describe plan; prompt collection stays with the app lifecycle.
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

    /// Execute the described argv directly; confirmation and refresh/reveal happen outside.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "described")
    }

    /// Summarize the effect without predicting jj's post-command graph state.
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
    /// Build a commit plan for `@`; selected graph rows are intentionally not accepted.
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

    /// Execute `jj commit`; follow-up navigation remains an app lifecycle concern.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            "committed current working-copy change",
        )
    }

    /// Summarize jj's working-copy commit effect without inspecting future state.
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

/// Preview-first plan for abandoning one exact revision.
///
/// Abandon safety owns the preflight probes and strong-confirm preview; keep it separate from
/// rewrite plans because it classifies destructive risk before command execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPlan {
    revision: String,
}

impl JjAbandonPlan {
    /// Build an abandon plan for one exact rendered revision.
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

    /// Load jj preflight text for the confirmation screen without deciding app transitions.
    pub fn run_preview(&self) -> Result<JjAbandonPreview> {
        let summary = run_direct_args_stdout(self.diff_summary_argv(), &self.diff_summary_label())?;
        let title = self.load_title().ok().flatten();

        Ok(JjAbandonPreview::new(self.revision.clone(), title, summary))
    }

    /// Execute `jj abandon`; refresh/reveal and result-screen routing are external policy.
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

/// Preflight result for an abandon confirmation screen.
///
/// The preview keeps jj's diff summary text and only classifies empty versus non-empty changes for
/// confirmation strength. It does not decide refresh or reveal behavior after abandon completes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPreview {
    revision: String,
    title: Option<String>,
    summary: String,
    change_state: AbandonChangeState,
}

impl JjAbandonPreview {
    /// Classify preflight output only by whether jj reported a non-empty diff summary.
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

    /// Return whether abandon can use the weaker empty-change confirmation flow.
    pub fn is_empty_change(&self) -> bool {
        self.change_state == AbandonChangeState::Empty
    }

    /// Build the confirmation text from jj preflight output without simulating abandon results.
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
mod tests;
