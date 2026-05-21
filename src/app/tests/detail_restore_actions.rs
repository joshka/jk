//! Detail-view action-menu, exact target, restore, and revert tests.

use super::support::*;

#[test]
fn detail_action_menu_from_exact_show_offers_duplicate_restore_and_revert() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let actions = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu
            .items()
            .iter()
            .map(|item| item.action())
            .collect::<Vec<_>>(),
        _ => panic!("expected detail action menu"),
    };
    assert_eq!(
        actions,
        vec![
            ActionKind::Duplicate,
            ActionKind::Restore,
            ActionKind::Revert
        ]
    );
}

#[test]
fn detail_action_menu_from_exact_file_list_offers_path_restore_first() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj_rows::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let menu = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu,
        _ => panic!("expected detail action menu"),
    };
    let actions = menu
        .items()
        .iter()
        .map(|item| item.action())
        .collect::<Vec<_>>();
    assert_eq!(
        actions,
        vec![
            ActionKind::Restore,
            ActionKind::FileChmodExecutable,
            ActionKind::FileChmodNormal,
            ActionKind::Duplicate,
            ActionKind::Restore,
            ActionKind::Revert
        ]
    );
    assert!(matches!(
        menu.items()[0].follow_up(),
        FollowUp::RestoreExactTarget { revision, path }
            if revision == "change-a" && path.as_deref() == Some("src/main.rs")
    ));
}

#[test]
fn open_action_menu_rejects_direct_show_startup_revset() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]),
    )));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "restore/revert from jk show main requires an exact graph-derived revision target"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn open_action_menu_rejects_bookmark_derived_show() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default).without_exact_change_target(),
    )));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "restore/revert from jk show change-a requires an exact graph-derived revision target"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn detail_navigation_marks_graph_targets_exact() {
    let app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(
        Vec::new(),
    )));

    let show = app
        .detail_spec(JjCommand::Show, "change-a".to_owned())
        .unwrap();
    let diff = app
        .detail_spec(JjCommand::Diff, "change-a".to_owned())
        .unwrap();

    assert_eq!(show.exact_change_target(), Some("change-a"));
    assert_eq!(diff.exact_change_target(), Some("change-a"));
}

#[test]
fn detail_navigation_from_bookmarks_is_not_exact() {
    let app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::jj_rows::BookmarkItem::new(
            Vec::new(),
            "feature".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    let show = app
        .detail_spec(JjCommand::Show, "change-a".to_owned())
        .unwrap();
    let diff = app
        .detail_spec(JjCommand::Diff, "change-a".to_owned())
        .unwrap();

    assert_eq!(show.exact_change_target(), None);
    assert_eq!(diff.exact_change_target(), None);
}

#[test]
fn detail_navigation_preserves_inexact_direct_startup_revsets() {
    let app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]),
    )));

    let diff = app.detail_spec(JjCommand::Diff, "main".to_owned()).unwrap();

    assert_eq!(diff.navigation_revset().as_deref(), Some("main"));
    assert_eq!(diff.exact_change_target(), None);
}

#[test]
fn file_show_navigation_preserves_source_exactness_only() {
    let exact_app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj_rows::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));
    let direct_app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned())),
            vec![crate::jj_rows::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));

    let exact = exact_app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();
    let direct = direct_app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();

    assert_eq!(exact.exact_change_target(), Some("change-a"));
    assert_eq!(direct.navigation_revset().as_deref(), Some("main"));
    assert_eq!(direct.exact_change_target(), None);
}

#[test]
fn file_show_navigation_from_resolve_uses_resolve_revision() {
    let app = test_app(ViewState::Resolve(
        crate::resolve::ResolveView::test_with_spec(
            ViewSpec::resolve(Some("main".to_owned())),
            vec![crate::jj_rows::ResolveEntry::parsed(
                Some("src/main.rs".to_owned()),
                Some("file".to_owned()),
                Some(3),
            )],
        ),
    ));

    let file_show = app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();

    assert_eq!(file_show.command(), JjCommand::FileShow);
    assert_eq!(file_show.args(), ["-r", "main", "src/main.rs"]);
    assert_eq!(file_show.navigation_revset().as_deref(), Some("main"));
    assert_eq!(file_show.exact_change_target(), None);
}

#[test]
fn file_show_navigation_from_default_resolve_uses_current_revision() {
    let app = test_app(ViewState::Resolve(
        crate::resolve::ResolveView::test_with_spec(
            ViewSpec::resolve(None),
            vec![crate::jj_rows::ResolveEntry::parsed(
                Some("src/main.rs".to_owned()),
                Some("file".to_owned()),
                Some(3),
            )],
        ),
    ));

    let file_show = app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();

    assert_eq!(file_show.command(), JjCommand::FileShow);
    assert_eq!(file_show.args(), ["-r", "@", "src/main.rs"]);
    assert_eq!(file_show.navigation_revset().as_deref(), Some("@"));
    assert_eq!(file_show.exact_change_target(), None);
}

