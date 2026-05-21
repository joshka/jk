//! Bookmark prompt, preview, and mutation result tests.

use super::support::*;

fn bookmark_row_with_state(
    name: &str,
    state: crate::bookmarks::BookmarkRowState,
) -> crate::bookmarks::BookmarkItem {
    crate::bookmarks::BookmarkItem::new(Vec::new(), name.to_owned(), None, None).with_state(state)
}

fn tracked_local_bookmark(name: &str) -> crate::bookmarks::BookmarkItem {
    bookmark_row_with_state(
        name,
        crate::bookmarks::BookmarkRowState::Local {
            tracking: crate::bookmarks::LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        },
    )
}

fn remote_only_bookmark(name: &str, remote: &str) -> crate::bookmarks::BookmarkItem {
    remote_bookmark_with_local_peer(
        name,
        remote,
        crate::bookmarks::BookmarkLocalPeerState::Absent,
    )
}

fn remote_bookmark_with_local_peer(
    name: &str,
    remote: &str,
    local_peer: crate::bookmarks::BookmarkLocalPeerState,
) -> crate::bookmarks::BookmarkItem {
    bookmark_row_with_state(
        name,
        crate::bookmarks::BookmarkRowState::Remote {
            remote: remote.to_owned(),
            tracking: crate::bookmarks::RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer,
        },
    )
}

fn untracked_remote_bookmark(name: &str, remote: &str) -> crate::bookmarks::BookmarkItem {
    crate::bookmarks::BookmarkItem::new(
        Vec::new(),
        name.to_owned(),
        Some("change-a".to_owned()),
        None,
    )
    .with_state(crate::bookmarks::BookmarkRowState::Remote {
        remote: remote.to_owned(),
        tracking: crate::bookmarks::RemoteBookmarkTrackingState::Untracked { synced: false },
        local_peer: crate::bookmarks::BookmarkLocalPeerState::Absent,
    })
}

fn tracked_remote_bookmark(name: &str, remote: &str) -> crate::bookmarks::BookmarkItem {
    crate::bookmarks::BookmarkItem::new(
        Vec::new(),
        name.to_owned(),
        Some("change-a".to_owned()),
        None,
    )
    .with_state(crate::bookmarks::BookmarkRowState::Remote {
        remote: remote.to_owned(),
        tracking: crate::bookmarks::RemoteBookmarkTrackingState::Tracked {
            local_present: true,
            synced: true,
        },
        local_peer: crate::bookmarks::BookmarkLocalPeerState::Unknown,
    })
}

#[test]
fn action_output_scroll_keys_clamp_to_visible_body() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
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
        crate::jj_rows::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::jj_rows::LogItem::new(Vec::new(), Some("second".to_owned()), None),
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
        crate::jj_rows::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
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
        crate::jj_rows::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
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
        crate::jj_rows::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
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
        crate::jj_rows::LogItem::new(Vec::new(), None, None),
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
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
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
    let remote = crate::bookmarks::BookmarkItem::new(Vec::new(), "@origin".to_owned(), None, None)
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
fn bookmark_forget_preview_uses_metadata_gated_selected_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![tracked_local_bookmark("feature/name")]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark forget preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Forget);
    assert_eq!(mutation.name(), "feature/name");
    assert_eq!(
        output.command_label(),
        "jj bookmark forget exact:\"feature/name\""
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("bookmark: feature/name"));
    assert!(body.contains("target: exact bookmark exact:\"feature/name\""));
    assert!(body.contains("visible state: local bookmark; tracked local bookmark"));
    assert!(body.contains("effect: forgets tracking relationship metadata"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark forget"));
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark forget 'feature/name' from jk bookmarks")
    );
}

#[test]
fn bookmark_forget_preview_uses_include_remotes_for_single_remote_only_row() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![remote_only_bookmark(
            "feature/name",
            "origin",
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark forget preview"),
    };
    assert_eq!(
        output.command_label(),
        "jj bookmark forget --include-remotes exact:\"feature/name\""
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("remote-only bookmark on origin"));
    assert!(body.contains("scope: one remote peer and no local peer; includes remotes"));
}

#[test]
fn bookmark_forget_rejects_remote_only_row_from_filtered_metadata() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![remote_bookmark_with_local_peer(
            "feature/name",
            "origin",
            crate::bookmarks::BookmarkLocalPeerState::Unknown,
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark forget disabled: selected remote bookmark has unknown local-peer metadata"
    );
}

