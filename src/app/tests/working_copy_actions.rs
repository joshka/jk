//! New, duplicate, split, and working-copy navigation action tests.

use super::support::*;

#[test]
fn rebase_plan_from_prompt_respects_explicit_roles() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "bbbbbbbb1111111111111111111111111111111111"),
            RolePromptOption::new("destination", "cccccccc2222222222222222222222222222222222"),
            RolePromptOption::new("source", "aaaaaaaa3333333333333333333333333333333333"),
        ],
        "Preview required before execution.",
    );

    let rebase =
        rebase_plan_from_prompt(&prompt).expect("role prompt should include a destination");

    assert_eq!(
        rebase.sources(),
        &[
            "bbbbbbbb1111111111111111111111111111111111",
            "aaaaaaaa3333333333333333333333333333333333"
        ]
    );
    assert_eq!(
        rebase.destination(),
        "cccccccc2222222222222222222222222222222222"
    );
}

#[test]
fn squash_plan_from_prompt_respects_explicit_roles() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "bbbbbbbb1111111111111111111111111111111111"),
            RolePromptOption::new("destination", "cccccccc2222222222222222222222222222222222"),
            RolePromptOption::new("source", "aaaaaaaa3333333333333333333333333333333333"),
        ],
        "Preview required before execution.",
    );

    let squash =
        squash_plan_from_prompt(&prompt).expect("role prompt should include a destination");

    assert_eq!(
        squash.sources(),
        &[
            "bbbbbbbb1111111111111111111111111111111111",
            "aaaaaaaa3333333333333333333333333333333333"
        ]
    );
    assert_eq!(
        squash.destination(),
        "cccccccc2222222222222222222222222222222222"
    );
}

#[test]
fn new_action_menu_enter_opens_preview_with_exact_parents() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::menus::build_action_menu(
            &crate::menus::ExactActionContext::with_current("current")
                .with_sources(["parent-a", "parent-b"]),
        ),
        selected: 0,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (parents, command_label, body) = match &app.mode {
        InteractionMode::NewPreview { new_change, output } => (
            new_change.parents().to_vec(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected new preview mode"),
    };
    assert_eq!(parents, ["parent-a", "parent-b"]);
    assert_eq!(command_label, "jj new parent-a parent-b");
    assert!(body.contains("parent: parent-a"));
    assert!(body.contains("parent: parent-b"));
    assert!(body.contains("undo path: jj undo"));
}

#[test]
fn action_menu_shortcut_opens_item_without_moving_selection() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::menus::build_action_menu(
            &crate::menus::ExactActionContext::with_current("current")
                .with_sources(["parent-a", "parent-b"]),
        ),
        selected: 3,
    };

    app.handle_mode_key(KeyCode::Char('n'), 12).unwrap();

    let parents = match &app.mode {
        InteractionMode::NewPreview { new_change, .. } => new_change.parents().to_vec(),
        _ => panic!("expected new preview mode"),
    };
    assert_eq!(parents, ["parent-a", "parent-b"]);
}

#[test]
fn action_menu_close_key_preserves_normal_context() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::menus::build_action_menu(&crate::menus::ExactActionContext::with_current(
            "change-a",
        )),
        selected: 4,
    };

    app.handle_mode_key(KeyCode::Char('q'), 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.graph_selected_revision().as_deref(), Some("change-a"));
}

#[test]
fn edit_action_menu_enter_opens_preview_with_exact_target() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::menus::build_action_menu(&crate::menus::ExactActionContext::with_current(
            "change-a",
        )),
        selected: 0,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (navigation, command_label, body) = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { navigation, output } => (
            navigation.clone(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected working-copy navigation preview"),
    };
    assert_eq!(navigation.kind(), JjWorkingCopyNavigationKind::Edit);
    assert_eq!(navigation.target_change_id(), Some("change-a"));
    assert_eq!(command_label, "jj edit exactly(change_id(\"change-a\"), 1)");
    assert!(body.contains("target: exact selected log revision change-a"));
    assert!(body.contains("moves @ to edit that revision directly"));
}

