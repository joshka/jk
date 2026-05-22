use color_eyre::Result;

use crate::actions::{
    JjAbandonPlan, JjAbsorbPlan, JjDuplicatePlan, JjFileMutationPlan, JjNewPlan, JjOperationTarget,
    JjRestorePlan, JjRevertPlan, JjSplitPlan, JjWorkingCopyNavigationPlan,
};
use crate::app::status_line::StatusLine;
use crate::command::ViewCommand;
use crate::jj::JjCommand;
use crate::menus::{ActionKind, ActionMenuItem, FollowUp, build_action_menu};
use crate::modes::InteractionMode;
use crate::view_state::ViewState;

use super::super::super::App;

impl App {
    /// Open the action menu for the current selection or exact restore/revert context.
    pub(in crate::app) fn open_action_menu(&mut self, viewport_height: u16) -> Result<bool> {
        if matches!(
            self.view.command(),
            JjCommand::Default | JjCommand::Log | JjCommand::OperationLog
        ) {
            let effect = self.execute_view(ViewCommand::OpenActionMenu, viewport_height);
            return self.apply_view_effect(effect, viewport_height);
        }

        let context = match self.view.exact_restore_revert_context() {
            Ok(Some(context)) => context,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "action menu is only available from log, show, diff, file list, or file show"
                        .to_owned(),
                );
                return Ok(false);
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                return Ok(false);
            }
        };

        let menu = build_action_menu(&context);
        if menu.is_empty() {
            self.status = StatusLine::error(
                &self.view,
                "no preview actions available for exact restore/revert context".to_owned(),
            );
            return Ok(false);
        }

        self.mode = InteractionMode::ActionMenu { menu, selected: 0 };
        Ok(false)
    }

    /// Consume one accepted menu item and route it into prompt, preview, or status flow.
    pub(in crate::app) fn apply_action_menu_item(&mut self, item: ActionMenuItem) {
        match item.follow_up() {
            FollowUp::StatusMessage(message) => {
                self.status = StatusLine::with_message(&self.view, message.as_str());
                self.mode = InteractionMode::Normal;
            }
            FollowUp::ExactRevision { revision } => {
                let action = item.action();
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                match action {
                    ActionKind::Abandon => self.open_abandon_preview(JjAbandonPlan::new(revision)),
                    ActionKind::Edit
                    | ActionKind::New
                    | ActionKind::Split
                    | ActionKind::Duplicate
                    | ActionKind::Restore
                    | ActionKind::Revert
                    | ActionKind::Rebase
                    | ActionKind::Squash
                    | ActionKind::Absorb
                    | ActionKind::FileTrack
                    | ActionKind::FileUntrack
                    | ActionKind::FileChmodExecutable
                    | ActionKind::FileChmodNormal => {
                        self.status =
                            StatusLine::with_message(&self.view, "preview not yet implemented");
                    }
                }
            }
            FollowUp::SplitExactTarget { revision } => {
                self.mode = InteractionMode::Normal;
                self.open_split_preview(JjSplitPlan::exact_change(revision.clone()));
            }
            FollowUp::SplitCurrentWorkingCopy => {
                self.mode = InteractionMode::Normal;
                self.open_split_preview(JjSplitPlan::current_working_copy());
            }
            FollowUp::DuplicateExactTarget { revision } => {
                self.mode = InteractionMode::Normal;
                self.open_duplicate_preview(JjDuplicatePlan::exact_change(revision.clone()));
            }
            FollowUp::EditExactTarget { revision } => {
                self.mode = InteractionMode::Normal;
                self.open_working_copy_navigation_preview(JjWorkingCopyNavigationPlan::edit(
                    revision.clone(),
                ));
            }
            FollowUp::RestoreExactTarget { revision, path } => {
                self.mode = InteractionMode::Normal;
                match path.clone() {
                    Some(path) => {
                        self.open_restore_preview(JjRestorePlan::for_path(revision.clone(), path));
                    }
                    None => {
                        self.open_restore_preview(JjRestorePlan::for_revision(revision.clone()));
                    }
                }
            }
            FollowUp::RestoreWorkingCopyPath { path } => {
                self.mode = InteractionMode::Normal;
                self.open_restore_preview(JjRestorePlan::for_working_copy_path(path.clone()));
            }
            FollowUp::RevertExactTarget { revision } => {
                self.mode = InteractionMode::Normal;
                self.open_revert_preview(JjRevertPlan::new(revision.clone()));
            }
            FollowUp::OperationRestoreExactTarget { operation_id } => {
                self.mode = InteractionMode::Normal;
                self.open_operation_target_preview(JjOperationTarget::restore(
                    operation_id.clone(),
                ));
            }
            FollowUp::OperationRevertExactTarget { operation_id } => {
                self.mode = InteractionMode::Normal;
                self.open_operation_target_preview(JjOperationTarget::revert(operation_id.clone()));
            }
            FollowUp::NewParents { parents } => {
                self.mode = InteractionMode::Normal;
                self.open_new_preview(JjNewPlan::new(parents.clone()));
            }
            FollowUp::RolePrompt(prompt) => {
                self.mode = InteractionMode::RolePrompt {
                    action: item.action(),
                    prompt: prompt.clone(),
                    selected: 0,
                };
            }
            FollowUp::AbsorbCandidates {
                source,
                destinations,
            } => {
                self.mode = InteractionMode::Normal;
                self.open_absorb_preview(JjAbsorbPlan::new(source.clone(), destinations.clone()));
            }
            FollowUp::FileTrack { path } => {
                self.mode = InteractionMode::Normal;
                self.open_file_mutation_preview(JjFileMutationPlan::track(path.clone()));
            }
            FollowUp::FileUntrack { path } => {
                self.mode = InteractionMode::Normal;
                self.open_file_mutation_preview(JjFileMutationPlan::untrack(path.clone()));
            }
            FollowUp::FileChmod {
                path,
                revision,
                mode,
            } => {
                self.mode = InteractionMode::Normal;
                let mutation = match revision.clone() {
                    Some(revision) => {
                        JjFileMutationPlan::chmod_exact_revision(revision, path.clone(), *mode)
                    }
                    None => JjFileMutationPlan::chmod_working_copy(path.clone(), *mode),
                };
                self.open_file_mutation_preview(mutation);
            }
        }
    }

    /// Return the selected graph revision when the current surface is a log-like graph view.
    pub(in crate::app) fn graph_selected_revision(&self) -> Option<String> {
        match &self.view {
            ViewState::Log(view) => view.selected_revision().map(str::to_owned),
            ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Status(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Bookmarks(_)
            | ViewState::Workspaces(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => None,
        }
    }
}
