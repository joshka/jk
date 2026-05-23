use crossterm::event::KeyCode;

use super::{ActionPreviewConfirmation, ActionPreviewEvent};
use crate::app::App;
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;

impl App {
    /// Route one key through the common preview/result-pane reducer shared by action overlays.
    pub fn handle_common_action_preview_key(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> bool {
        let Some(event) = self.mode.common_action_preview_event(code, viewport_height) else {
            return false;
        };

        self.apply_action_preview_event(event, viewport_height);
        true
    }

    /// Apply one reduced preview event to app-owned mode, status, or confirmation flow.
    fn apply_action_preview_event(&mut self, event: ActionPreviewEvent, viewport_height: u16) {
        match event {
            ActionPreviewEvent::StayOpen => {}
            ActionPreviewEvent::CloseCompleted => self.mode = InteractionMode::Normal,
            ActionPreviewEvent::CancelPending(message) => {
                self.mode = InteractionMode::Normal;
                self.status = StatusLine::with_message(&self.view, message);
            }
            ActionPreviewEvent::Confirm(confirmation) => {
                self.confirm_action_preview(confirmation, viewport_height);
            }
        }
    }

    /// Dispatch one accepted preview confirmation into the matching completion method.
    fn confirm_action_preview(
        &mut self,
        confirmation: ActionPreviewConfirmation,
        viewport_height: u16,
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
            } => self.request_interactive_split(split, status_context, viewport_height),
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
