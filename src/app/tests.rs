use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::KeybindConfig;

use crate::flow::{FlowAction, PromptKind};

use super::preview::{confirmation_preview_tokens, is_dangerous, toggle_patch_flag};
use super::selection::{
    build_row_revision_map, extract_revision, is_change_id, is_commit_id,
    looks_like_graph_commit_row, metadata_log_tokens, startup_action, trim_to_width,
};
use super::view::{
    bookmark_mutation_summary, decorate_command_output, git_fetch_summary, git_push_summary,
    keymap_overview_lines, operation_mutation_summary, render_bookmark_list_view,
    render_bookmark_mutation_view, render_diff_view, render_file_annotate_view,
    render_file_chmod_view, render_file_list_view, render_file_search_view, render_file_show_view,
    render_file_track_view, render_file_untrack_view, render_git_fetch_view, render_git_push_view,
    render_operation_diff_view, render_operation_log_view, render_operation_mutation_view,
    render_operation_show_view, render_resolve_list_view, render_root_view, render_show_view,
    render_status_view, render_tag_delete_view, render_tag_list_view, render_tag_set_view,
    render_top_level_mutation_view, render_version_view, render_workspace_list_view,
    render_workspace_mutation_view, top_level_mutation_summary, workspace_mutation_summary,
};
use super::{App, Mode};

#[test]
fn extracts_change_id_from_log_line() {
    let line = "@  abcdefgh joshka@example.com 2026-02-07 0123abcd";
    assert_eq!(extract_revision(line), Some("abcdefgh".to_string()));
}

#[test]
fn extracts_commit_id_when_change_missing() {
    let line = "Commit hash 0123abcd updated";
    assert_eq!(extract_revision(line), Some("0123abcd".to_string()));
}

#[test]
fn extracts_revision_from_ansi_colored_log_line() {
    let line = "\u{1b}[32m@\u{1b}[0m  \u{1b}[36mabcdefgh\u{1b}[0m user 2026-02-07 0123abcd";
    assert_eq!(extract_revision(line), Some("abcdefgh".to_string()));
}

#[test]
fn detects_ansi_colored_graph_commit_rows() {
    let line = "\u{1b}[32m@\u{1b}[0m  \u{1b}[36mabcdefgh\u{1b}[0m user message";
    assert!(looks_like_graph_commit_row(line));
}

#[test]
fn extracts_revision_from_graph_row_without_commit_hash() {
    let line = "@  abcdefgh joshka 2026-02-07 add feature";
    assert_eq!(extract_revision(line), Some("abcdefgh".to_string()));
}

#[test]
fn trim_to_width_preserves_ansi_sequences() {
    let line = "\u{1b}[31m@a\u{1b}[0m trailing";
    let trimmed = trim_to_width(line, 1);
    assert_eq!(trimmed, "\u{1b}[31m@\u{1b}[0m");
}

#[test]
fn snapshot_renders_basic_frame() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.lines = vec![
        "@  abcdefgh 0123abcd message".to_string(),
        "○  hgfedcba 89abcdef parent".to_string(),
    ];
    app.status_line = "Ready".to_string();
    insta::assert_snapshot!(app.render_for_snapshot(60, 6));
}

#[test]
fn marks_tier_c_commands_as_dangerous() {
    assert!(is_dangerous(&["rebase".to_string()]));
    assert!(is_dangerous(&["squash".to_string()]));
    assert!(is_dangerous(&["split".to_string()]));
    assert!(is_dangerous(&["diffedit".to_string()]));
    assert!(is_dangerous(&["fix".to_string()]));
    assert!(is_dangerous(&["abandon".to_string()]));
    assert!(is_dangerous(&["undo".to_string()]));
    assert!(is_dangerous(&["redo".to_string()]));
    assert!(is_dangerous(&["restore".to_string()]));
    assert!(is_dangerous(&["revert".to_string()]));
    assert!(is_dangerous(&["git".to_string(), "push".to_string()]));
    assert!(is_dangerous(&["bookmark".to_string(), "set".to_string()]));
    assert!(is_dangerous(&["bookmark".to_string(), "move".to_string()]));
    assert!(is_dangerous(&[
        "bookmark".to_string(),
        "delete".to_string()
    ]));
    assert!(is_dangerous(&[
        "bookmark".to_string(),
        "forget".to_string()
    ]));
    assert!(is_dangerous(&[
        "bookmark".to_string(),
        "rename".to_string()
    ]));
}

#[test]
fn leaves_read_and_low_risk_commands_unguarded() {
    assert!(!is_dangerous(&["log".to_string()]));
    assert!(!is_dangerous(&["status".to_string()]));
    assert!(!is_dangerous(&["show".to_string()]));
    assert!(!is_dangerous(&["diff".to_string()]));
    assert!(!is_dangerous(&["git".to_string(), "fetch".to_string()]));
    assert!(!is_dangerous(&["bookmark".to_string(), "list".to_string()]));
    assert!(!is_dangerous(&[
        "bookmark".to_string(),
        "create".to_string()
    ]));
    assert!(!is_dangerous(&["next".to_string()]));
    assert!(!is_dangerous(&["prev".to_string()]));
    assert!(!is_dangerous(&["edit".to_string()]));
}

#[test]
fn selected_revision_falls_back_to_previous_revision_line() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.lines = vec![
        "@  abcdefgh 0123abcd top commit".to_string(),
        "│  detailed message line without ids".to_string(),
        "○  hgfedcba 89abcdef parent commit".to_string(),
    ];
    app.cursor = 1;

    assert_eq!(app.selected_revision(), Some("abcdefgh".to_string()));
}

#[test]
fn log_navigation_moves_by_revision_item_not_rendered_line() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.lines = vec![
        "@  aaaaaaaa 11111111 top".to_string(),
        "│  detail line one".to_string(),
        "│  detail line two".to_string(),
        "○  bbbbbbbb 22222222 middle".to_string(),
        "│  middle detail".to_string(),
        "○  cccccccc 33333333 bottom".to_string(),
    ];
    app.row_revision_map = vec![
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("bbbbbbbb".to_string()),
        Some("bbbbbbbb".to_string()),
        Some("cccccccc".to_string()),
    ];

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move to next item");
    assert_eq!(app.cursor, 3);
    assert_eq!(app.selected_revision(), Some("bbbbbbbb".to_string()));

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move to third item");
    assert_eq!(app.cursor, 5);
    assert_eq!(app.selected_revision(), Some("cccccccc".to_string()));

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should move to previous item");
    assert_eq!(app.cursor, 3);
    assert_eq!(app.selected_revision(), Some("bbbbbbbb".to_string()));
}

#[test]
fn log_navigation_uses_visible_graph_rows_when_revision_ids_repeat() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.last_command = vec!["log".to_string()];
    app.lines = vec![
        "@  aaaaaaaa 11111111 first".to_string(),
        "│  detail first".to_string(),
        "○  bbbbbbbb 22222222 second".to_string(),
        "│  detail second".to_string(),
        "○  cccccccc 33333333 third".to_string(),
    ];
    // Simulate metadata ambiguity where adjacent visible items resolve to repeated revision ids.
    app.row_revision_map = vec![
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("bbbbbbbb".to_string()),
    ];

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("first down should move to second visible item");
    assert_eq!(app.cursor, 2);
    assert_eq!(app.selected_revision(), Some("aaaaaaaa".to_string()));

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("second down should move to third visible item");
    assert_eq!(app.cursor, 4);
    assert_eq!(app.selected_revision(), Some("bbbbbbbb".to_string()));

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should return to second visible item");
    assert_eq!(app.cursor, 2);

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should return to first visible item");
    assert_eq!(app.cursor, 0);
}

#[test]
fn page_navigation_supports_page_keys_and_ctrl_bindings() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.viewport_rows = 6;
    app.lines = (0..40).map(|index| format!("line {index}")).collect();

    app.handle_key(KeyEvent::from(KeyCode::PageDown))
        .expect("pagedown should move cursor by one viewport");
    assert_eq!(app.cursor, 5);

    app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL))
        .expect("ctrl+d should page down");
    assert_eq!(app.cursor, 10);

    app.handle_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL))
        .expect("ctrl+u should page up");
    assert_eq!(app.cursor, 5);

    app.handle_key(KeyEvent::from(KeyCode::PageUp))
        .expect("pageup should move cursor by one viewport");
    assert_eq!(app.cursor, 0);
}

#[test]
fn log_page_navigation_moves_by_viewport_rows_not_item_count() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.viewport_rows = 6;
    app.lines = vec![
        "@  aaaaaaaa 11111111 one".to_string(),
        "│  detail one".to_string(),
        "○  bbbbbbbb 22222222 two".to_string(),
        "│  detail two".to_string(),
        "○  cccccccc 33333333 three".to_string(),
        "│  detail three".to_string(),
        "○  dddddddd 44444444 four".to_string(),
        "│  detail four".to_string(),
        "○  eeeeeeee 55555555 five".to_string(),
        "│  detail five".to_string(),
        "○  ffffffff 66666666 six".to_string(),
        "│  detail six".to_string(),
    ];
    app.row_revision_map = vec![
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("bbbbbbbb".to_string()),
        Some("bbbbbbbb".to_string()),
        Some("cccccccc".to_string()),
        Some("cccccccc".to_string()),
        Some("dddddddd".to_string()),
        Some("dddddddd".to_string()),
        Some("eeeeeeee".to_string()),
        Some("eeeeeeee".to_string()),
        Some("ffffffff".to_string()),
        Some("ffffffff".to_string()),
    ];

    app.handle_key(KeyEvent::from(KeyCode::PageDown))
        .expect("pagedown should target next item near viewport boundary");
    assert_eq!(app.cursor, 6);
    assert_eq!(app.selected_revision(), Some("dddddddd".to_string()));

    app.handle_key(KeyEvent::from(KeyCode::PageDown))
        .expect("pagedown should clamp to last item start");
    assert_eq!(app.cursor, 10);
    assert_eq!(app.selected_revision(), Some("ffffffff".to_string()));

    app.handle_key(KeyEvent::from(KeyCode::PageUp))
        .expect("pageup should target prior item near viewport boundary");
    assert_eq!(app.cursor, 4);
    assert_eq!(app.selected_revision(), Some("cccccccc".to_string()));
}

#[test]
fn log_navigation_uses_graph_rows_when_metadata_map_is_missing() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.last_command = vec!["log".to_string()];
    app.lines = vec![
        "@  abcdefgh top".to_string(),
        "│  detail".to_string(),
        "○  hgfedcba parent".to_string(),
        "│  parent detail".to_string(),
        "○  qwertyui root".to_string(),
    ];
    app.row_revision_map = vec![None; app.lines.len()];

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move to next graph row");
    assert_eq!(app.cursor, 2);

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move to third graph row");
    assert_eq!(app.cursor, 4);

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should move to previous graph row");
    assert_eq!(app.cursor, 2);
}

