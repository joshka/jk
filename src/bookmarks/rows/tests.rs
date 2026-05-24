use super::metadata::{BookmarkMetadata, BookmarkMetadataCoverage, parse_bookmark_metadata_line};
use super::pairing::pair_bookmark_lines;
use super::*;
use crate::jj::{JjCommand, ViewSpec};

#[test]
fn parses_bookmark_metadata_lines() {
    assert_eq!(
        parse_bookmark_metadata_line(
            r#"{"name":"main","remote":null,"tracked":false,"tracking_present":false,"synced":true,"target_change_id":"wuqolszplkmommqzmxpmmwtwrpuuwkmo","target_commit_id":"2f81d8af4234fef19b84d1495383a55999bb37fa"}"#
        ),
        Some(BookmarkMetadata {
            name: "main".to_owned(),
            remote: None,
            tracked: false,
            tracking_present: false,
            synced: true,
            target_change_id: Some("wuqolszplkmommqzmxpmmwtwrpuuwkmo".to_owned()),
            target_commit_id: Some("2f81d8af4234fef19b84d1495383a55999bb37fa".to_owned()),
        })
    );
    assert_eq!(
        parse_bookmark_metadata_line(
            r#"{"name":"main","remote":"origin","tracked":true,"tracking_present":true,"synced":false,"target_change_id":"","target_commit_id":"","future_field":"ignored"}"#
        ),
        Some(BookmarkMetadata {
            name: "main".to_owned(),
            remote: Some("origin".to_owned()),
            tracked: true,
            tracking_present: true,
            synced: false,
            target_change_id: None,
            target_commit_id: None,
        })
    );
    assert_eq!(parse_bookmark_metadata_line(r#"{"name":"main"}"#), None);
    assert_eq!(parse_bookmark_metadata_line("main\torigin\t\t"), None);
    assert_eq!(parse_bookmark_metadata_line(""), None);
}

#[test]
fn pairs_bookmark_rows_in_render_order() {
    let lines = b"main: okrnpmzv d10e26b6 Update agent repository guidance\nprototype: nqvrkyps f65c4354 docs: add explicit unsupported warning\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
    let metadata = vec![
        bookmark_metadata(
            "main",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        bookmark_metadata(
            "prototype",
            Some("nqvrkypsnqvrkypsnqvrkypsnqvrkyps"),
            Some("f65c4354f65c4354f65c4354f65c4354f65c4354"),
        ),
    ];

    let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::VisibleRowsOnly);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].line_count(), 1);
    assert_eq!(items[0].bookmark_name(), "main");
    assert_eq!(
        items[0].state(),
        &BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Ambiguous,
        }
    );
    assert_eq!(
        items[0].target_change_id(),
        Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv")
    );
    assert_eq!(
        items[0].target_commit_id(),
        Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6")
    );
    assert_eq!(items[1].bookmark_name(), "prototype");
}

#[test]
fn remote_bookmark_rows_do_not_advance_local_metadata() {
    let lines = b"main: okrnpmzv d10e26b6 Update agent repository guidance\nmain@origin: okrnpmzv d10e26b6 Update agent repository guidance\nprototype: nqvrkyps f65c4354 docs: add explicit unsupported warning\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
    let metadata = vec![
        bookmark_metadata(
            "main",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        remote_bookmark_metadata(
            "main",
            "origin",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        bookmark_metadata(
            "prototype",
            Some("nqvrkypsnqvrkypsnqvrkypsnqvrkyps"),
            Some("f65c4354f65c4354f65c4354f65c4354f65c4354"),
        ),
    ];

    let items = pair_bookmark_lines(
        lines,
        metadata,
        BookmarkMetadataCoverage::UnfilteredAllRemotes,
    );

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].bookmark_name(), "main");
    assert!(items[0].is_local());
    assert_eq!(
        items[0].state(),
        &BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        }
    );
    assert_eq!(items[1].bookmark_name(), "main");
    assert!(!items[1].is_local());
    assert_eq!(
        items[1].state(),
        &BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        }
    );
    assert_eq!(
        items[1].target_change_id(),
        Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv")
    );
    assert_eq!(
        items[1].target_commit_id(),
        Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6")
    );
    assert_eq!(items[2].bookmark_name(), "prototype");
    assert!(items[2].is_local());
}

#[test]
fn all_remote_bookmark_metadata_marks_local_only_and_untracked_remote_rows() {
    let lines = b"local-only: okrnpmzv d10e26b6\nremote-only@origin: okrnpmzv d10e26b6\nlocal-with-untracked: okrnpmzv d10e26b6\nlocal-with-untracked@origin: okrnpmzv d10e26b6\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
    let metadata = vec![
        bookmark_metadata(
            "local-only",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        remote_bookmark_metadata(
            "remote-only",
            "origin",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        )
        .with_tracking(false, false, false),
        bookmark_metadata(
            "local-with-untracked",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        remote_bookmark_metadata(
            "local-with-untracked",
            "origin",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        )
        .with_tracking(false, false, false),
    ];

    let items = pair_bookmark_lines(
        lines,
        metadata,
        BookmarkMetadataCoverage::UnfilteredAllRemotes,
    );

    assert_eq!(
        items[0].state(),
        &BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::LocalOnly,
        }
    );
    assert_eq!(
        items[1].state(),
        &BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Absent,
        }
    );
    assert_eq!(
        items[2].state(),
        &BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::UntrackedRemotePresent,
        }
    );
    assert_eq!(
        items[3].state(),
        &BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Present,
        }
    );
}

