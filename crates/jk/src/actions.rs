use crossterm::event::{KeyCode, KeyEvent};
use jk_cli::{
    JjAbandon, JjDiff, JjEdit, JjEvolog, JjLog, JjNew, JjOperation, JjRecovery, JjRestore, JjShow,
    JjSquash, JjStatus, JjWorkspaces, RecoveryCommand,
};
use jk_tui::log_view::LogAction;

use crate::key::AppKey;
use crate::state::{AppState, AppView, InputMode};
use crate::workspace_lifecycle::WorkspaceLifecycleKind;
use crate::{
    AppLoop, SearchDirection, abandon_or_preview, apply_action, apply_search_action,
    copy_selected_command, edit_command_output, execute_edit_action, execute_new_action,
    execute_recovery_action, handle_back_with_log_source, open_action_menu, open_command_discovery,
    open_command_history, open_command_history_operation, open_diff_file_list,
    open_jj_command_mode, open_operation_log, open_restore_preview, open_squash_preview,
    open_view_options, open_workspaces, push_selected_command_history_details,
    push_selected_evolog, push_selected_operation_show, push_selected_show,
    push_selected_workspace_status, push_status,
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
    pub(crate) squash: &'a JjSquash,
    pub(crate) restore: &'a JjRestore,
    pub(crate) rebase: &'a jk_cli::JjRebase,
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
        AppKey::OpenActionMenu => {
            open_action_menu(state);
        }
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
                crate::workspace_routes::open_workspace_lifecycle(
                    state,
                    sources.workspaces,
                    WorkspaceLifecycleKind::UpdateStale,
                );
            }
        }
        AppKey::RunUndo => {
            execute_recovery_action(state, sources.log, sources.recovery, RecoveryCommand::Undo);
        }
        AppKey::RunRedo => {
            execute_recovery_action(state, sources.log, sources.recovery, RecoveryCommand::Redo);
        }
        AppKey::StartDescribe => {
            crate::open_describe_message(state);
        }
        AppKey::StartNew => {
            execute_new_action(state, sources.log, sources.new_change);
        }
        AppKey::StartEdit => {
            execute_edit_action(state, sources.log, sources.edit);
        }
        AppKey::StartAbandon => {
            abandon_or_preview(state, sources.log, sources.abandon);
        }
        AppKey::StartSquash => {
            open_squash_preview(state, sources.squash);
        }
        AppKey::StartRestore => {
            open_restore_preview(state, sources.restore);
        }
        AppKey::StartRebase => crate::rebase::open_rebase_destination(state, sources.rebase),
        AppKey::StartWorkspaceAdd => crate::workspace_routes::open_workspace_lifecycle(
            state,
            sources.workspaces,
            WorkspaceLifecycleKind::Add,
        ),
        AppKey::StartWorkspaceRename => crate::workspace_routes::open_workspace_lifecycle(
            state,
            sources.workspaces,
            WorkspaceLifecycleKind::Rename,
        ),
        AppKey::StartWorkspaceForget => crate::workspace_routes::open_workspace_lifecycle(
            state,
            sources.workspaces,
            WorkspaceLifecycleKind::Forget,
        ),
        AppKey::StartWorkspaceUpdateStale => crate::workspace_routes::open_workspace_lifecycle(
            state,
            sources.workspaces,
            WorkspaceLifecycleKind::UpdateStale,
        ),
        AppKey::OpenViewOptions => {
            if !matches!(state.views.active(), AppView::CommandHistory { .. }) {
                open_view_options(state);
            }
        }
        AppKey::StartCommandMode => {
            open_jj_command_mode(state);
        }
        AppKey::EditCommandOutput if matches!(state.views.active(), AppView::Log(_)) => {}
        AppKey::EditCommandOutput => {
            edit_command_output(state);
        }
        AppKey::OpenDiffFileList => {
            open_diff_file_list(state);
        }
        AppKey::StartSearch if active_view_supports_search(state) => {
            state.modes.push(search_input_mode(state));
        }
        AppKey::SearchNext if matches!(state.views.active(), AppView::Log(_)) => {}
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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    #[test]
    fn direct_mutation_keys_are_menu_only_on_log() {
        let mut state = AppState::new(AppView::Log(jk_tui::log_view::LogView::default()));
        let before = state.views.active().clone();

        let mut log = JjLog::default();
        let diff = JjDiff::default();
        let evolog = JjEvolog::default();
        let show = JjShow::default();
        let status = JjStatus::default();
        let abandon = JjAbandon::default();
        let new_change = JjNew::default();
        let edit = JjEdit::default();
        let squash = JjSquash::default();
        let restore = JjRestore::default();
        let rebase = jk_cli::JjRebase::default();
        let operation = JjOperation::default();
        let recovery = JjRecovery::default();
        let workspaces = JjWorkspaces::default();
        let mut sources = AppSources {
            log: &mut log,
            diff: &diff,
            evolog: &evolog,
            show: &show,
            status: &status,
            abandon: &abandon,
            new_change: &new_change,
            edit: &edit,
            squash: &squash,
            restore: &restore,
            rebase: &rebase,
            operation: &operation,
            recovery: &recovery,
            workspaces: &workspaces,
        };

        assert_eq!(
            dispatch_app_key(
                &mut state,
                &mut sources,
                KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
                AppKey::SearchNext,
            ),
            DispatchResult::Continue
        );
        assert_eq!(state.views.active(), &before);

        assert_eq!(
            dispatch_app_key(
                &mut state,
                &mut sources,
                KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
                AppKey::EditCommandOutput,
            ),
            DispatchResult::Continue
        );
        assert_eq!(state.views.active(), &before);
        assert_eq!(state.modes.active(), None);

        assert_eq!(
            dispatch_app_key(
                &mut state,
                &mut sources,
                KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE),
                AppKey::Ignore,
            ),
            DispatchResult::Continue
        );
        assert_eq!(state.views.active(), &before);
        assert_eq!(state.modes.active(), None);

        assert_eq!(
            dispatch_app_key(
                &mut state,
                &mut sources,
                KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE),
                AppKey::StartUndo,
            ),
            DispatchResult::Continue
        );
        assert_eq!(state.views.active(), &before);
        assert_eq!(state.modes.active(), None);
    }
}
