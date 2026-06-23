use crossterm::event::{KeyCode, KeyEvent};
use jk_cli::{
    JjAbandon, JjDiff, JjEdit, JjEvolog, JjLog, JjNew, JjOperation, JjRecovery, JjShow, JjStatus,
    JjWorkspaces, RecoveryCommand,
};
use jk_tui::log_view::LogAction;

use crate::key::AppKey;
use crate::state::{AppState, AppView, InputMode};
use crate::{
    AppLoop, SearchDirection, apply_action, apply_search_action, copy_selected_command,
    edit_command_output, handle_back_with_log_source, open_abandon_preview, open_command_discovery,
    open_command_history, open_command_history_operation, open_diff_file_list, open_edit_preview,
    open_jj_command_mode, open_new_preview, open_operation_log, open_recovery_preview,
    open_view_options, open_workspaces, push_selected_command_history_details,
    push_selected_evolog, push_selected_operation_show, push_selected_show,
    push_selected_workspace_status, push_status, update_selected_workspace_stale,
};

pub struct AppSources<'a> {
    pub(crate) log: &'a mut JjLog,
    pub(crate) diff: &'a JjDiff,
    pub(crate) evolog: &'a JjEvolog,
    pub(crate) show: &'a JjShow,
    pub(crate) status: &'a JjStatus,
    pub(crate) abandon: &'a JjAbandon,
    pub(crate) new_change: &'a JjNew,
    pub(crate) edit: &'a JjEdit,
    pub(crate) operation: &'a JjOperation,
    pub(crate) recovery: &'a JjRecovery,
    pub(crate) workspaces: &'a JjWorkspaces,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchResult {
    Continue,
    Quit,
}

pub fn dispatch_app_key(
    state: &mut AppState,
    sources: &mut AppSources<'_>,
    key: KeyEvent,
    app_key: AppKey,
) -> DispatchResult {
    if matches!(
        state.views.active(),
        AppView::Workspaces { .. } | AppView::CommandHistory { .. } | AppView::OperationLog { .. }
    ) && matches!(key.code, KeyCode::Esc)
    {
        handle_back_with_log_source(state, sources.log);
        return DispatchResult::Continue;
    }

    let AppKey::Action(action) = app_key else {
        dispatch_direct_app_key(state, sources, app_key);
        return DispatchResult::Continue;
    };

    if matches!(action, LogAction::CollapseExpanded)
        && matches!(key.code, KeyCode::Left)
        && state.pop_log_drill(sources.log)
    {
        return DispatchResult::Continue;
    }

    if matches!(app_key, AppKey::Action(LogAction::ToggleHelp)) {
        open_command_discovery(state);
        return DispatchResult::Continue;
    }

    if apply_action(
        state,
        sources.log,
        sources.diff,
        sources.evolog,
        sources.show,
        sources.status,
        sources.operation,
        sources.workspaces,
        action,
    ) == AppLoop::Quit
    {
        DispatchResult::Quit
    } else {
        DispatchResult::Continue
    }
}

