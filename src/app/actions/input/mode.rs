use crossterm::event::KeyCode;

use crate::app::actions::action_pane_visible_lines;
use crate::modes::InteractionMode;

use super::{ActionPreviewConfirmation, ActionPreviewEvent, action_preview_event};

impl InteractionMode {
    /// Reduce one key for any preview/result mode that shares the common action-pane behavior.
    pub fn common_action_preview_event(
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
