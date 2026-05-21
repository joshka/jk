//! Fetch and push action lifecycle tests.

use super::support::*;

#[test]
fn default_fetch_runs_immediately_and_keeps_result_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "fetch: fetched");
    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), None);
            output
        }
        _ => panic!("expected fetch result mode"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git fetch");
    assert_eq!(
        output.body_lines(),
        [
            "command: jj git fetch",
            "context: default fetch uses jj git fetch remote resolution",
            "output:",
            "  fetched",
        ]
    );
}

#[test]
fn graph_remote_fetch_key_opens_remote_prompt_for_multiple_remotes() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.handle_normal_key(key(KeyCode::Char('g'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();

    match &app.mode {
        InteractionMode::FetchRemotePrompt { remotes, selected } => {
            assert_eq!(remotes, &["origin".to_owned(), "upstream".to_owned()]);
            assert_eq!(*selected, 0);
        }
        _ => panic!("expected fetch remote prompt"),
    }
}

#[test]
fn fetch_remote_prompt_selects_remote_for_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.open_fetch_remote_prompt();

    app.handle_mode_key(crossterm::event::KeyCode::Down, 12)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), Some("upstream"));
            output
        }
        _ => panic!("expected fetch preview"),
    };
    assert!(!output.completed());
    assert_eq!(
        output.command_label(),
        "jj git fetch --remote exact:\"upstream\""
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("fetch targets exact remote 'upstream' with pattern exact:\"upstream\"")
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("remote pattern: exact:\"upstream\"")
    );
}

#[test]
fn fetch_remote_skips_prompt_for_single_remote() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_remotes_load = mock_single_remote;

    app.open_fetch_remote_prompt();

    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), Some("origin"));
            output
        }
        _ => panic!("expected fetch preview"),
    };
    assert!(!output.completed());
    assert_eq!(
        output.command_label(),
        "jj git fetch --remote exact:\"origin\""
    );
}

#[test]
fn fetch_remote_reports_no_remotes_with_readable_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_remotes_load = mock_no_remotes;

    app.open_fetch_remote_prompt();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(
        app.status.message(),
        "no git remotes found; run default fetch or add a remote before choosing one"
    );
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch output"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git remote list");
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("fetch remote selection found no remotes")
    );
}

#[test]
fn fetch_remote_reports_remote_list_errors_with_readable_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_remotes_load = mock_remotes_failure;

    app.open_fetch_remote_prompt();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(app.status.message(), "jj git remote list failed: denied");
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch output"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git remote list");
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj git remote list failed: denied")
    );
}

#[test]
fn fetch_preview_enter_runs_remote_fetch_and_keeps_result_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.open_fetch_preview("origin".to_owned());

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    assert_eq!(app.status.message(), "fetch origin: fetched origin");
    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), Some("origin"));
            output
        }
        _ => panic!("expected fetch result"),
    };
    assert!(output.completed());
    assert_eq!(
        output.command_label(),
        "jj git fetch --remote exact:\"origin\""
    );
    assert!(output.body_lines().join("\n").contains("fetched origin"));
}

#[test]
fn fetch_failure_keeps_error_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_fetch_run = mock_fetch_failure;
    app.open_fetch_preview("origin".to_owned());

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(
        app.status.message(),
        "jj git fetch --remote exact:\"origin\" failed: denied"
    );
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj git fetch --remote exact:\"origin\" failed: denied")
    );
}

#[test]
fn fetch_success_with_refresh_error_keeps_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.refresh_view = mock_refresh_failure;
    app.open_fetch_preview("origin".to_owned());

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(app.status.message(), "view refresh failed");
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("fetched origin"));
    assert!(body.contains("refresh failed: view refresh failed"));
}

#[test]
fn open_push_prompt_requires_exact_graph_revision() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), None, None),
    ])));

    assert!(!app.open_push_prompt().unwrap());
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "push from graph requires a selected row with an exact revision"
    );
}

#[test]
fn open_push_prompt_skips_remote_prompt_for_single_remote() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.git_remotes_load = mock_single_remote;

    assert!(!app.open_push_prompt().unwrap());

    let output = match &app.mode {
        InteractionMode::PushPreview { push, output } => {
            assert_eq!(push.remote(), Some("origin"));
            output
        }
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.command_label(),
        "jj git push --dry-run --remote origin --revision abcdef"
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("graph push targets exact selected revision 'abcdef' on remote origin")
    );
}

#[test]
fn open_push_prompt_keeps_remote_prompt_for_multiple_remotes() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.git_remotes_load = mock_multiple_remotes;

    assert!(!app.open_push_prompt().unwrap());

    match &app.mode {
        InteractionMode::PushRemotePrompt {
            target,
            remotes,
            selected,
        } => {
            assert_eq!(target, &JjGitPushTarget::Revision("abcdef".to_owned()));
            assert_eq!(remotes, &["origin".to_owned(), "upstream".to_owned()]);
            assert_eq!(*selected, 0);
        }
        _ => panic!("expected push remote prompt"),
    }
}

#[test]
fn open_push_prompt_reports_no_remote_error() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.git_remotes_load = mock_no_remotes;

    assert!(!app.open_push_prompt().unwrap());

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "no git remotes found; add a remote before pushing"
    );
}

#[test]
fn open_push_prompt_reports_unsupported_view_error() {
    let mut app = test_app(ViewState::OperationLog(
        crate::operation_log::OperationLogView::test_new(Vec::new()),
    ));

    assert!(!app.open_push_prompt().unwrap());

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "push is only available from graph, status, or bookmarks views"
    );
}

#[test]
fn push_preview_context_names_status_default_resolution() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));

    app.open_push_preview(JjGitPushTarget::Status, "origin".to_owned());

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("status push uses jj default target resolution for remote origin")
    );
    assert_eq!(
        output.command_label(),
        "jj git push --dry-run --remote origin"
    );
}

#[test]
fn push_preview_context_names_exact_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "feature".to_owned(),
            Some("abcdef".to_owned()),
            None,
        )]),
    ));

    app.open_push_preview(
        JjGitPushTarget::Bookmark("feature".to_owned()),
        "origin".to_owned(),
    );

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark push targets exact bookmark 'feature' on remote origin")
    );
    assert_eq!(
        output.command_label(),
        "jj git push --dry-run --remote origin --bookmark feature"
    );
}

#[test]
fn push_result_keeps_context_until_closed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_revision("abcdef".to_owned()).with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --dry-run --remote origin --revision abcdef".to_owned(),
            "preview only".to_owned(),
            Some("graph push targets exact selected revision 'abcdef' on remote origin".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push result mode"),
    };
    assert!(output.completed());
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("graph push targets exact selected revision 'abcdef' on remote origin")
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("pushed: jj git push --remote origin --revision abcdef")
    );

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
}

#[test]
fn push_preview_entering_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --remote origin --revision abcdef".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "push cancelled");
}

#[test]
fn push_confirm_success_with_refresh_error_keeps_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_revision("abcdef".to_owned()).with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --dry-run --remote origin --revision abcdef".to_owned(),
            "preview only".to_owned(),
            Some("graph push targets exact selected revision 'abcdef' on remote origin".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("pushed: jj git push --remote origin --revision abcdef"));
    assert!(body.contains("refresh failed: view refresh failed"));
    assert_eq!(app.status.message(), "view refresh failed");
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn push_preview_completion_stays_until_closed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::finished(
            "jj git push --remote origin".to_owned(),
            "pushed".to_owned(),
            Some("status push uses jj default target resolution for remote origin".to_owned()),
        ),
    };
    app.status = StatusLine::with_message(&app.view, "pushed");

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "pushed");
}