#[test]
fn snapshot_log_item_navigation_and_paging_sequence() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.viewport_rows = 6;
    app.status_line = "ok: jj log".to_string();
    app.last_command = vec!["log".to_string()];
    app.lines = vec![
        "@  aaaaaaaa 11111111 one".to_string(),
        "│  detail one".to_string(),
        "│  detail one-b".to_string(),
        "○  bbbbbbbb 22222222 two".to_string(),
        "│  detail two".to_string(),
        "○  cccccccc 33333333 three".to_string(),
        "│  detail three".to_string(),
        "○  dddddddd 44444444 four".to_string(),
    ];
    app.row_revision_map = vec![
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("bbbbbbbb".to_string()),
        Some("bbbbbbbb".to_string()),
        Some("cccccccc".to_string()),
        Some("cccccccc".to_string()),
        Some("dddddddd".to_string()),
    ];

    let mut frames = Vec::new();
    frames.push(format!("start\n{}", app.render_for_snapshot(60, 8)));

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move to next item start");
    frames.push(format!("down\n{}", app.render_for_snapshot(60, 8)));

    app.handle_key(KeyEvent::from(KeyCode::PageDown))
        .expect("pagedown should move by viewport target");
    frames.push(format!("pagedown\n{}", app.render_for_snapshot(60, 8)));

    app.handle_key(KeyEvent::from(KeyCode::PageUp))
        .expect("pageup should move by viewport target");
    frames.push(format!("pageup\n{}", app.render_for_snapshot(60, 8)));

    insta::assert_snapshot!(frames.join("\n\n---\n\n"));
}

#[test]
fn snapshot_log_navigation_round_trip_three_items() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.status_line = "ok: jj log".to_string();
    app.last_command = vec!["log".to_string()];
    app.lines = vec![
        "@  aaaaaaaa 11111111 first".to_string(),
        "│  detail first".to_string(),
        "○  bbbbbbbb 22222222 second".to_string(),
        "│  detail second".to_string(),
        "○  cccccccc 33333333 third".to_string(),
    ];
    app.row_revision_map = vec![
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("aaaaaaaa".to_string()),
        Some("bbbbbbbb".to_string()),
    ];

    let mut frames = Vec::new();
    frames.push(format!("start\n{}", app.render_for_snapshot(64, 8)));

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move to second visible item");
    frames.push(format!("down-1\n{}", app.render_for_snapshot(64, 8)));

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move to third visible item");
    frames.push(format!("down-2\n{}", app.render_for_snapshot(64, 8)));

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should move to second visible item");
    frames.push(format!("up-1\n{}", app.render_for_snapshot(64, 8)));

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should move to first visible item");
    frames.push(format!("up-2\n{}", app.render_for_snapshot(64, 8)));

    insta::assert_snapshot!(frames.join("\n\n---\n\n"));
}

#[test]
fn view_history_moves_back_and_forward_between_screens() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.execute_command_line("commands")
        .expect("commands screen should render");
    app.execute_command_line("keys")
        .expect("keys screen should render");

    app.handle_key(KeyEvent::from(KeyCode::Left))
        .expect("left should navigate back");
    assert!(
        app.lines
            .iter()
            .any(|line| line.contains("jk command registry"))
    );
    assert!(app.status_line.contains("back: commands"));

    app.handle_key(KeyEvent::from(KeyCode::Right))
        .expect("right should navigate forward");
    assert!(app.lines.iter().any(|line| line.contains("jk keymap")));
    assert!(app.status_line.contains("forward: keys"));
}

#[test]
fn snapshot_screen_transition_history_sequence() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    let mut captures = Vec::new();

    app.execute_command_line("commands")
        .expect("commands screen should render");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("commands\nstatus: {status}\n{frame}"));

    app.execute_command_line("keys")
        .expect("keys screen should render");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("keys\nstatus: {status}\n{frame}"));

    app.execute_command_line("aliases")
        .expect("aliases screen should render");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("aliases\nstatus: {status}\n{frame}"));

    app.execute_command_line("help inspect")
        .expect("workflow help should render");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("help-inspect\nstatus: {status}\n{frame}"));

    app.handle_key(KeyEvent::from(KeyCode::Left))
        .expect("left should go back to aliases");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("back-1\nstatus: {status}\n{frame}"));

    app.handle_key(KeyEvent::from(KeyCode::Left))
        .expect("left should go back to keys");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("back-2\nstatus: {status}\n{frame}"));

    app.handle_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::CONTROL))
        .expect("ctrl+i should go forward to aliases");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("forward-1\nstatus: {status}\n{frame}"));

    app.handle_key(KeyEvent::from(KeyCode::Right))
        .expect("right should go forward to help-inspect");
    let status = app.status_line.clone();
    let frame = app.render_for_snapshot(90, 8);
    captures.push(format!("forward-2\nstatus: {status}\n{frame}"));

    insta::assert_snapshot!(captures.join("\n\n---\n\n"));
}

#[test]
fn does_not_extract_revision_from_message_line_without_commit_id() {
    let line = "│  detailed message line without ids";
    assert_eq!(extract_revision(line), None);
}

#[test]
fn recognizes_change_and_commit_id_formats() {
    assert!(is_change_id("abcdefgh"));
    assert!(is_change_id("abcdefgh/12"));
    assert!(!is_change_id("abc-defgh"));

    assert!(is_commit_id("0123abcd"));
    assert!(!is_commit_id("abcdefgh"));
}

#[test]
fn startup_action_defaults_to_log() {
    assert_eq!(
        startup_action(&[]),
        FlowAction::Execute(vec!["log".to_string()])
    );
}

#[test]
fn startup_action_uses_guided_planner() {
    let tokens = vec!["new".to_string()];
    match startup_action(&tokens) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::NewMessage);
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn startup_action_supports_core_jj_default_aliases() {
    assert_eq!(
        startup_action(&["st".to_string()]),
        FlowAction::Execute(vec!["status".to_string()])
    );
    assert_eq!(
        startup_action(&["b".to_string()]),
        FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
    );
    assert_eq!(
        startup_action(&["op".to_string()]),
        FlowAction::Execute(vec!["operation".to_string(), "log".to_string()])
    );

    match startup_action(&["ci".to_string()]) {
        FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::CommitMessage),
        other => panic!("expected prompt, got {other:?}"),
    }

    match startup_action(&["desc".to_string()]) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::DescribeMessage {
                    revision: "@".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn startup_action_supports_high_frequency_omz_aliases() {
    assert_eq!(
        startup_action(&["jjst".to_string()]),
        FlowAction::Execute(vec!["status".to_string()])
    );
    assert_eq!(
        startup_action(&["jjl".to_string()]),
        FlowAction::Execute(vec!["log".to_string()])
    );
    assert_eq!(
        startup_action(&["jjd".to_string()]),
        FlowAction::Execute(vec!["diff".to_string(), "-r".to_string(), "@".to_string()])
    );
    assert_eq!(
        startup_action(&["jjrbm".to_string()]),
        FlowAction::Execute(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "trunk()".to_string()
        ])
    );

    match startup_action(&["jjgf".to_string()]) {
        FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::GitFetchRemote),
        other => panic!("expected prompt, got {other:?}"),
    }

    match startup_action(&["jjgp".to_string()]) {
        FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::GitPushBookmark),
        other => panic!("expected prompt, got {other:?}"),
    }

    match startup_action(&["jjc".to_string()]) {
        FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::CommitMessage),
        other => panic!("expected prompt, got {other:?}"),
    }

    match startup_action(&["jjds".to_string()]) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::DescribeMessage {
                    revision: "@".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn startup_dangerous_command_requires_confirmation() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.apply_startup_tokens(vec![
        "rebase".to_string(),
        "-d".to_string(),
        "main".to_string(),
    ])
    .expect("startup action should succeed");

    assert_eq!(app.mode, Mode::Confirm);
    assert_eq!(
        app.pending_confirm,
        Some(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "main".to_string()
        ])
    );
}

#[test]
fn startup_commands_view_renders_without_running_jj() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.apply_startup_tokens(vec!["commands".to_string()])
        .expect("startup action should succeed");

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.status_line, "Showing command registry".to_string());
    assert!(
        app.lines
            .iter()
            .any(|line| line.contains("jk command registry"))
    );
}

#[test]
fn startup_keys_view_renders_without_running_jj() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.apply_startup_tokens(vec!["keys".to_string()])
        .expect("startup action should succeed");

    assert_eq!(app.mode, Mode::Normal);
    assert_eq!(app.status_line, "Showing keymap".to_string());
    assert!(app.lines.iter().any(|line| line.contains("jk keymap")));
    assert!(app.lines.iter().any(|line| line.contains("push")));
    assert!(app.lines.iter().any(|line| line.contains("file list")));
    assert!(app.lines.iter().any(|line| line.contains("resolve list")));
    assert!(app.lines.iter().any(|line| line.contains("tag list")));
}

#[test]
fn filters_keymap_view_by_query() {
    let lines = keymap_overview_lines(
        &KeybindConfig::load().expect("keybind config should parse"),
        Some("push"),
    );

    assert!(lines.iter().any(|line| line.contains("push")));
    assert!(!lines.iter().any(|line| line.contains("quit")));
}

#[test]
fn command_keys_view_renders_and_filters() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.execute_command_line("keys push")
        .expect("keys command should render");

    assert_eq!(app.status_line, "Showing keymap for `push`".to_string());
    assert!(app.lines.iter().any(|line| line.contains("push")));
    assert!(!app.lines.iter().any(|line| line.contains("quit")));
}

#[test]
fn snapshot_renders_condensed_keymap_layout() {
    let lines = keymap_overview_lines(
        &KeybindConfig::load().expect("keybind config should parse"),
        None,
    );

    insta::assert_snapshot!(lines.join("\n"));
}

#[test]
fn confirm_preview_renders_header_for_tier_c_commands() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.execute_with_confirmation(vec![
        "rebase".to_string(),
        "-d".to_string(),
        "main".to_string(),
    ])
    .expect("confirmation setup should succeed");

    assert_eq!(app.mode, Mode::Confirm);
    assert!(
        app.lines
            .iter()
            .any(|line| line.contains("Confirm: jj rebase -d main"))
    );
}

