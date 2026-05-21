//! Operation recovery, operation target, and operation-detail navigation tests.

use super::support::*;

#[test]
fn push_remote_prompt_without_selection_stays_ready() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushRemotePrompt {
        target: JjGitPushTarget::Revision("abcdef".to_owned()),
        remotes: Vec::new(),
        selected: 0,
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "no remote selected for push");
}

#[test]
fn operation_log_undo_key_opens_global_preview_without_selected_operation_id() {
    let selected_operation_id = "b".repeat(128);
    let mut operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("a".repeat(128)),
        ),
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("○  selected")],
            Some(selected_operation_id.clone()),
        ),
    ]);
    operation_log.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::OperationRecoveryPreview { recovery, output } => {
            assert_eq!(recovery.kind(), JjOperationRecoveryKind::Undo);
            output
        }
        _ => panic!("expected operation recovery preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj undo");
    assert!(body.contains("global current-repo undo from jk operation log"));
    assert!(body.contains("selected operation-log row is not an argument"));
    assert!(!body.contains(&selected_operation_id));
}

#[test]
fn operation_recovery_preview_can_cancel_or_confirm_success() {
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("a".repeat(128)),
        ),
    ]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "undo cancelled");

    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationRecoveryPreview { output, .. } => output,
        _ => panic!("expected operation recovery result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("undone operation | jj redo")
    );
    assert_eq!(app.status.message(), "undone operation | jj redo");
}

#[test]
fn operation_redo_failure_keeps_command_output_readable() {
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("a".repeat(128)),
        ),
    ]);
    let mut app = test_app(ViewState::OperationLog(operation_log));
    app.services.operation_recovery_run = mock_operation_recovery_failure;

    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::CONTROL), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationRecoveryPreview { recovery, output } => {
            assert_eq!(recovery.kind(), JjOperationRecoveryKind::Redo);
            output
        }
        _ => panic!("expected operation recovery result"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj redo");
    assert!(output.completed());
    assert!(body.contains("jj redo failed: no operation to redo available"));
    assert!(body.contains("hint: run the opposite recovery command first"));
    assert_eq!(
        app.status.message(),
        "jj redo failed: no operation to redo available\nhint: run the opposite recovery command first"
    );
}

#[test]
fn operation_action_menu_requires_exact_operation_id() {
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            None,
        ),
    ]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "operation recovery actions unavailable: selected row has no operation id"
    );
}

#[test]
fn operation_restore_preview_can_cancel_or_confirm_success() {
    let operation_id = "e".repeat(128);
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        ),
    ]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { target, output } => {
            assert_eq!(target.status_action(), "restore");
            assert_eq!(target.operation_id(), operation_id.as_str());
            output
        }
        _ => panic!("expected operation target preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        format!("jj operation restore {operation_id}")
    );
    assert!(body.contains(&format!("operation id: {operation_id}")));
    assert!(body.contains(&format!("command: jj operation restore {operation_id}")));
    assert!(body.contains(&format!(
        "confirmation: press Enter to run jj operation restore {operation_id}"
    )));

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "operation restore cancelled");

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation restore result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains(&format!("operation restore {operation_id}")));
    assert!(body.contains("new operation recorded | jj undo"));
    assert_eq!(
        app.status.message(),
        format!("operation restore {operation_id}\nnew operation recorded | jj undo")
    );
}

#[test]
fn operation_restore_confirm_refreshes_non_empty_repo_stack() {
    OPERATION_RESTORE_REFRESH_CALLS.store(0, Ordering::SeqCst);
    let operation_id = "e".repeat(128);
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        ),
    ]);
    let mut app = test_app(ViewState::OperationLog(operation_log));
    app.services.refresh_view = mock_operation_restore_counting_refresh_ok;
    app.stack
        .push(ViewState::Status(crate::status::StatusView::test_new(&[
            "Working copy changes:",
        ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    assert_eq!(OPERATION_RESTORE_REFRESH_CALLS.load(Ordering::SeqCst), 2);
    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation restore result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("new operation recorded | jj undo")
    );
    assert_eq!(
        app.status.message(),
        format!("operation restore {operation_id}\nnew operation recorded | jj undo")
    );
}

#[test]
fn operation_revert_confirm_keeps_stacked_refresh_failure_inspectable() {
    OPERATION_REVERT_REFRESH_CALLS.store(0, Ordering::SeqCst);
    let operation_id = "f".repeat(128);
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        ),
    ]);
    let mut app = test_app(ViewState::OperationLog(operation_log));
    app.services.refresh_view = mock_operation_revert_second_refresh_failure;
    app.stack
        .push(ViewState::Status(crate::status::StatusView::test_new(&[
            "Working copy changes:",
        ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Down, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    assert_eq!(OPERATION_REVERT_REFRESH_CALLS.load(Ordering::SeqCst), 2);
    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation revert result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains(&format!("operation revert {operation_id}")));
    assert!(body.contains("stacked view refresh failed: view refresh failed | jj undo"));
    assert_eq!(app.status.message(), "view refresh failed");
}

#[test]
fn operation_revert_preview_confirm_failure_keeps_output_readable() {
    let operation_id = "f".repeat(128);
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        ),
    ]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Down, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { target, output } => {
            assert_eq!(target.status_action(), "revert");
            assert_eq!(target.operation_id(), operation_id.as_str());
            output
        }
        _ => panic!("expected operation target preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        format!("jj operation revert {operation_id}")
    );
    assert!(body.contains(&format!("operation id: {operation_id}")));
    assert!(body.contains("revert exactly the selected operation by applying its inverse"));

    app.services.operation_target_run = mock_operation_target_failure;
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation revert result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains(&format!(
        "jj operation revert {operation_id} failed: first line"
    )));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        format!("jj operation revert {operation_id} failed: first line\nsecond line")
    );
}

#[test]
fn back_from_operation_detail_returns_to_operation_log() {
    let operation_id = "abcdef".to_owned();
    let operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::operation_log::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some(operation_id.clone()),
        ),
    ]);
    let detail = crate::operation_log::detail::OperationDetailView::test_new(
        ViewSpec::operation_show(operation_id),
        crate::rendered_jj::DocumentLines::new(vec![ratatui::text::Line::from(
            "operation details",
        )]),
    );
    let mut app = test_app(ViewState::OperationDetail(detail));
    app.stack.push(ViewState::OperationLog(operation_log));

    app.pop_view();

    assert!(matches!(app.view, ViewState::OperationLog(_)));
    assert_eq!(app.status.title(), "jk operation log");
    assert_eq!(app.status.message(), "1 operations");
}
