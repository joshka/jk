//! Startup, help, view switching, command-prefix, and stack navigation tests.

use super::support::*;

#[test]
fn parses_default_startup_view() {
    let spec = initial_view(Vec::new()).unwrap();

    assert_eq!(spec.command(), jj::Command::Default);
    assert!(spec.args().is_empty());
}

#[test]
fn parses_passthrough_startup_view() {
    let spec = initial_view(vec!["log".into(), "-r".into(), "::".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Log);
    assert_eq!(spec.args(), ["-r", "::"]);
}

#[test]
fn parses_show_startup_view() {
    let spec = initial_view(vec!["show".into(), "--git".into(), "main".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Show);
    assert_eq!(spec.args(), ["--git", "main"]);
    assert_eq!(spec.diff_format(), DiffFormat::Git);
}

#[test]
fn parses_diff_startup_view() {
    let spec = initial_view(vec!["diff".into(), "-r".into(), "main".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Diff);
    assert_eq!(spec.args(), ["-r", "main"]);
}

#[test]
fn parses_status_startup_view() {
    let spec = initial_view(vec!["status".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Status);
    assert!(spec.args().is_empty());
}

#[test]
fn parses_resolve_startup_view() {
    let spec = initial_view(vec!["resolve".into(), "-r".into(), "main".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Resolve);
    assert_eq!(spec.args(), ["-r", "main"]);
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
}

#[test]
fn parses_default_resolve_startup_view() {
    let spec = initial_view(vec!["resolve".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Resolve);
    assert_eq!(spec.args(), ["-r", "@"]);
    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
}

#[test]
fn open_resolve_uses_default_target() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));

    app.open_resolve().unwrap();

    assert_eq!(app.view.spec().command(), jj::Command::Resolve);
    assert_eq!(app.view.spec().args(), ["-r", "@"]);
    assert_eq!(app.view.spec().navigation_revset().as_deref(), Some("@"));
}

#[test]
fn parses_operation_log_startup_view() {
    let spec = initial_view(vec!["operation-log".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::OperationLog);
    assert!(spec.args().is_empty());
}

#[test]
fn parses_bookmarks_startup_view() {
    let spec = initial_view(vec!["bookmarks".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Bookmarks);
    assert!(spec.args().is_empty());
}

#[test]
fn parses_workspaces_startup_view() {
    let spec = initial_view(vec!["workspaces".into()]).unwrap();

    assert_eq!(spec.command(), jj::Command::Workspaces);
    assert!(spec.args().is_empty());
}

#[test]
fn rejects_unknown_startup_command() {
    assert!(initial_view(vec!["bookmark".into()]).is_err());
}

#[test]
fn direct_view_entry_keys_open_shipped_top_level_views() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('S'), KeyModifiers::SHIFT),
        12,
    )
    .unwrap();
    assert_eq!(app.view.command(), jj::Command::Status);
    assert!(app.pending_command.is_none());

    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('B'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert_eq!(app.view.command(), jj::Command::Bookmarks);
    assert!(app.pending_command.is_none());

    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('X'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert_eq!(app.view.command(), jj::Command::Workspaces);
    assert!(app.pending_command.is_none());

    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('O'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert_eq!(app.view.command(), jj::Command::OperationLog);
    assert!(app.pending_command.is_none());
}

#[test]
fn direct_log_key_loads_plain_log_and_clears_stack() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[])));
    app.services.load_view = mock_load_view;
    app.stack.push(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![]),
    ));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('L'), KeyModifiers::NONE),
        12,
    )
    .unwrap();

    assert_eq!(app.view.command(), jj::Command::Log);
    assert!(app.view.spec().args().is_empty());
    assert!(app.stack.is_empty());
    assert!(app.pending_command.is_none());
}

#[test]
fn direct_default_key_loads_default_view_and_clears_stack() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[])));
    app.services.load_view = mock_load_view;
    app.stack.push(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![]),
    ));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('J'), KeyModifiers::NONE),
        12,
    )
    .unwrap();

    assert_eq!(app.view.command(), jj::Command::Default);
    assert!(app.view.spec().args().is_empty());
    assert!(app.stack.is_empty());
    assert!(app.pending_command.is_none());
}

