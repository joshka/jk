//! Rebase, squash, and absorb preview/result tests.

use super::support::*;

#[test]
fn rebase_preview_entering_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            Some("rebase preview context".to_owned()),
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "rebase cancelled");
}

#[test]
fn rebase_preview_completion_stays_until_closed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::finished(
            "jj rebase -r source-a -o dest".to_owned(),
            "rebased".to_owned(),
            None,
        ),
    };
    app.status = StatusLine::with_message(&app.view, "rebased");

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "rebased");
}

#[test]
fn rebase_confirm_success_with_reveal_failure_stays_completed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.rebase_run = mock_rebase_success;
    app.services.refresh_view = mock_refresh_ok;
    app.services.reveal_graph_change = mock_reveal_graph_change_error;
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            Some("rebase preview context".to_owned()),
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );

    let output = match app.mode {
        InteractionMode::RebasePreview { ref output, .. } => output,
        _ => panic!("expected rebase preview mode"),
    };
    assert_eq!(output.command_label(), "jj rebase -r source-a -o dest");
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("rebase preview context")
    );
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("reveal failed: refreshed graph did not include the new working-copy change")
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj undo | jj op show -p")
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(
        app.status.message(),
        "refreshed graph did not include the new working-copy change"
    );

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "refreshed graph did not include the new working-copy change"
    );
}

#[test]
fn rebase_confirm_success_keeps_review_and_undo_paths_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("source-a".to_owned()), None),
    ])));
    app.services.rebase_run = mock_rebase_success;
    app.services.refresh_view = mock_refresh_ok;
    app.services.reveal_graph_change = mock_reveal_rebased_source_in_recent;
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            Some("rebase preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::RebasePreview { output, .. } => output,
        _ => panic!("expected rebase result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("rebased | showing recent work | jj undo | jj op show -p"));
    assert_eq!(
        app.status.message(),
        "rebased | showing recent work | jj undo | jj op show -p"
    );
}

#[test]
fn rebase_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("source-a".to_owned()), None),
    ])));
    app.services.rebase_run = mock_rebase_failure;
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::RebasePreview { output, .. } => output,
        _ => panic!("expected rebase result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj rebase -r source-a -o dest");
    assert!(output.completed());
    assert!(body.contains("jj rebase failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj rebase failed: first line\nsecond line"
    );
}

#[test]
fn rebase_role_prompt_enters_preview_with_explicit_plan() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "source-a".to_owned()),
            RolePromptOption::new("destination", "dest".to_owned()),
        ],
        "Preview required before execution.",
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RolePrompt {
        action: ActionKind::Rebase,
        prompt,
        selected: 0,
    };

    let result = app.handle_mode_key(crossterm::event::KeyCode::Enter, 12);
    assert!(result.is_ok());
    let (command_label, status_context, preview_output) = match app.mode {
        InteractionMode::RebasePreview { ref output, .. } => (
            output.command_label().to_owned(),
            output.status_context().cloned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected rebase preview mode"),
    };
    assert_eq!(command_label, "jj rebase -r source-a -o dest");
    assert_eq!(
        status_context.as_deref(),
        Some("rebase from 1 source(s) into dest from jk | source(s): source-a")
    );
    assert!(preview_output.contains("command: jj rebase -r source-a -o dest"));
    assert!(preview_output.contains("source revision: source-a"));
    assert!(preview_output.contains("destination revision: dest"));
    assert!(preview_output.contains("current graph context:"));
    assert!(preview_output.contains("source rows are selected in jk"));
    assert!(preview_output.contains("destination is the current row"));
    assert!(preview_output.contains("expected jj effect:"));
    assert!(
        preview_output.contains("semantics: jj rebase --revision <source> --onto <destination>")
    );
    assert!(preview_output.contains("not a graph preview"));
    assert!(preview_output.contains("review after run: jj op show -p"));
    assert!(preview_output.contains("undo path: jj undo"));
    assert!(preview_output.contains("confirmation: press Enter to run jj rebase"));
}