#[test]
fn open_action_menu_rejects_status_without_exact_path() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[])));

    app.open_action_menu(12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "status file action unavailable: status output is empty"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn status_action_menu_opens_working_copy_path_restore_preview() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "Working copy changes:",
        "M src/status.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('j'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let menu = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu,
        _ => panic!("expected status action menu"),
    };
    assert_eq!(menu.items().len(), 4);
    assert!(matches!(
        menu.items()[0].follow_up(),
        FollowUp::RestoreWorkingCopyPath { path } if path == "src/status.rs"
    ));

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (path, command_label, body) = match &app.mode {
        InteractionMode::RestorePreview { restore, output } => (
            restore.path().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected restore preview mode"),
    };
    assert_eq!(path.as_deref(), Some("src/status.rs"));
    assert_eq!(command_label, "jj restore root-file:\"src/status.rs\"");
    assert!(body.contains("target revision: @"));
    assert!(body.contains("selected path: src/status.rs"));
}

#[test]
fn status_action_menu_reports_disabled_ambiguous_row() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "R {old.rs => new.rs}",
    ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "status file action unavailable: renamed status rows contain multiple paths"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn restore_action_menu_enter_opens_path_preview() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj_rows::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::detail("change-a")
                .with_selected_path("src/main.rs"),
        ),
        selected: 0,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (path, command_label, body) = match &app.mode {
        InteractionMode::RestorePreview { restore, output } => (
            restore.path().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected restore preview mode"),
    };
    assert_eq!(path.as_deref(), Some("src/main.rs"));
    assert_eq!(
        command_label,
        "jj restore --changes-in exactly(change_id(\"change-a\"), 1) root-file:\"src/main.rs\""
    );
    assert!(body.contains("target revision: change-a"));
    assert!(body.contains("selected path: src/main.rs"));
    assert!(body.contains("undo path: jj undo"));
}

#[test]
fn restore_action_menu_path_shortcut_opens_path_preview() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj_rows::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::detail("change-a")
                .with_selected_path("src/main.rs"),
        ),
        selected: 1,
    };

    app.handle_mode_key(KeyCode::Char('p'), 12).unwrap();

    let path = match &app.mode {
        InteractionMode::RestorePreview { restore, .. } => restore.path().map(str::to_owned),
        _ => panic!("expected restore preview mode"),
    };
    assert_eq!(path.as_deref(), Some("src/main.rs"));
}

#[test]
fn restore_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::RestorePreview {
        restore: JjRestorePlan::for_revision("change-a"),
        output: ActionOutput::pending(
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("restore preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "restore cancelled");
}

#[test]
fn restore_confirm_success_and_failure_keep_output_readable() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::RestorePreview {
        restore: JjRestorePlan::for_path("change-a", "src/main.rs"),
        output: ActionOutput::pending(
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1) root-file:\"src/main.rs\""
                .to_owned(),
            "preview only".to_owned(),
            Some("restore preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RestorePreview { output, .. } => output,
        _ => panic!("expected restore result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("restored src/main.rs from change-a | jj undo"));
    assert_eq!(
        app.status.message(),
        "restored src/main.rs from change-a | jj undo"
    );

    app.services.restore_run = mock_restore_failure;
    app.mode = InteractionMode::RestorePreview {
        restore: JjRestorePlan::for_revision("change-a"),
        output: ActionOutput::pending(
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RestorePreview { output, .. } => output,
        _ => panic!("expected restore result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj restore failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj restore failed: first line\nsecond line"
    );
}

#[test]
fn revert_action_menu_enter_opens_preview_and_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::detail("change-a"),
        ),
        selected: 2,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (command_label, body) = match &app.mode {
        InteractionMode::RevertPreview { output, .. } => (
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected revert preview mode"),
    };
    assert_eq!(
        command_label,
        "jj revert -r exactly(change_id(\"change-a\"), 1) -o @"
    );
    assert!(body.contains("target revision: change-a"));

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "revert cancelled");
}

#[test]
fn revert_confirm_success_and_failure_keep_output_readable() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::RevertPreview {
        revert: JjRevertPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj revert -r exactly(change_id(\"change-a\"), 1) -o @".to_owned(),
            "preview only".to_owned(),
            Some("revert preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RevertPreview { output, .. } => output,
        _ => panic!("expected revert result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("reverted change-a | jj undo"));
    assert_eq!(app.status.message(), "reverted change-a | jj undo");

    app.services.revert_run = mock_revert_failure;
    app.mode = InteractionMode::RevertPreview {
        revert: JjRevertPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj revert -r exactly(change_id(\"change-a\"), 1) -o @".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RevertPreview { output, .. } => output,
        _ => panic!("expected revert result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj revert failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj revert failed: first line\nsecond line"
    );
}
