//! Common action preview key flow for app-owned mutation screens.
//!
//! Action output owns scrolling. The app owns what cancellation, completion, and confirmation mean
//! for each pending jj action.

mod app;
mod mode;

use crossterm::event::KeyCode;

use crate::actions::{
    JjAbsorbPlan, JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjNewPlan, JjOperationRecovery, JjOperationTarget,
    JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};
use crate::app::actions::{ActionPane, ActionPaneKey, handle_action_pane_key};

/// Reduced preview-pane event returned after applying one key to a shared action pane.
pub enum ActionPreviewEvent {
    /// The pane remains open after a scroll or ignored key.
    StayOpen,
    /// A completed result pane should close and return to normal mode.
    CloseCompleted,
    /// A pending preview pane should close and surface a cancellation message.
    CancelPending(String),
    /// The preview was accepted and should run the corresponding command.
    Confirm(ActionPreviewConfirmation),
}

/// Deferred confirmation payload that preserves the plan and status context for one action pane.
pub enum ActionPreviewConfirmation {
    Describe {
        /// Prepared describe plan that should now run.
        describe: JjDescribePlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Commit {
        /// Prepared commit plan that should now run.
        commit: JjCommitPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    BookmarkMutation {
        /// Prepared bookmark mutation plan that should now run.
        mutation: JjBookmarkMutationPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    FileMutation {
        /// Prepared file mutation plan that should now run.
        mutation: JjFileMutationPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    New {
        /// Prepared new-change plan that should now run.
        new_change: JjNewPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Duplicate {
        /// Prepared duplicate plan that should now run.
        duplicate: JjDuplicatePlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Rebase {
        /// Prepared rebase plan that should now run.
        rebase: JjRebasePlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Split {
        /// Prepared split plan that should now run.
        split: JjSplitPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Restore {
        /// Prepared restore plan that should now run.
        restore: JjRestorePlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Revert {
        /// Prepared revert plan that should now run.
        revert: JjRevertPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Squash {
        /// Prepared squash plan that should now run.
        squash: JjSquashPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Absorb {
        /// Prepared absorb plan that should now run.
        absorb: JjAbsorbPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Push {
        /// Prepared push action that should now run.
        push: JjGitPush,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    Fetch {
        /// Prepared fetch action that should now run.
        fetch: JjGitFetch,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    OperationRecovery {
        /// Prepared undo/redo recovery action that should now run.
        recovery: JjOperationRecovery,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    OperationTarget {
        /// Prepared restore/revert operation action that should now run.
        target: JjOperationTarget,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
    WorkingCopyNavigation {
        /// Prepared edit/next/prev navigation action that should now run.
        navigation: JjWorkingCopyNavigationPlan,
        /// Preview status context that should remain visible on the result pane.
        status_context: Option<String>,
    },
}

/// Reduce one key for a preview/result pane into close, cancel, or confirm behavior.
fn action_preview_event(
    code: KeyCode,
    output: &mut ActionPane,
    visible_lines: u16,
    cancel_message: String,
    confirm: impl FnOnce(Option<String>) -> ActionPreviewConfirmation,
) -> ActionPreviewEvent {
    let completed = output.completed();
    let status_context = output.status_context().cloned();

    match handle_action_pane_key(code, output, visible_lines) {
        ActionPaneKey::Cancel => {
            if completed {
                ActionPreviewEvent::CloseCompleted
            } else {
                ActionPreviewEvent::CancelPending(cancel_message)
            }
        }
        ActionPaneKey::Primary => {
            if completed {
                ActionPreviewEvent::CloseCompleted
            } else {
                ActionPreviewEvent::Confirm(confirm(status_context))
            }
        }
        ActionPaneKey::Handled | ActionPaneKey::Ignored => ActionPreviewEvent::StayOpen,
    }
}
