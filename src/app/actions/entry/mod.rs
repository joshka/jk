//! Entry routing for app-owned action flows.
//!
//! This module owns the app lifecycle step after an accepted menu item or key action has already
//! been chosen: route it to a prompt, open the corresponding preview, or report a status. Feature
//! views and action menus own whether an action is available and the exact target values they
//! carry. [`super::preview`] owns preview pane construction and preview status contexts.
//! [`super::completion`] and [`super::shared`] own confirmed command result handling.
//! [`crate::actions`] owns command-plan argv, preview, and run contracts.

use color_eyre::Result;

use crate::action_pane::ActionPane;
use crate::actions::{
    JjAbandonPlan, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan, JjDescribeTarget,
    JjDuplicatePlan, JjFileMutationPlan, JjGitFetch, JjGitPushTarget, JjNewPlan, JjOperationTarget,
    JjRestorePlan, JjRevertPlan, JjSplitPlan, JjWorkingCopyNavigationPlan,
};
use crate::command::ViewCommand;
use crate::jj::JjCommand;
use crate::menus::{ActionKind, ActionMenuItem, FollowUp, build_action_menu};
use crate::modes::InteractionMode;
use crate::status_line::StatusLine;
use crate::view_state::ViewState;

use super::super::App;

const PUSH_NO_REMOTES_MESSAGE: &str = "no git remotes found; add a remote before pushing";
const FETCH_NO_REMOTES_MESSAGE: &str =
    "no git remotes found; run default fetch or add a remote before choosing one";
const FETCH_REMOTE_LIST_COMMAND_LABEL: &str = "jj git remote list";
const FETCH_NO_REMOTES_CONTEXT: &str = "fetch remote selection found no remotes";
const FETCH_REMOTE_LIST_ERROR_CONTEXT: &str = "fetch remote selection failed to list remotes";

#[derive(Debug, Eq, PartialEq)]
enum PushRemotePromptDecision {
    MissingRemotes { message: String },
    OpenPreview { remote: String },
    Prompt { remotes: Vec<String> },
    RemoteListError { message: String },
}

#[derive(Debug, Eq, PartialEq)]
enum FetchRemotePromptDecision {
    MissingRemotes {
        message: String,
        status_context: String,
    },
    OpenPreview {
        remote: String,
    },
    Prompt {
        remotes: Vec<String>,
    },
    RemoteListError {
        message: String,
        status_context: String,
    },
}

impl App {
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

