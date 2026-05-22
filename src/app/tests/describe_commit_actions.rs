//! Describe and commit prompt, preview, and result tests.

use super::support::*;

#[test]
fn describe_prompt_types_backspaces_and_opens_preview_for_exact_log_target() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "Mesx".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Backspace, 12).unwrap();
    app.handle_mode_key(KeyCode::Char('g'), 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (describe, output) = match &app.mode {
        InteractionMode::DescribePreview { describe, output } => (describe, output),
        _ => panic!("expected describe preview"),
    };
    assert_eq!(
        describe.target(),
        &JjDescribeTarget::ExactChange("change-a".to_owned())
    );
    assert_eq!(
        output.command_label(),
        "jj describe change-a --message Mesg"
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("target: exact selected revision change-a"));
    assert!(body.contains("message: Mesg"));
    assert!(body.contains("without opening an editor"));
}

#[test]
fn describe_prompt_types_and_opens_preview_for_status_target() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "working copy changes:",
        "M src/app.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "Message".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (describe, output) = match &app.mode {
        InteractionMode::DescribePreview { describe, output } => (describe, output),
        _ => panic!("expected describe preview"),
    };
    assert_eq!(describe.target(), &JjDescribeTarget::CurrentWorkingCopy);
    assert_eq!(output.command_label(), "jj describe @ --message Message");
    let body = output.body_lines().join("\n");
    assert!(body.contains("target: current working-copy change (@)"));
    assert!(body.contains("message: Message"));
    assert!(body.contains("without opening an editor"));
}

#[test]
fn describe_prompt_cancel_and_empty_input_do_not_open_preview() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "describe cancelled");

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "describe cancelled: empty description"
    );
}

#[test]
fn describe_requires_exact_log_target_and_rejects_unsupported_context() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "describe from log requires a selected row with an exact revision"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "main".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(
        app.status.message(),
        "describe is only available from log or status views"
    );
}

#[test]
fn describe_confirm_success_refreshes_reveals_and_keeps_undo_visible() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.reveal_log_change = mock_reveal_described_change_in_recent;
    app.mode = InteractionMode::DescribePreview {
        describe: JjDescribePlan::new(
            JjDescribeTarget::exact_change("change-a"),
            "New description",
        ),
        output: ActionPane::pending(
            "jj describe change-a --message New description".to_owned(),
            "preview only".to_owned(),
            Some("describe change-a from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DescribePreview { output, .. } => output,
        _ => panic!("expected describe result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("described change-a | jj undo")
    );
    assert_eq!(app.status.message(), "described change-a | jj undo");
}

#[test]
fn describe_failure_and_refresh_failure_remain_inspectable() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.describe_run = mock_describe_failure;
    app.mode = InteractionMode::DescribePreview {
        describe: JjDescribePlan::new(JjDescribeTarget::exact_change("change-a"), "New"),
        output: ActionPane::pending(
            "jj describe change-a --message New".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DescribePreview { output, .. } => output,
        _ => panic!("expected describe result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj describe failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj describe failed: first line\nsecond line"
    );

    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::DescribePreview {
        describe: JjDescribePlan::new(JjDescribeTarget::exact_change("change-a"), "New"),
        output: ActionPane::pending(
            "jj describe change-a --message New".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DescribePreview { output, .. } => output,
        _ => panic!("expected describe result"),
    };
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("described change-a | refresh failed: view refresh failed | jj undo")
    );
    assert_eq!(app.status.message(), "view refresh failed");
}

#[test]
fn commit_prompt_is_honest_about_current_working_copy_target() {
    let mut graph = crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("historical".to_owned()), None),
        crate::log::LogItem::new(Vec::new(), Some("selected-row".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::Log(graph));

    app.handle_normal_key(key(KeyCode::Char('C'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "Commitx".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Backspace, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj commit --message Commit");
    assert!(body.contains("target: current working-copy change (@)"));
    assert!(body.contains("selected log rows are not arguments"));
    assert!(!body.contains("selected-row"));
}

#[test]
fn commit_prompt_cancel_and_empty_input_do_not_open_preview() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('C'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "commit cancelled");

    app.handle_normal_key(key(KeyCode::Char('C'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "commit cancelled: empty description");
}

#[test]
fn commit_confirm_success_and_failure_keep_output_readable() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::CommitPreview {
        commit: JjCommitPlan::new("Commit"),
        output: ActionPane::pending(
            "jj commit --message Commit".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(
        body.contains("committed working copy | new working-copy change created on top | jj undo")
    );
    assert_eq!(
        app.status.message(),
        "committed working copy | new working-copy change created on top | jj undo"
    );

    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.commit_run = mock_commit_failure;
    app.mode = InteractionMode::CommitPreview {
        commit: JjCommitPlan::new("Commit"),
        output: ActionPane::pending(
            "jj commit --message Commit".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj commit failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj commit failed: first line\nsecond line"
    );
}

#[test]
fn commit_refresh_failure_keeps_undo_and_new_working_copy_effect_visible() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::CommitPreview {
        commit: JjCommitPlan::new("Commit"),
        output: ActionPane::pending(
            "jj commit --message Commit".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit result"),
    };
    assert!(output.body_lines().join("\n").contains(
        "committed working copy | refresh failed: view refresh failed | new working-copy change created on top | jj undo"
    ));
    assert_eq!(app.status.message(), "view refresh failed");
}