#[test]
fn edit_direct_key_requires_exact_selected_log_row() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('e'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "edit from log requires a selected row with an exact revision"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn next_direct_key_opens_preview_without_selected_row_targeting() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char(']'), KeyModifiers::NONE), 12)
        .unwrap();

    let (navigation, command_label, body) = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { navigation, output } => (
            navigation.clone(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected working-copy navigation preview"),
    };
    assert_eq!(navigation.kind(), JjWorkingCopyNavigationKind::Next);
    assert_eq!(command_label, "jj next --edit");
    assert!(body.contains("highlighted log row is not an argument to jj next --edit"));
    assert!(body.contains("runs jj topology movement relative to @"));
}

#[test]
fn working_copy_navigation_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::edit("change-a"),
        output: ActionPane::pending(
            "jj edit exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("edit preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "edit cancelled");
}

#[test]
fn edit_confirm_success_refreshes_and_reveals_target() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.reveal_log_change = mock_reveal_edit_target_in_recent;
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::edit("change-a"),
        output: ActionPane::pending(
            "jj edit exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("edit preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { output, .. } => output,
        _ => panic!("expected working-copy navigation result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("editing change-a | jj undo"));
    assert_eq!(app.status.message(), "editing change-a | jj undo");
}

#[test]
fn split_action_menu_enter_opens_exact_target_preview() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(
            vec![ratatui::text::Line::from("○  change")],
            Some("change-a".to_owned()),
            None,
        ),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::menus::build_action_menu(&crate::menus::ExactActionContext::with_current(
            "change-a",
        )),
        selected: 2,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (target, command_label, body) = match &app.mode {
        InteractionMode::SplitPreview { split, output } => (
            split.target().exact_change_id().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected split preview"),
    };
    assert_eq!(target.as_deref(), Some("change-a"));
    assert_eq!(
        command_label,
        "jj split --revision exactly(change_id(\"change-a\"), 1)"
    );
    assert!(body.contains("target: exact selected log revision change-a"));
    assert!(body.contains("jj's diff editor"));
    assert!(body.contains("jk is not an in-app patch editor"));
}

#[test]
fn duplicate_action_menu_enter_opens_exact_source_preview() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(
            vec![ratatui::text::Line::from("○  change")],
            Some("change-a".to_owned()),
            None,
        ),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::menus::build_action_menu(&crate::menus::ExactActionContext::with_current(
            "change-a",
        )),
        selected: 4,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (source, command_label, body) = match &app.mode {
        InteractionMode::DuplicatePreview { duplicate, output } => (
            duplicate.source().to_owned(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected duplicate preview"),
    };
    assert_eq!(source, "change-a");
    assert_eq!(
        command_label,
        "jj duplicate exactly(change_id(\"change-a\"), 1)"
    );
    assert!(body.contains("source revision: change-a"));
    assert!(body.contains("source count: 1 exact selected change"));
    assert!(body.contains("does not parse duplicate output for the new change id"));
    assert!(body.contains("recovery: jj undo"));
}

#[test]
fn duplicate_preview_cancel_preserves_log_selection() {
    let mut graph = crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::log::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            size: ratatui::layout::Size {
                height: 12,
                width: 80,
            },
            search: None,
        },
    );
    let mut app = test_app(ViewState::Log(graph));
    app.mode = InteractionMode::DuplicatePreview {
        duplicate: JjDuplicatePlan::exact_change("second"),
        output: ActionPane::pending(
            "jj duplicate exactly(change_id(\"second\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("duplicate preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    let ViewState::Log(graph) = &app.view else {
        panic!("expected log view");
    };
    assert_eq!(graph.selected_revision(), Some("second"));
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "duplicate cancelled");
}

#[test]
fn duplicate_confirm_success_refreshes_and_uses_recent_source_fallback() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.reveal_log_change = mock_reveal_duplicate_source_in_recent;
    app.mode = InteractionMode::DuplicatePreview {
        duplicate: JjDuplicatePlan::exact_change("change-a"),
        output: ActionPane::pending(
            "jj duplicate exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("duplicate exact source change-a from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DuplicatePreview { output, .. } => output,
        _ => panic!("expected duplicate result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("Duplicated source change-a"));
    assert!(body.contains("refresh: active view refreshed"));
    assert!(body.contains(
        "reveal: selected original source change-a because jk does not parse duplicate output"
    ));
    assert_eq!(
        app.status.message(),
        "duplicate completed | showing recent work fallback for source | jj undo | jj op show -p"
    );
}

#[test]
fn duplicate_confirm_success_from_exact_detail_view_refreshes_without_graph_reveal() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default).with_exact_change_target(),
    )));
    app.services.reveal_log_change = mock_reveal_log_change_unexpected;
    app.mode = InteractionMode::DuplicatePreview {
        duplicate: JjDuplicatePlan::exact_change("change-a"),
        output: ActionPane::pending(
            "jj duplicate exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("duplicate exact source change-a from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DuplicatePreview { output, .. } => output,
        _ => panic!("expected duplicate result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("Duplicated source change-a"));
    assert!(body.contains("refresh: active view refreshed"));
    assert!(body.contains(
        "reveal: source fallback not attempted because the active view cannot reveal log changes"
    ));
    assert!(body.contains("recovery: jj undo"));
    assert!(body.contains("review: jj op show -p"));
    assert_eq!(
        app.status.message(),
        "duplicate completed | active view refreshed | source reveal unavailable | jj undo | jj op show -p"
    );
}

#[test]
fn duplicate_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.duplicate_run = mock_duplicate_failure;
    app.mode = InteractionMode::DuplicatePreview {
        duplicate: JjDuplicatePlan::exact_change("change-a"),
        output: ActionPane::pending(
            "jj duplicate exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DuplicatePreview { output, .. } => output,
        _ => panic!("expected duplicate result"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        "jj duplicate exactly(change_id(\"change-a\"), 1)"
    );
    assert!(output.completed());
    assert!(body.contains("jj duplicate exactly(change_id(\"change-a\"), 1) failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj duplicate exactly(change_id(\"change-a\"), 1) failed: first line\nsecond line"
    );
}

#[test]
fn split_visible_working_copy_uses_bare_command() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("current-change".to_owned()),
            None,
        ),
    ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('s'), 12).unwrap();

    let (target, command_label, body) = match &app.mode {
        InteractionMode::SplitPreview { split, output } => (
            split.target().exact_change_id().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected split preview"),
    };
    assert_eq!(target, None);
    assert_eq!(command_label, "jj split");
    assert!(body.contains("target: current working-copy change (@)"));
    assert!(body.contains("fileset: no fileset is passed"));
}

#[test]
fn split_preview_cancel_preserves_log_selection() {
    let mut graph = crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::log::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            size: ratatui::layout::Size {
                height: 12,
                width: 80,
            },
            search: None,
        },
    );
    let mut app = test_app(ViewState::Log(graph));
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::exact_change("second"),
        output: ActionPane::pending(
            "jj split --revision exactly(change_id(\"second\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("split preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    let ViewState::Log(graph) = &app.view else {
        panic!("expected log view");
    };
    assert_eq!(graph.selected_revision(), Some("second"));
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "split cancelled");
}

