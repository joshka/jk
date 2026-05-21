//! File track, untrack, and chmod action lifecycle tests.

use super::support::*;

#[test]
fn status_untracked_path_opens_track_preview() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "Working copy changes:",
        "? scratch file.txt",
    ])));

    app.handle_normal_key(key(KeyCode::Char('j'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let menu = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu,
        _ => panic!("expected status file action menu"),
    };
    assert_eq!(menu.items().len(), 1);
    assert_eq!(menu.items()[0].action(), ActionKind::FileTrack);

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (command_label, body, path) = match &app.mode {
        InteractionMode::FileMutationPreview { mutation, output } => (
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
            mutation.path().to_owned(),
        ),
        _ => panic!("expected file mutation preview"),
    };
    assert_eq!(
        command_label,
        "jj file track -- root-file:\"scratch file.txt\""
    );
    assert_eq!(path, "scratch file.txt");
    assert!(body.contains("selected path: scratch file.txt"));
    assert!(body.contains("exact fileset: root-file:\"scratch file.txt\""));
    assert!(body.contains("effect: starts tracking this exact untracked working-copy path"));
}

#[test]
fn status_tracked_path_opens_untrack_and_chmod_previews() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "Working copy changes:",
        "M src/main.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('j'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let actions = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu
            .items()
            .iter()
            .map(|item| item.action())
            .collect::<Vec<_>>(),
        _ => panic!("expected status file action menu"),
    };
    assert_eq!(
        actions,
        vec![
            ActionKind::Restore,
            ActionKind::FileUntrack,
            ActionKind::FileChmodExecutable,
            ActionKind::FileChmodNormal,
        ]
    );

    app.handle_mode_key(KeyCode::Char('u'), 12).unwrap();

    let (command_label, body) = match &app.mode {
        InteractionMode::FileMutationPreview { output, .. } => (
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected file untrack preview"),
    };
    assert_eq!(
        command_label,
        "jj file untrack -- root-file:\"src/main.rs\""
    );
    assert!(body.contains("jj requires the path to already be ignored"));

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert_eq!(app.status.message(), "file untrack cancelled");
}

#[test]
fn file_list_working_copy_offers_untrack_and_chmod() {
    let mut app = test_app(ViewState::FileList(
        crate::files::list::FileListView::test_with_spec(
            ViewSpec::file_list(None, None),
            vec![crate::files::list::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let actions = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu
            .items()
            .iter()
            .map(|item| item.action())
            .collect::<Vec<_>>(),
        _ => panic!("expected file list action menu"),
    };
    assert_eq!(
        actions,
        vec![
            ActionKind::FileUntrack,
            ActionKind::FileChmodExecutable,
            ActionKind::FileChmodNormal,
        ]
    );
}

#[test]
fn exact_file_list_chmod_passes_exact_revision() {
    let mut app = test_app(ViewState::FileList(
        crate::files::list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), None).with_exact_change_target(),
            vec![crate::files::list::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();

    let command_label = match &app.mode {
        InteractionMode::FileMutationPreview { output, .. } => output.command_label().to_owned(),
        _ => panic!("expected exact revision chmod preview"),
    };
    assert_eq!(
        command_label,
        "jj file chmod -r exactly(change_id(\"change-a\"), 1) x -- root-file:\"src/main.rs\""
    );
}

#[test]
fn direct_file_revset_and_resolve_file_show_disable_file_actions() {
    let mut direct = test_app(ViewState::FileList(
        crate::files::list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("main".to_owned()), None),
            vec![crate::files::list::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));

    direct
        .handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(direct.mode, InteractionMode::Normal));
    assert_eq!(
        direct.status.message(),
        "file actions from jk file list -r main require a working-copy file list or exact graph-derived revision target"
    );

    let mut resolve = test_app(ViewState::FileShow(crate::files::show::FileShowView::new(
        ViewSpec::file_show(Some("@".to_owned()), "src/main.rs".to_owned()),
        "src/main.rs",
        crate::rendered_jj::DocumentLines::new(Vec::new()),
    )));

    resolve
        .handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(resolve.mode, InteractionMode::Normal));
    assert_eq!(
        resolve.status.message(),
        "file actions from jk file show -r @ src/main.rs require a working-copy file show or exact graph-derived revision target"
    );
}

#[test]
fn file_mutation_confirm_preserves_result_output_and_refreshes() {
    let mut app = test_app(ViewState::FileShow(crate::files::show::FileShowView::new(
        ViewSpec::file_show(None, "src/main.rs".to_owned()),
        "src/main.rs",
        crate::rendered_jj::DocumentLines::new(Vec::new()),
    )));
    app.mode = InteractionMode::FileMutationPreview {
        mutation: JjFileMutationPlan::chmod_working_copy(
            "src/main.rs",
            crate::jj_actions::JjFileChmodMode::Normal,
        ),
        output: ActionOutput::pending(
            "jj file chmod n -- root-file:\"src/main.rs\"".to_owned(),
            "preview only".to_owned(),
            Some("file chmod context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::FileMutationPreview { output, .. } => output,
        _ => panic!("expected file mutation result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("file chmod n src/main.rs | jj undo | jj op show -p")
    );

    app.services.file_mutation_run = mock_file_mutation_failure;
    app.mode = InteractionMode::FileMutationPreview {
        mutation: JjFileMutationPlan::untrack("src/main.rs"),
        output: ActionOutput::pending(
            "jj file untrack -- root-file:\"src/main.rs\"".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::FileMutationPreview { output, .. } => output,
        _ => panic!("expected file mutation failure result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj file failed: first line")
    );
    assert_eq!(
        app.status.message(),
        "jj file failed: first line\nsecond line"
    );
}
