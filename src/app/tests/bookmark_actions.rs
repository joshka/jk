//! Bookmark prompt, preview, and mutation result tests.

use super::support::*;

#[test]
fn action_output_scroll_keys_clamp_to_visible_body() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --preview --remote origin".to_owned(),
            (0..8)
                .map(|line| format!("line {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Char('j'), 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.scroll(),
        output.max_scroll(action_output_visible_lines(4))
    );

    app.handle_mode_key(crossterm::event::KeyCode::PageUp, 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Char('k'), 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Char('g'), 4)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(output.scroll(), 0);
}

#[test]
fn closing_action_output_preserves_graph_selection() {
    let mut graph = crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::Graph(graph));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --preview --remote origin".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    let ViewState::Graph(graph) = &app.view else {
        panic!("expected graph view");
    };
    assert_eq!(graph.selected_revision(), Some("second"));
    assert!(matches!(app.mode, InteractionMode::Normal));
}

#[test]
fn bookmark_create_prompt_uses_exact_graph_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "feature/name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Create);
    assert_eq!(mutation.name(), "feature/name");
    assert_eq!(
        output.command_label(),
        "jj bookmark create --revision exactly(change_id(\"change-a\"), 1) feature/name"
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("destination: exact selected revision change-a"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark create"));
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark create 'feature/name' targets change-a from jk")
    );
}

#[test]
fn bookmark_set_prompt_uses_status_current_working_copy_target() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "working copy changes:",
        "M src/app.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('='), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "feature/name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Set);
    assert_eq!(
        output.command_label(),
        "jj bookmark set --revision @ feature/name"
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("destination: current working-copy change (@)")
    );
}

#[test]
fn bookmark_move_prompt_uses_exact_pattern_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('m'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "feature/name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark preview"),
    };
    assert_eq!(
        output.command_label(),
        "jj bookmark move --to exactly(change_id(\"change-a\"), 1) exact:\"feature/name\""
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("source/current: exact pattern exact:\"feature/name\"")
    );
}

#[test]
fn bookmark_prompt_cancel_and_empty_input_do_not_open_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark create cancelled");

    app.handle_normal_key(key(KeyCode::Char('='), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark set cancelled: empty bookmark name"
    );
}

#[test]
fn bookmark_mutation_rejects_unsupported_and_inexact_contexts() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark mutation from graph requires a selected row with an exact revision"
    );

    let mut app = test_app(ViewState::OperationLog(
        crate::operation_log::OperationLogView::test_new(Vec::new()),
    ));
    app.handle_normal_key(key(KeyCode::Char('m'), KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(
        app.status.message(),
        "bookmark move is only available from graph or status views"
    );
}

#[test]
fn bookmark_delete_preview_uses_selected_exact_local_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::jj::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('x'), KeyModifiers::NONE), 12)
        .unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark delete preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Delete);
    assert_eq!(
        output.command_label(),
        "jj bookmark delete exact:\"feature/name\""
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("effect: deletes one local bookmark; this is delete, not forget"));
    assert!(body.contains("track/untrack stay disabled"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark delete"));
}

#[test]
fn bookmark_delete_rejects_nonlocal_bookmark_rows() {
    let remote = crate::jj::BookmarkItem::new(Vec::new(), "@origin".to_owned(), None, None)
        .with_local(false);
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![remote]),
    ));

    app.handle_normal_key(key(KeyCode::Char('x'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "delete requires a selected exact local bookmark"
    );
}

#[test]
fn file_list_x_is_not_bookmark_delete() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_new(vec![crate::jj::FileListItem::new(
            vec![ratatui::text::Line::from("src/lib.rs")],
            "src/lib.rs".to_owned(),
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('x'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "1 files");
}

#[test]
fn bookmark_mutation_confirm_success_failure_and_cancel_are_inspectable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::create(
            "feature/name",
            JjBookmarkTarget::exact_change("change-a"),
        ),
        output: ActionOutput::pending(
            "jj bookmark create --revision exactly(change_id(\"change-a\"), 1) feature/name"
                .to_owned(),
            "preview only".to_owned(),
            Some("bookmark create context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("bookmark create feature/name | jj undo")
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark create context")
    );
    assert_eq!(
        app.status.message(),
        "bookmark create feature/name | jj undo"
    );

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.bookmark_mutation_run = mock_bookmark_mutation_failure;
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::set(
            "feature/name",
            JjBookmarkTarget::exact_change("change-a"),
        ),
        output: ActionOutput::pending(
            "jj bookmark set --revision exactly(change_id(\"change-a\"), 1) feature/name"
                .to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj bookmark failed: first line")
    );
    assert_eq!(
        app.status.message(),
        "jj bookmark failed: first line\nsecond line"
    );

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::move_to(
            "feature/name",
            JjBookmarkTarget::exact_change("change-a"),
        ),
        output: ActionOutput::pending(
            "jj bookmark move --to exactly(change_id(\"change-a\"), 1) exact:\"feature/name\""
                .to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark move cancelled");
}