#[test]
fn squash_role_prompt_enters_preview_with_explicit_plan() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "source-a".to_owned()),
            RolePromptOption::new("source", "source-b".to_owned()),
            RolePromptOption::new("destination", "dest".to_owned()),
        ],
        "Preview required before execution.",
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RolePrompt {
        action: ActionKind::Squash,
        prompt,
        selected: 0,
    };

    let result = app.handle_mode_key(crossterm::event::KeyCode::Enter, 12);
    assert!(result.is_ok());
    let (command_label, status_context, preview_output) = match app.mode {
        InteractionMode::SquashPreview { ref output, .. } => (
            output.command_label().to_owned(),
            output.status_context().cloned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected squash preview mode"),
    };
    assert_eq!(
        command_label,
        "jj squash --from source-a --from source-b --into dest --use-destination-message"
    );
    assert_eq!(
        status_context.as_deref(),
        Some("squash from 2 source(s) into dest from jk | source(s): source-a, source-b")
    );
    assert!(preview_output.contains("source: source-a"));
    assert!(preview_output.contains("source: source-b"));
    assert!(preview_output.contains("destination: dest"));
    assert!(preview_output.contains("--use-destination-message keeps the destination description"));
    assert!(preview_output.contains("confirmation: press Enter to run jj squash"));
    assert!(preview_output.contains("undo path: jj undo"));
}

#[test]
fn squash_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            Some("squash preview context".to_owned()),
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "squash cancelled");
}

#[test]
fn squash_confirm_success_refreshes_and_reveals_destination() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.reveal_graph_change = mock_reveal_squash_destination_in_recent;
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            Some("squash preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::SquashPreview { output, .. } => output,
        _ => panic!("expected squash result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        "jj squash --from source-a --into dest --use-destination-message"
    );
    assert!(output.completed());
    assert!(body.contains("squashed | showing recent work | jj undo"));
    assert_eq!(
        app.status.message(),
        "squashed | showing recent work | jj undo"
    );
}

#[test]
fn squash_confirm_refresh_failure_keeps_undo_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            Some("squash preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::SquashPreview { output, .. } => output,
        _ => panic!("expected squash result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("squashed | refresh failed: view refresh failed | jj undo"));
    assert_eq!(app.status.message(), "view refresh failed");
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn squash_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.squash_run = mock_squash_failure;
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::SquashPreview { output, .. } => output,
        _ => panic!("expected squash result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        "jj squash --from source-a --into dest --use-destination-message"
    );
    assert!(output.completed());
    assert!(body.contains("jj squash failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj squash failed: first line\nsecond line"
    );
}

#[test]
fn absorb_action_menu_enter_opens_preview_with_current_source_and_candidates() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("source".to_owned()), None),
        crate::jj_rows::LogItem::new(Vec::new(), Some("dest-a".to_owned()), None),
        crate::jj_rows::LogItem::new(Vec::new(), Some("dest-b".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("source")
                .with_sources(["source", "dest-a", "dest-b"]),
        ),
        selected: 3,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (source, destinations, command_label, body) = match &app.mode {
        InteractionMode::AbsorbPreview { absorb, output } => (
            absorb.source().to_owned(),
            absorb.destinations().to_vec(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected absorb preview mode"),
    };
    assert_eq!(source, "source");
    assert_eq!(destinations, ["dest-a", "dest-b"]);
    assert_eq!(
        command_label,
        "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1) --into exactly(change_id(\"dest-b\"), 1)"
    );
    assert!(body.contains("source: source"));
    assert!(body.contains("candidate destination: dest-a"));
    assert!(body.contains("candidate destination: dest-b"));
    assert!(body.contains("only considers selected revisions that are ancestors"));
    assert!(body.contains("jj op show -p"));
}

#[test]
fn absorb_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));
    app.mode = InteractionMode::AbsorbPreview {
        absorb: JjAbsorbPlan::new("source", vec!["dest-a".to_owned()]),
        output: ActionOutput::pending(
            "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("absorb preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "absorb cancelled");
}

#[test]
fn absorb_confirm_success_keeps_undo_and_operation_review_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));
    app.mode = InteractionMode::AbsorbPreview {
        absorb: JjAbsorbPlan::new("source", vec!["dest-a".to_owned()]),
        output: ActionOutput::pending(
            "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("absorb preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbsorbPreview { output, .. } => output,
        _ => panic!("expected absorb result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("absorbed | jj undo | jj op show -p"));
    assert_eq!(app.status.message(), "absorbed | jj undo | jj op show -p");
}

#[test]
fn absorb_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));
    app.services.absorb_run = mock_absorb_failure;
    app.mode = InteractionMode::AbsorbPreview {
        absorb: JjAbsorbPlan::new("source", vec!["dest-a".to_owned()]),
        output: ActionOutput::pending(
            "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbsorbPreview { output, .. } => output,
        _ => panic!("expected absorb result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj absorb failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj absorb failed: first line\nsecond line"
    );
}

#[test]
fn absorb_without_candidates_returns_clear_status() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj_rows::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));

    app.open_absorb_preview(JjAbsorbPlan::new("source", Vec::new()));

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "absorb requires at least one selected exact candidate destination"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}
