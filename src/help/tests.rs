use crossterm::event::{KeyCode, KeyModifiers};

use super::*;
use crate::command::KeyPattern;

#[test]
fn project_help_groups_bindings_by_command() {
    let global = [
        Binding::new(KeyPattern::char('r'), Command::Refresh),
        Binding::new(KeyPattern::code(KeyCode::Char('R')), Command::Refresh),
    ];
    let view = [Binding::new(
        KeyPattern::char('s'),
        Command::View(ViewCommand::OpenShow),
    )];

    let sections = project_help(&global, &view, HelpContext::Log);

    assert_eq!(sections[0].title(), "View Switching");
    assert_eq!(sections[0].rows()[0], HelpRow::new("s", "open show"));
    assert_eq!(sections[1].title(), "App");
    assert_eq!(sections[1].rows()[0], HelpRow::new("r, R", "refresh"));
}

#[test]
fn project_help_exposes_push_only_in_supported_contexts() {
    let global = [Binding::new(KeyPattern::char('p'), Command::Push)];

    let graph_help = project_help(&global, &[], HelpContext::Log);
    let status_help = project_help(&global, &[], HelpContext::Status);
    let bookmarks_help = project_help(&global, &[], HelpContext::Bookmarks);
    let show_help = project_help(&global, &[], HelpContext::Show);

    assert_eq!(graph_help[0].title(), "Action Previews");
    assert_eq!(
        graph_help[0].rows()[0],
        HelpRow::new("p", "push selected revision")
    );
    assert_eq!(status_help[0].title(), "Action Previews");
    assert_eq!(status_help[0].rows()[0], HelpRow::new("p", "push status"));
    assert_eq!(
        bookmarks_help[0].rows()[0],
        HelpRow::new("p", "push selected bookmark")
    );
    assert!(!show_help.iter().any(|section| {
        section
            .rows()
            .iter()
            .any(|row| row.keys() == "p" && row.action().contains("push"))
    }));
}

#[test]
fn project_help_exposes_describe_and_commit_in_honest_contexts() {
    let global = [
        Binding::new(KeyPattern::char('D'), Command::Describe),
        Binding::new(KeyPattern::char('C'), Command::Commit),
    ];

    let graph_help = project_help(&global, &[], HelpContext::Log);
    let status_help = project_help(&global, &[], HelpContext::Status);
    let show_help = project_help(&global, &[], HelpContext::Show);

    assert_eq!(graph_help[0].title(), "Action Previews");
    assert_eq!(
        graph_help[0].rows()[0],
        HelpRow::new("D", "describe selected revision")
    );
    assert_eq!(
        graph_help[0].rows()[1],
        HelpRow::new("C", "commit @ and create new change (ignores selection)")
    );
    assert_eq!(status_help[0].rows()[0], HelpRow::new("D", "describe @"));
    assert_eq!(
        status_help[0].rows()[1],
        HelpRow::new("C", "commit @ and create new change")
    );
    assert!(!show_help.iter().any(|section| {
        section
            .rows()
            .iter()
            .any(|row| row.keys() == "D" || row.keys() == "C")
    }));
}

#[test]
fn project_help_exposes_log_edit_next_and_prev_only_in_log() {
    let view = [
        Binding::new(KeyPattern::char('e'), Command::Edit),
        Binding::new(KeyPattern::char(']'), Command::NextEdit),
        Binding::new(KeyPattern::char('['), Command::PrevEdit),
    ];

    let graph_help = project_help(&[], &view, HelpContext::Log);
    let show_help = project_help(&[], &view, HelpContext::Show);

    assert_eq!(graph_help[0].title(), "Action Previews");
    assert_eq!(
        graph_help[0].rows(),
        &[
            HelpRow::new("e", "edit selected revision"),
            HelpRow::new("]", "next editable change from @ (ignores selection)"),
            HelpRow::new("[", "previous editable change from @ (ignores selection)"),
        ]
    );
    assert!(show_help.is_empty());
}

