use color_eyre::Result;

use super::super::ViewState;
use crate::actions::{JjBookmarkForgetTarget, JjBookmarkTarget, JjGitPushTarget};

pub fn push_target(view: &ViewState) -> Result<Option<JjGitPushTarget>> {
    match view {
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

pub fn bookmark_target(view: &ViewState) -> Result<Option<JjBookmarkTarget>> {
    match view {
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

pub fn selected_local_bookmark_name_for<'a>(
    view: &'a ViewState,
    action: &str,
) -> Result<Option<&'a str>> {
    match view {
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

pub fn bookmark_forget_target(
    view: &ViewState,
) -> Result<Option<(String, JjBookmarkForgetTarget)>> {
    match view {
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