#[test]
fn bookmark_rows_without_metadata_are_not_marked_local() {
    let lines = b"remote-looking-but-not-trusted: okrnpmzv d10e26b6\n"
        .to_vec()
        .into_text()
        .unwrap()
        .lines;

    let items = pair_bookmark_lines(lines, Vec::new(), BookmarkMetadataCoverage::VisibleRowsOnly);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].bookmark_name(), "remote-looking-but-not-trusted");
    assert!(!items[0].is_local());
    assert_eq!(items[0].state(), &BookmarkRowState::Unknown);
}

#[test]
fn bookmark_rows_with_extra_metadata_are_not_marked_local() {
    let lines = b"main: okrnpmzv d10e26b6\n"
        .to_vec()
        .into_text()
        .unwrap()
        .lines;
    let metadata = vec![
        bookmark_metadata(
            "main",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        remote_bookmark_metadata(
            "main",
            "origin",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
    ];

    let items = pair_bookmark_lines(
        lines,
        metadata,
        BookmarkMetadataCoverage::UnfilteredAllRemotes,
    );

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].bookmark_name(), "main");
    assert_eq!(items[0].target_change_id(), None);
    assert_eq!(items[0].state(), &BookmarkRowState::Unknown);
}

#[test]
fn tracked_local_bookmark_state_preserves_untracked_remote_peer() {
    let lines = b"main: okrnpmzv d10e26b6\nmain@origin: okrnpmzv d10e26b6\nmain@upstream: okrnpmzv d10e26b6\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
    let metadata = vec![
        bookmark_metadata(
            "main",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        remote_bookmark_metadata(
            "main",
            "origin",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        remote_bookmark_metadata(
            "main",
            "upstream",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        )
        .with_tracking(false, false, false),
    ];

    let items = pair_bookmark_lines(
        lines,
        metadata,
        BookmarkMetadataCoverage::UnfilteredAllRemotes,
    );

    assert_eq!(
        items[0].state(),
        &BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: true,
            },
        }
    );
    assert_eq!(
        items[1].state(),
        &BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        }
    );
    assert_eq!(
        items[2].state(),
        &BookmarkRowState::Remote {
            remote: "upstream".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Present,
        }
    );
}

#[test]
fn bookmark_metadata_coverage_requires_unfiltered_all_remotes_args() {
    assert_eq!(
        bookmark_metadata_coverage(&ViewSpec::new(
            JjCommand::Bookmarks,
            vec!["--all-remotes".to_owned()],
        )),
        BookmarkMetadataCoverage::UnfilteredAllRemotes
    );
    assert_eq!(
        bookmark_metadata_coverage(&ViewSpec::new(JjCommand::Bookmarks, vec!["-a".to_owned()])),
        BookmarkMetadataCoverage::UnfilteredAllRemotes
    );

    for args in [
        vec!["--all-remotes", "--remote", "origin"],
        vec!["--all-remotes", "--remote=origin"],
        vec!["--all-remotes", "--tracked"],
        vec!["--all-remotes", "--conflicted"],
        vec!["--all-remotes", "-r", "main"],
        vec!["--all-remotes", "feature"],
    ] {
        let args = args.into_iter().map(str::to_owned).collect();
        assert_eq!(
            bookmark_metadata_coverage(&ViewSpec::new(JjCommand::Bookmarks, args)),
            BookmarkMetadataCoverage::VisibleRowsOnly
        );
    }
}

#[test]
fn filtered_all_remote_bookmark_metadata_does_not_prove_remote_only_exactness() {
    let lines = b"feature@origin: okrnpmzv d10e26b6\n"
        .to_vec()
        .into_text()
        .unwrap()
        .lines;
    let metadata = vec![
        remote_bookmark_metadata(
            "feature",
            "origin",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        )
        .with_tracking(false, false, false),
    ];

    let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::VisibleRowsOnly);

    assert_eq!(
        items[0].state(),
        &BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
            local_peer: BookmarkLocalPeerState::Unknown,
        }
    );
}

#[test]
fn visible_bookmark_metadata_still_proves_local_rows_with_visible_remote_peers() {
    let lines = b"feature: okrnpmzv d10e26b6\nfeature@origin: okrnpmzv d10e26b6\n"
        .to_vec()
        .into_text()
        .unwrap()
        .lines;
    let metadata = vec![
        bookmark_metadata(
            "feature",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
        remote_bookmark_metadata(
            "feature",
            "origin",
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
        ),
    ];

    let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::VisibleRowsOnly);

    assert_eq!(
        items[0].state(),
        &BookmarkRowState::Local {
            tracking: LocalBookmarkRemoteState::Tracked {
                untracked_remote_present: false,
            },
        }
    );
    assert_eq!(
        items[1].state(),
        &BookmarkRowState::Remote {
            remote: "origin".to_owned(),
            tracking: RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                synced: true,
            },
            local_peer: BookmarkLocalPeerState::Present,
        }
    );
}

fn bookmark_metadata(
    name: &str,
    target_change_id: Option<&str>,
    target_commit_id: Option<&str>,
) -> BookmarkMetadata {
    bookmark_metadata_with_remote(name, None, target_change_id, target_commit_id)
}

fn remote_bookmark_metadata(
    name: &str,
    remote: &str,
    target_change_id: Option<&str>,
    target_commit_id: Option<&str>,
) -> BookmarkMetadata {
    bookmark_metadata_with_remote(name, Some(remote), target_change_id, target_commit_id)
}

fn bookmark_metadata_with_remote(
    name: &str,
    remote: Option<&str>,
    target_change_id: Option<&str>,
    target_commit_id: Option<&str>,
) -> BookmarkMetadata {
    BookmarkMetadata {
        name: name.to_owned(),
        remote: remote.map(str::to_owned),
        tracked: remote.is_some(),
        tracking_present: remote.is_some(),
        synced: remote.is_some(),
        target_change_id: target_change_id.map(str::to_owned),
        target_commit_id: target_commit_id.map(str::to_owned),
    }
}