#[test]
fn builds_dry_run_preview_for_git_push() {
    let preview = confirmation_preview_tokens(&["git".to_string(), "push".to_string()]);
    assert_eq!(
        preview,
        Some(vec![
            "git".to_string(),
            "push".to_string(),
            "--dry-run".to_string()
        ])
    );

    let existing = confirmation_preview_tokens(&[
        "git".to_string(),
        "push".to_string(),
        "--dry-run".to_string(),
    ]);
    assert_eq!(existing, None);
}

#[test]
fn builds_operation_preview_for_restore_and_revert() {
    let restore = confirmation_preview_tokens(&[
        "operation".to_string(),
        "restore".to_string(),
        "abc123".to_string(),
    ]);
    assert_eq!(
        restore,
        Some(vec![
            "operation".to_string(),
            "show".to_string(),
            "abc123".to_string(),
            "--no-op-diff".to_string()
        ])
    );

    let revert_default =
        confirmation_preview_tokens(&["operation".to_string(), "revert".to_string()]);
    assert_eq!(
        revert_default,
        Some(vec![
            "operation".to_string(),
            "show".to_string(),
            "@".to_string(),
            "--no-op-diff".to_string()
        ])
    );
}

#[test]
fn builds_rebase_and_squash_preview_revsets() {
    let rebase =
        confirmation_preview_tokens(&["rebase".to_string(), "-d".to_string(), "main".to_string()]);
    assert_eq!(
        rebase,
        Some(vec![
            "log".to_string(),
            "-r".to_string(),
            "@ | main".to_string(),
            "-n".to_string(),
            "20".to_string()
        ])
    );

    let squash = confirmation_preview_tokens(&[
        "squash".to_string(),
        "--from".to_string(),
        "abc123".to_string(),
        "--into".to_string(),
        "@-".to_string(),
    ]);
    assert_eq!(
        squash,
        Some(vec![
            "log".to_string(),
            "-r".to_string(),
            "abc123 | @-".to_string(),
            "-n".to_string(),
            "20".to_string()
        ])
    );
}

#[test]
fn builds_split_and_abandon_preview_commands() {
    let split = confirmation_preview_tokens(&[
        "split".to_string(),
        "-r".to_string(),
        "abc123".to_string(),
        "src/main.rs".to_string(),
    ]);
    assert_eq!(split, Some(vec!["show".to_string(), "abc123".to_string()]));

    let abandon = confirmation_preview_tokens(&["abandon".to_string(), "abc123".to_string()]);
    assert_eq!(
        abandon,
        Some(vec![
            "log".to_string(),
            "-r".to_string(),
            "abc123".to_string(),
            "-n".to_string(),
            "20".to_string()
        ])
    );
}

#[test]
fn builds_restore_revert_bookmark_and_operation_log_previews() {
    let restore = confirmation_preview_tokens(&[
        "restore".to_string(),
        "--from".to_string(),
        "@-".to_string(),
        "--to".to_string(),
        "@".to_string(),
    ]);
    assert_eq!(
        restore,
        Some(vec![
            "log".to_string(),
            "-r".to_string(),
            "@- | @".to_string(),
            "-n".to_string(),
            "20".to_string()
        ])
    );

    let revert = confirmation_preview_tokens(&[
        "revert".to_string(),
        "-r".to_string(),
        "abc123".to_string(),
        "-o".to_string(),
        "@".to_string(),
    ]);
    assert_eq!(
        revert,
        Some(vec![
            "log".to_string(),
            "-r".to_string(),
            "abc123 | @".to_string(),
            "-n".to_string(),
            "20".to_string()
        ])
    );

    let bookmark = confirmation_preview_tokens(&[
        "bookmark".to_string(),
        "set".to_string(),
        "feature".to_string(),
        "-r".to_string(),
        "@".to_string(),
    ]);
    assert_eq!(
        bookmark,
        Some(vec![
            "bookmark".to_string(),
            "list".to_string(),
            "--all".to_string()
        ])
    );

    let undo = confirmation_preview_tokens(&["undo".to_string()]);
    assert_eq!(
        undo,
        Some(vec![
            "operation".to_string(),
            "log".to_string(),
            "-n".to_string(),
            "5".to_string()
        ])
    );
}

#[test]
fn falls_back_to_operation_log_for_unknown_tier_c_commands() {
    let preview = confirmation_preview_tokens(&[
        "simplify-parents".to_string(),
        "-r".to_string(),
        "@".to_string(),
    ]);
    assert_eq!(
        preview,
        Some(vec![
            "operation".to_string(),
            "log".to_string(),
            "-n".to_string(),
            "5".to_string()
        ])
    );

    let safe = confirmation_preview_tokens(&["status".to_string()]);
    assert_eq!(safe, None);
}

#[test]
fn metadata_log_tokens_strip_template_and_patch_options() {
    let tokens = vec![
        "log".to_string(),
        "-r".to_string(),
        "all()".to_string(),
        "-T".to_string(),
        "user_template".to_string(),
        "--patch".to_string(),
    ];
    assert_eq!(
        metadata_log_tokens(&tokens),
        Some(vec![
            "log".to_string(),
            "--no-graph".to_string(),
            "-T".to_string(),
            "change_id.short() ++ \" \" ++ commit_id.short()".to_string(),
            "-r".to_string(),
            "all()".to_string(),
        ])
    );
}

#[test]
fn graph_row_detection_handles_connectors() {
    assert!(looks_like_graph_commit_row("@  abcdefgh 0123abcd message"));
    assert!(looks_like_graph_commit_row("│ ○ hgfedcba 89abcdef parent"));
    assert!(!looks_like_graph_commit_row("│ detailed message line"));
}

#[test]
fn row_revision_map_falls_back_to_metadata_order() {
    let lines = vec![
        "@  no explicit ids".to_string(),
        "│ detail row".to_string(),
        "○ also missing ids".to_string(),
        "│ detail row two".to_string(),
    ];
    let metadata = vec!["abcdefgh".to_string(), "hgfedcba".to_string()];
    let map = build_row_revision_map(&lines, &metadata);

    assert_eq!(
        map,
        vec![
            Some("abcdefgh".to_string()),
            Some("abcdefgh".to_string()),
            Some("hgfedcba".to_string()),
            Some("hgfedcba".to_string()),
        ]
    );
}

#[test]
fn renders_status_output_as_scannable_sections() {
    let rendered = render_status_view(vec![
        "Working copy changes:".to_string(),
        "M src/app.rs".to_string(),
        "A src/new.rs".to_string(),
        "Working copy  (@) : abcdefgh 0123abcd summary".to_string(),
        "Parent commit (@-): hgfedcba 89abcdef parent".to_string(),
        "Conflicted bookmarks:".to_string(),
        "  feature".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Status Overview".to_string()));
    assert!(rendered.iter().any(|line| line == "Working copy changes:"));
    assert!(rendered.iter().any(|line| line == "  M src/app.rs"));
    assert!(rendered.iter().any(|line| line == "Conflicted bookmarks:"));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 working-copy changes"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("1 conflicted bookmark"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Shortcuts: s status"))
    );
}

#[test]
fn renders_show_view_with_header_and_shortcuts() {
    let rendered = render_show_view(vec![
        "Commit ID: abcdef0123456789".to_string(),
        "Change ID: abcdefghijklmnop".to_string(),
        "Modified regular file src/app.rs:".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Show View".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line == "Commit ID: abcdef0123456789")
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Shortcuts: Enter show selected"))
    );
}

#[test]
fn inserts_section_spacing_in_show_view() {
    let rendered = render_show_view(vec![
        "Commit ID: abcdef0123456789".to_string(),
        "Change ID: abcdefghijklmnop".to_string(),
        "Modified regular file src/app.rs:".to_string(),
        "  1: old".to_string(),
        "Modified regular file src/config.rs:".to_string(),
        "  1: new".to_string(),
    ]);

    let second_section_index = rendered
        .iter()
        .position(|line| line == "Modified regular file src/config.rs:")
        .expect("second show section should exist");
    assert_eq!(rendered.get(second_section_index - 1), Some(&String::new()));
}

#[test]
fn renders_diff_view_with_header_and_shortcuts() {
    let rendered = render_diff_view(vec![
        "Modified regular file src/app.rs:".to_string(),
        "  1  1: use std::collections::HashMap;".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Diff View".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line == "Modified regular file src/app.rs:")
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Shortcuts: d diff selected"))
    );
}

#[test]
fn inserts_file_spacing_in_diff_view() {
    let rendered = render_diff_view(vec![
        "Modified regular file src/app.rs:".to_string(),
        "  1  1: use std::collections::HashMap;".to_string(),
        "Modified regular file src/config.rs:".to_string(),
        "  1  1: use std::env;".to_string(),
    ]);

    let second_file_index = rendered
        .iter()
        .position(|line| line == "Modified regular file src/config.rs:")
        .expect("second diff file should exist");
    assert_eq!(rendered.get(second_file_index - 1), Some(&String::new()));
}

#[test]
fn renders_root_view_with_header_and_tip() {
    let rendered = render_root_view(vec!["/Users/joshka/local/jk".to_string()]);

    assert_eq!(rendered.first(), Some(&"Workspace Root".to_string()));
    assert!(rendered.iter().any(|line| line == "/Users/joshka/local/jk"));
    assert!(rendered.iter().any(|line| line.contains("jjrt/jk root")));
}

#[test]
fn renders_version_view_with_summary_and_tip() {
    let rendered = render_version_view(vec!["jj 0.31.0".to_string()]);

    assert_eq!(rendered.first(), Some(&"Version".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: jj 0.31.0"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("confirm toolchain state"))
    );
}

#[test]
fn renders_resolve_list_view_with_no_conflicts_summary() {
    let rendered = render_resolve_list_view(vec![
        "Error: No conflicts found at this revision".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Resolve List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: no conflicts listed"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("No conflicts found at this revision"))
    );
}

#[test]
fn renders_resolve_list_view_with_conflict_count() {
    let rendered =
        render_resolve_list_view(vec!["src/app.rs".to_string(), "src/flows.rs".to_string()]);

    assert_eq!(rendered.first(), Some(&"Resolve List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 conflicted paths listed"))
    );
    assert!(rendered.iter().any(|line| line == "src/app.rs"));
}

#[test]
fn decorates_resolve_list_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["resolve".to_string(), "-l".to_string()],
        vec!["Error: No conflicts found at this revision".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Resolve List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: no conflicts listed"))
    );
}

#[test]
fn decorates_resolve_action_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["resolve".to_string(), "src/app.rs".to_string()],
        vec!["Resolved conflict in src/app.rs".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Resolve".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Resolved conflict in src/app.rs"))
    );
}

