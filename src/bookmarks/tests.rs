use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use ratatui::text::Line;

use super::*;
use crate::bookmarks::actions::{
    JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkTrackingTarget,
};
use crate::bookmarks::{
    BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState, RemoteBookmarkTrackingState,
};
use crate::command::{Binding, Command, CommandContext, ViewCommand, ViewEffect, find_binding};
use crate::jj::{self, ViewSpec};
use crate::search::SearchQuery;
use crate::selection::Selection;

fn bookmark_item(
    text: &[&str],
    bookmark_name: &str,
    target_change_id: Option<&str>,
    target_commit_id: Option<&str>,
) -> BookmarkItem {
    BookmarkItem::new(
        text.iter()
            .map(|line| Line::from((*line).to_owned()))
            .collect::<Vec<_>>(),
        bookmark_name.to_owned(),
        target_change_id.map(str::to_owned),
        target_commit_id.map(str::to_owned),
    )
}

fn bookmarks_view(entries: Vec<BookmarkItem>) -> BookmarksView {
    BookmarksView {
        spec: ViewSpec::new(jj::Command::Bookmarks, Vec::new()),
        entries,
        selection: Selection::default(),
    }
}

fn all_remotes_bookmarks_view(entries: Vec<BookmarkItem>) -> BookmarksView {
    BookmarksView {
        spec: ViewSpec::new(jj::Command::Bookmarks, vec!["--all-remotes".to_owned()]),
        entries,
        selection: Selection::default(),
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

#[test]
fn movement_is_bookmark_item_based() {
    let mut view = bookmarks_view(vec![
        bookmark_item(
            &["@  feature", "│  target change"],
            "feature",
            Some("a"),
            Some("aa"),
        ),
        bookmark_item(&["○  trunk"], "trunk", Some("b"), Some("bb")),
    ]);

    view.execute(
        ViewCommand::MoveDown,
        CommandContext {
            size: ratatui::layout::Size {
                height: 10,
                width: 80,
            },
            search: None,
        },
    );

    assert_eq!(view.selection.index(), 1);
    view.execute(
        ViewCommand::MoveUp,
        CommandContext {
            size: ratatui::layout::Size {
                height: 10,
                width: 80,
            },
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 0);
    view.execute(
        ViewCommand::MoveLast,
        CommandContext {
            size: ratatui::layout::Size {
                height: 10,
                width: 80,
            },
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 1);
}

#[test]
fn copy_options_include_exact_name_and_target_ids_when_known() {
    let view = bookmarks_view(vec![bookmark_item(
        &["@  feature", "│  target change"],
        "feature",
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        Some("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
    )]);

    let options = view.copy_options();

    assert_eq!(options.len(), 4);
    assert_eq!(options[0].label(), "bookmark name");
    assert_eq!(options[0].value(), "feature");
    assert_eq!(options[1].label(), "change id");
    assert_eq!(options[1].value().len(), 64);
    assert_eq!(options[2].label(), "commit id");
    assert_eq!(options[2].value().len(), 64);
    assert_eq!(options[3].label(), "row text");
    assert_eq!(options[3].value(), "@  feature\n│  target change");
}

#[test]
fn refresh_preserves_selected_bookmark_name() {
    let mut view = bookmarks_view(vec![
        bookmark_item(&["@  first"], "first", Some("a"), Some("aa")),
        bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
    ]);
    view.selection.set(1, view.entries.len());

    view.refresh_with_loader(|_| {
        Ok(vec![
            bookmark_item(&["@  second"], "second", Some("b2"), Some("bb2")),
            bookmark_item(&["○  third"], "third", Some("c"), Some("cc")),
        ])
    })
    .unwrap();

    assert_eq!(view.selection.index(), 0);
    assert_eq!(view.entries[0].bookmark_name(), "second");
}

#[test]
fn refresh_clamps_when_selected_bookmark_disappears() {
    let mut view = bookmarks_view(vec![
        bookmark_item(&["@  first"], "first", Some("a"), Some("aa")),
        bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
    ]);
    view.selection.set(1, view.entries.len());

    view.refresh_with_loader(|_| {
        Ok(vec![bookmark_item(
            &["@  first"],
            "first",
            Some("a"),
            Some("aa"),
        )])
    })
    .unwrap();

    assert_eq!(view.selection.index(), 0);
}

#[test]
fn selected_bookmark_name_returns_current_row() {
    let view = bookmarks_view(vec![
        bookmark_item(
            &["@  feature", "│  target"],
            "feature",
            Some("a"),
            Some("aa"),
        ),
        bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
    ]);

    assert_eq!(view.selected_bookmark_name(), Some("feature"));
}

#[test]
fn selected_local_bookmark_name_ignores_nonlocal_rows() {
    let remote = bookmark_item(&["  @origin: abc"], "@origin", None, None).with_local(false);
    let view = bookmarks_view(vec![remote]);

    assert_eq!(view.selected_bookmark_name(), Some("@origin"));
    assert_eq!(view.selected_local_bookmark_name(), None);
}

#[test]
fn selected_local_bookmark_name_ignores_unknown_metadata_rows() {
    let unknown = bookmark_item(&["maybe-local: abc"], "maybe-local", None, None)
        .with_state(BookmarkRowState::Unknown);
    let view = bookmarks_view(vec![unknown]);

    assert_eq!(view.selected_bookmark_name(), Some("maybe-local"));
    assert_eq!(view.selected_local_bookmark_name(), None);
}

#[test]
fn bookmark_forget_target_accepts_tracked_or_remote_backed_local_rows() {
    let tracked =
        bookmark_item(&["main: abc"], "main", None, None).with_state(BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        });
    let view = bookmarks_view(vec![tracked]);

    let (name, target) = view.selected_bookmark_forget_target().unwrap().unwrap();

    assert_eq!(name, "main");
    assert_eq!(
        target,
        JjBookmarkForgetTarget::local("tracked local bookmark")
    );

    let remote_backed = bookmark_item(&["feature: abc"], "feature", None, None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::UntrackedRemotePresent,
        },
    );
    let view = bookmarks_view(vec![remote_backed]);

    let (name, target) = view.selected_bookmark_forget_target().unwrap().unwrap();

    assert_eq!(name, "feature");
    assert_eq!(
        target,
        JjBookmarkForgetTarget::local("local bookmark with an untracked remote peer present")
    );
}

#[test]
fn bookmark_forget_target_accepts_single_remote_without_local_peer() {
    let remote = bookmark_item(&["main@origin: abc"], "main", None, None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Absent,
        },
    );
    let view = bookmarks_view(vec![remote]);

    let (name, target) = view.selected_bookmark_forget_target().unwrap().unwrap();

    assert_eq!(name, "main");
    assert_eq!(
        target,
        JjBookmarkForgetTarget::remote_only("origin", "untracked remote bookmark; synced: false")
    );
}

#[test]
fn bookmark_forget_target_rejects_disabled_metadata_states() {
    let local_only = bookmark_item(&["scratch: abc"], "scratch", None, None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::LocalOnly,
        },
    );
    let view = bookmarks_view(vec![local_only]);
    assert_eq!(
        view.selected_bookmark_forget_target()
            .unwrap_err()
            .to_string(),
        "bookmark forget disabled: selected local bookmark is local-only"
    );

    let ambiguous =
        bookmark_item(&["main: abc"], "main", None, None).with_state(BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Ambiguous,
        });
    let view = bookmarks_view(vec![ambiguous]);
    assert_eq!(
        view.selected_bookmark_forget_target()
            .unwrap_err()
            .to_string(),
        "bookmark forget disabled: selected local bookmark has ambiguous remote metadata"
    );

    let unknown =
        bookmark_item(&["main?: abc"], "main", None, None).with_state(BookmarkRowState::Unknown);
    let view = bookmarks_view(vec![unknown]);
    assert_eq!(
        view.selected_bookmark_forget_target()
            .unwrap_err()
            .to_string(),
        "bookmark forget requires trusted bookmark metadata for the selected row"
    );
}

