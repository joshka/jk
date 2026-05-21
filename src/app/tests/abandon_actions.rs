//! Abandon preview, confirmation, cancellation, and result tests.

use super::support::*;

#[test]
fn abandon_action_menu_enter_opens_preview_with_exact_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_preview_load = mock_non_empty_abandon_preview;
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        ),
        selected: 3,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (revision, command_label, body) = match &app.mode {
        InteractionMode::AbandonPreview {
            abandon, output, ..
        } => (
            abandon.revision().to_owned(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected abandon preview mode"),
    };
    assert_eq!(revision, "change-a");
    assert_eq!(command_label, "jj abandon change-a");
    assert!(body.contains("change: change-a"));
    assert!(body.contains("title: Edit change"));
}

#[test]
fn empty_abandon_preview_enter_runs_and_keeps_undo_visible() {
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            Some("abandon exact revision change-a from jk".to_owned()),
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("abandoned | jj undo")
    );
    assert_eq!(app.status.message(), "abandoned | jj undo");
}

#[test]
fn empty_abandon_rechecks_before_running_and_requires_confirmation_after_drift() {
    ABANDON_DRIFT_RECHECK_CALLS.store(1, Ordering::SeqCst);
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_preview_load = mock_abandon_preview_drifts_to_non_empty;
    app.services.abandon_run = panic_abandon_run;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            Some("abandon exact revision change-a from jk".to_owned()),
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (input, body) = match &app.mode {
        InteractionMode::AbandonConfirm { input, output, .. } => {
            (input.as_str(), output.body_lines().join("\n"))
        }
        _ => panic!("expected abandon confirmation after recheck drift"),
    };
    assert_eq!(input, "");
    assert!(body.contains("change is no longer empty"));
    assert!(body.contains("M src/main.rs"));
    assert_eq!(
        app.status.message(),
        "change is no longer empty; type exact revision to confirm abandon"
    );

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert_eq!(
        app.status.message(),
        "confirmation did not match; abandon not run"
    );

    app.services.abandon_run = mock_abandon_success;
    for character in "change-a".chars() {
        app.handle_mode_key(crossterm::event::KeyCode::Char(character), 12)
            .unwrap();
    }
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("abandoned | jj undo")
    );
}

#[test]
fn empty_abandon_recheck_failure_stays_readable_without_running() {
    ABANDON_FAILED_RECHECK_CALLS.store(1, Ordering::SeqCst);
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_preview_load = mock_abandon_preview_recheck_failure;
    app.services.abandon_run = panic_abandon_run;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            None,
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected readable abandon recheck failure"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj diff -r change-a --summary failed: disappeared"));
    assert_eq!(
        app.status.message(),
        "jj diff -r change-a --summary failed: disappeared"
    );
}

#[test]
fn non_empty_abandon_requires_exact_typed_revision() {
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Edit change".to_owned()),
        "M src/main.rs\n".to_owned(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_run = panic_abandon_run;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            Some("abandon exact revision change-a from jk".to_owned()),
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert!(matches!(app.mode, InteractionMode::AbandonConfirm { .. }));

    app.handle_mode_key(crossterm::event::KeyCode::Char('x'), 12)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert_eq!(
        app.status.message(),
        "confirmation did not match; abandon not run"
    );

    app.services.abandon_run = mock_abandon_success;
    app.handle_mode_key(crossterm::event::KeyCode::Backspace, 12)
        .unwrap();
    for character in "change-a".chars() {
        app.handle_mode_key(crossterm::event::KeyCode::Char(character), 12)
            .unwrap();
    }
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("abandoned | jj undo")
    );
}

#[test]
fn abandon_cancel_restores_normal_mode_and_selection() {
    let mut graph = crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::graph::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let preview = JjAbandonPreview::new("second".to_owned(), None, String::new());
    let mut app = test_app(ViewState::Graph(graph));
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("second"),
        output: ActionOutput::pending("jj abandon second".to_owned(), preview.preview_text(), None),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    let ViewState::Graph(graph) = &app.view else {
        panic!("expected graph view");
    };
    assert_eq!(graph.selected_revision(), Some("second"));
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "abandon cancelled");
}

#[test]
fn abandon_failure_keeps_full_error_output_readable() {
    let preview = JjAbandonPreview::new("change-a".to_owned(), None, String::new());
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::graph::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_run = mock_abandon_failure;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            None,
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj abandon change-a failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj abandon change-a failed: first line\nsecond line"
    );
}