#[test]
fn bookmark_forget_cancel_confirm_success_and_failure_are_inspectable() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![tracked_local_bookmark("feature/name")]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark forget cancelled");

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark forget result"),
    };
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("bookmark forget feature/name | jj undo")
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark forget 'feature/name' from jk bookmarks")
    );
    assert_eq!(
        app.status.message(),
        "bookmark forget feature/name | jj undo"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![tracked_local_bookmark("feature/name")]),
    ));
    app.services.bookmark_mutation_run = mock_bookmark_mutation_failure;

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark forget result"),
    };
    assert!(output.body_lines().join("\n").contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj bookmark failed: first line\nsecond line"
    );
}

#[test]
fn bookmark_forget_reports_disabled_rows_without_preview() {
    let local_only = crate::bookmarks::BookmarkItem::new(
        Vec::new(),
        "scratch".to_owned(),
        Some("change-a".to_owned()),
        None,
    )
    .with_state(crate::bookmarks::BookmarkRowState::Local {
        tracking: crate::bookmarks::LocalBookmarkRemoteState::LocalOnly,
    });
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new_with_args(
            vec![local_only],
            vec!["--all-remotes".to_owned()],
        ),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark forget disabled: selected local bookmark is local-only"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![
            remote_only_bookmark("main", "origin"),
            remote_only_bookmark("main", "upstream"),
        ]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark forget disabled: selected remote bookmark is not unique; found 2 remote peers named 'main'"
    );
}

#[test]
fn bookmark_track_preview_uses_exact_selected_remote_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![untracked_remote_bookmark(
            "feature/name",
            "origin",
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('t'), KeyModifiers::NONE), 12)
        .unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark track preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Track);
    assert_eq!(mutation.name(), "feature/name");
    assert_eq!(
        output.command_label(),
        "jj bookmark track --remote exact:\"origin\" exact:\"feature/name\""
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("local bookmark: absent"));
    assert!(body.contains("remote bookmark: feature/name"));
    assert!(body.contains("remote: origin"));
    assert!(body.contains("remote pattern: exact:\"origin\""));
    assert!(body.contains("bookmark pattern: exact:\"feature/name\""));
    assert!(body.contains("visible state: remote bookmark on origin"));
    assert!(body.contains("effect: tracks the exact remote bookmark"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark track"));
    assert!(body.contains("recovery: jj undo; review: jj op show -p"));
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark track 'feature/name' from jk bookmarks")
    );
}

#[test]
fn bookmark_untrack_preview_uses_exact_selected_remote_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![tracked_remote_bookmark(
            "feature/name",
            "origin",
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark untrack preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Untrack);
    assert_eq!(
        output.command_label(),
        "jj bookmark untrack --remote exact:\"origin\" exact:\"feature/name\""
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("remote bookmark: feature/name"));
    assert!(body.contains("remote: origin"));
    assert!(body.contains("effect: untracks the exact remote bookmark relationship"));
    assert!(body.contains("does not delete the local or remote bookmark"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark untrack"));
}

#[test]
fn bookmark_track_and_untrack_report_disabled_rows_without_preview() {
    let local_only = crate::bookmarks::BookmarkItem::new(
        Vec::new(),
        "scratch".to_owned(),
        Some("change-a".to_owned()),
        None,
    )
    .with_state(crate::bookmarks::BookmarkRowState::Local {
        tracking: crate::bookmarks::LocalBookmarkRemoteState::LocalOnly,
    });
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new_with_args(
            vec![local_only],
            vec!["--all-remotes".to_owned()],
        ),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('t'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark track disabled: selected local bookmark is local-only"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![untracked_remote_bookmark(
            "feature/name",
            "origin",
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark untrack disabled: selected remote bookmark is not tracked"
    );
}

#[test]
fn bookmark_track_confirm_success_failure_and_cancel_are_inspectable() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![untracked_remote_bookmark(
            "feature/name",
            "origin",
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('t'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark track cancelled");

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('t'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark track result"),
    };
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("bookmark track feature/name | jj undo")
    );
    assert_eq!(
        app.status.message(),
        "bookmark track feature/name | jj undo"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![untracked_remote_bookmark(
            "feature/name",
            "origin",
        )]),
    ));
    app.services.bookmark_mutation_run = mock_bookmark_mutation_failure;

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('t'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark track result"),
    };
    assert!(output.body_lines().join("\n").contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj bookmark failed: first line\nsecond line"
    );
}

#[test]
fn bookmark_delete_rename_and_forget_use_distinct_actions() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![tracked_local_bookmark("feature/name")]),
    ));

    app.handle_normal_key(key(KeyCode::Char('x'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(matches!(
        app.mode,
        InteractionMode::BookmarkMutationPreview { ref mutation, .. }
            if mutation.kind() == JjBookmarkMutationKind::Delete
    ));

    app.mode = InteractionMode::Normal;
    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(matches!(
        app.mode,
        InteractionMode::BookmarkRenamePrompt { .. }
    ));

    app.mode = InteractionMode::Normal;
    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(matches!(
        app.mode,
        InteractionMode::BookmarkMutationPreview { ref mutation, .. }
            if mutation.kind() == JjBookmarkMutationKind::Forget
    ));
}

#[test]
fn bookmark_rename_prompt_uses_selected_exact_local_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(matches!(
        app.mode,
        InteractionMode::BookmarkRenamePrompt { ref old_name, .. }
            if old_name == "feature/name"
    ));

    for character in "feature/renamed".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark rename preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Rename);
    assert_eq!(mutation.name(), "feature/name");
    assert_eq!(mutation.new_name(), Some("feature/renamed"));
    assert_eq!(
        output.command_label(),
        "jj bookmark rename feature/name feature/renamed"
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("old name: feature/name"));
    assert!(body.contains("new name: feature/renamed"));
    assert!(body.contains("without --overwrite-existing"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark rename"));
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark rename 'feature/name' to 'feature/renamed' from jk bookmarks")
    );
}

#[test]
fn bookmark_rename_prompt_rejects_whitespace_wrapped_input_before_preview() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in " feature/renamed ".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark rename cancelled: bookmark name must not contain whitespace or control characters"
    );
}