#[test]
fn renders_file_list_view_with_summary_and_tip() {
    let rendered =
        render_file_list_view(vec!["src/app.rs".to_string(), "src/flows.rs".to_string()]);

    assert_eq!(rendered.first(), Some(&"File List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 files listed"))
    );
    assert!(rendered.iter().any(|line| line == "src/app.rs"));
    assert!(rendered.iter().any(|line| line.contains("show`/`diff")));
}

#[test]
fn renders_file_show_view_with_summary_and_tip() {
    let rendered = render_file_show_view(vec![
        "fn main() {".to_string(),
        String::new(),
        "}".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"File Show".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 3 content lines"))
    );
    assert!(rendered.iter().any(|line| line == "fn main() {"));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("surrounding change context"))
    );
}

#[test]
fn renders_file_search_view_with_summary_and_tip() {
    let rendered = render_file_search_view(vec![
        "src/app.rs:120:render_status_view".to_string(),
        "src/flows.rs:88:plan_command".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"File Search".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 match lines"))
    );
    assert!(rendered.iter().any(|line| line.contains("src/app.rs:120")));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("refine search patterns"))
    );
}

#[test]
fn renders_file_annotate_view_with_summary_and_tip() {
    let rendered = render_file_annotate_view(vec![
        "uxqqtlkq src/app.rs:1 use std::io;".to_string(),
        "qtswpusn src/app.rs:2 fn main() {}".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"File Annotate".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 annotated lines"))
    );
    assert!(rendered.iter().any(|line| line.contains("src/app.rs:1")));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("source revision details"))
    );
}

#[test]
fn renders_file_track_view_with_summary_and_tip() {
    let rendered = render_file_track_view(vec![
        "Started tracking 2 paths".to_string(),
        "src/new.rs".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"File Track".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 output lines"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("review tracked paths"))
    );
}

#[test]
fn renders_file_untrack_view_with_summary_and_tip() {
    let rendered =
        render_file_untrack_view(vec!["Stopped tracking target/generated.txt".to_string()]);

    assert_eq!(rendered.first(), Some(&"File Untrack".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("ensure paths are ignored"))
    );
}

#[test]
fn renders_file_chmod_view_with_summary_and_tip() {
    let rendered = render_file_chmod_view(vec![
        "Updated mode to executable for scripts/deploy.sh".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"File Chmod".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("verify executable-bit updates"))
    );
}

#[test]
fn renders_tag_list_view_with_empty_state() {
    let rendered = render_tag_list_view(vec!["(no output)".to_string()]);

    assert_eq!(rendered.first(), Some(&"Tag List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 0 tags listed"))
    );
    assert!(rendered.iter().any(|line| line == "(no tags listed)"));
}

#[test]
fn renders_tag_set_view_with_summary_and_tip() {
    let rendered = render_tag_set_view(vec!["Created tag v0.2.0 pointing to abc12345".to_string()]);

    assert_eq!(rendered.first(), Some(&"Tag Set".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("confirm updated tag targets"))
    );
}

#[test]
fn renders_tag_delete_view_with_summary_and_tip() {
    let rendered = render_tag_delete_view(vec!["Deleted tag v0.1.0".to_string()]);

    assert_eq!(rendered.first(), Some(&"Tag Delete".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("confirm removed tags"))
    );
}

#[test]
fn decorates_file_list_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["file".to_string(), "list".to_string()],
        vec!["src/main.rs".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"File List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 file listed"))
    );
}

#[test]
fn decorates_file_show_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["file".to_string(), "show".to_string()],
        vec!["fn main() {}".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"File Show".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 content line"))
    );
}

#[test]
fn decorates_file_search_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["file".to_string(), "search".to_string()],
        vec!["src/app.rs:1:fn main()".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"File Search".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 match line"))
    );
}

#[test]
fn decorates_file_annotate_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["file".to_string(), "annotate".to_string()],
        vec!["uxqqtlkq src/app.rs:1 use std::io;".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"File Annotate".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 annotated line"))
    );
}

#[test]
fn decorates_file_track_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["file".to_string(), "track".to_string()],
        vec!["Started tracking src/new.rs".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"File Track".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
}

#[test]
fn decorates_file_untrack_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["file".to_string(), "untrack".to_string()],
        vec!["Stopped tracking target/generated.txt".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"File Untrack".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
}

#[test]
fn decorates_file_chmod_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["file".to_string(), "chmod".to_string()],
        vec!["Updated mode for scripts/deploy.sh".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"File Chmod".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
}

#[test]
fn decorates_tag_list_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["tag".to_string(), "list".to_string()],
        vec!["v0.1.0: abcdef12".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Tag List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 tag listed"))
    );
}

#[test]
fn decorates_tag_set_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["tag".to_string(), "set".to_string()],
        vec!["Created tag v0.2.0".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Tag Set".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
}

#[test]
fn decorates_tag_delete_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["tag".to_string(), "delete".to_string()],
        vec!["Deleted tag v0.1.0".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Tag Delete".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 output line"))
    );
}

#[test]
fn decorates_workspace_root_output_with_root_wrapper() {
    let rendered = decorate_command_output(
        &["workspace".to_string(), "root".to_string()],
        vec!["/Users/joshka/local/jk".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Workspace Root".to_string()));
    assert!(rendered.iter().any(|line| line == "/Users/joshka/local/jk"));
}

#[test]
fn decorates_version_output_with_wrapper() {
    let rendered = decorate_command_output(&["version".to_string()], vec!["jj 0.31.0".to_string()]);

    assert_eq!(rendered.first(), Some(&"Version".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: jj 0.31.0"))
    );
}

#[test]
fn renders_workspace_list_view_with_summary_and_tip() {
    let rendered = render_workspace_list_view(vec![
        "default: abcdef12 main workspace".to_string(),
        "staging: 0123abcd staging workspace".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Workspace List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 workspaces listed"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line == "default: abcdef12 main workspace")
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("workspace add/forget/rename"))
    );
}

#[test]
fn decorates_workspace_list_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["workspace".to_string(), "list".to_string()],
        vec!["default: abcdef12 main workspace".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Workspace List".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 workspace listed"))
    );
}

#[test]
fn renders_git_fetch_view_with_summary_and_tip() {
    let rendered = render_git_fetch_view(vec![
        "Nothing changed.".to_string(),
        "Hint: use -b to fetch bookmarks".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Git Fetch".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: no remote updates fetched"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("run `log` or `status`"))
    );
}

#[test]
fn renders_git_push_view_with_summary_and_tip() {
    let rendered = render_git_push_view(vec![
        "Pushed bookmark main to origin".to_string(),
        "Done.".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Git Push".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Pushed bookmark main to origin"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("confirm-gated with a dry-run preview"))
    );
}

#[test]
fn decorates_git_fetch_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["git".to_string(), "fetch".to_string()],
        vec!["Nothing changed.".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Git Fetch".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: no remote updates fetched"))
    );
}

#[test]
fn decorates_git_push_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["git".to_string(), "push".to_string()],
        vec!["Pushed bookmark main to origin".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Git Push".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Pushed bookmark main to origin"))
    );
}

#[test]
fn renders_bookmark_list_view_with_header_and_tip() {
    let rendered = render_bookmark_list_view(vec![
        "main: abcdef12".to_string(),
        "feature: 0123abcd".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Bookmark List".to_string()));
    assert!(rendered.iter().any(|line| line == "main: abcdef12"));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("bookmark set/move/track"))
    );
}

#[test]
fn renders_bookmark_mutation_view_with_summary_and_tip() {
    let rendered = render_bookmark_mutation_view(
        Some("set"),
        vec!["Moved bookmark main to abcdef12".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Bookmark Set".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Moved bookmark main to abcdef12"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line == "Moved bookmark main to abcdef12")
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("run `bookmark list`"))
    );
}

#[test]
fn decorates_bookmark_set_output_with_wrapper() {
    let rendered = decorate_command_output(
        &[
            "bookmark".to_string(),
            "set".to_string(),
            "main".to_string(),
        ],
        vec!["Moved bookmark main to abcdef12".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Bookmark Set".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Moved bookmark main to abcdef12"))
    );
}

#[test]
fn decorates_all_bookmark_mutation_subcommands_with_wrappers() {
    let cases = [
        ("create", "Bookmark Create"),
        ("set", "Bookmark Set"),
        ("move", "Bookmark Move"),
        ("track", "Bookmark Track"),
        ("untrack", "Bookmark Untrack"),
        ("delete", "Bookmark Delete"),
        ("forget", "Bookmark Forget"),
        ("rename", "Bookmark Rename"),
    ];

    for (subcommand, expected_header) in cases {
        let rendered = decorate_command_output(
            &[
                "bookmark".to_string(),
                subcommand.to_string(),
                "feature".to_string(),
            ],
            vec![format!("{subcommand} bookmark output")],
        );

        assert_eq!(
            rendered.first(),
            Some(&expected_header.to_string()),
            "expected wrapper header for bookmark {subcommand}",
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line")),
            "expected summary line for bookmark {subcommand}",
        );
    }
}

#[test]
fn renders_workspace_mutation_view_with_summary_and_tip() {
    let rendered = render_workspace_mutation_view(
        Some("add"),
        vec!["Created workspace docs at ../jk-docs".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Workspace Add".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Created workspace docs at ../jk-docs"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("run `workspace list`"))
    );
}

#[test]
fn decorates_workspace_add_output_with_wrapper() {
    let rendered = decorate_command_output(
        &[
            "workspace".to_string(),
            "add".to_string(),
            "../jk-docs".to_string(),
        ],
        vec!["Created workspace docs at ../jk-docs".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Workspace Add".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Created workspace docs at ../jk-docs"))
    );
}

#[test]
fn decorates_all_workspace_mutation_subcommands_with_wrappers() {
    let cases = [
        ("add", "Workspace Add"),
        ("forget", "Workspace Forget"),
        ("rename", "Workspace Rename"),
        ("update-stale", "Workspace Update-stale"),
    ];

    for (subcommand, expected_header) in cases {
        let rendered = decorate_command_output(
            &[
                "workspace".to_string(),
                subcommand.to_string(),
                "demo".to_string(),
            ],
            vec![format!("{subcommand} workspace output")],
        );

        assert_eq!(
            rendered.first(),
            Some(&expected_header.to_string()),
            "expected wrapper header for workspace {subcommand}",
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line")),
            "expected summary line for workspace {subcommand}",
        );
    }
}

#[test]
fn renders_operation_restore_view_with_summary_and_tip() {
    let rendered = render_operation_mutation_view(
        "restore",
        vec!["Restored to operation 7699d9773e37".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Operation Restore".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Restored to operation 7699d9773e37"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("operation log` and `status"))
    );
}