#[test]
fn project_help_exposes_bookmark_mutations_only_in_honest_contexts() {
    const BOOKMARK_RENAME: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('r')];
    const BOOKMARK_FORGET: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('f')];
    const BOOKMARK_TRACK: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('t')];
    const BOOKMARK_UNTRACK: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('u')];
    let global = [
        Binding::new(KeyPattern::char('b'), Command::BookmarkCreate),
        Binding::new(KeyPattern::char('='), Command::BookmarkSet),
        Binding::new(KeyPattern::char('m'), Command::BookmarkMove),
        Binding::sequence(BOOKMARK_RENAME, Command::BookmarkRename),
        Binding::new(KeyPattern::char('x'), Command::BookmarkDelete),
        Binding::sequence(BOOKMARK_FORGET, Command::BookmarkForget),
        Binding::sequence(BOOKMARK_TRACK, Command::BookmarkTrack),
        Binding::sequence(BOOKMARK_UNTRACK, Command::BookmarkUntrack),
    ];

    let graph_help = project_help(&global, &[], HelpContext::Log);
    let status_help = project_help(&global, &[], HelpContext::Status);
    let bookmarks_help = project_help(&global, &[], HelpContext::Bookmarks);
    let show_help = project_help(&global, &[], HelpContext::Show);

    assert_eq!(
        graph_help[0].rows(),
        &[
            HelpRow::new("b", "create bookmark here"),
            HelpRow::new("=", "set bookmark here"),
            HelpRow::new("m", "move bookmark here"),
        ]
    );
    assert_eq!(
        status_help[0].rows(),
        &[
            HelpRow::new("b", "create bookmark at @"),
            HelpRow::new("=", "set bookmark to @"),
            HelpRow::new("m", "move bookmark to @"),
        ]
    );
    assert_eq!(
        bookmarks_help[0].rows(),
        &[
            HelpRow::new("br", "rename local bookmark"),
            HelpRow::new("x", "delete local bookmark"),
            HelpRow::new("bf", "forget tracked or single remote-only bookmark"),
            HelpRow::new("bt", "track exact remote bookmark"),
            HelpRow::new("bu", "untrack exact remote bookmark"),
        ]
    );
    assert!(!show_help.iter().any(|section| {
        section
            .rows()
            .iter()
            .any(|row| row.action().contains("bookmark") || row.action().contains("track"))
    }));
}

#[test]
fn operation_help_exposes_show_and_diff_without_placeholder_text() {
    let view = [
        Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
        Binding::new(KeyPattern::char('d'), Command::View(ViewCommand::OpenDiff)),
    ];

    let sections = project_help(&[], &view, HelpContext::OperationLog);

    assert_eq!(sections[0].title(), "View Switching");
    assert_eq!(sections[0].rows()[0], HelpRow::new("s", "operation show"));
    assert_eq!(sections[0].rows()[1], HelpRow::new("d", "operation diff"));
}

#[test]
fn operation_help_exposes_global_undo_and_redo_recovery_actions() {
    let view = [
        Binding::new(KeyPattern::char('u'), Command::OperationUndo),
        Binding::new(
            KeyPattern::modified_char('r', KeyModifiers::CONTROL),
            Command::OperationRedo,
        ),
    ];

    let sections = project_help(&[], &view, HelpContext::OperationLog);

    assert_eq!(sections[0].title(), "Recovery");
    assert_eq!(
        sections[0].rows()[0],
        HelpRow::new("u", "undo last repo operation (global)")
    );
    assert_eq!(
        sections[0].rows()[1],
        HelpRow::new("C-r", "redo most recently undone operation (global)")
    );
}

#[test]
fn operation_help_exposes_exact_id_action_menu_separately_from_global_recovery() {
    let view = [Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    )];

    let sections = project_help(&[], &view, HelpContext::OperationLog);

    assert_eq!(sections[0].title(), "Action Previews");
    assert_eq!(
        sections[0].rows()[0],
        HelpRow::new("a", "open action menu (preview required)")
    );
}

#[test]
fn document_help_exposes_wrap_commands_only_in_document_contexts() {
    const TOGGLE_WRAP: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('w')];
    const SCROLL_LEFT: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('h')];
    const SCROLL_RIGHT: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('l')];
    let view = [
        Binding::sequence(TOGGLE_WRAP, Command::View(ViewCommand::ToggleWrap)),
        Binding::sequence(SCROLL_LEFT, Command::View(ViewCommand::ScrollLeft)),
        Binding::sequence(SCROLL_RIGHT, Command::View(ViewCommand::ScrollRight)),
    ];

    let file_show_help = project_help(&[], &view, HelpContext::FileShow);
    let graph_help = project_help(&[], &view, HelpContext::Log);

    assert_eq!(file_show_help[0].title(), "Navigation");
    assert_eq!(
        file_show_help[0].rows(),
        &[
            HelpRow::new("zh", "scroll left"),
            HelpRow::new("zl", "scroll right"),
        ]
    );
    assert_eq!(file_show_help[1].title(), "View Switching");
    assert_eq!(
        file_show_help[1].rows(),
        &[HelpRow::new("zw", "toggle wrap")]
    );
    assert!(graph_help.is_empty());
}

#[test]
fn resolve_help_exposes_global_entry_and_inspect_action() {
    let global = [Binding::new(KeyPattern::char('R'), Command::OpenResolve)];
    let view = [Binding::new(
        KeyPattern::code(KeyCode::Enter),
        Command::View(ViewCommand::OpenItem),
    )];

    let sections = project_help(&global, &view, HelpContext::Resolve);

    assert_eq!(sections[0].title(), "View Switching");
    assert_eq!(sections[0].rows()[0], HelpRow::new("R", "resolve"));
    assert_eq!(
        sections[0].rows()[1],
        HelpRow::new("Enter", "inspect conflict")
    );
}
