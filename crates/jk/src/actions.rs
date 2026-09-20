use crossterm::event::{KeyCode, KeyEvent};
use jk_cli::{
    JjAbandon, JjBookmarks, JjDiff, JjEdit, JjEvolog, JjGitRemote, JjLog, JjNew, JjOperation,
    JjRecovery, JjShow, JjStatus, JjWorkspaces, RecoveryCommand,
};
use jk_tui::log_view::LogAction;

use crate::key::AppKey;
use crate::state::{AppState, AppView, InputMode};
use crate::{
    AppLoop, SearchDirection, abandon_or_preview, apply_action, apply_bookmark_action,
    apply_search_action, copy_selected_command, edit_command_output, execute_edit_action,
    execute_new_action, execute_recovery_action, handle_back_with_log_source, open_action_menu,
    open_bookmarks, open_command_discovery, open_command_history, open_command_history_operation,
    open_diff_file_list, open_jj_command_mode, open_operation_log, open_remote_preview,
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
    pub(crate) bookmarks: &'a JjBookmarks,
    pub(crate) remotes: &'a JjGitRemote,
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
    if let AppView::Bookmarks { view } = state.views.active_mut() {
        if view.help_visible() {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('q' | '?')) {
                let _ = view.apply(jk_tui::bookmark_view::BookmarkAction::ToggleHelp);
            }
            return DispatchResult::Continue;
        }
        if matches!(app_key, AppKey::Action(LogAction::Quit)) {
            return DispatchResult::Quit;
        }
    }
    if matches!(state.views.active(), AppView::Bookmarks { .. }) {
        let action = match key.code {
            KeyCode::Char('m') => Some(jk_tui::bookmark_view::BookmarkAction::Move),
            KeyCode::Char('?') => Some(jk_tui::bookmark_view::BookmarkAction::ToggleHelp),
            _ => None,
        };
        if let Some(action) = action {
            apply_bookmark_action(state, sources.bookmarks, sources.show, action);
            return DispatchResult::Continue;
        }
    }
    if matches!(
        state.views.active(),
        AppView::Bookmarks { .. }
            | AppView::Workspaces { .. }
            | AppView::CommandHistory { .. }
            | AppView::OperationLog { .. }
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

    if matches!(state.views.active(), AppView::Bookmarks { .. }) {
        apply_bookmark_action(
            state,
            sources.bookmarks,
            sources.show,
            bookmark_action_for_log_action(action),
        );
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

fn bookmark_action_for_log_action(action: LogAction) -> jk_tui::bookmark_view::BookmarkAction {
    use jk_tui::bookmark_view::BookmarkAction;
    match action {
        LogAction::Previous | LogAction::ScrollPreviousLine => BookmarkAction::Previous,
        LogAction::Next | LogAction::ScrollNextLine => BookmarkAction::Next,
        LogAction::First => BookmarkAction::First,
        LogAction::Last => BookmarkAction::Last,
        LogAction::Refresh => BookmarkAction::Refresh,
        LogAction::ClearMarks => BookmarkAction::Create,
        LogAction::ToggleHelp => BookmarkAction::ToggleHelp,
        LogAction::Quit => BookmarkAction::Quit,
        LogAction::CollapseExpanded | LogAction::Home | LogAction::Log => {
            BookmarkAction::ReturnBack
        }
        _ => BookmarkAction::Continue,
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
            if matches!(state.views.active(), AppView::Bookmarks { .. }) {
                apply_bookmark_action(
                    state,
                    sources.bookmarks,
                    sources.show,
                    jk_tui::bookmark_view::BookmarkAction::OpenTarget,
                );
            } else if matches!(state.views.active(), AppView::OperationLog { .. }) {
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
        AppKey::OpenBookmarks => {
            open_bookmarks(state, sources.bookmarks);
        }
        AppKey::FetchRemote => {
            if matches!(state.views.active(), AppView::Bookmarks { .. }) {
                open_remote_preview(state, sources.remotes, false);
            }
        }
        AppKey::PushDryRun => {
            if matches!(state.views.active(), AppView::Bookmarks { .. }) {
                open_remote_preview(state, sources.remotes, true);
            }
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
            }
        }
        AppKey::RunUndo => {
            execute_recovery_action(state, sources.log, sources.recovery, RecoveryCommand::Undo);
        }
        AppKey::RunRedo => {
            execute_recovery_action(state, sources.log, sources.recovery, RecoveryCommand::Redo);
        }
        AppKey::BookmarkMove | AppKey::StartDescribe => {
            if matches!(state.views.active(), AppView::Bookmarks { .. }) {
                apply_bookmark_action(
                    state,
                    sources.bookmarks,
                    sources.show,
                    jk_tui::bookmark_view::BookmarkAction::Move,
                );
            } else if app_key == AppKey::StartDescribe {
                crate::open_describe_message(state);
            }
        }
        AppKey::DeleteBookmark => {
            if matches!(state.views.active(), AppView::Bookmarks { .. }) {
                apply_bookmark_action(
                    state,
                    sources.bookmarks,
                    sources.show,
                    jk_tui::bookmark_view::BookmarkAction::Delete,
                );
            }
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
        | AppView::Bookmarks { .. }
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
            operation: &operation,
            recovery: &recovery,
            workspaces: &workspaces,
            bookmarks: &JjBookmarks::default(),
            remotes: &JjGitRemote::default(),
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