#[test]
fn decorates_operation_restore_output_with_wrapper() {
    let rendered = decorate_command_output(
        &[
            "operation".to_string(),
            "restore".to_string(),
            "7699d9773e37".to_string(),
        ],
        vec!["Restored to operation 7699d9773e37".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Operation Restore".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Restored to operation 7699d9773e37"))
    );
}

#[test]
fn decorates_operation_revert_output_with_wrapper() {
    let rendered = decorate_command_output(
        &[
            "operation".to_string(),
            "revert".to_string(),
            "7699d9773e37".to_string(),
        ],
        vec!["Reverted operation 7699d9773e37".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Operation Revert".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Reverted operation 7699d9773e37"))
    );
}

#[test]
fn renders_top_level_mutation_view_with_summary_and_tip() {
    let rendered = render_top_level_mutation_view(
        "commit",
        vec!["Working copy now at: abcdef12 commit message".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Commit Result".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Working copy now at: abcdef12"))
    );
    assert!(rendered.iter().any(|line| line.contains("show` or `log")));
}

#[test]
fn decorates_commit_output_with_wrapper() {
    let rendered = decorate_command_output(
        &[
            "commit".to_string(),
            "-m".to_string(),
            "message".to_string(),
        ],
        vec!["Working copy now at: abcdef12 commit message".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Commit Result".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Working copy now at: abcdef12"))
    );
}

#[test]
fn decorates_rebase_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["rebase".to_string(), "-d".to_string(), "main".to_string()],
        vec!["Rebased 3 commits onto main".to_string()],
    );

    assert_eq!(rendered.first(), Some(&"Rebase Result".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: Rebased 3 commits onto main"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("log`, `status`, or `diff`"))
    );
}

#[test]
fn mutation_summary_uses_signal_line_when_available() {
    let summary =
        top_level_mutation_summary("undo", &[String::from("Undid operation 67d547b627fb")]);
    assert_eq!(summary, "Summary: Undid operation 67d547b627fb");
}

#[test]
fn mutation_summary_detects_signal_lines_for_lifted_rewrite_commands() {
    assert_eq!(
        top_level_mutation_summary(
            "absorb",
            &[String::from("Absorbed changes into 2 destination commits")]
        ),
        "Summary: Absorbed changes into 2 destination commits"
    );
    assert_eq!(
        top_level_mutation_summary(
            "duplicate",
            &[String::from("Duplicated 1 commits onto 0123abcd")]
        ),
        "Summary: Duplicated 1 commits onto 0123abcd"
    );
    assert_eq!(
        top_level_mutation_summary(
            "parallelize",
            &[String::from(
                "Parallelized 3 revisions by making them siblings"
            )]
        ),
        "Summary: Parallelized 3 revisions by making them siblings"
    );
    assert_eq!(
        top_level_mutation_summary("fix", &[String::from("Fixed 5 files across 3 revisions")]),
        "Summary: Fixed 5 files across 3 revisions"
    );
    assert_eq!(
        top_level_mutation_summary(
            "rebase",
            &[String::from(
                "\u{1b}[32mRebased 2 commits onto main\u{1b}[0m"
            )]
        ),
        "Summary: Rebased 2 commits onto main"
    );
}

#[test]
fn mutation_summary_falls_back_to_line_count() {
    let summary = top_level_mutation_summary(
        "abandon",
        &[String::from("Random line"), String::from("Second line")],
    );
    assert_eq!(summary, "Summary: 2 output lines");
}

#[test]
fn bookmark_mutation_summary_uses_signal_line_when_available() {
    let summary = bookmark_mutation_summary(
        "rename",
        &[String::from("Renamed bookmark main to primary")],
    );
    assert_eq!(summary, "Summary: Renamed bookmark main to primary");
}

#[test]
fn bookmark_mutation_summary_falls_back_to_line_count() {
    let summary = bookmark_mutation_summary(
        "set",
        &[String::from("bookmark set output"), String::from("done")],
    );
    assert_eq!(summary, "Summary: 2 output lines");
}

#[test]
fn workspace_mutation_summary_uses_signal_line_when_available() {
    let summary = workspace_mutation_summary(
        "update-stale",
        &[String::from("Updated 2 stale workspaces")],
    );
    assert_eq!(summary, "Summary: Updated 2 stale workspaces");
}

#[test]
fn workspace_mutation_summary_falls_back_to_line_count() {
    let summary = workspace_mutation_summary(
        "rename",
        &[
            String::from("workspace rename output"),
            String::from("done"),
        ],
    );
    assert_eq!(summary, "Summary: 2 output lines");
}

#[test]
fn operation_mutation_summary_uses_signal_line_when_available() {
    let summary = operation_mutation_summary(
        "restore",
        &[String::from("Restored to operation 7699d9773e37")],
    );
    assert_eq!(summary, "Summary: Restored to operation 7699d9773e37");
}

#[test]
fn operation_mutation_summary_falls_back_to_line_count() {
    let summary = operation_mutation_summary(
        "restore",
        &[
            String::from("operation restore output"),
            String::from("done"),
        ],
    );
    assert_eq!(summary, "Summary: 2 output lines");
}

#[test]
fn git_fetch_summary_uses_signal_line_when_available() {
    let summary = git_fetch_summary(&[String::from("Fetched 3 bookmarks from origin")]);
    assert_eq!(summary, "Summary: Fetched 3 bookmarks from origin");
}

#[test]
fn git_fetch_summary_falls_back_to_line_count() {
    let summary = git_fetch_summary(&[String::from("fetch output"), String::from("done")]);
    assert_eq!(summary, "Summary: 2 output lines");
}

#[test]
fn git_push_summary_uses_signal_line_when_available() {
    let summary = git_push_summary(&[String::from("Pushed bookmark main to origin")]);
    assert_eq!(summary, "Summary: Pushed bookmark main to origin");
}

#[test]
fn git_push_summary_falls_back_to_line_count() {
    let summary = git_push_summary(&[String::from("push output"), String::from("done")]);
    assert_eq!(summary, "Summary: 2 output lines");
}

#[test]
fn decorates_all_top_level_mutation_commands_with_wrappers() {
    let cases = [
        ("new", "New Result"),
        ("describe", "Describe Result"),
        ("commit", "Commit Result"),
        ("metaedit", "Metaedit Result"),
        ("edit", "Edit Result"),
        ("next", "Next Result"),
        ("prev", "Prev Result"),
        ("rebase", "Rebase Result"),
        ("squash", "Squash Result"),
        ("split", "Split Result"),
        ("diffedit", "Diffedit Result"),
        ("simplify-parents", "Simplify-parents Result"),
        ("fix", "Fix Result"),
        ("abandon", "Abandon Result"),
        ("undo", "Undo Result"),
        ("redo", "Redo Result"),
        ("restore", "Restore Result"),
        ("revert", "Revert Result"),
        ("absorb", "Absorb Result"),
        ("duplicate", "Duplicate Result"),
        ("parallelize", "Parallelize Result"),
    ];

    for (command, expected_header) in cases {
        let rendered = decorate_command_output(
            &[command.to_string()],
            vec![format!("{command} output line")],
        );

        assert_eq!(
            rendered.first(),
            Some(&expected_header.to_string()),
            "expected wrapper header for command `{command}`",
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line")),
            "expected summary line for command `{command}`",
        );
    }
}

#[test]
fn decorates_interdiff_output_with_wrapper() {
    let rendered = decorate_command_output(
        &[
            "interdiff".to_string(),
            "--from".to_string(),
            "@-".to_string(),
            "--to".to_string(),
            "@".to_string(),
        ],
        vec![
            "Modified regular file src/app.rs:".to_string(),
            "    1      1: use std::collections::HashMap;".to_string(),
        ],
    );

    assert_eq!(rendered.first(), Some(&"Interdiff View".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("compare patch intent"))
    );
}

#[test]
fn decorates_evolog_output_with_wrapper() {
    let rendered = decorate_command_output(
        &[
            "evolog".to_string(),
            "-r".to_string(),
            "abc12345".to_string(),
        ],
        vec![
            "@  abcdef12 user 2026-02-07 00:00:00 0123abcd".to_string(),
            "│  evolve message".to_string(),
        ],
    );

    assert_eq!(rendered.first(), Some(&"Evolution Log".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 evolution entry shown"))
    );
}

#[test]
fn top_level_mutation_wrappers_use_command_specific_tips() {
    let commit_rendered = render_top_level_mutation_view(
        "commit",
        vec!["Working copy now at: abcdef12 commit message".to_string()],
    );
    assert!(
        commit_rendered
            .iter()
            .any(|line| line.contains("show` or `log"))
    );

    let undo_rendered =
        render_top_level_mutation_view("undo", vec!["Undid operation 67d547b627fb".to_string()]);
    assert!(
        undo_rendered
            .iter()
            .any(|line| line.contains("operation log"))
    );

    let rebase_rendered =
        render_top_level_mutation_view("rebase", vec!["Rebased 3 commits onto main".to_string()]);
    assert!(
        rebase_rendered
            .iter()
            .any(|line| line.contains("log`, `status`, or `diff`"))
    );

    let next_rendered = render_top_level_mutation_view(
        "next",
        vec!["Working copy now at: abcdef12 next change".to_string()],
    );
    assert!(
        next_rendered
            .iter()
            .any(|line| line.contains("show` or `diff"))
    );
}