fn dispatch_direct_app_key(state: &mut AppState, sources: &mut AppSources<'_>, app_key: AppKey) {
    match app_key {
        AppKey::Back => {
            handle_back_with_log_source(state, sources.log);
        }
        AppKey::OpenShow => {
            if matches!(state.views.active(), AppView::OperationLog { .. }) {
                push_selected_operation_show(state, sources.operation);
            } else if matches!(state.views.active(), AppView::Workspaces { .. }) {
                push_selected_workspace_status(state, sources.workspaces);
            } else if matches!(state.views.active(), AppView::CommandHistory { .. }) {
                push_selected_command_history_details(state);
            } else if active_log_has_selected_elision(state) {
                let _ = apply_action(
                    state,
                    sources.log,
                    sources.diff,
                    sources.evolog,
                    sources.show,
                    sources.status,
                    sources.operation,
                    sources.workspaces,
                    LogAction::ToggleExpanded,
                );
            } else {
                push_selected_show(state, sources.show);
            }
        }
        AppKey::OpenEvolog => {
            push_selected_evolog(state, sources.evolog);
        }
        AppKey::OpenStatus => {
            if matches!(state.views.active(), AppView::Workspaces { .. }) {
                push_selected_workspace_status(state, sources.workspaces);
            } else {
                push_status(state, sources.status);
            }
        }
        AppKey::OpenWorkspaces => {
            open_workspaces(state, sources.workspaces);
        }
        AppKey::OpenCommandHistory => {
            open_command_history(state);
        }
        AppKey::OpenOperationLog => {
            if matches!(state.views.active(), AppView::CommandHistory { .. }) {
                open_command_history_operation(state, sources.operation);
            } else {
                open_operation_log(state, sources.operation);
            }
        }
        AppKey::CopyCommand => {
            copy_selected_command(state);
        }
        AppKey::StartUndo => {
            if matches!(state.views.active(), AppView::Workspaces { .. }) {
                update_selected_workspace_stale(state, sources.workspaces);
            } else {
                open_recovery_preview(state, sources.recovery, RecoveryCommand::Undo);
            }
        }
        AppKey::StartRedo => {
            open_recovery_preview(state, sources.recovery, RecoveryCommand::Redo);
        }
        AppKey::StartDescribe => {
            crate::open_describe_message(state);
        }
        AppKey::StartAbandon => {
            open_abandon_preview(state, sources.abandon);
        }
        AppKey::OpenViewOptions => {
            if !matches!(state.views.active(), AppView::CommandHistory { .. }) {
                open_view_options(state);
            }
        }
        AppKey::StartCommandMode => {
            open_jj_command_mode(state);
        }
        AppKey::EditCommandOutput => {
            if matches!(state.views.active(), AppView::Log(_)) {
                open_edit_preview(state, sources.edit);
            } else {
                edit_command_output(state);
            }
        }
        AppKey::OpenDiffFileList => {
            open_diff_file_list(state);
        }
        AppKey::StartSearch if active_view_supports_search(state) => {
            state.modes.push(search_input_mode(state));
        }
        AppKey::SearchNext if matches!(state.views.active(), AppView::Log(_)) => {
            open_new_preview(state, sources.new_change);
        }
        AppKey::SearchNext => {
            apply_search_action(state, SearchDirection::Next);
        }
        AppKey::SearchPrevious => {
            apply_search_action(state, SearchDirection::Previous);
        }
        AppKey::Action(_) | AppKey::Ignore | AppKey::StartSearch => {}
    }
}

fn active_log_has_selected_elision(state: &AppState) -> bool {
    let AppView::Log(log) = state.views.active() else {
        return false;
    };
    log.selected_elision_revset().is_some()
}

fn active_view_supports_search(state: &AppState) -> bool {
    matches!(
        state.views.active(),
        AppView::Diff { .. }
            | AppView::Show { .. }
            | AppView::Evolog { .. }
            | AppView::Status { .. }
            | AppView::WorkspaceLog { .. }
            | AppView::WorkspaceStatus { .. }
            | AppView::WorkspaceDiff { .. }
            | AppView::OperationShow { .. }
            | AppView::OperationDiff { .. }
            | AppView::CommandOutput { .. }
            | AppView::CommandHistoryDetails { .. }
    )
}

fn search_input_mode(state: &AppState) -> InputMode {
    match state.views.active() {
        AppView::Diff { .. } => InputMode::DiffSearch {
            query: String::new(),
        },
        AppView::Show { .. }
        | AppView::Evolog { .. }
        | AppView::Status { .. }
        | AppView::WorkspaceLog { .. }
        | AppView::WorkspaceStatus { .. }
        | AppView::WorkspaceDiff { .. }
        | AppView::OperationShow { .. }
        | AppView::OperationDiff { .. }
        | AppView::CommandHistoryDetails { .. }
        | AppView::CommandOutput { .. } => InputMode::InspectionSearch {
            query: String::new(),
        },
        AppView::Log(_)
        | AppView::Workspaces { .. }
        | AppView::OperationLog { .. }
        | AppView::CommandHistory { .. } => unreachable!("search support checked before call"),
    }
}