#[test]
fn generated_help_uses_same_multikey_and_view_entry_bindings_as_dispatch() {
    let sections = crate::command::project_help(
        APP_BINDINGS,
        crate::log::BINDINGS,
        crate::command::HelpContext::Log,
    );
    let rows = sections
        .iter()
        .flat_map(|section| section.rows())
        .map(|row| (row.keys(), row.action()))
        .collect::<Vec<_>>();

    assert!(rows.contains(&("S", "status")));
    assert!(rows.contains(&("B", "bookmarks")));
    assert!(rows.contains(&("X", "workspaces")));
    assert!(rows.contains(&("O", "operation log")));
    assert!(rows.contains(&("b, bc", "create bookmark here")));
    assert!(rows.contains(&("f", "fetch")));
    assert!(rows.contains(&("gf", "fetch")));
    assert!(rows.contains(&("F", "fetch remote")));
    assert!(rows.contains(&("gr", "fetch remote")));
    assert!(rows.contains(&("p", "push selected revision")));
    assert!(rows.contains(&("gp", "push selected revision")));
    assert!(rows.contains(&("v", "view menu")));

    let status_sections = crate::command::project_help(
        APP_BINDINGS,
        crate::status::BINDINGS,
        crate::command::HelpContext::Status,
    );
    let status_rows = status_sections
        .iter()
        .flat_map(|section| section.rows())
        .map(|row| (row.keys(), row.action()))
        .collect::<Vec<_>>();
    assert!(status_rows.contains(&("f", "fetch")));
    assert!(status_rows.contains(&("F", "fetch remote")));
    assert!(!status_rows.contains(&("f, gf", "fetch")));
}

#[test]
fn help_menu_executes_listed_command_and_closes() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert!(matches!(app.mode, InteractionMode::Help));

    app.handle_mode_key_at_viewport_height(KeyCode::Char('S'), 12)
        .unwrap();

    assert_eq!(app.view.command(), jj::Command::Status);
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert!(app.pending_command.is_none());
}

#[test]
fn help_menu_close_key_closes_without_executing() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Esc, 12)
        .unwrap();

    assert_eq!(app.view.command(), jj::Command::Default);
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert!(app.pending_command.is_none());
}

#[test]
fn help_menu_close_key_accepts_shifted_question_mark() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT))
        .unwrap();

    assert_eq!(app.view.command(), jj::Command::Default);
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert!(app.pending_command.is_none());
}

#[test]
fn help_menu_ignores_arrow_keys_without_moving_log_selection() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        log_item("first"),
        log_item("second"),
    ])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::SHIFT),
        12,
    )
    .unwrap();
    assert!(matches!(app.mode, InteractionMode::Help));

    app.handle_mode_key_at_viewport_height(KeyCode::Down, 12)
        .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Up, 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Help));
    assert_eq!(app.graph_selected_revision().as_deref(), Some("first"));
    assert!(app.pending_command.is_none());
}

#[test]
fn help_menu_does_not_execute_hidden_commands() {
    let show =
        crate::show::ShowView::test_new(ViewSpec::show("change-a".to_owned(), DiffFormat::Default));
    let mut app = test_app(ViewState::Show(show));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Char('D'), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Help));
    assert_eq!(app.view.command(), jj::Command::Show);
    assert_eq!(app.status.message(), "not available from help menu");
}

#[test]
fn help_menu_supports_multikey_options_and_fallbacks() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.git_fetch_run = mock_fetch_success;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Char('g'), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Help));
    assert!(app.pending_command.is_some());
    assert_eq!(app.status.message(), "help: g -> f/p/r");

    app.handle_mode_key_at_viewport_height(KeyCode::Char('f'), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::FetchPreview { .. }));
    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "fetch: fetched");
}

#[test]
fn expired_help_prefix_runs_fallback_before_routing_next_key_to_opened_mode() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        log_item("change-a"),
    ])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Char('b'), 12)
        .unwrap();
    app.pending_command.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);

    app.handle_mode_key_at_viewport_height(KeyCode::Char('x'), 12)
        .unwrap();

    assert!(app.pending_command.is_none());
    let InteractionMode::BookmarkNamePrompt { input, .. } = &app.mode else {
        panic!("expired bare b fallback should open bookmark prompt");
    };
    assert_eq!(input, "x");
}

#[test]
fn help_prefix_nonmatching_suffix_runs_fallback_then_routes_suffix() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        log_item("first"),
        log_item("second"),
    ])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('?'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Char('g'), 12)
        .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Char('j'), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert!(app.pending_command.is_none());
    assert_eq!(app.graph_selected_revision().as_deref(), Some("second"));
}

#[test]
fn help_menu_projection_groups_commands_by_user_operation() {
    let view = ViewState::Log(crate::log::LogView::test_new(vec![]));
    let mode = InteractionMode::Help;

    let Overlay::Help { sections } = mode.overlay(&view, APP_BINDINGS) else {
        panic!("help mode should project a help overlay");
    };

    let titles = sections
        .iter()
        .map(crate::command::HelpSection::title)
        .collect::<Vec<_>>();

    assert_eq!(
        titles,
        vec![
            "Navigation",
            "View Switching",
            "Search / Copy",
            "Repository Actions",
            "Action Previews",
            "App",
        ]
    );
}