    pub(in crate::app) fn apply_action_menu_item(&mut self, item: ActionMenuItem) {
        // Accepted menu items leave modal input here because the app lifecycle owns the
        // mode transition and any immediate preview/status side effects.
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
                    ActionKind::Abandon => {
                        self.open_abandon_preview(JjAbandonPlan::new(revision));
                    }
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
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                self.open_split_preview(JjSplitPlan::exact_change(revision));
            }
            FollowUp::SplitCurrentWorkingCopy => {
                self.mode = InteractionMode::Normal;
                self.open_split_preview(JjSplitPlan::current_working_copy());
            }
            FollowUp::DuplicateExactTarget { revision } => {
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                self.open_duplicate_preview(JjDuplicatePlan::exact_change(revision));
            }
            FollowUp::EditExactTarget { revision } => {
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                self.open_working_copy_navigation_preview(JjWorkingCopyNavigationPlan::edit(
                    revision,
                ));
            }
            FollowUp::RestoreExactTarget { revision, path } => {
                let revision = revision.clone();
                let path = path.clone();
                self.mode = InteractionMode::Normal;
                match path {
                    Some(path) => {
                        self.open_restore_preview(JjRestorePlan::for_path(revision, path));
                    }
                    None => {
                        self.open_restore_preview(JjRestorePlan::for_revision(revision));
                    }
                }
            }
            FollowUp::RestoreWorkingCopyPath { path } => {
                let path = path.clone();
                self.mode = InteractionMode::Normal;
                self.open_restore_preview(JjRestorePlan::for_working_copy_path(path));
            }
            FollowUp::RevertExactTarget { revision } => {
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                self.open_revert_preview(JjRevertPlan::new(revision));
            }
            FollowUp::OperationRestoreExactTarget { operation_id } => {
                let operation_id = operation_id.clone();
                self.mode = InteractionMode::Normal;
                self.open_operation_target_preview(JjOperationTarget::restore(operation_id));
            }
            FollowUp::OperationRevertExactTarget { operation_id } => {
                let operation_id = operation_id.clone();
                self.mode = InteractionMode::Normal;
                self.open_operation_target_preview(JjOperationTarget::revert(operation_id));
            }
            FollowUp::NewParents { parents } => {
                let parents = parents.clone();
                self.mode = InteractionMode::Normal;
                self.open_new_preview(JjNewPlan::new(parents));
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
                let source = source.clone();
                let destinations = destinations.clone();
                self.mode = InteractionMode::Normal;
                self.open_absorb_preview(JjAbsorbPlan::new(source, destinations));
            }
            FollowUp::FileTrack { path } => {
                let path = path.clone();
                self.mode = InteractionMode::Normal;
                self.open_file_mutation_preview(JjFileMutationPlan::track(path));
            }
            FollowUp::FileUntrack { path } => {
                let path = path.clone();
                self.mode = InteractionMode::Normal;
                self.open_file_mutation_preview(JjFileMutationPlan::untrack(path));
            }
            FollowUp::FileChmod {
                path,
                revision,
                mode,
            } => {
                let path = path.clone();
                let revision = revision.clone();
                let mode = *mode;
                self.mode = InteractionMode::Normal;
                let mutation = match revision {
                    Some(revision) => {
                        JjFileMutationPlan::chmod_exact_revision(revision, path, mode)
                    }
                    None => JjFileMutationPlan::chmod_working_copy(path, mode),
                };
                self.open_file_mutation_preview(mutation);
            }
        }
    }

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

    pub(in crate::app) fn open_describe_prompt(&mut self) {
        let target = match self.view.command() {
            JjCommand::Default | JjCommand::Log => match self.view.push_target() {
                Ok(Some(JjGitPushTarget::Revision(revision))) => {
                    JjDescribeTarget::exact_change(revision)
                }
                Ok(_) | Err(_) => {
                    self.status = StatusLine::error(
                        &self.view,
                        "describe from log requires a selected row with an exact revision"
                            .to_owned(),
                    );
                    return;
                }
            },
            JjCommand::Status => JjDescribeTarget::current_working_copy(),
            JjCommand::Show
            | JjCommand::Diff
            | JjCommand::Resolve
            | JjCommand::FileList
            | JjCommand::FileShow
            | JjCommand::Bookmarks
            | JjCommand::Workspaces
            | JjCommand::OperationLog
            | JjCommand::OperationShow
            | JjCommand::OperationDiff => {
                self.status = StatusLine::error(
                    &self.view,
                    "describe is only available from log or status views".to_owned(),
                );
                return;
            }
        };

        self.mode = InteractionMode::DescribePrompt {
            target,
            input: String::new(),
        };
    }

    pub(in crate::app) fn open_commit_prompt(&mut self) {
        if matches!(
            self.view.command(),
            JjCommand::Default | JjCommand::Log | JjCommand::Status
        ) {
            self.mode = InteractionMode::CommitPrompt(String::new());
        } else {
            self.status = StatusLine::error(
                &self.view,
                "commit is only available from log or status because jj commit always acts on @"
                    .to_owned(),
            );
        }
    }

    pub(in crate::app) fn open_bookmark_name_prompt(&mut self, kind: JjBookmarkMutationKind) {
        let target = match self.view.bookmark_target() {
            Ok(Some(target)) => target,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    format!(
                        "bookmark {} is only available from log or status views",
                        kind.label()
                    ),
                );
                return;
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                return;
            }
        };

        self.mode = InteractionMode::BookmarkNamePrompt {
            kind,
            target,
            input: String::new(),
        };
    }

    pub(in crate::app) fn open_bookmark_delete_preview(&mut self) {
        let name = match self.view.selected_local_bookmark_name() {
            Ok(Some(name)) => name.to_owned(),
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "bookmark delete is only available from bookmarks view".to_owned(),
                );
                return;
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                return;
            }
        };

        self.open_bookmark_mutation_preview(JjBookmarkMutationPlan::delete(name));
    }

    pub(in crate::app) fn open_bookmark_forget_preview(&mut self) {
        let (name, target) = match self.view.bookmark_forget_target() {
            Ok(Some(target)) => target,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "bookmark forget is only available from bookmarks view".to_owned(),
                );
                return;
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                return;
            }
        };

        self.open_bookmark_mutation_preview(JjBookmarkMutationPlan::forget(name, target));
    }

    pub(in crate::app) fn open_bookmark_tracking_preview(&mut self, kind: JjBookmarkMutationKind) {
        let (name, target) = match &self.view {
            ViewState::Bookmarks(view) => match view.selected_bookmark_tracking_target(kind) {
                Ok(Some((name, target))) => (name.to_owned(), target),
                Ok(None) => {
                    self.status = StatusLine::error(
                        &self.view,
                        format!(
                            "bookmark {} is only available from bookmarks view",
                            kind.label()
                        ),
                    );
                    return;
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    return;
                }
            },
            ViewState::Log(_)
            | ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Status(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Workspaces(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => {
                self.status = StatusLine::error(
                    &self.view,
                    format!(
                        "bookmark {} is only available from bookmarks view",
                        kind.label()
                    ),
                );
                return;
            }
        };

        let mutation = match kind {
            JjBookmarkMutationKind::Track => JjBookmarkMutationPlan::track(name, target),
            JjBookmarkMutationKind::Untrack => JjBookmarkMutationPlan::untrack(name, target),
            JjBookmarkMutationKind::Create
            | JjBookmarkMutationKind::Set
            | JjBookmarkMutationKind::Move
            | JjBookmarkMutationKind::Rename
            | JjBookmarkMutationKind::Delete
            | JjBookmarkMutationKind::Forget => {
                self.status = StatusLine::error(
                    &self.view,
                    "bookmark tracking preview requires track or untrack".to_owned(),
                );
                return;
            }
        };
        self.open_bookmark_mutation_preview(mutation);
    }

    pub(in crate::app) fn open_bookmark_rename_prompt(&mut self) {
        let old_name = match self.view.selected_local_bookmark_name_for("rename") {
            Ok(Some(name)) => name.to_owned(),
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "bookmark rename is only available from bookmarks view".to_owned(),
                );
                return;
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                return;
            }
        };

        self.mode = InteractionMode::BookmarkRenamePrompt {
            old_name,
            input: String::new(),
        };
    }

    pub(in crate::app) fn open_push_prompt(&mut self) -> Result<bool> {
        let target = match self.view.push_target() {
            Ok(Some(target)) => target,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "push is only available from log, status, or bookmarks views".to_owned(),
                );
                return Ok(false);
            }
            Err(message) => {
                self.status = StatusLine::error(&self.view, message.to_string());
                return Ok(false);
            }
        };

        let decision =
            decide_push_remote_prompt(self.load_git_remotes().map_err(|error| error.to_string()));
        // The reducer classifies loaded remotes; this entry point applies the resulting app
        // side effect after preserving the feature-selected push target.
        match decision {
            PushRemotePromptDecision::MissingRemotes { message }
            | PushRemotePromptDecision::RemoteListError { message } => {
                self.status = StatusLine::error(&self.view, message);
            }
            PushRemotePromptDecision::OpenPreview { remote } => {
                self.open_push_preview(target, remote);
            }
            PushRemotePromptDecision::Prompt { remotes } => {
                self.mode = InteractionMode::PushRemotePrompt {
                    target,
                    remotes,
                    selected: 0,
                }
            }
        }
        Ok(false)
    }

    pub(in crate::app) fn open_fetch_remote_prompt(&mut self) {
        let decision =
            decide_fetch_remote_prompt(self.load_git_remotes().map_err(|error| error.to_string()));
        // Fetch without an explicit remote still opens a preview pane so the failed remote-list
        // command has the same status/output surface as other action previews.
        match decision {
            FetchRemotePromptDecision::MissingRemotes {
                message,
                status_context,
            }
            | FetchRemotePromptDecision::RemoteListError {
                message,
                status_context,
            } => {
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::FetchPreview {
                    fetch: JjGitFetch::default_remotes(),
                    output: ActionPane::finished(
                        FETCH_REMOTE_LIST_COMMAND_LABEL.to_owned(),
                        message,
                        Some(status_context),
                    ),
                };
            }
            FetchRemotePromptDecision::OpenPreview { remote } => {
                self.open_fetch_preview(remote);
            }
            FetchRemotePromptDecision::Prompt { remotes } => {
                self.mode = InteractionMode::FetchRemotePrompt {
                    remotes,
                    selected: 0,
                };
            }
        }
    }
}

