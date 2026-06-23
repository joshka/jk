use jk_cli::{
    DiffQuery, EvologQuery, JjCommandRunner, JjDiff, JjEvolog, JjLog, JjLogCommand, JjOperation,
    JjShow, JjStatus, JjWorkspaces, LogTemplateSelection, OperationQuery, RecordingJjCommandRunner,
    ShowQuery, StatusQuery, SystemJjCommandRunner, WorkspaceInspectionQuery,
};
use jk_core::{CommandHistory, CommandSource, SourceAction, SourceView};
use jk_tui::diff_view::DiffView;
use jk_tui::log_view::LogView;
use jk_tui::operation_log_view::OperationLogView;
use jk_tui::rendered_view::RenderedView;
use jk_tui::workspaces_view::WorkspacesView;

use crate::operation_log::operation_log_snapshot;
use crate::state::{AppState, AppView};
use crate::workspaces::workspace_view_snapshot;
use crate::{AppTransition, WorkspaceInspectionKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationRenderedKind {
    Show,
    Diff,
}

/// Reloads the current command without replacing the view on failure.
pub fn refresh_log(
    app: &mut LogView,
    history: &mut CommandHistory,
    source: &JjLog,
    command_source: CommandSource,
) -> bool {
    refresh_log_with_runner(app, history, source, command_source, SystemJjCommandRunner)
}

pub fn refresh_log_with_runner<R: JjCommandRunner>(
    app: &mut LogView,
    history: &mut CommandHistory,
    source: &JjLog,
    command_source: CommandSource,
    runner: R,
) -> bool {
    let mut runner = RecordingJjCommandRunner::new(runner, history, command_source);
    match source.load_with_runner(&mut runner) {
        Ok(snapshot) => {
            app.refresh(snapshot);
            true
        }
        Err(error) => {
            app.show_error(error.to_string());
            false
        }
    }
}

/// Reloads the active diff without replacing the view on failure.
pub fn refresh_diff(
    app: &mut DiffView,
    query: &DiffQuery,
    history: &mut CommandHistory,
    source: &JjDiff,
) {
    let mut runner = crate::recording_runner(
        history,
        CommandSource::new(SourceView::Diff, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active show/details view without replacing it on failure.
pub fn refresh_show(
    app: &mut RenderedView,
    query: &ShowQuery,
    history: &mut CommandHistory,
    source: &JjShow,
) {
    let mut runner = crate::recording_runner(
        history,
        CommandSource::new(SourceView::Show, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active evolog view without replacing it on failure.
pub fn refresh_evolog(
    app: &mut RenderedView,
    query: &EvologQuery,
    history: &mut CommandHistory,
    source: &JjEvolog,
) {
    let mut runner = crate::recording_runner(
        history,
        CommandSource::new(SourceView::Evolog, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active status view without replacing it on failure.
pub fn refresh_status(
    app: &mut RenderedView,
    query: &StatusQuery,
    history: &mut CommandHistory,
    source: &JjStatus,
) {
    let mut runner = crate::recording_runner(
        history,
        CommandSource::new(SourceView::Status, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the workspace list without replacing the view on failure.
pub fn refresh_workspaces(
    app: &mut WorkspacesView,
    history: &mut CommandHistory,
    source: &JjWorkspaces,
) {
    refresh_workspaces_with_runner(app, history, source, SystemJjCommandRunner);
}

pub fn refresh_workspaces_with_runner<R: JjCommandRunner>(
    app: &mut WorkspacesView,
    history: &mut CommandHistory,
    source: &JjWorkspaces,
    runner: R,
) {
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        history,
        CommandSource::new(SourceView::Workspaces, SourceAction::Refresh),
    );
    match source.load_list_with_runner(&mut runner) {
        Ok(snapshot) => app.refresh(workspace_view_snapshot(snapshot)),
        Err(error) => app.show_error(error.to_string()),
    }
}

pub fn refresh_operation_log(
    app: &mut OperationLogView,
    history: &mut CommandHistory,
    source: &JjOperation,
) {
    let mut runner = crate::recording_runner(
        history,
        CommandSource::new(SourceView::OperationLog, SourceAction::Refresh),
    );
    match source.load_query_with_runner(&OperationQuery::log(), &mut runner) {
        Ok(snapshot) => app.refresh(operation_log_snapshot(
            snapshot.title(),
            snapshot.rendered(),
        )),
        Err(_error) => {}
    }
}

pub fn operation_rendered_transition(
    history: &mut CommandHistory,
    source: &JjOperation,
    query: OperationQuery,
    source_view: SourceView,
    action: SourceAction,
    kind: OperationRenderedKind,
) -> AppTransition {
    operation_rendered_transition_with_runner(
        history,
        source,
        query,
        source_view,
        action,
        kind,
        SystemJjCommandRunner,
    )
}

pub fn operation_rendered_transition_with_runner<R: JjCommandRunner>(
    history: &mut CommandHistory,
    source: &JjOperation,
    query: OperationQuery,
    source_view: SourceView,
    action: SourceAction,
    kind: OperationRenderedKind,
    runner: R,
) -> AppTransition {
    let mut runner =
        RecordingJjCommandRunner::new(runner, history, CommandSource::new(source_view, action));
    match source.load_query_with_runner(&query, &mut runner) {
        Ok(snapshot) => {
            let view = RenderedView::new(snapshot);
            let app_view = match kind {
                OperationRenderedKind::Show => AppView::OperationShow { view, query },
                OperationRenderedKind::Diff => AppView::OperationDiff { view, query },
            };
            AppTransition::Push(app_view)
        }
        Err(_error) => AppTransition::Continue,
    }
}

pub fn refresh_operation_rendered(
    app: &mut RenderedView,
    query: &OperationQuery,
    history: &mut CommandHistory,
    source: &JjOperation,
    view: SourceView,
) {
    let mut runner =
        crate::recording_runner(history, CommandSource::new(view, SourceAction::Refresh));
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads a selected-workspace inspection view without changing its workspace root.
pub fn refresh_workspace_inspection(
    app: &mut RenderedView,
    query: &WorkspaceInspectionQuery,
    history: &mut CommandHistory,
    source: &JjWorkspaces,
    kind: WorkspaceInspectionKind,
) {
    let command_source = match kind {
        WorkspaceInspectionKind::Log => {
            CommandSource::new(SourceView::WorkspaceLog, SourceAction::Refresh)
        }
        WorkspaceInspectionKind::Status => {
            CommandSource::new(SourceView::WorkspaceStatus, SourceAction::Refresh)
        }
        WorkspaceInspectionKind::Diff => {
            CommandSource::new(SourceView::WorkspaceDiff, SourceAction::Refresh)
        }
    };
    let mut runner = crate::recording_runner(history, command_source);
    let snapshot = match kind {
        WorkspaceInspectionKind::Log => source.load_log_with_runner(query, &mut runner),
        WorkspaceInspectionKind::Status => source.load_status_with_runner(query, &mut runner),
        WorkspaceInspectionKind::Diff => source.load_diff_with_runner(query, &mut runner),
    };
    match snapshot {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Switches the command context only after the replacement log loads.
pub fn switch_log_command(
    app: &mut LogView,
    history: &mut CommandHistory,
    source: &mut JjLog,
    command: JjLogCommand,
) {
    let mut next_source = source.clone().with_command(command);
    if command == JjLogCommand::ConfiguredDefault {
        next_source = next_source.with_configured_template();
    }
    let mut runner = crate::recording_runner(
        history,
        CommandSource::new(SourceView::Log, SourceAction::Refresh),
    );
    match next_source.load_with_runner(&mut runner) {
        Ok(snapshot) => {
            *source = next_source;
            app.refresh(snapshot);
        }
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Switches the rendered log template only after the replacement log loads.
pub fn apply_log_template_selection(
    state: &mut AppState,
    source: &mut JjLog,
    template: LogTemplateSelection,
) {
    if !matches!(state.views.active(), AppView::Log(_)) {
        return;
    }

    let next_source = source
        .clone()
        .with_command(JjLogCommand::Log)
        .with_template(template);
    let mut runner = crate::recording_runner(
        &mut state.history,
        CommandSource::new(SourceView::Log, SourceAction::Refresh),
    );
    match next_source.load_with_runner(&mut runner) {
        Ok(snapshot) => {
            *source = next_source;
            if let AppView::Log(log) = state.views.active_mut() {
                log.refresh(snapshot);
            }
        }
        Err(error) => show_log_template_load_error(state, error.to_string()),
    }
}

pub fn show_log_template_load_error(state: &mut AppState, error: String) {
    if let AppView::Log(log) = state.views.active_mut() {
        log.show_error(error);
    }
}