#[test]
fn decorates_gold_command_set_with_native_wrapper_headers() {
    let cases = vec![
        (
            vec!["status"],
            vec!["Working copy changes:", "M src/app.rs"],
            "Status Overview",
        ),
        (
            vec!["show", "-r", "abc12345"],
            vec!["Commit ID: abc12345", "Change ID: qtswpusn"],
            "Show View",
        ),
        (
            vec!["diff", "-r", "abc12345"],
            vec!["Modified regular file src/app.rs:", "@@ -1,1 +1,2 @@"],
            "Diff View",
        ),
        (
            vec!["new"],
            vec!["Working copy now at: abcdef12 new change"],
            "New Result",
        ),
        (
            vec!["describe"],
            vec!["Working copy now at: abcdef12 described change"],
            "Describe Result",
        ),
        (
            vec!["commit"],
            vec!["Working copy now at: abcdef12 commit change"],
            "Commit Result",
        ),
        (
            vec!["next"],
            vec!["Working copy now at: abcdef12 next change"],
            "Next Result",
        ),
        (
            vec!["prev"],
            vec!["Working copy now at: abcdef12 prev change"],
            "Prev Result",
        ),
        (
            vec!["edit"],
            vec!["Working copy now at: abcdef12 edit change"],
            "Edit Result",
        ),
        (
            vec!["rebase", "-d", "main"],
            vec!["Rebased 3 commits onto main"],
            "Rebase Result",
        ),
        (
            vec!["squash", "--into", "main"],
            vec!["Rebased 1 commits onto main"],
            "Squash Result",
        ),
        (
            vec!["split", "-r", "abc12345"],
            vec!["Rebased 1 commits onto abc12345"],
            "Split Result",
        ),
        (
            vec!["diffedit", "-r", "abc12345"],
            vec!["Rebased 1 descendant commits after diff edit"],
            "Diffedit Result",
        ),
        (
            vec!["simplify-parents", "abc12345"],
            vec!["Rebased 2 commits onto simplified parent set"],
            "Simplify-parents Result",
        ),
        (
            vec!["fix", "-s", "abc12345"],
            vec!["Fixed 5 files across 3 revisions"],
            "Fix Result",
        ),
        (
            vec!["abandon", "abc12345"],
            vec!["Abandoned 1 commits."],
            "Abandon Result",
        ),
        (
            vec!["undo"],
            vec!["Undid operation 67d547b627fb"],
            "Undo Result",
        ),
        (
            vec!["redo"],
            vec!["Redid operation 67d547b627fb"],
            "Redo Result",
        ),
        (
            vec!["bookmark", "list"],
            vec!["main: abcdef12", "feature: 0123abcd"],
            "Bookmark List",
        ),
        (
            vec!["bookmark", "create", "feature"],
            vec!["Created bookmark feature at abcdef12"],
            "Bookmark Create",
        ),
        (
            vec!["bookmark", "set", "main"],
            vec!["Moved bookmark main to abcdef12"],
            "Bookmark Set",
        ),
        (
            vec!["bookmark", "move", "main"],
            vec!["Moved bookmark main to abcdef12"],
            "Bookmark Move",
        ),
        (
            vec!["bookmark", "track", "main"],
            vec!["Started tracking bookmark main@origin"],
            "Bookmark Track",
        ),
        (
            vec!["bookmark", "untrack", "main"],
            vec!["Stopped tracking bookmark main@origin"],
            "Bookmark Untrack",
        ),
        (
            vec!["git", "fetch"],
            vec!["Fetched 2 commits from origin"],
            "Git Fetch",
        ),
        (
            vec!["git", "push"],
            vec!["Pushed bookmark main to origin"],
            "Git Push",
        ),
    ];

    for (command, output, expected_header) in cases {
        let command_tokens = command
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>();
        let output_lines = output
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<_>>();

        let rendered = decorate_command_output(&command_tokens, output_lines);
        assert_eq!(
            rendered.first(),
            Some(&expected_header.to_string()),
            "expected native wrapper header for command `{}`",
            command.join(" "),
        );
    }
}

#[test]
fn renders_operation_log_view_with_header_and_tip() {
    let rendered = render_operation_log_view(vec![
        "@  fac974146f86 user 5 seconds ago".to_string(),
        "│  snapshot working copy".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Operation Log".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line == "@  fac974146f86 user 5 seconds ago")
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 operation entry shown"))
    );
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("restore/revert operations"))
    );
}

#[test]
fn renders_operation_show_view_with_header_and_tip() {
    let rendered = render_operation_show_view(vec![
        "7699d9773e37 user 41 seconds ago, lasted 19 milliseconds".to_string(),
        "snapshot working copy".to_string(),
        "Changed commits:".to_string(),
        "○  + qqrxwkpt c245343e feat(help): surface local views".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Operation Details".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: operation 7699d9773e37"))
    );
    assert!(rendered.iter().any(|line| line == "Changed commits:"));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("operation restore/revert remain confirm-gated"))
    );
}

#[test]
fn renders_operation_diff_view_with_header_and_summary() {
    let rendered = render_operation_diff_view(vec![
        "From operation: abc123 (2026-02-07) describe commit old".to_string(),
        "  To operation: def456 (2026-02-07) describe commit new".to_string(),
        "Changed commits:".to_string(),
        "○  + uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
        "Changed working copy default@:".to_string(),
        "+ uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
    ]);

    assert_eq!(rendered.first(), Some(&"Operation Diff".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 changed commit entry shown"))
    );
    assert!(rendered.iter().any(|line| line == "Changed commits:"));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("operation show/restore/revert"))
    );
}

#[test]
fn decorates_operation_diff_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["operation".to_string(), "diff".to_string()],
        vec![
            "From operation: abc123 (2026-02-07) describe commit old".to_string(),
            "Changed commits:".to_string(),
            "○  + uxqqtlkq 722c112d feature".to_string(),
        ],
    );

    assert_eq!(rendered.first(), Some(&"Operation Diff".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 1 changed commit entry shown"))
    );
}

