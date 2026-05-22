use super::{
    App, DiffFormat, InteractionMode, JjCommand, KeyCode, KeyEvent, KeyEventKind, KeyEventState,
    KeyModifiers, LogViewMode, Result, StatusLine, ViewSpec, ViewState,
};

pub fn test_app(view: ViewState) -> App {
    App {
        status: StatusLine::ready(&view),
        view,
        stack: Vec::new(),
        startup_log_args: None,
        diff_format: DiffFormat::Default,
        mode: InteractionMode::Normal,
        pending_command: None,
        search: None,
        should_quit: false,
        services: super::services::test_services(),
    }
}

pub fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

pub fn log_item(change_id: &str) -> crate::log::LogItem {
    crate::log::LogItem::new(
        vec![ratatui::text::Line::from(change_id.to_owned())],
        Some(change_id.to_owned()),
        None,
    )
}

pub fn mock_load_view(spec: ViewSpec) -> Result<ViewState> {
    let view = match spec.command() {
        JjCommand::Default | JjCommand::Log => {
            ViewState::Log(crate::log::LogView::test_with_spec(spec, vec![]))
        }
        JjCommand::Show => ViewState::Show(crate::show::ShowView::test_new(spec)),
        JjCommand::Diff => ViewState::Diff(crate::diff::DiffView::test_new(spec)),
        JjCommand::Status => ViewState::Status(crate::status::StatusView::test_new(&[])),
        JjCommand::Resolve => ViewState::Resolve(crate::resolve::ResolveView::test_new(vec![])),
        JjCommand::FileList => {
            ViewState::FileList(crate::files::list::FileListView::test_new(vec![]))
        }
        JjCommand::FileShow => ViewState::FileShow(crate::files::show::FileShowView::new(
            spec,
            "src/main.rs",
            crate::documents::DocumentLines::new(Vec::new()),
        )),
        JjCommand::Bookmarks => {
            ViewState::Bookmarks(crate::bookmarks::BookmarksView::test_new(vec![]))
        }
        JjCommand::Workspaces => {
            ViewState::Workspaces(crate::workspaces::WorkspacesView::test_new(
                crate::workspaces::WorkspaceContext::default(),
            ))
        }
        JjCommand::OperationLog => {
            ViewState::OperationLog(crate::operation_log::OperationLogView::test_new(vec![]))
        }
        JjCommand::OperationShow | JjCommand::OperationDiff => {
            ViewState::OperationDetail(crate::operation_log::detail::OperationDetailView::test_new(
                spec,
                crate::documents::DocumentLines::new(Vec::new()),
            ))
        }
    };
    Ok(view)
}

pub fn default_reveal_log_change(
    view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    view.reveal_log_change(change_id, fallback_mode)
}
