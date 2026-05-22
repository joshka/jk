//! Preview-first action and mutation plans for `jj` commands.
//!
//! These plans own argv labels, argv construction, preview summaries, and direct execution
//! envelopes for user-confirmed mutation flows. They preserve preview honesty by showing the exact
//! `jj` command that will run, exact revsets/filesets where a target comes from rendered metadata,
//! and `jj`'s own preview output where available instead of simulating final graph or file results.
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
//! transitions after a plan runs. Syntax quoting helpers stay in [`crate::jj`]; rendered row
//! loading stays in [`crate::rendered_rows`] and view-spec command construction stays in [`crate::jj`].
//! This root is intentionally shared as a vocabulary and re-export boundary after target selection.
//! Do not move feature-owned availability rules, row eligibility checks, or prompt routing here;
//! those stay with the feature or app action lifecycle that chose the target.

mod abandon;
mod describe;
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
pub use abandon::{JjAbandonPlan, JjAbandonPreview};
pub use describe::{JjCommitPlan, JjDescribePlan, JjDescribeTarget};
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

/// Shared result envelope for preview and confirmed command output.
///
/// `CommandOutput` deliberately carries only presentation-ready text. Preview plans put their
/// honest preview summary here; confirmed execution plans put preserved `jj` stdout/stderr or a
/// narrow fallback message here. Callers display the message in the action-output pane instead of
/// reparsing command output, reconstructing jj wording, or inferring follow-up state transitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    /// Presentation-ready text shown in the preview or result pane.
    message: String,
}

impl CommandOutput {
    /// Wrap presentation-ready output from a preview or confirmed execution path.
    pub fn new(message: String) -> Self {
        Self { message }
    }

    /// Return output exactly as the result pane should present it.
    pub fn message(&self) -> &str {
        &self.message
    }
}
