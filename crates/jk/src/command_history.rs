use jk_cli::{
    JjCommandRunner, JjOperation, OperationQuery, RecordingJjCommandRunner, SystemJjCommandRunner,
};
use jk_core::{CommandHistory, CommandSource, SourceAction, SourceView};
use jk_tui::command_history_view::{
    CommandHistoryAction, CommandHistoryActionResult, CommandHistorySnapshot, CommandHistoryView,
};
use jk_tui::rendered_view::RenderedView;

use crate::AppTransition;
use crate::operation_log::operation_log_snapshot;
use crate::refresh::{
    OperationRenderedKind, operation_rendered_transition_with_runner, refresh_operation_log,
};
use crate::state::{AppState, AppView};

pub fn open_command_history(state: &mut AppState) {
    let snapshot = command_history_snapshot(&state.history);
    if let AppView::CommandHistory { view } = state.views.active_mut() {
        view.refresh(snapshot);
        return;
    }

    let view = CommandHistoryView::new_focused_on_latest_operation(snapshot);
    state.views.push(AppView::CommandHistory { view });
}

pub fn command_history_snapshot(history: &CommandHistory) -> CommandHistorySnapshot {
    CommandHistorySnapshot::from_records(history.records())
}

pub fn push_selected_command_history_details(state: &mut AppState) {
    let action = {
        let AppView::CommandHistory { view } = state.views.active_mut() else {
            return;
        };
        view.apply(CommandHistoryAction::OpenDetails)
    };

    if let CommandHistoryActionResult::OpenDetails { details } = action {
        state.views.push(AppView::CommandHistoryDetails {
            view: RenderedView::new(details.into_snapshot()),
        });
    }
}

pub fn open_command_history_operation(state: &mut AppState, operation_source: &JjOperation) {
    open_command_history_operation_with_runner(state, operation_source, SystemJjCommandRunner);
}

pub fn open_command_history_operation_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    operation_source: &JjOperation,
    runner: R,
) {
    let action = {
        let AppView::CommandHistory { view } = state.views.active_mut() else {
            return;
        };
        view.apply(CommandHistoryAction::OpenOperation)
    };

    match action {
        CommandHistoryActionResult::OpenOperation { operation_id } => {
            let query = OperationQuery::show(operation_id);
            let transition = operation_rendered_transition_with_runner(
                &mut state.history,
                operation_source,
                query,
                SourceView::CommandHistory,
                SourceAction::OperationShow,
                OperationRenderedKind::Show,
                runner,
            );
            if let AppTransition::Push(view) = transition {
                state.views.push(view);
            }
        }
        CommandHistoryActionResult::OpenOperationLog => {
            open_operation_log_from_with_runner(
                state,
                operation_source,
                CommandSource::new(SourceView::CommandHistory, SourceAction::OperationLog),
                runner,
            );
        }
        CommandHistoryActionResult::Continue
        | CommandHistoryActionResult::Refresh
        | CommandHistoryActionResult::ReturnBack
        | CommandHistoryActionResult::Quit => {}
        _ => {}
    }
}

pub fn open_operation_log(state: &mut AppState, operation_source: &JjOperation) {
    open_operation_log_from(
        state,
        operation_source,
        CommandSource::new(SourceView::Log, SourceAction::OperationLog),
    );
}

pub fn open_operation_log_from(
    state: &mut AppState,
    operation_source: &JjOperation,
    source: CommandSource,
) {
    open_operation_log_from_with_runner(state, operation_source, source, SystemJjCommandRunner);
}

pub fn open_operation_log_from_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    operation_source: &JjOperation,
    source: CommandSource,
    runner: R,
) {
    if matches!(state.views.active(), AppView::OperationLog { .. }) {
        let AppState { views, history, .. } = state;
        if let AppView::OperationLog { view } = views.active_mut() {
            refresh_operation_log(view, history, operation_source);
        }
        return;
    }

    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, source);
    let query = OperationQuery::log();
    match operation_source.load_query_with_runner(&query, &mut runner) {
        Ok(snapshot) => {
            let view = jk_tui::operation_log_view::OperationLogView::new(operation_log_snapshot(
                snapshot.title(),
                snapshot.rendered(),
            ));
            state.views.push(AppView::OperationLog { view });
        }
        Err(error) => {
            if let AppView::Log(log) = state.views.active_mut() {
                log.show_error(error.to_string());
            }
        }
    }
}

pub fn apply_command_history_action(
    view: &mut CommandHistoryView,
    history: &CommandHistory,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let history_action = match action {
        jk_tui::log_view::LogAction::Previous => CommandHistoryAction::Previous,
        jk_tui::log_view::LogAction::Next => CommandHistoryAction::Next,
        jk_tui::log_view::LogAction::ScrollPreviousLine => CommandHistoryAction::Previous,
        jk_tui::log_view::LogAction::ScrollNextLine => CommandHistoryAction::Next,
        jk_tui::log_view::LogAction::PagePrevious => CommandHistoryAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext | jk_tui::log_view::LogAction::ToggleMark => {
            CommandHistoryAction::PageNext
        }
        jk_tui::log_view::LogAction::First => CommandHistoryAction::First,
        jk_tui::log_view::LogAction::Last => CommandHistoryAction::Last,
        jk_tui::log_view::LogAction::Refresh => CommandHistoryAction::Refresh,
        jk_tui::log_view::LogAction::ToggleHelp => CommandHistoryAction::ToggleHelp,
        jk_tui::log_view::LogAction::Quit => CommandHistoryAction::Quit,
        _ => return AppTransition::Continue,
    };

    match view.apply(history_action) {
        CommandHistoryActionResult::Refresh => view.refresh(command_history_snapshot(history)),
        CommandHistoryActionResult::OpenDetails { details } => {
            return AppTransition::Push(AppView::CommandHistoryDetails {
                view: RenderedView::new(details.into_snapshot()),
            });
        }
        CommandHistoryActionResult::ReturnBack => return AppTransition::PopView,
        CommandHistoryActionResult::Quit => return AppTransition::Quit,
        CommandHistoryActionResult::Continue => {}
        _ => {}
    }

    AppTransition::Continue
}
