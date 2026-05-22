//! Action target projection for the active view.
//!
//! `ViewState` owns mechanical view dispatch. This module owns the policy that
//! turns the active view selection into mutation targets and exact file/change
//! contexts for action menus.

use color_eyre::Result;

use crate::actions::{JjBookmarkForgetTarget, JjBookmarkTarget, JjGitPushTarget};
use crate::menus::ExactActionContext;
use crate::status::StatusFileAction;
use crate::view_state::ViewState;

pub(crate) struct ViewActionTargets<'a> {
    view: &'a ViewState,
}

impl<'a> ViewActionTargets<'a> {
    pub(crate) fn new(view: &'a ViewState) -> Self {
        Self { view }
    }

    pub(crate) fn push_target(&self) -> Result<Option<JjGitPushTarget>> {
        match self.view {
            ViewState::Log(view) => view
                .selected_revision()
                .map(|revision| JjGitPushTarget::Revision(revision.to_owned()))
                .map_or_else(
                    || {
                        Err(color_eyre::eyre::eyre!(
                            "push from log requires a selected row with an exact revision"
                        ))
                    },
                    |target| Ok(Some(target)),
                ),
            ViewState::Bookmarks(view) => view
                .selected_bookmark_name()
                .map(|name| JjGitPushTarget::Bookmark(name.to_owned()))
                .map_or_else(
                    || {
                        Err(color_eyre::eyre::eyre!(
                            "selected bookmark has no target name for push"
                        ))
                    },
                    |target| Ok(Some(target)),
                ),
            ViewState::Status(_) => Ok(Some(JjGitPushTarget::Status)),
            ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Workspaces(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => Ok(None),
        }
    }

    pub(crate) fn bookmark_target(&self) -> Result<Option<JjBookmarkTarget>> {
        match self.view {
            ViewState::Log(view) => view
                .selected_revision()
                .map(|revision| JjBookmarkTarget::exact_change(revision.to_owned()))
                .map_or_else(
                    || {
                        Err(color_eyre::eyre::eyre!(
                            "bookmark mutation from log requires a selected row with an exact revision"
                        ))
                    },
                    |target| Ok(Some(target)),
                ),
            ViewState::Status(_) => Ok(Some(JjBookmarkTarget::current_working_copy())),
            ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Bookmarks(_)
            | ViewState::Workspaces(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => Ok(None),
        }
    }

    pub(crate) fn selected_local_bookmark_name(&self) -> Result<Option<&'a str>> {
        self.selected_local_bookmark_name_for("delete")
    }

    pub(crate) fn selected_local_bookmark_name_for(&self, action: &str) -> Result<Option<&'a str>> {
        match self.view {
            ViewState::Bookmarks(view) => view.selected_local_bookmark_name().map_or_else(
                || {
                    Err(color_eyre::eyre::eyre!(
                        "{} requires a selected exact local bookmark",
                        action
                    ))
                },
                |name| Ok(Some(name)),
            ),
            ViewState::Log(_)
            | ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Status(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Workspaces(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => Ok(None),
        }
    }

    pub(crate) fn bookmark_forget_target(
        &self,
    ) -> Result<Option<(String, JjBookmarkForgetTarget)>> {
        match self.view {
            ViewState::Bookmarks(view) => view
                .selected_bookmark_forget_target()
                .map(|target| target.map(|(name, forget_target)| (name.to_owned(), forget_target))),
            ViewState::Log(_)
            | ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Status(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Workspaces(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => Ok(None),
        }
    }

    pub(crate) fn exact_restore_revert_context(&self) -> Result<Option<ExactActionContext>> {
        match self.view {
            ViewState::Log(_) => Ok(None),
            ViewState::Show(view) => view
                .spec()
                .exact_change_target()
                .map(ExactActionContext::detail)
                .map(Some)
                .ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "restore/revert from {} requires an exact log-derived revision target",
                        view.spec().app_label()
                    )
                }),
            ViewState::Diff(view) => view
                .spec()
                .exact_change_target()
                .map(ExactActionContext::detail)
                .map(Some)
                .ok_or_else(|| {
                    color_eyre::eyre::eyre!(
                        "restore/revert from {} requires an exact log-derived revision target",
                        view.spec().app_label()
                    )
                }),
            ViewState::FileList(view) => {
                let Some(path) = view.selected_path() else {
                    return Err(color_eyre::eyre::eyre!(
                        "file action from {} requires a selected exact path",
                        view.spec().app_label()
                    ));
                };
                if let Some(revision) = view.spec().exact_change_target() {
                    return Ok(Some(
                        ExactActionContext::detail(revision).with_selected_path(path),
                    ));
                }
                if view.spec().target().is_none() {
                    return Ok(Some(ExactActionContext::working_copy_file_path(path)));
                }
                Err(color_eyre::eyre::eyre!(
                    "file actions from {} require a working-copy file list or exact log-derived revision target",
                    view.spec().app_label()
                ))
            }
            ViewState::FileShow(view) => {
                let path = view.path();
                if path.is_empty() {
                    return Err(color_eyre::eyre::eyre!(
                        "file action from {} requires a selected exact path",
                        view.spec().app_label()
                    ));
                }
                if let Some(revision) = view.spec().exact_change_target() {
                    return Ok(Some(
                        ExactActionContext::detail(revision).with_selected_path(path),
                    ));
                }
                if view.spec().target().is_none() {
                    return Ok(Some(ExactActionContext::working_copy_file_path(path)));
                }
                Err(color_eyre::eyre::eyre!(
                    "file actions from {} require a working-copy file show or exact log-derived revision target",
                    view.spec().app_label()
                ))
            }
            ViewState::Status(view) => {
                let action = view
                    .selected_file_action()
                    .map_err(|message| color_eyre::eyre::eyre!(message))?;
                Ok(Some(status_file_action_context(action)))
            }
            ViewState::Resolve(_)
            | ViewState::Bookmarks(_)
            | ViewState::Workspaces(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => Ok(None),
        }
    }
}

fn status_file_action_context(action: StatusFileAction) -> ExactActionContext {
    match action {
        StatusFileAction::Track { path } => ExactActionContext::status_untracked_path(path),
        StatusFileAction::Tracked {
            path,
            restore_allowed,
            chmod_allowed,
        } => ExactActionContext::status_tracked_path(path, restore_allowed, chmod_allowed),
    }
}
