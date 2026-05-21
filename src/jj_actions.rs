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
//! - [`crate::operation_log::actions`] owns operation recovery plans.
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
use crate::jj_syntax::exact_change_id_revset;
use color_eyre::Result;
use color_eyre::eyre::eyre;

mod files;
mod git_sync;
mod rewrite;
mod working_copy;

// Re-export plan types as the boundary consumed by views, menus, and the app lifecycle. The
// owning modules keep family-specific policy local while this root module keeps the top-level
// action vocabulary discoverable from one import path.
pub use crate::bookmarks::actions::{
    JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkMutationPlan, JjBookmarkTarget,
    validate_bookmark_rename_new_name,
};
pub use crate::operation_log::actions::{
    JjOperationRecovery, JjOperationRecoveryKind, JjOperationTarget,
};
#[allow(unused_imports)]
pub use files::{
    JjFileChmodMode, JjFileMutationKind, JjFileMutationPlan, JjFileMutationTarget, JjRestorePlan,
    JjRevertPlan,
};
pub use git_sync::{JjGitFetch, JjGitPush, JjGitPushTarget};
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
