//! Action-menu entry and prompt setup for app-owned action flows.
//!
//! This module decides which prompt or preview follows a selected action/menu item. Preview
//! rendering and confirmed command completion live in sibling lifecycle modules.

use color_eyre::Result;

use crate::action_menu::{ActionKind, ActionMenuItem, FollowUp, build_action_menu};
use crate::action_output::ActionOutput;
use crate::app_screen::InteractionMode;
use crate::app_status::StatusLine;
use crate::command::ViewCommand;
use crate::jj::{
    JjAbandonPlan, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan, JjCommand,
    JjDescribeTarget, JjDuplicatePlan, JjFileMutationPlan, JjGitFetch, JjGitPushTarget, JjNewPlan,
    JjOperationTarget, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjWorkingCopyNavigationPlan,
};
use crate::view_state::ViewState;

use super::super::App;

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
                    "action menu is only available from graph, show, diff, file list, or file show"
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
            ViewState::Graph(view) => view.selected_revision().map(str::to_owned),
            ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Status(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Bookmarks(_)
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
                        "describe from graph requires a selected row with an exact revision"
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
            | JjCommand::OperationLog
            | JjCommand::OperationShow
            | JjCommand::OperationDiff => {
                self.status = StatusLine::error(
                    &self.view,
                    "describe is only available from graph or status views".to_owned(),
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
                "commit is only available from graph or status because jj commit always acts on @"
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
                        "bookmark {} is only available from graph or status views",
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
            ViewState::Graph(_)
            | ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Status(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
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
                    "push is only available from graph, status, or bookmarks views".to_owned(),
                );
                return Ok(false);
            }
            Err(message) => {
                self.status = StatusLine::error(&self.view, message.to_string());
                return Ok(false);
            }
        };

        match self.load_git_remotes() {
            Ok(remotes) => {
                match remotes.as_slice() {
                    [] => {
                        self.status = StatusLine::error(
                            &self.view,
                            "no git remotes found; add a remote before pushing".to_owned(),
                        );
                    }
                    [remote] => self.open_push_preview(target, remote.to_owned()),
                    _ => {
                        self.mode = InteractionMode::PushRemotePrompt {
                            target,
                            remotes,
                            selected: 0,
                        };
                    }
                }
                Ok(false)
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                Ok(false)
            }
        }
    }

    pub(in crate::app) fn open_fetch_remote_prompt(&mut self) {
        match self.load_git_remotes() {
            Ok(remotes) => match remotes.as_slice() {
                [] => {
                    let message = "no git remotes found; run default fetch or add a remote before choosing one"
                        .to_owned();
                    self.status = StatusLine::error(&self.view, message.clone());
                    self.mode = InteractionMode::FetchPreview {
                        fetch: JjGitFetch::default_remotes(),
                        output: ActionOutput::finished(
                            "jj git remote list".to_owned(),
                            message,
                            Some("fetch remote selection found no remotes".to_owned()),
                        ),
                    };
                }
                [remote] => self.open_fetch_preview(remote.to_owned()),
                _ => {
                    self.mode = InteractionMode::FetchRemotePrompt {
                        remotes,
                        selected: 0,
                    };
                }
            },
            Err(error) => {
                let message = error.to_string();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::FetchPreview {
                    fetch: JjGitFetch::default_remotes(),
                    output: ActionOutput::finished(
                        "jj git remote list".to_owned(),
                        message,
                        Some("fetch remote selection failed to list remotes".to_owned()),
                    ),
                };
            }
        }
    }
}