#[test]
fn bookmark_forget_target_rejects_remote_rows_with_local_or_nonunique_peers() {
    let remote_with_local = bookmark_item(&["main@origin: abc"], "main", None, None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let view = bookmarks_view(vec![remote_with_local]);
    assert_eq!(
        view.selected_bookmark_forget_target()
            .unwrap_err()
            .to_string(),
        "bookmark forget disabled: selected remote bookmark has a local peer"
    );

    let first = bookmark_item(&["main@origin: abc"], "main", None, None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Absent,
        },
    );
    let second = bookmark_item(&["main@upstream: abc"], "main", None, None).with_state(
        BookmarkRowState::Remote {
            remote: "upstream".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Absent,
        },
    );
    let view = bookmarks_view(vec![first, second]);
    assert_eq!(
        view.selected_bookmark_forget_target()
            .unwrap_err()
            .to_string(),
        "bookmark forget disabled: selected remote bookmark is not unique; found 2 remote peers named 'main'"
    );
}

#[test]
fn bookmark_track_target_accepts_remote_untracked_rows() {
    let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Absent,
        },
    );
    let view = bookmarks_view(vec![remote]);

    let (name, target) = view
        .selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
        .unwrap()
        .unwrap();

    assert_eq!(name, "main");
    assert_eq!(
        target,
        JjBookmarkTrackingTarget::remote_only(
            "main",
            "origin",
            "remote bookmark on origin; untracked remote bookmark; synced: false; local peer: absent",
        )
    );
}

