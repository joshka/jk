use color_eyre::Result;

use crate::menus::{ExactActionContext, StatusPathActionAvailability};
use crate::status::StatusFileAction;

use super::super::ViewState;

pub fn exact_restore_revert_context(view: &ViewState) -> Result<Option<ExactActionContext>> {
    match view {
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

/// Converts a status-file action into the exact action context used by shared file action menus.
fn status_file_action_context(action: StatusFileAction) -> ExactActionContext {
    match action {
        StatusFileAction::Track { path } => ExactActionContext::status_untracked_path(path),
        StatusFileAction::Tracked {
            path,
            restore_allowed,
            chmod_allowed,
        } => ExactActionContext::status_tracked_path(
            path,
            StatusPathActionAvailability {
                restore_allowed,
                chmod_allowed,
            },
        ),
    }
}
