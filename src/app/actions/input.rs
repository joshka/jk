//! Common action preview key flow for app-owned mutation screens.
//!
//! Action output owns scrolling. The app owns what cancellation, completion, and confirmation mean
//! for each pending jj action.

use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;

use crate::action_pane::{
    ActionPane, ActionPaneKey, action_pane_visible_lines, handle_action_pane_key,
};
use crate::actions::{
    JjAbsorbPlan, JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjNewPlan, JjOperationRecovery, JjOperationTarget,
    JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};
use crate::modes::InteractionMode;
use crate::status_line::StatusLine;

use super::super::App;

impl App {
    /// Route one key through the common preview/result-pane reducer shared by action overlays.
    pub(in crate::app) fn handle_common_action_preview_key(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
        terminal: Option<&mut DefaultTerminal>,
    ) -> bool {
        let Some(event) = self.mode.common_action_preview_event(code, viewport_height) else {
            return false;
        };

        self.apply_action_preview_event(event, viewport_height, terminal);
        true
    }

    /// Apply one reduced preview event to app-owned mode, status, or confirmation flow.
    fn apply_action_preview_event(
        &mut self,
        event: ActionPreviewEvent,
        viewport_height: u16,
        terminal: Option<&mut DefaultTerminal>,
    ) {
        match event {
            ActionPreviewEvent::StayOpen => {}
            ActionPreviewEvent::CloseCompleted => self.mode = InteractionMode::Normal,
            ActionPreviewEvent::CancelPending(message) => {
                self.mode = InteractionMode::Normal;
                self.status = StatusLine::with_message(&self.view, message);
            }
            ActionPreviewEvent::Confirm(confirmation) => {
                self.confirm_action_preview(confirmation, viewport_height, terminal);
            }
        }
    }

    /// Dispatch one accepted preview confirmation into the matching completion method.
    fn confirm_action_preview(
        &mut self,
        confirmation: ActionPreviewConfirmation,
        viewport_height: u16,
        terminal: Option<&mut DefaultTerminal>,
    ) {
        match confirmation {
            ActionPreviewConfirmation::Describe {
                describe,
                status_context,
            } => self.confirm_describe(describe, status_context, viewport_height),
            ActionPreviewConfirmation::Commit {
                commit,
                status_context,
            } => self.confirm_commit(commit, status_context, viewport_height),
            ActionPreviewConfirmation::BookmarkMutation {
                mutation,
                status_context,
            } => self.confirm_bookmark_mutation(mutation, status_context, viewport_height),
            ActionPreviewConfirmation::FileMutation {
                mutation,
                status_context,
            } => self.confirm_file_mutation(mutation, status_context, viewport_height),
            ActionPreviewConfirmation::New {
                new_change,
                status_context,
            } => self.confirm_new_change(new_change, status_context, viewport_height),
            ActionPreviewConfirmation::Duplicate {
                duplicate,
                status_context,
            } => self.confirm_duplicate(duplicate, status_context, viewport_height),
            ActionPreviewConfirmation::Rebase {
                rebase,
                status_context,
            } => self.confirm_rebase(rebase, status_context, viewport_height),
            ActionPreviewConfirmation::Split {
                split,
                status_context,
            } => self.confirm_split(split, status_context, viewport_height, terminal),
            ActionPreviewConfirmation::Restore {
                restore,
                status_context,
            } => self.confirm_restore(restore, status_context, viewport_height),
            ActionPreviewConfirmation::Revert {
                revert,
                status_context,
            } => self.confirm_revert(revert, status_context, viewport_height),
            ActionPreviewConfirmation::Squash {
                squash,
                status_context,
            } => self.confirm_squash(squash, status_context, viewport_height),
            ActionPreviewConfirmation::Absorb {
                absorb,
                status_context,
            } => self.confirm_absorb(absorb, status_context, viewport_height),
            ActionPreviewConfirmation::Push {
                push,
                status_context,
            } => self.confirm_push(push, status_context, viewport_height),
            ActionPreviewConfirmation::Fetch {
                fetch,
                status_context,
            } => self.confirm_fetch(fetch, status_context, viewport_height),
            ActionPreviewConfirmation::OperationRecovery {
                recovery,
                status_context,
            } => self.confirm_operation_recovery(recovery, status_context, viewport_height),
            ActionPreviewConfirmation::OperationTarget {
                target,
                status_context,
            } => self.confirm_operation_target(target, status_context, viewport_height),
            ActionPreviewConfirmation::WorkingCopyNavigation {
                navigation,
                status_context,
            } => self.confirm_working_copy_navigation(navigation, status_context, viewport_height),
        }
    }
}

