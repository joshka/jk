//! Common action preview key flow for app-owned mutation screens.
//!
//! Action output owns scrolling. The app owns what cancellation, completion, and confirmation mean
//! for each pending jj action.

use crossterm::event::KeyCode;
use ratatui::DefaultTerminal;

use crate::action_output::{
    ActionOutput, ActionOutputKey, action_output_visible_lines, handle_action_output_key,
};
use crate::app_screen::InteractionMode;
use crate::app_status::StatusLine;
use crate::jj::{
    JjAbsorbPlan, JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjNewPlan, JjOperationRecovery, JjOperationTarget,
    JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};

use super::App;

impl App {
    pub(super) fn handle_common_action_preview_key(
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
    fn common_action_preview_event(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> Option<ActionPreviewEvent> {
        let visible_lines = action_output_visible_lines(viewport_height);
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

enum ActionPreviewEvent {
    StayOpen,
    CloseCompleted,
    CancelPending(String),
    Confirm(ActionPreviewConfirmation),
}

enum ActionPreviewConfirmation {
    Describe {
        describe: JjDescribePlan,
        status_context: Option<String>,
    },
    Commit {
        commit: JjCommitPlan,
        status_context: Option<String>,
    },
    BookmarkMutation {
        mutation: JjBookmarkMutationPlan,
        status_context: Option<String>,
    },
    FileMutation {
        mutation: JjFileMutationPlan,
        status_context: Option<String>,
    },
    New {
        new_change: JjNewPlan,
        status_context: Option<String>,
    },
    Duplicate {
        duplicate: JjDuplicatePlan,
        status_context: Option<String>,
    },
    Rebase {
        rebase: JjRebasePlan,
        status_context: Option<String>,
    },
    Split {
        split: JjSplitPlan,
        status_context: Option<String>,
    },
    Restore {
        restore: JjRestorePlan,
        status_context: Option<String>,
    },
    Revert {
        revert: JjRevertPlan,
        status_context: Option<String>,
    },
    Squash {
        squash: JjSquashPlan,
        status_context: Option<String>,
    },
    Absorb {
        absorb: JjAbsorbPlan,
        status_context: Option<String>,
    },
    Push {
        push: JjGitPush,
        status_context: Option<String>,
    },
    Fetch {
        fetch: JjGitFetch,
        status_context: Option<String>,
    },
    OperationRecovery {
        recovery: JjOperationRecovery,
        status_context: Option<String>,
    },
    OperationTarget {
        target: JjOperationTarget,
        status_context: Option<String>,
    },
    WorkingCopyNavigation {
        navigation: JjWorkingCopyNavigationPlan,
        status_context: Option<String>,
    },
}

fn action_preview_event(
    code: KeyCode,
    output: &mut ActionOutput,
    visible_lines: u16,
    cancel_message: String,
    confirm: impl FnOnce(Option<String>) -> ActionPreviewConfirmation,
) -> ActionPreviewEvent {
    let completed = output.completed();
    let status_context = output.status_context().cloned();

    match handle_action_output_key(code, output, visible_lines) {
        ActionOutputKey::Cancel => {
            if completed {
                ActionPreviewEvent::CloseCompleted
            } else {
                ActionPreviewEvent::CancelPending(cancel_message)
            }
        }
        ActionOutputKey::Primary => {
            if completed {
                ActionPreviewEvent::CloseCompleted
            } else {
                ActionPreviewEvent::Confirm(confirm(status_context))
            }
        }
        ActionOutputKey::Handled | ActionOutputKey::Ignored => ActionPreviewEvent::StayOpen,
    }
}