fn decide_push_remote_prompt(
    remotes: std::result::Result<Vec<String>, String>,
) -> PushRemotePromptDecision {
    // Keep remote-list classification pure; callers own status, prompt, and preview mutation.
    match remotes {
        Ok(remotes) => match remotes.as_slice() {
            [] => PushRemotePromptDecision::MissingRemotes {
                message: PUSH_NO_REMOTES_MESSAGE.to_owned(),
            },
            [remote] => PushRemotePromptDecision::OpenPreview {
                remote: remote.to_owned(),
            },
            _ => PushRemotePromptDecision::Prompt { remotes },
        },
        Err(message) => PushRemotePromptDecision::RemoteListError { message },
    }
}

fn decide_fetch_remote_prompt(
    remotes: std::result::Result<Vec<String>, String>,
) -> FetchRemotePromptDecision {
    // Keep remote-list classification pure; callers own status, prompt, and preview mutation.
    match remotes {
        Ok(remotes) => match remotes.as_slice() {
            [] => FetchRemotePromptDecision::MissingRemotes {
                message: FETCH_NO_REMOTES_MESSAGE.to_owned(),
                status_context: FETCH_NO_REMOTES_CONTEXT.to_owned(),
            },
            [remote] => FetchRemotePromptDecision::OpenPreview {
                remote: remote.to_owned(),
            },
            _ => FetchRemotePromptDecision::Prompt { remotes },
        },
        Err(message) => FetchRemotePromptDecision::RemoteListError {
            message,
            status_context: FETCH_REMOTE_LIST_ERROR_CONTEXT.to_owned(),
        },
    }
}

#[cfg(test)]
mod tests;