#[test]
fn bookmark_rename_prompt_cancel_and_invalid_inputs_do_not_open_preview() {
    let view = || {
        ViewState::Bookmarks(crate::bookmarks::BookmarksView::test_new(vec![
            crate::bookmarks::BookmarkItem::new(
                Vec::new(),
                "feature/name".to_owned(),
                Some("change-a".to_owned()),
                None,
            ),
        ]))
    };

    let mut app = test_app(view());
    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark rename cancelled");

    let mut app = test_app(view());
    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark rename cancelled: empty bookmark name"
    );

    let mut app = test_app(view());
    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "feature/name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark rename cancelled: new bookmark name is unchanged"
    );

    let mut app = test_app(view());
    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "bad name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark rename cancelled: bookmark name must not contain whitespace or control characters"
    );
}

#[test]
fn bookmark_rename_rejects_nonlocal_bookmark_rows() {
    let remote = crate::bookmarks::BookmarkItem::new(Vec::new(), "@origin".to_owned(), None, None)
        .with_state(crate::bookmarks::BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: crate::bookmarks::RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: crate::bookmarks::BookmarkLocalPeerState::Unknown,
        });
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![remote]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "rename requires a selected exact local bookmark"
    );

    let unknown = crate::bookmarks::BookmarkItem::new(
        Vec::new(),
        "maybe-local".to_owned(),
        Some("change-a".to_owned()),
        None,
    )
    .with_state(crate::bookmarks::BookmarkRowState::Unknown);
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![unknown]),
    ));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "rename requires a selected exact local bookmark"
    );
}

#[test]
fn bookmark_rename_confirm_success_failure_and_cancel_are_inspectable() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::rename("feature/name", "feature/renamed"),
        output: ActionOutput::pending(
            "jj bookmark rename feature/name feature/renamed".to_owned(),
            "preview only".to_owned(),
            Some("bookmark rename context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark rename result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("bookmark rename feature/name -> feature/renamed | jj undo")
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark rename context")
    );
    assert_eq!(
        app.status.message(),
        "bookmark rename feature/name -> feature/renamed | jj undo"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));
    app.services.bookmark_mutation_run = mock_bookmark_mutation_failure;
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::rename("feature/name", "main"),
        output: ActionOutput::pending(
            "jj bookmark rename feature/name main".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark rename result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj bookmark failed: first line")
    );
    assert!(output.body_lines().join("\n").contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj bookmark failed: first line\nsecond line"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::rename("feature/name", "feature/renamed"),
        output: ActionOutput::pending(
            "jj bookmark rename feature/name feature/renamed".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark rename cancelled");
}

#[test]
fn bookmark_rename_confirm_duplicate_name_failure_preserves_error_output() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));
    app.services.bookmark_mutation_run = mock_bookmark_mutation_duplicate_name_failure;
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::rename("feature/name", "feature/renamed"),
        output: ActionOutput::pending(
            "jj bookmark rename feature/name feature/renamed".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark rename result"),
    };
    let body = output.body_lines().join("\n");
    assert!(body.contains("Error: Bookmark already exists: feature/renamed"));
    assert_eq!(
        app.status.message(),
        "Error: Bookmark already exists: feature/renamed"
    );
}

#[test]
fn file_list_x_is_not_bookmark_delete() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_new(vec![crate::jj_rows::FileListItem::new(
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
        crate::jj_rows::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
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
        crate::jj_rows::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
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
        crate::jj_rows::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
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
