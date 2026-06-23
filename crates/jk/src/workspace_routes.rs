use jk_cli::{
    JjCommandRunner, JjStatus, JjWorkspaces, RecordingJjCommandRunner, StatusQuery,
    SystemJjCommandRunner, WorkspaceInspectionQuery,
};
use jk_core::{CommandSource, SourceAction, SourceView};
use jk_tui::rendered_view::RenderedView;
use jk_tui::workspaces_view::{WorkspaceViewSnapshot, WorkspacesView};

use crate::refresh::refresh_workspaces_with_runner;
use crate::state::{AppState, AppView};
use crate::workspaces::{update_stale_success_message, workspace_view_snapshot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceInspectionKind {
    Log,
    Status,
    Diff,
}

pub fn push_status(state: &mut AppState, status_source: &JjStatus) {
    push_status_with_runner(state, status_source, SystemJjCommandRunner);
}

pub fn push_status_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    status_source: &JjStatus,
    runner: R,
) {
    if !matches!(state.views.active(), AppView::Log(_)) {
        return;
    }

    let query = StatusQuery::default();
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        &mut state.history,
        CommandSource::new(SourceView::Log, SourceAction::OpenStatus),
    );
    match status_source.load_query_with_runner(&query, &mut runner) {
        Ok(snapshot) => {
            state.views.push(AppView::Status {
                view: RenderedView::new(snapshot),
                query,
            });
        }
        Err(error) => {
            if let AppView::Log(log) = state.views.active_mut() {
                log.show_error(error.to_string());
            }
        }
    }
}

pub fn open_workspaces(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    open_workspaces_with_runner(state, workspaces_source, SystemJjCommandRunner);
}

pub fn open_workspaces_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    workspaces_source: &JjWorkspaces,
    runner: R,
) {
    if matches!(state.views.active(), AppView::Workspaces { .. }) {
        let AppState { views, history, .. } = state;
        if let AppView::Workspaces { view } = views.active_mut() {
            refresh_workspaces_with_runner(view, history, workspaces_source, runner);
        }
        return;
    }

    push_workspaces_with_runner(
        state,
        workspaces_source,
        CommandSource::new(SourceView::Log, SourceAction::WorkspaceList),
        runner,
    );
}

fn push_workspaces_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    workspaces_source: &JjWorkspaces,
    source: CommandSource,
    runner: R,
) {
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, source);
    let view = match workspaces_source.load_list_with_runner(&mut runner) {
        Ok(snapshot) => WorkspacesView::new(workspace_view_snapshot(snapshot)),
        Err(error) => {
            let mut view = WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new()));
            view.show_error(error.to_string());
            view
        }
    };
    push_workspace_view(state, view);
}

pub fn push_workspace_view(state: &mut AppState, view: WorkspacesView) {
    state.views.push(AppView::Workspaces { view });
}

pub fn push_selected_workspace_status(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    push_selected_workspace_inspection_with_runner(
        state,
        workspaces_source,
        WorkspaceInspectionKind::Status,
        SystemJjCommandRunner,
    );
}

pub fn push_selected_workspace_log(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    push_selected_workspace_inspection_with_runner(
        state,
        workspaces_source,
        WorkspaceInspectionKind::Log,
        SystemJjCommandRunner,
    );
}

#[cfg(test)]
pub fn push_selected_workspace_log_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    workspaces_source: &JjWorkspaces,
    runner: R,
) {
    push_selected_workspace_inspection_with_runner(
        state,
        workspaces_source,
        WorkspaceInspectionKind::Log,
        runner,
    );
}

pub fn push_selected_workspace_diff(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    push_selected_workspace_inspection_with_runner(
        state,
        workspaces_source,
        WorkspaceInspectionKind::Diff,
        SystemJjCommandRunner,
    );
}

fn push_selected_workspace_inspection_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    workspaces_source: &JjWorkspaces,
    kind: WorkspaceInspectionKind,
    runner: R,
) {
    let query = {
        let AppView::Workspaces { view } = state.views.active_mut() else {
            return;
        };
        let Some(row) = view.selected_row() else {
            return;
        };
        let Some(root) = row.root().map(ToOwned::to_owned) else {
            view.show_error(format!("workspace `{}` has no root", row.name));
            return;
        };
        WorkspaceInspectionQuery::new(root)
    };

    let command_source = match kind {
        WorkspaceInspectionKind::Log => {
            CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceLog)
        }
        WorkspaceInspectionKind::Status => {
            CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceStatus)
        }
        WorkspaceInspectionKind::Diff => {
            CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceDiff)
        }
    };
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, command_source);
    let snapshot = match kind {
        WorkspaceInspectionKind::Log => workspaces_source.load_log_with_runner(&query, &mut runner),
        WorkspaceInspectionKind::Status => {
            workspaces_source.load_status_with_runner(&query, &mut runner)
        }
        WorkspaceInspectionKind::Diff => {
            workspaces_source.load_diff_with_runner(&query, &mut runner)
        }
    };
    match snapshot {
        Ok(snapshot) => {
            let view = RenderedView::new(snapshot);
            let app_view = match kind {
                WorkspaceInspectionKind::Log => AppView::WorkspaceLog { view, query },
                WorkspaceInspectionKind::Status => AppView::WorkspaceStatus { view, query },
                WorkspaceInspectionKind::Diff => AppView::WorkspaceDiff { view, query },
            };
            state.views.push(app_view);
        }
        Err(error) => {
            if let AppView::Workspaces { view } = state.views.active_mut() {
                view.show_error(error.to_string());
            }
        }
    }
}

pub fn update_selected_workspace_stale(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    let (workspace_name, query) = {
        let AppView::Workspaces { view } = state.views.active_mut() else {
            return;
        };
        let Some(row) = view.selected_row() else {
            view.show_status("No workspace selected");
            return;
        };
        let workspace_name = row.name.clone();
        let Some(root) = row.root().map(ToOwned::to_owned) else {
            view.show_error(format!("workspace `{}` has no root", row.name));
            return;
        };
        (workspace_name, WorkspaceInspectionQuery::new(root))
    };

    let mut update_runner = crate::recording_runner(
        &mut state.history,
        CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceUpdateStale),
    );
    match workspaces_source.update_stale_with_runner(&query, &mut update_runner) {
        Ok(outcome) => {
            let success = update_stale_success_message(
                &workspace_name,
                &outcome.title,
                &outcome.stderr,
                &outcome.stdout,
            );
            let refresh_result = {
                let mut refresh_runner = crate::recording_runner(
                    &mut state.history,
                    CommandSource::new(SourceView::Workspaces, SourceAction::Refresh),
                );
                workspaces_source.load_list_with_runner(&mut refresh_runner)
            };
            if let AppView::Workspaces { view } = state.views.active_mut() {
                match refresh_result {
                    Ok(snapshot) => {
                        view.refresh(workspace_view_snapshot(snapshot));
                        view.show_status(success);
                    }
                    Err(error) => {
                        view.show_status(format!("{success}; refresh failed: {error}"));
                    }
                }
            }
        }
        Err(error) => {
            if let AppView::Workspaces { view } = state.views.active_mut() {
                view.show_error(error.to_string());
            }
        }
    }
}