#[test]
fn decorates_operation_show_output_with_wrapper() {
    let rendered = decorate_command_output(
        &["operation".to_string(), "show".to_string()],
        vec![
            "7699d9773e37 user 41 seconds ago, lasted 19 milliseconds".to_string(),
            "snapshot working copy".to_string(),
        ],
    );

    assert_eq!(rendered.first(), Some(&"Operation Details".to_string()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: operation 7699d9773e37"))
    );
}

#[test]
fn inserts_spacing_between_operation_entries() {
    let rendered = render_operation_log_view(vec![
        "@  fac974146f86 user 5 seconds ago".to_string(),
        "│  snapshot working copy".to_string(),
        "○  4a8a95e95f6f user 22 seconds ago".to_string(),
        "│  snapshot working copy".to_string(),
    ]);

    let second_entry_index = rendered
        .iter()
        .position(|line| line == "○  4a8a95e95f6f user 22 seconds ago")
        .expect("second operation entry should exist");
    assert_eq!(rendered.get(second_entry_index - 1), Some(&String::new()));
    assert!(
        rendered
            .iter()
            .any(|line| line.contains("Summary: 2 operation entries shown"))
    );
}

#[test]
fn snapshot_renders_bookmark_list_wrapper_view() {
    let rendered = render_bookmark_list_view(vec![
        "main: abcdef12".to_string(),
        "feature: 0123abcd".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_set_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("set"),
        vec!["Moved bookmark main to abcdef12".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_create_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("create"),
        vec!["Created bookmark feature at abcdef12".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_move_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("move"),
        vec!["Moved bookmark main to abcdef12".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_track_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("track"),
        vec!["Started tracking bookmark main@origin".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_untrack_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("untrack"),
        vec!["Stopped tracking bookmark main@origin".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_delete_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("delete"),
        vec!["Deleted bookmark stale-feature".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_forget_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("forget"),
        vec!["Forgot bookmark stale-feature@origin".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_bookmark_rename_wrapper_view() {
    let rendered = render_bookmark_mutation_view(
        Some("rename"),
        vec!["Renamed bookmark main to primary".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_file_list_wrapper_view() {
    let rendered = render_file_list_view(vec![
        "src/app.rs".to_string(),
        "src/flows.rs".to_string(),
        "README.md".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_file_show_wrapper_view() {
    let rendered = render_file_show_view(vec![
        "fn main() {".to_string(),
        "    println!(\"hi\");".to_string(),
        "}".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_file_search_wrapper_view() {
    let rendered = render_file_search_view(vec![
        "src/app.rs:627:decorate_command_output".to_string(),
        "src/flows.rs:264:plan_command".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_file_annotate_wrapper_view() {
    let rendered = render_file_annotate_view(vec![
        "uxqqtlkq src/app.rs:1 use std::io;".to_string(),
        "qtswpusn src/app.rs:2 fn main() {}".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_file_track_wrapper_view() {
    let rendered = render_file_track_view(vec![
        "Started tracking 2 paths".to_string(),
        "src/new.rs".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_file_untrack_wrapper_view() {
    let rendered =
        render_file_untrack_view(vec!["Stopped tracking target/generated.txt".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_file_chmod_wrapper_view() {
    let rendered = render_file_chmod_view(vec![
        "Updated mode to executable for scripts/deploy.sh".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_resolve_list_wrapper_view() {
    let rendered = render_resolve_list_view(vec![
        "Error: No conflicts found at this revision".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_resolve_action_wrapper_view() {
    let rendered = decorate_command_output(
        &["resolve".to_string(), "src/app.rs".to_string()],
        vec!["Resolved conflict in src/app.rs".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_workspace_list_wrapper_view() {
    let rendered = render_workspace_list_view(vec![
        "default: qqrxwkpt c245343e feature workspace".to_string(),
        "staging: abcdef12 release workspace".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_workspace_add_wrapper_view() {
    let rendered = render_workspace_mutation_view(
        Some("add"),
        vec!["Created workspace docs at ../jk-docs".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_workspace_forget_wrapper_view() {
    let rendered =
        render_workspace_mutation_view(Some("forget"), vec!["Forgot workspace docs".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_workspace_rename_wrapper_view() {
    let rendered = render_workspace_mutation_view(
        Some("rename"),
        vec!["Renamed workspace docs to docs-v2".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_workspace_update_stale_wrapper_view() {
    let rendered = render_workspace_mutation_view(
        Some("update-stale"),
        vec!["Updated 2 stale workspaces".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_status_wrapper_view() {
    let rendered = render_status_view(vec![
        "Working copy changes:".to_string(),
        "M src/app.rs".to_string(),
        "A src/new.rs".to_string(),
        "Working copy  (@) : abcdefgh 0123abcd summary".to_string(),
        "Parent commit (@-): hgfedcba 89abcdef parent".to_string(),
        "Conflicted bookmarks:".to_string(),
        "feature".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_operation_log_wrapper_view() {
    let rendered = render_operation_log_view(vec![
        "@  fac974146f86 user 5 seconds ago".to_string(),
        "│  snapshot working copy".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_operation_show_wrapper_view() {
    let rendered = render_operation_show_view(vec![
        "7699d9773e37 user 41 seconds ago, lasted 19 milliseconds".to_string(),
        "snapshot working copy".to_string(),
        "Changed commits:".to_string(),
        "○  + qqrxwkpt c245343e feature".to_string(),
        "Changed working copy default@:".to_string(),
        "+ qqrxwkpt c245343e feature".to_string(),
        "- qqrxwkpt/1 2fb0ae09 (hidden) feature".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_operation_restore_wrapper_view() {
    let rendered = render_operation_mutation_view(
        "restore",
        vec![
            "Restored to operation 7699d9773e37".to_string(),
            "Working copy now matches restored operation".to_string(),
        ],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_operation_revert_wrapper_view() {
    let rendered = render_operation_mutation_view(
        "revert",
        vec![
            "Reverted operation 7699d9773e37".to_string(),
            "Created undo operation 89abcdef0123".to_string(),
        ],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_operation_diff_wrapper_view() {
    let rendered = render_operation_diff_view(vec![
        "From operation: 67d547b627fb (2026-02-07) describe commit old".to_string(),
        "  To operation: 3c63d5e89db3 (2026-02-07) describe commit new".to_string(),
        "Changed commits:".to_string(),
        "○  + uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
        "   - uxqqtlkq/1 f8cbf93c (hidden) feat(ux): expand read-mode wrappers and shortcuts"
            .to_string(),
        "Changed working copy default@:".to_string(),
        "+ uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
        "- uxqqtlkq/1 f8cbf93c (hidden) feat(ux): expand read-mode wrappers and shortcuts"
            .to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_interdiff_wrapper_view() {
    let rendered = decorate_command_output(
        &[
            "interdiff".to_string(),
            "--from".to_string(),
            "@-".to_string(),
            "--to".to_string(),
            "@".to_string(),
        ],
        vec![
            "Modified regular file src/app.rs:".to_string(),
            "    1      1: use std::collections::HashMap;".to_string(),
        ],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_evolog_wrapper_view() {
    let rendered = decorate_command_output(
        &[
            "evolog".to_string(),
            "-r".to_string(),
            "abc12345".to_string(),
        ],
        vec![
            "@  abcdef12 user 2026-02-07 00:00:00 0123abcd".to_string(),
            "│  evolve message".to_string(),
        ],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_git_fetch_wrapper_view() {
    let rendered = render_git_fetch_view(vec![
        "Nothing changed.".to_string(),
        "Hint: use -b to fetch bookmarks".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_version_wrapper_view() {
    let rendered = render_version_view(vec!["jj 0.31.0".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_git_push_wrapper_view() {
    let rendered = render_git_push_view(vec![
        "Pushed bookmark main to origin".to_string(),
        "Done.".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_commit_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "commit",
        vec!["Working copy now at: abcdef12 commit message".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_metaedit_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "metaedit",
        vec!["Working copy now at: abcdef12 updated metadata".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_new_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "new",
        vec!["Working copy now at: abcdef12 new change".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_describe_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "describe",
        vec!["Working copy now at: abcdef12 updated description".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_edit_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "edit",
        vec!["Working copy now at: abcdef12 edit target".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_next_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "next",
        vec!["Working copy now at: abcdef12 next revision".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_prev_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "prev",
        vec!["Working copy now at: abcdef12 previous revision".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_undo_wrapper_view() {
    let rendered =
        render_top_level_mutation_view("undo", vec!["Undid operation 67d547b627fb".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_abandon_wrapper_view() {
    let rendered =
        render_top_level_mutation_view("abandon", vec!["Abandoned 1 commits.".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_restore_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "restore",
        vec!["Restored 2 paths from revision abcdef12".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_redo_wrapper_view() {
    let rendered =
        render_top_level_mutation_view("redo", vec!["Redid operation 67d547b627fb".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_revert_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "revert",
        vec!["Reverted 2 paths from revision abcdef12".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_rebase_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "rebase",
        vec!["Rebased 3 commits onto 0123abcd".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_squash_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "squash",
        vec!["Rebased 1 commits onto abcdef12".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_split_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "split",
        vec!["Rebased 1 commits onto abcdef12".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_diffedit_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "diffedit",
        vec!["Rebased 1 descendant commits after diff edit".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_simplify_parents_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "simplify-parents",
        vec!["Rebased 2 commits onto simplified parent set".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_fix_wrapper_view() {
    let rendered =
        render_top_level_mutation_view("fix", vec!["Fixed 5 files across 3 revisions".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_absorb_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "absorb",
        vec!["Absorbed changes into 2 destination commits".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_duplicate_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "duplicate",
        vec!["Duplicated 1 commits onto 0123abcd".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_parallelize_wrapper_view() {
    let rendered = render_top_level_mutation_view(
        "parallelize",
        vec!["Parallelized 3 revisions by making them siblings".to_string()],
    );
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_tag_list_wrapper_view() {
    let rendered = render_tag_list_view(vec!["(no output)".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_tag_set_wrapper_view() {
    let rendered = render_tag_set_view(vec!["Created tag v0.2.0 pointing to abc12345".to_string()]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn snapshot_renders_tag_delete_wrapper_view() {
    let rendered = render_tag_delete_view(vec![
        "Deleted tag v0.1.0".to_string(),
        "Deleted tag release-1".to_string(),
    ]);
    insta::assert_snapshot!(rendered.join("\n"));
}

#[test]
fn toggles_patch_flag_for_log_commands() {
    assert_eq!(
        toggle_patch_flag(&["log".to_string(), "-r".to_string(), "all()".to_string()]),
        vec![
            "log".to_string(),
            "-r".to_string(),
            "all()".to_string(),
            "--patch".to_string()
        ]
    );

    assert_eq!(
        toggle_patch_flag(&[
            "log".to_string(),
            "--patch".to_string(),
            "-r".to_string(),
            "all()".to_string()
        ]),
        vec!["log".to_string(), "-r".to_string(), "all()".to_string()]
    );
}

#[test]
fn normal_mode_shortcuts_route_to_expected_flows() {
    let mut fetch_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    fetch_app
        .handle_key(KeyEvent::from(KeyCode::Char('F')))
        .expect("fetch shortcut should be handled");
    assert_eq!(fetch_app.mode, Mode::Prompt);
    assert_eq!(
        fetch_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::GitFetchRemote)
    );

    let mut push_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    push_app
        .handle_key(KeyEvent::from(KeyCode::Char('P')))
        .expect("push shortcut should be handled");
    assert_eq!(push_app.mode, Mode::Prompt);
    assert_eq!(
        push_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::GitPushBookmark)
    );

    let mut rebase_main_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    rebase_main_app
        .handle_key(KeyEvent::from(KeyCode::Char('M')))
        .expect("rebase-main shortcut should be handled");
    assert_eq!(rebase_main_app.mode, Mode::Confirm);
    assert_eq!(
        rebase_main_app.pending_confirm,
        Some(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "main".to_string()
        ])
    );

    let mut rebase_trunk_app =
        App::new(KeybindConfig::load().expect("keybind config should parse"));
    rebase_trunk_app
        .handle_key(KeyEvent::from(KeyCode::Char('T')))
        .expect("rebase-trunk shortcut should be handled");
    assert_eq!(rebase_trunk_app.mode, Mode::Confirm);
    assert_eq!(
        rebase_trunk_app.pending_confirm,
        Some(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "trunk()".to_string()
        ])
    );

    let mut new_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    new_app
        .handle_key(KeyEvent::from(KeyCode::Char('n')))
        .expect("new shortcut should be handled");
    assert_eq!(new_app.mode, Mode::Prompt);
    assert_eq!(
        new_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::NewMessage)
    );

    let mut commit_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    commit_app
        .handle_key(KeyEvent::from(KeyCode::Char('c')))
        .expect("commit shortcut should be handled");
    assert_eq!(commit_app.mode, Mode::Prompt);
    assert_eq!(
        commit_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::CommitMessage)
    );

    let mut describe_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    describe_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    describe_app
        .handle_key(KeyEvent::from(KeyCode::Char('D')))
        .expect("describe shortcut should be handled");
    assert_eq!(describe_app.mode, Mode::Prompt);
    assert_eq!(
        describe_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::DescribeMessage {
            revision: "abcdefgh".to_string()
        })
    );

    let mut bookmark_set_app =
        App::new(KeybindConfig::load().expect("keybind config should parse"));
    bookmark_set_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    bookmark_set_app
        .handle_key(KeyEvent::from(KeyCode::Char('b')))
        .expect("bookmark-set shortcut should be handled");
    assert_eq!(bookmark_set_app.mode, Mode::Prompt);
    assert_eq!(
        bookmark_set_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::BookmarkSet {
            target_revision: "abcdefgh".to_string()
        })
    );

    let mut abandon_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    abandon_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    abandon_app
        .handle_key(KeyEvent::from(KeyCode::Char('a')))
        .expect("abandon shortcut should be handled");
    assert_eq!(abandon_app.mode, Mode::Confirm);
    assert_eq!(
        abandon_app.pending_confirm,
        Some(vec!["abandon".to_string(), "abcdefgh".to_string()])
    );

    let mut rebase_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    rebase_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    rebase_app
        .handle_key(KeyEvent::from(KeyCode::Char('B')))
        .expect("rebase shortcut should be handled");
    assert_eq!(rebase_app.mode, Mode::Prompt);
    assert_eq!(
        rebase_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::RebaseDestination {
            source_revision: "abcdefgh".to_string()
        })
    );

    let mut squash_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    squash_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    squash_app
        .handle_key(KeyEvent::from(KeyCode::Char('S')))
        .expect("squash shortcut should be handled");
    assert_eq!(squash_app.mode, Mode::Prompt);
    assert_eq!(
        squash_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::SquashInto {
            from_revision: "abcdefgh".to_string()
        })
    );

    let mut split_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    split_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    split_app
        .handle_key(KeyEvent::from(KeyCode::Char('X')))
        .expect("split shortcut should be handled");
    assert_eq!(split_app.mode, Mode::Prompt);
    assert_eq!(
        split_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::SplitFileset {
            revision: "abcdefgh".to_string()
        })
    );

    let mut restore_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    restore_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    restore_app
        .handle_key(KeyEvent::from(KeyCode::Char('O')))
        .expect("restore shortcut should be handled");
    assert_eq!(restore_app.mode, Mode::Prompt);
    assert_eq!(
        restore_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::RestoreFrom {
            target_revision: "abcdefgh".to_string()
        })
    );

    let mut revert_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    revert_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
    revert_app
        .handle_key(KeyEvent::from(KeyCode::Char('R')))
        .expect("revert shortcut should be handled");
    assert_eq!(revert_app.mode, Mode::Prompt);
    assert_eq!(
        revert_app.pending_prompt.map(|prompt| prompt.kind),
        Some(PromptKind::RevertRevisions {
            default_revisions: "abcdefgh".to_string(),
            onto_revision: "@".to_string()
        })
    );

    let mut undo_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    undo_app
        .handle_key(KeyEvent::from(KeyCode::Char('u')))
        .expect("undo shortcut should be handled");
    assert_eq!(undo_app.mode, Mode::Confirm);
    assert_eq!(undo_app.pending_confirm, Some(vec!["undo".to_string()]));

    let mut redo_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    redo_app
        .handle_key(KeyEvent::from(KeyCode::Char('U')))
        .expect("redo shortcut should be handled");
    assert_eq!(redo_app.mode, Mode::Confirm);
    assert_eq!(redo_app.pending_confirm, Some(vec!["redo".to_string()]));

    let mut status_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    status_app
        .handle_key(KeyEvent::from(KeyCode::Char('s')))
        .expect("status shortcut should be handled");
    assert_eq!(status_app.mode, Mode::Normal);
    assert_eq!(status_app.last_command, vec!["status".to_string()]);
    assert!(
        status_app
            .lines
            .iter()
            .any(|line| line.contains("Status Overview"))
    );

    let mut log_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    log_app
        .handle_key(KeyEvent::from(KeyCode::Char('l')))
        .expect("log shortcut should be handled");
    assert_eq!(log_app.mode, Mode::Normal);
    assert_eq!(log_app.last_command, vec!["log".to_string()]);
    assert!(!log_app.lines.is_empty());

    let mut operation_log_app =
        App::new(KeybindConfig::load().expect("keybind config should parse"));
    operation_log_app
        .handle_key(KeyEvent::from(KeyCode::Char('o')))
        .expect("operation-log shortcut should be handled");
    assert_eq!(operation_log_app.mode, Mode::Normal);
    assert_eq!(
        operation_log_app.last_command,
        vec!["operation".to_string(), "log".to_string()]
    );
    assert!(
        operation_log_app
            .lines
            .iter()
            .any(|line| line.contains("Operation Log"))
    );

    let mut bookmark_list_app =
        App::new(KeybindConfig::load().expect("keybind config should parse"));
    bookmark_list_app
        .handle_key(KeyEvent::from(KeyCode::Char('L')))
        .expect("bookmark-list shortcut should be handled");
    assert_eq!(bookmark_list_app.mode, Mode::Normal);
    assert_eq!(
        bookmark_list_app.last_command,
        vec!["bookmark".to_string(), "list".to_string()]
    );
    assert!(
        bookmark_list_app
            .lines
            .iter()
            .any(|line| line.contains("Bookmark List"))
            || bookmark_list_app.lines == vec!["(no output)".to_string()]
    );

    let mut resolve_list_app =
        App::new(KeybindConfig::load().expect("keybind config should parse"));
    resolve_list_app
        .handle_key(KeyEvent::from(KeyCode::Char('v')))
        .expect("resolve-list shortcut should be handled");
    assert_eq!(resolve_list_app.mode, Mode::Normal);
    assert_eq!(
        resolve_list_app.last_command,
        vec!["resolve".to_string(), "-l".to_string()]
    );
    assert!(
        resolve_list_app
            .lines
            .iter()
            .any(|line| line.contains("Resolve List"))
    );

    let mut file_list_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    file_list_app
        .handle_key(KeyEvent::from(KeyCode::Char('f')))
        .expect("file-list shortcut should be handled");
    assert_eq!(file_list_app.mode, Mode::Normal);
    assert_eq!(
        file_list_app.last_command,
        vec!["file".to_string(), "list".to_string()]
    );
    assert!(
        file_list_app
            .lines
            .iter()
            .any(|line| line.contains("File List"))
    );

    let mut tag_list_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    tag_list_app
        .handle_key(KeyEvent::from(KeyCode::Char('t')))
        .expect("tag-list shortcut should be handled");
    assert_eq!(tag_list_app.mode, Mode::Normal);
    assert_eq!(
        tag_list_app.last_command,
        vec!["tag".to_string(), "list".to_string()]
    );
    assert!(
        tag_list_app
            .lines
            .iter()
            .any(|line| line.contains("Tag List"))
    );

    let mut root_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    root_app
        .handle_key(KeyEvent::from(KeyCode::Char('w')))
        .expect("root shortcut should be handled");
    assert_eq!(root_app.mode, Mode::Normal);
    assert_eq!(root_app.last_command, vec!["root".to_string()]);
    assert!(
        root_app
            .lines
            .iter()
            .any(|line| line.contains("Workspace Root"))
    );

    let mut help_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    help_app
        .handle_key(KeyEvent::from(KeyCode::Char('?')))
        .expect("help shortcut should be handled");
    assert_eq!(help_app.mode, Mode::Normal);
    assert_eq!(help_app.status_line, "Showing command registry".to_string());
    assert!(
        help_app
            .lines
            .iter()
            .any(|line| line.contains("jk command registry"))
    );

    let mut keymap_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    keymap_app
        .handle_key(KeyEvent::from(KeyCode::Char('K')))
        .expect("keymap shortcut should be handled");
    assert_eq!(keymap_app.mode, Mode::Normal);
    assert_eq!(keymap_app.status_line, "Showing keymap".to_string());
    assert!(
        keymap_app
            .lines
            .iter()
            .any(|line| line.contains("jk keymap"))
    );

    let mut aliases_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    aliases_app
        .handle_key(KeyEvent::from(KeyCode::Char('A')))
        .expect("aliases shortcut should be handled");
    assert_eq!(aliases_app.mode, Mode::Normal);
    assert_eq!(aliases_app.status_line, "Showing alias catalog".to_string());
    assert!(
        aliases_app
            .lines
            .iter()
            .any(|line| line.contains("jk alias catalog"))
    );

    let mut repeat_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    repeat_app.last_command = vec!["status".to_string()];
    repeat_app
        .handle_key(KeyEvent::from(KeyCode::Char('.')))
        .expect("repeat-last shortcut should be handled");
    assert_eq!(repeat_app.mode, Mode::Normal);
    assert_eq!(repeat_app.last_command, vec!["status".to_string()]);
    assert!(
        repeat_app
            .lines
            .iter()
            .any(|line| line.contains("Status Overview"))
    );
}

#[test]
fn command_history_navigates_previous_and_next_entries() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.mode = Mode::Command;
    app.command_history = vec!["status".to_string(), "log -n 5".to_string()];

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("history previous should succeed");
    assert_eq!(app.command_input, "log -n 5".to_string());

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("history previous should stay at oldest");
    assert_eq!(app.command_input, "status".to_string());

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("history next should succeed");
    assert_eq!(app.command_input, "log -n 5".to_string());
}

#[test]
fn command_history_traverses_three_entries_both_directions() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.mode = Mode::Command;
    app.command_history = vec![
        "status".to_string(),
        "log -n 5".to_string(),
        "show @".to_string(),
    ];

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should select newest history entry");
    assert_eq!(app.command_input, "show @");

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should select second entry");
    assert_eq!(app.command_input, "log -n 5");

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("up should select first entry");
    assert_eq!(app.command_input, "status");

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should return to second entry");
    assert_eq!(app.command_input, "log -n 5");

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should return to third entry");
    assert_eq!(app.command_input, "show @");

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("down should move past newest entry to draft");
    assert_eq!(app.command_input, String::new());
}

#[test]
fn snapshot_command_history_three_entry_navigation() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.mode = Mode::Command;
    app.command_history = vec![
        "status".to_string(),
        "log -n 5".to_string(),
        "show @".to_string(),
    ];

    let mut captures = Vec::new();
    captures.push(format!("start\n{}", app.render_for_snapshot(80, 6)));

    for (label, key) in [
        ("up-1", KeyEvent::from(KeyCode::Up)),
        ("up-2", KeyEvent::from(KeyCode::Up)),
        ("up-3", KeyEvent::from(KeyCode::Up)),
        ("down-1", KeyEvent::from(KeyCode::Down)),
        ("down-2", KeyEvent::from(KeyCode::Down)),
        ("down-3", KeyEvent::from(KeyCode::Down)),
    ] {
        app.handle_key(key)
            .expect("command history navigation should succeed");
        captures.push(format!("{label}\n{}", app.render_for_snapshot(80, 6)));
    }

    insta::assert_snapshot!(captures.join("\n\n---\n\n"));
}

#[test]
fn command_history_restores_draft_after_navigation() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.mode = Mode::Command;
    app.command_history = vec!["status".to_string(), "log -n 5".to_string()];
    app.command_input = "boo".to_string();

    app.handle_key(KeyEvent::from(KeyCode::Up))
        .expect("history previous should succeed");
    assert_eq!(app.command_input, "log -n 5".to_string());

    app.handle_key(KeyEvent::from(KeyCode::Down))
        .expect("history next should restore draft");
    assert_eq!(app.command_input, "boo".to_string());
}

#[test]
fn command_mode_ranks_suggestions_by_usage_and_recency() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.record_intent("status");
    app.record_intent("status");
    app.record_intent("show");
    app.record_intent("status");
    app.mode = Mode::Command;
    app.command_input = "s".to_string();

    let rendered = app.render_for_snapshot(120, 6);
    assert!(rendered.contains("suggest: status"));
}

#[test]
fn confirm_mode_supports_dry_run_preview_key() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.execute_with_confirmation(vec!["git".to_string(), "push".to_string()])
        .expect("confirm setup should succeed");
    assert_eq!(app.mode, Mode::Confirm);

    app.handle_key(KeyEvent::from(KeyCode::Char('d')))
        .expect("dry-run preview key should succeed");

    assert_eq!(app.mode, Mode::Confirm);
    assert_eq!(
        app.pending_confirm,
        Some(vec!["git".to_string(), "push".to_string()])
    );
    assert!(
        app.lines
            .iter()
            .any(|line| line.contains("Preview: jj git push --dry-run"))
    );
}

#[test]
fn help_workflow_render_includes_recent_intents() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.record_intent("status");
    app.record_intent("log");

    app.execute_command_line("help inspect")
        .expect("workflow help should render");

    assert_eq!(
        app.status_line,
        "Showing workflow help for `inspect`".to_string()
    );
    assert!(
        app.lines
            .iter()
            .any(|line| line.contains("recent intents: :log, :status"))
    );
}

#[test]
fn footer_shows_onboarding_and_history_hints_until_complete() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.lines = vec![
        "@  abcdefgh 0123abcd message".to_string(),
        "○  hgfedcba 89abcdef parent".to_string(),
    ];
    app.last_command = vec!["log".to_string()];
    app.view_back_stack = vec!["status".to_string()];
    let rendered = app.render_for_snapshot(260, 6);
    assert!(rendered.contains("onboarding:"));
    assert!(rendered.contains("history: back status"));
    assert!(rendered.contains("quick (abcdefgh)"));

    app.onboarding.complete = true;
    let completed = app.render_for_snapshot(260, 6);
    assert!(!completed.contains("onboarding:"));
}

#[test]
fn snapshot_renders_workflow_help_modes() {
    let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
    app.record_intent("status");
    app.record_intent("log");

    let mut captures = Vec::new();
    for workflow in ["inspect", "rewrite", "sync", "recover"] {
        app.execute_command_line(&format!("help {workflow}"))
            .expect("workflow help should render");
        captures.push(format!("{workflow}\n{}", app.render_for_snapshot(100, 10)));
    }

    insta::assert_snapshot!(captures.join("\n\n---\n\n"));
}