#[test]
fn view_menu_selects_shipped_top_level_views() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('v'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    for _ in 0..3 {
        app.handle_mode_key_at_viewport_height(KeyCode::Down, 12)
            .unwrap();
    }
    app.handle_mode_key_at_viewport_height(KeyCode::Enter, 12)
        .unwrap();

    assert_eq!(app.view.command(), jj::Command::Bookmarks);
    assert!(matches!(app.mode, InteractionMode::Normal));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('v'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Down, 12)
        .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Enter, 12)
        .unwrap();

    assert_eq!(app.view.command(), jj::Command::Workspaces);

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('v'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Down, 12)
        .unwrap();
    app.handle_mode_key_at_viewport_height(KeyCode::Enter, 12)
        .unwrap();

    assert_eq!(app.view.command(), jj::Command::OperationLog);
}

#[test]
fn view_menu_diff_format_status_names_show_diff_scope() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));

    app.apply_view_menu_action(ViewMenuAction::DiffFormat(DiffFormat::Git), 12)
        .unwrap();

    assert_eq!(app.status.message(), "show/diff format: git");
}

#[test]
fn multi_key_bookmark_create_dispatches_without_typing_prefix_suffix() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('b'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert!(app.pending_command.is_some());
    assert_eq!(app.status.message(), "prefix: b -> c/r/f/t/u");

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('c'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    match &app.mode {
        InteractionMode::BookmarkNamePrompt { input, .. } => assert_eq!(input, ""),
        _ => panic!("expected bookmark name prompt"),
    }
    assert!(app.pending_command.is_none());
}

#[test]
fn multi_key_fetch_dispatches_from_git_prefix() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('g'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert!(app.pending_command.is_some());
    assert_eq!(app.status.message(), "prefix: g -> f/p/r");

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('f'), KeyModifiers::NONE),
        12,
    )
    .unwrap();

    assert_eq!(app.status.message(), "fetch: fetched");
    assert!(app.pending_command.is_none());
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
        output.status_context().map(String::as_str),
        Some("default fetch uses jj git fetch remote resolution")
    );
}

#[test]
fn git_fetch_prefix_does_not_delay_non_log_g_navigation() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "working copy changes:",
        "M src/app.rs",
    ])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('g'), KeyModifiers::NONE),
        12,
    )
    .unwrap();

    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "1/2 lines");
}

#[test]
fn expired_bookmark_prefix_runs_fallback_before_next_key() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('b'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.pending_command.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);
    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('c'), KeyModifiers::NONE),
        12,
    )
    .unwrap();

    match &app.mode {
        InteractionMode::BookmarkNamePrompt { input, .. } => assert_eq!(input, "c"),
        _ => panic!("expected bookmark name prompt"),
    }
    assert!(app.pending_command.is_none());
}

#[test]
fn idle_command_prefix_timeout_runs_exact_fallback_and_refreshes_status() {
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

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('g'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.pending_command.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);
    app.flush_expired_pending_command_at_viewport_height(12)
        .unwrap();

    let ViewState::Log(graph) = &app.view else {
        panic!("expected log view");
    };
    assert_eq!(graph.selected_revision(), Some("first"));
    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "2 items | default work");
}

#[test]
fn command_prefix_cancel_does_not_run_global_escape() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![])));

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('b'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    app.handle_normal_key_at_viewport_height_for_test(key(KeyCode::Esc, KeyModifiers::NONE), 12)
        .unwrap();

    assert!(!app.should_quit);
    assert!(app.pending_command.is_none());
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "prefix cancelled");
}

#[test]
fn right_and_l_open_expandable_detail_and_h_or_left_backs_out() {
    let mut app = test_app(ViewState::Log(crate::log::LogView::test_new(vec![
        crate::log::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('l'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert_eq!(app.view.command(), jj::Command::Show);

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('h'), KeyModifiers::NONE),
        12,
    )
    .unwrap();
    assert_eq!(app.view.command(), jj::Command::Default);

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::bookmarks::BookmarkItem::new(
            Vec::new(),
            "main".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(key(KeyCode::Right, KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), jj::Command::Show);

    app.handle_normal_key_at_viewport_height_for_test(key(KeyCode::Left, KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), jj::Command::Bookmarks);
}

#[test]
fn operation_log_l_opens_operation_detail() {
    let operation_id = "op123".to_owned();
    let mut app = test_app(ViewState::OperationLog(
        crate::operation_log::OperationLogView::test_new(vec![
            crate::operation_log::OperationLogItem::new(
                vec![ratatui::text::Line::from("@  current")],
                Some(operation_id),
            ),
        ]),
    ));
    app.services.load_view = mock_load_view;

    app.handle_normal_key_at_viewport_height_for_test(
        key(KeyCode::Char('l'), KeyModifiers::NONE),
        12,
    )
    .unwrap();

    assert_eq!(app.view.command(), jj::Command::OperationShow);
}