#[test]
fn bookmark_untrack_target_accepts_remote_tracked_rows() {
    let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Unknown,
        },
    );
    let view = bookmarks_view(vec![remote]);

    let (name, target) = view
        .selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
        .unwrap()
        .unwrap();

    assert_eq!(name, "main");
    assert_eq!(
        target,
        JjBookmarkTrackingTarget::remote_only(
            "main",
            "origin",
            "remote bookmark on origin; tracked remote bookmark; local present: true; synced: true; local peer: unknown in visible rows",
        )
    );
}

#[test]
fn bookmark_tracking_targets_accept_safe_local_rows() {
    let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        },
    );
    let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let view = all_remotes_bookmarks_view(vec![local, remote]);

    let (name, target) = view
        .selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
        .unwrap()
        .unwrap();

    assert_eq!(name, "main");
    assert_eq!(
        target,
        JjBookmarkTrackingTarget::local(
            "main",
            "main",
            "origin",
            "local bookmark with one tracked remote peer on origin; tracked remote bookmark; local present: true; synced: true",
        )
    );

    let local = bookmark_item(&["topic: abc"], "topic", Some("b"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::UntrackedRemotePresent,
        },
    );
    let remote = bookmark_item(&["topic@origin: abc"], "topic", Some("b"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let view = all_remotes_bookmarks_view(vec![local, remote]);

    let (name, target) = view
        .selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
        .unwrap()
        .unwrap();

    assert_eq!(name, "topic");
    assert_eq!(
        target,
        JjBookmarkTrackingTarget::local(
            "topic",
            "topic",
            "origin",
            "local bookmark with one untracked remote peer on origin; untracked remote bookmark; synced: false",
        )
    );
}

#[test]
fn bookmark_tracking_targets_reject_disabled_row_states() {
    let local_only = bookmark_item(&["scratch: abc"], "scratch", Some("a"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::LocalOnly,
        },
    );
    let view = all_remotes_bookmarks_view(vec![local_only]);

    assert_eq!(
        view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
            .unwrap_err()
            .to_string(),
        "bookmark track disabled: selected local bookmark is local-only"
    );
    assert_eq!(
        view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
            .unwrap_err()
            .to_string(),
        "bookmark untrack disabled: selected local bookmark is local-only"
    );

    let unknown = bookmark_item(&["maybe: abc"], "maybe", Some("a"), None)
        .with_state(BookmarkRowState::Unknown);
    let view = bookmarks_view(vec![unknown]);
    assert_eq!(
        view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
            .unwrap_err()
            .to_string(),
        "bookmark track disabled: selected row has unknown bookmark metadata"
    );
}

#[test]
fn bookmark_tracking_targets_reject_unsafe_local_context_and_drift() {
    let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        },
    );
    let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let view = bookmarks_view(vec![local, remote]);

    assert_eq!(
        view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
            .unwrap_err()
            .to_string(),
        "bookmark untrack disabled: selected local bookmark requires unfiltered all-remotes metadata"
    );

    let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        },
    );
    let remote = bookmark_item(&["main@origin: def"], "main", Some("b"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: false,
            },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let view = all_remotes_bookmarks_view(vec![local, remote]);

    assert_eq!(
        view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
            .unwrap_err()
            .to_string(),
        "bookmark untrack disabled: local and remote bookmark targets differ"
    );
}