#[test]
fn split_confirm_success_refreshes_reveals_and_keeps_recovery_visible() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.reveal_log_change = mock_reveal_edit_target_in_recent;
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::exact_change("change-a"),
        output: ActionPane::pending(
            "jj split --revision exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("split exact log revision change-a from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::SplitPreview { output, .. } => output,
        _ => panic!("expected split result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("child exit status: exit status: 0"));
    assert!(body.contains("did not capture that output"));
    assert!(body.contains("refresh: split completed | jj undo | jj op show -p"));
    assert_eq!(
        app.status.message(),
        "split completed | jj undo | jj op show -p"
    );
}

#[test]
fn split_current_confirm_success_reveals_current_working_copy_when_possible() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("current-change".to_owned()), None),
    ])));
    app.services.reveal_log_change = mock_reveal_current_working_copy_in_recent;
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::current_working_copy(),
        output: ActionPane::pending(
            "jj split".to_owned(),
            "preview only".to_owned(),
            Some("split current working-copy change (@) from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::SplitPreview { output, .. } => output,
        _ => panic!("expected split result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(
        body.contains("refresh: split completed | showing recent work | jj undo | jj op show -p")
    );
    assert_eq!(
        app.status.message(),
        "split completed | showing recent work | jj undo | jj op show -p"
    );
}

#[test]
fn split_failure_keeps_app_owned_result_without_claiming_captured_stderr() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("current-change".to_owned()), None),
    ])));
    app.services.split_run = mock_split_failure_service;
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::current_working_copy(),
        output: ActionPane::pending("jj split".to_owned(), "preview only".to_owned(), None),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::SplitPreview { output, .. } => output,
        _ => panic!("expected split result"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj split");
    assert!(output.completed());
    assert!(body.contains("result: split command failed or did not complete"));
    assert!(body.contains("runner status: jj split failed with status exit status: 1"));
    assert!(body.contains("did not capture stderr"));
    assert!(body.contains("if jj recorded an operation, use jj undo"));
    assert!(body.contains("review: jj op show -p"));
    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert!(
        app.status
            .message()
            .contains("runner status: jj split failed")
    );
}

