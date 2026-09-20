use jk_cli::{
    JjCommandRunner, JjStatus, JjWorkspaces, RecordingJjCommandRunner, StatusQuery,
    SystemJjCommandRunner, WorkspaceInspectionQuery,
};
use jk_core::{CommandSource, JjCommandSpec, SourceAction, SourceView};
use jk_tui::rendered_view::RenderedView;
use jk_tui::workspaces_view::{WorkspaceViewSnapshot, WorkspacesView};

use crate::refresh::refresh_workspaces_with_runner;
use crate::state::{AppState, AppView};
use crate::workspace_lifecycle::{WorkspaceLifecycleDialog, WorkspaceLifecycleKind};
use crate::workspaces::{update_stale_success_message, workspace_view_snapshot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceInspectionKind {
    Log,
    Status,
    Diff,
}

pub fn open_workspace_lifecycle(
    state: &mut AppState,
    source: &JjWorkspaces,
    kind: WorkspaceLifecycleKind,
) {
    let AppView::Workspaces { view } = state.views.active_mut() else {
        return;
    };
    let Some(row) = view.selected_row() else {
        view.show_status("No workspace selected");
        return;
    };
    let Some(root) = row.root().map(ToOwned::to_owned) else {
        view.show_error(format!("workspace `{}` has no root", row.name));
        return;
    };
    let dialog = WorkspaceLifecycleDialog::new(kind, row.name.clone(), root, source);
    state
        .modes
        .push(crate::state::InputMode::WorkspaceLifecycle {
            dialog: Box::new(dialog),
        });
}

pub fn execute_workspace_lifecycle(
    state: &mut AppState,
    source: &JjWorkspaces,
    spec: JjCommandSpec,
    kind: WorkspaceLifecycleKind,
    preferred: Option<String>,
) {
    execute_workspace_lifecycle_with_runner(
        state,
        source,
        spec,
        kind,
        preferred,
        SystemJjCommandRunner,
    );
}

pub(crate) fn execute_workspace_lifecycle_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &JjWorkspaces,
    spec: JjCommandSpec,
    kind: WorkspaceLifecycleKind,
    preferred: Option<String>,
    runner: R,
) {
    state.cancel_log_refresh();
    let command_source = CommandSource::new(SourceView::Workspaces, kind.action());
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, command_source);
    let result = runner.run_confirmed_mutation(&spec);
    let runner = runner.into_inner();
    let action_title = kind.title().trim_end_matches('?');
    let success_message = match result {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Some(if kind == WorkspaceLifecycleKind::UpdateStale {
                update_stale_success_message(
                    preferred.as_deref().unwrap_or("workspace"),
                    spec.title(),
                    &stderr,
                    &stdout,
                )
            } else {
                format!("{action_title} succeeded · C history · o operations")
            })
        }
        Ok(output) => {
            let message = crate::mutation_preview::command_failure_message(
                action_title,
                &output.stderr,
                &output.stdout,
            );
            if let AppView::Workspaces { view } = state.views.active_mut() {
                view.show_error(message.lines().next().unwrap_or(&message));
            }
            None
        }
        Err(error) => {
            if let AppView::Workspaces { view } = state.views.active_mut() {
                view.show_error(format!("{action_title} failed: {error}"));
            }
            None
        }
    };
    let Some(success_message) = success_message else {
        return;
    };
    let refresh = {
        let mut runner = RecordingJjCommandRunner::new(
            runner,
            &mut state.history,
            CommandSource::new(SourceView::Workspaces, SourceAction::Refresh),
        );
        source.load_list_with_runner(&mut runner)
    };
    if let AppView::Workspaces { view } = state.views.active_mut() {
        match refresh {
            Ok(snapshot) => {
                view.refresh_selecting(workspace_view_snapshot(snapshot), preferred.as_deref());
                view.show_status(success_message);
            }
            Err(error) => view.show_status(format!("{success_message}; refresh failed: {error}")),
        }
    }
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

#[cfg(test)]
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

#[cfg(test)]
mod tests {
    use std::io;

    use jk_core::SourceAction;

    use super::*;
    use crate::test_support::{SequencedRunner, output};

    #[test]
    fn successful_lifecycle_records_operation_and_refreshes_preferred_workspace() {
        let source = JjWorkspaces::default().with_repository("/repo/default");
        let spec = source.add_spec(std::path::Path::new("/repo/new"), "new");
        let mut state = workspace_state();
        let runner = SequencedRunner::results(vec![
            Ok(output(0, "111111111111\n", "")),
            Ok(output(0, "", "")),
            Ok(output(0, "222222222222\n", "")),
            Ok(output(
                0,
                "default\t/repo/default\tchange-1\tcommit-1\nnew\t/repo/new\tchange-2\tcommit-2\n",
                "",
            )),
            Ok(output(0, "/repo/default\n", "")),
        ]);

        execute_workspace_lifecycle_with_runner(
            &mut state,
            &source,
            spec,
            WorkspaceLifecycleKind::Add,
            Some("new".to_owned()),
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].source.action, SourceAction::WorkspaceAdd);
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        assert!(
            records[1..]
                .iter()
                .all(|record| record.source.action == SourceAction::Refresh)
        );
        let AppView::Workspaces { view } = state.views.active() else {
            panic!("workspace lifecycle keeps the workspace view open");
        };
        assert_eq!(view.selected_workspace_name(), Some("new"));
    }

    #[test]
    fn failed_lifecycle_does_not_refresh() {
        let source = JjWorkspaces::default().with_repository("/repo/default");
        let spec = source.forget_spec(&"long-workspace-name".repeat(20));
        let mut state = workspace_state();
        let runner = SequencedRunner::results(vec![
            Ok(output(0, "111111111111\n", "")),
            Ok(output(1, "", "\u{1b}[31mcannot forget\u{1b}[0m\n")),
        ]);

        execute_workspace_lifecycle_with_runner(
            &mut state,
            &source,
            spec,
            WorkspaceLifecycleKind::Forget,
            None,
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].source.action, SourceAction::WorkspaceForget);
        assert_eq!(records[0].operation_id, None);

        let AppView::Workspaces { view } = state.views.active_mut() else {
            panic!("failure keeps workspace view open");
        };
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 12))
            .expect("test terminal");
        terminal
            .draw(|frame| view.render(frame))
            .expect("failure renders");
        let screen = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(screen.contains("Forget workspace metadata failed: cannot forget"));
    }

    #[test]
    fn spawn_failure_does_not_refresh() {
        let source = JjWorkspaces::default().with_repository("/repo/default");
        let spec = source.forget_spec("scratch");
        let mut state = workspace_state();
        let runner = SequencedRunner::results(vec![
            Ok(output(0, "111111111111\n", "")),
            Err(io::Error::new(io::ErrorKind::NotFound, "jj unavailable")),
        ]);

        execute_workspace_lifecycle_with_runner(
            &mut state,
            &source,
            spec,
            WorkspaceLifecycleKind::Forget,
            None,
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].source.action, SourceAction::WorkspaceForget);
        assert_eq!(
            records[0].result.spawn_error.as_deref(),
            Some("jj unavailable")
        );
    }

    fn workspace_state() -> AppState {
        AppState::new(AppView::Workspaces {
            view: WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new())),
        })
    }
}