#[test]
fn bookmark_tracking_targets_reject_ambiguous_local_remote_siblings() {
    let local = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        },
    );
    let origin = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let upstream = bookmark_item(&["main@upstream: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Remote {
            remote: "upstream".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let view = all_remotes_bookmarks_view(vec![local, origin, upstream]);

    assert_eq!(
        view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Untrack)
            .unwrap_err()
            .to_string(),
        "bookmark untrack disabled: selected local bookmark has ambiguous remote siblings; found 2 eligible remote rows named 'main'"
    );
}

#[test]
fn bookmark_tracking_targets_reject_remote_rows_with_ambiguous_local_peers() {
    let remote = bookmark_item(&["main@origin: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Present,
        },
    );
    let local_one = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::UntrackedRemotePresent,
        },
    );
    let local_two = bookmark_item(&["main: abc"], "main", Some("a"), None).with_state(
        BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::UntrackedRemotePresent,
        },
    );
    let view = all_remotes_bookmarks_view(vec![remote, local_one, local_two]);

    assert_eq!(
        view.selected_bookmark_tracking_target(JjBookmarkMutationKind::Track)
            .unwrap_err()
            .to_string(),
        "bookmark track disabled: selected bookmark has ambiguous local peers; found 2 local rows named 'main'"
    );
}

#[test]
fn search_wraps_by_bookmark_item() {
    let mut view = bookmarks_view(vec![
        bookmark_item(&["@  alpha", "│  target"], "alpha", Some("a"), Some("aa")),
        bookmark_item(&["○  beta"], "beta", Some("b"), Some("bb")),
        bookmark_item(&["○  alpha"], "gamma", Some("c"), Some("cc")),
    ]);
    view.selection.set(1, view.entries.len());
    let query = SearchQuery::new("alpha".to_owned()).unwrap();

    assert_eq!(view.search_matches(&query), 2);
    assert!(view.next_match(&query));
    assert_eq!(view.selection.index(), 2);
    assert!(view.next_match(&query));
    assert_eq!(view.selection.index(), 0);
    assert!(view.previous_match(&query));
    assert_eq!(view.selection.index(), 2);
}

#[test]
fn open_show_uses_target_change_id_and_reports_missing_targets() {
    let view = bookmarks_view(vec![bookmark_item(
        &["@  feature", "│  target change"],
        "feature",
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
        Some("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
    )]);

    assert_eq!(
        find_binding(view.bindings(), key(KeyCode::Enter)).map(Binding::command),
        Some(Command::View(ViewCommand::OpenShow))
    );
    assert_eq!(
        find_binding(view.bindings(), key(KeyCode::Char('s'))).map(Binding::command),
        Some(Command::View(ViewCommand::OpenShow))
    );

    let mut view = view;
    assert_eq!(
        view.execute(
            ViewCommand::OpenShow,
            CommandContext {
                size: ratatui::layout::Size {
                    height: 10,
                    width: 80
                },
                search: None,
            },
        ),
        ViewEffect::OpenDetail(
            jj::Command::Show,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned()
        )
    );

    let mut view = bookmarks_view(vec![bookmark_item(&["○  feature"], "feature", None, None)]);
    assert_eq!(
        view.execute(
            ViewCommand::OpenShow,
            CommandContext {
                size: ratatui::layout::Size {
                    height: 10,
                    width: 80
                },
                search: None,
            },
        ),
        ViewEffect::StatusMessage("selected bookmark has no target change id".to_owned())
    );
}