#[test]
fn prev_confirm_success_resolves_current_working_copy_and_reveals_recent() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), None, None),
    ])));
    app.services.reveal_log_change = mock_reveal_current_working_copy_in_recent;
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::prev(),
        output: ActionPane::pending(
            "jj prev --edit".to_owned(),
            "preview only".to_owned(),
            Some("prev preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { output, .. } => output,
        _ => panic!("expected working-copy navigation result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("moved to previous editable change | showing recent work | jj undo"));
    assert_eq!(
        app.status.message(),
        "moved to previous editable change | showing recent work | jj undo"
    );
}

#[test]
fn working_copy_navigation_failure_keeps_output_readable() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), None, None),
    ])));
    app.services.working_copy_navigation_run = mock_working_copy_navigation_failure;
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::next(),
        output: ActionPane::pending("jj next --edit".to_owned(), "preview only".to_owned(), None),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { output, .. } => output,
        _ => panic!("expected working-copy navigation result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj next --edit");
    assert!(output.completed());
    assert!(body.contains("jj next --edit failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj next --edit failed: first line\nsecond line"
    );
}

#[test]
fn new_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::NewPreview {
        new_change: JjNewPlan::new(vec!["parent-a".to_owned()]),
        output: ActionPane::pending(
            "jj new parent-a".to_owned(),
            "preview only".to_owned(),
            Some("new preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "new change cancelled");
}

#[test]
fn new_confirm_success_refreshes_and_reveals_working_copy() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.services.reveal_log_change = mock_reveal_new_change_in_recent;
    app.mode = InteractionMode::NewPreview {
        new_change: JjNewPlan::new(vec!["parent-a".to_owned(), "parent-b".to_owned()]),
        output: ActionPane::pending(
            "jj new parent-a parent-b".to_owned(),
            "preview only".to_owned(),
            Some("new preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::NewPreview { output, .. } => output,
        _ => panic!("expected new result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj new parent-a parent-b");
    assert!(output.completed());
    assert!(body.contains("new parents: parent-a,parent-b | showing recent work | jj undo"));
    assert_eq!(
        app.status.message(),
        "new parents: parent-a,parent-b | showing recent work | jj undo"
    );
}

#[test]
fn graph_new_trunk_uses_test_service_and_reveals_working_copy() {
    NEW_TRUNK_CALLS.store(0, Ordering::SeqCst);
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.resolve_revision = mock_resolve_trunk_and_current_change_id;
    app.services.reveal_log_change = mock_reveal_new_change_in_recent;

    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(NEW_TRUNK_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(
        app.status.message(),
        "created new change from trunk | showing recent work | jj undo"
    );
}

#[test]
fn new_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.services.new_run = mock_new_failure;
    app.mode = InteractionMode::NewPreview {
        new_change: JjNewPlan::new(vec!["parent-a".to_owned()]),
        output: ActionPane::pending(
            "jj new parent-a".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::NewPreview { output, .. } => output,
        _ => panic!("expected new result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj new parent-a");
    assert!(output.completed());
    assert!(body.contains("jj new failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj new failed: first line\nsecond line"
    );
}