impl InteractionMode {
    /// Reduce one key for any preview/result mode that shares the common action-pane behavior.
    fn common_action_preview_event(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> Option<ActionPreviewEvent> {
        let visible_lines = action_pane_visible_lines(viewport_height);
        match self {
            Self::DescribePreview { describe, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "describe cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Describe {
                    describe: describe.clone(),
                    status_context,
                },
            )),
            Self::CommitPreview { commit, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "commit cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Commit {
                    commit: commit.clone(),
                    status_context,
                },
            )),
            Self::BookmarkMutationPreview { mutation, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                format!("bookmark {} cancelled", mutation.kind().label()),
                |status_context| ActionPreviewConfirmation::BookmarkMutation {
                    mutation: mutation.clone(),
                    status_context,
                },
            )),
            Self::FileMutationPreview { mutation, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                format!("file {} cancelled", mutation.kind().label()),
                |status_context| ActionPreviewConfirmation::FileMutation {
                    mutation: mutation.clone(),
                    status_context,
                },
            )),
            Self::NewPreview { new_change, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "new change cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::New {
                    new_change: new_change.clone(),
                    status_context,
                },
            )),
            Self::DuplicatePreview { duplicate, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "duplicate cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Duplicate {
                    duplicate: duplicate.clone(),
                    status_context,
                },
            )),
            Self::RebasePreview { rebase, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "rebase cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Rebase {
                    rebase: rebase.clone(),
                    status_context,
                },
            )),
            Self::SplitPreview { split, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "split cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Split {
                    split: split.clone(),
                    status_context,
                },
            )),
            Self::RestorePreview { restore, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "restore cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Restore {
                    restore: restore.clone(),
                    status_context,
                },
            )),
            Self::RevertPreview { revert, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "revert cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Revert {
                    revert: revert.clone(),
                    status_context,
                },
            )),
            Self::SquashPreview { squash, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "squash cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Squash {
                    squash: squash.clone(),
                    status_context,
                },
            )),
            Self::AbsorbPreview { absorb, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "absorb cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Absorb {
                    absorb: absorb.clone(),
                    status_context,
                },
            )),
            Self::PushPreview { push, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "push cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Push {
                    push: push.clone(),
                    status_context,
                },
            )),
            Self::FetchPreview { fetch, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                "fetch cancelled".to_owned(),
                |status_context| ActionPreviewConfirmation::Fetch {
                    fetch: fetch.clone(),
                    status_context,
                },
            )),
            Self::OperationRecoveryPreview { recovery, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                format!("{} cancelled", recovery.status_action()),
                |status_context| ActionPreviewConfirmation::OperationRecovery {
                    recovery: recovery.clone(),
                    status_context,
                },
            )),
            Self::OperationTargetPreview { target, output } => Some(action_preview_event(
                code,
                output,
                visible_lines,
                format!("operation {} cancelled", target.status_action()),
                |status_context| ActionPreviewConfirmation::OperationTarget {
                    target: target.clone(),
                    status_context,
                },
            )),
            Self::WorkingCopyNavigationPreview { navigation, output } => {
                Some(action_preview_event(
                    code,
                    output,
                    visible_lines,
                    navigation.cancel_message().to_owned(),
                    |status_context| ActionPreviewConfirmation::WorkingCopyNavigation {
                        navigation: navigation.clone(),
                        status_context,
                    },
                ))
            }
            _ => None,
        }
    }
}

/// Reduced preview-pane event returned after applying one key to a shared action pane.
enum ActionPreviewEvent {
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
enum ActionPreviewConfirmation {
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
