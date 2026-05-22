use crate::actions::{JjBookmarkMutationKind, JjBookmarkMutationPlan, JjDescribeTarget};
use crate::app::status_line::StatusLine;
use crate::jj::JjCommand;
use crate::modes::InteractionMode;
use crate::view_state::ViewState;

use super::super::super::App;

impl App {
    /// Open the describe prompt for the current exact change or selected graph revision.
    pub(in crate::app) fn open_describe_prompt(&mut self) {
        let target = match self.view.command() {
            JjCommand::Default | JjCommand::Log => match self.view.push_target() {
                Ok(Some(crate::actions::JjGitPushTarget::Revision(revision))) => {
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

    /// Open the commit prompt when the current surface can commit the working copy.
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

    /// Open the bookmark-name prompt for create, set, or move flows on the current target.
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

    /// Open the delete preview for the selected bookmark.
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

    /// Open the forget preview for the selected bookmark and its current target.
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

    /// Open the track or untrack preview for the selected bookmark.
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

    /// Open the bookmark-rename prompt for the selected bookmark.
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
}
