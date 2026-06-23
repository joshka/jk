use jk_cli::{DiffQuery, JjDiff, JjLog, JjShow, JjStatus, JjWorkspaces, ShowQuery, StatusQuery};
use jk_core::{CommandHistory, CommandSource, SourceAction, SourceView};
use jk_tui::diff_view::DiffView;
use jk_tui::log_view::LogView;
use jk_tui::rendered_view::RenderedView;
use jk_tui::workspaces_view::{WorkspaceViewSnapshot, WorkspacesView};

use crate::runner::recording_runner;
use crate::state::AppView;
use crate::workspaces::workspace_view_snapshot;

pub fn root_log_view(source: &JjLog, history: &mut CommandHistory) -> color_eyre::Result<AppView> {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Log, SourceAction::InitialLoad),
    );
    let entries = source.load_with_runner(&mut runner)?;
    Ok(AppView::Log(LogView::new(entries)))
}

pub fn root_diff_view(
    diff_source: &JjDiff,
    query: DiffQuery,
    history: &mut CommandHistory,
) -> AppView {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Diff, SourceAction::InitialLoad),
    );
    let snapshot = diff_source.load_query_with_runner(&query, &mut runner);
    let diff = match snapshot {
        Ok(snapshot) => DiffView::new(snapshot),
        Err(error) => DiffView::from_error(
            query.target_label(),
            diff_source.spec_for(&query).title().to_owned(),
            error.to_string(),
        ),
    };
    AppView::Diff { view: diff, query }
}

pub fn root_show_view(
    show_source: &JjShow,
    query: ShowQuery,
    history: &mut CommandHistory,
) -> AppView {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Show, SourceAction::InitialLoad),
    );
    let snapshot = show_source.load_query_with_runner(&query, &mut runner);
    let show = match snapshot {
        Ok(snapshot) => RenderedView::new(snapshot),
        Err(error) => RenderedView::from_error(
            query.target_label(),
            show_source.spec_for(&query).title().to_owned(),
            error.to_string(),
        ),
    };
    AppView::Show { view: show, query }
}

pub fn root_status_view(
    status_source: &JjStatus,
    query: StatusQuery,
    history: &mut CommandHistory,
) -> AppView {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Status, SourceAction::InitialLoad),
    );
    let snapshot = status_source.load_query_with_runner(&query, &mut runner);
    let status = match snapshot {
        Ok(snapshot) => RenderedView::new(snapshot),
        Err(error) => RenderedView::from_error(
            query.target_label(),
            status_source.spec_for(&query).title().to_owned(),
            error.to_string(),
        ),
    };
    AppView::Status {
        view: status,
        query,
    }
}

pub fn root_workspaces_view(
    workspaces_source: &JjWorkspaces,
    history: &mut CommandHistory,
) -> AppView {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Workspaces, SourceAction::InitialLoad),
    );
    let view = match workspaces_source.load_list_with_runner(&mut runner) {
        Ok(snapshot) => WorkspacesView::new(workspace_view_snapshot(snapshot)),
        Err(error) => {
            let mut view = WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new()));
            view.show_error(error.to_string());
            view
        }
    };
    AppView::Workspaces { view }
}
