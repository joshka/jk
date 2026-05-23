use color_eyre::Result;
use color_eyre::eyre::eyre;

use super::resolver::BookmarkActionTargetResolver;
use crate::bookmarks::actions::{JjBookmarkForgetTarget, JjBookmarkTrackingTarget};
use crate::bookmarks::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState,
};

pub fn remote_bookmark_track_target(
    name: &str,
    remote: &str,
    tracking: &RemoteBookmarkTrackingState,
    local_peer: &BookmarkLocalPeerState,
    entry: &BookmarkItem,
    resolver: &BookmarkActionTargetResolver<'_>,
) -> Result<JjBookmarkTrackingTarget> {
    match tracking {
        RemoteBookmarkTrackingState::Untracked { .. } => {
            let local = validated_visible_local_peer("track", name, local_peer, entry, resolver)?;
            let visible_state = format!(
                "remote bookmark on {remote}; {}; local peer: {}",
                remote_tracking_summary(tracking),
                local_peer_summary(local_peer)
            );
            Ok(tracking_target(name, remote, visible_state, local))
        }
        RemoteBookmarkTrackingState::Tracked { .. } => Err(eyre!(
            "bookmark track disabled: selected remote bookmark is already tracked"
        )),
    }
}

pub fn remote_bookmark_untrack_target(
    name: &str,
    remote: &str,
    tracking: &RemoteBookmarkTrackingState,
    local_peer: &BookmarkLocalPeerState,
    entry: &BookmarkItem,
    resolver: &BookmarkActionTargetResolver<'_>,
) -> Result<JjBookmarkTrackingTarget> {
    match tracking {
        RemoteBookmarkTrackingState::Tracked { .. } => {
            let local = validated_visible_local_peer("untrack", name, local_peer, entry, resolver)?;
            let visible_state = format!(
                "remote bookmark on {remote}; {}; local peer: {}",
                remote_tracking_summary(tracking),
                local_peer_summary(local_peer)
            );
            Ok(tracking_target(name, remote, visible_state, local))
        }
        RemoteBookmarkTrackingState::Untracked { .. } => Err(eyre!(
            "bookmark untrack disabled: selected remote bookmark is not tracked"
        )),
    }
}

pub fn remote_peer_is_tracked(entry: &BookmarkItem) -> bool {
    matches!(
        entry.state(),
        BookmarkRowState::Remote {
            tracking: RemoteBookmarkTrackingState::Tracked { .. },
            ..
        }
    )
}

pub fn remote_peer_is_untracked(entry: &BookmarkItem) -> bool {
    matches!(
        entry.state(),
        BookmarkRowState::Remote {
            tracking: RemoteBookmarkTrackingState::Untracked { .. },
            ..
        }
    )
}

pub fn remote_name(entry: &BookmarkItem) -> Option<&str> {
    match entry.state() {
        BookmarkRowState::Remote { remote, .. } => Some(remote),
        BookmarkRowState::Local { .. } | BookmarkRowState::Unknown => None,
    }
}

pub fn remote_tracking(entry: &BookmarkItem) -> Option<&RemoteBookmarkTrackingState> {
    match entry.state() {
        BookmarkRowState::Remote { tracking, .. } => Some(tracking),
        BookmarkRowState::Local { .. } | BookmarkRowState::Unknown => None,
    }
}

pub fn require_selected_target(action: &str, entry: &BookmarkItem) -> Result<()> {
    if entry.target_change_id().is_none() {
        return Err(eyre!(
            "bookmark {action} disabled: selected bookmark row has no exact target metadata"
        ));
    }
    Ok(())
}

pub fn require_matching_peer_targets(
    action: &str,
    local: &BookmarkItem,
    remote: &BookmarkItem,
) -> Result<()> {
    let Some(local_target) = local.target_change_id() else {
        return Err(eyre!(
            "bookmark {action} disabled: selected local bookmark has no exact target metadata"
        ));
    };
    let Some(remote_target) = remote.target_change_id() else {
        return Err(eyre!(
            "bookmark {action} disabled: selected remote sibling has no exact target metadata"
        ));
    };
    if local_target != remote_target {
        return Err(eyre!(
            "bookmark {action} disabled: local and remote bookmark targets differ"
        ));
    }
    Ok(())
}

pub fn local_forget_target(tracking: &LocalBookmarkRemoteState) -> Result<JjBookmarkForgetTarget> {
    match tracking {
        LocalBookmarkRemoteState::LocalOnly => Err(eyre!(
            "bookmark forget disabled: selected local bookmark is local-only"
        )),
        LocalBookmarkRemoteState::Tracked {
            untracked_remote_present,
        } => Ok(JjBookmarkForgetTarget::local(
            if *untracked_remote_present {
                "tracked local bookmark with an untracked remote peer present"
            } else {
                "tracked local bookmark"
            },
        )),
        LocalBookmarkRemoteState::UntrackedRemotePresent => Ok(JjBookmarkForgetTarget::local(
            "local bookmark with an untracked remote peer present",
        )),
        LocalBookmarkRemoteState::Ambiguous => Err(eyre!(
            "bookmark forget disabled: selected local bookmark has ambiguous remote metadata"
        )),
    }
}

pub fn remote_tracking_summary(tracking: &RemoteBookmarkTrackingState) -> String {
    match tracking {
        RemoteBookmarkTrackingState::Tracked {
            local_present,
            synced,
        } => format!("tracked remote bookmark; local present: {local_present}; synced: {synced}"),
        RemoteBookmarkTrackingState::Untracked { synced } => {
            format!("untracked remote bookmark; synced: {synced}")
        }
    }
}

fn validated_visible_local_peer<'a>(
    action: &str,
    name: &str,
    local_peer: &BookmarkLocalPeerState,
    remote: &BookmarkItem,
    resolver: &'a BookmarkActionTargetResolver<'_>,
) -> Result<Option<&'a BookmarkItem>> {
    match local_peer {
        BookmarkLocalPeerState::Present => {
            let Some(local) = resolver.visible_local_peer_target(action, name)? else {
                return Err(eyre!(
                    "bookmark {action} disabled: selected remote bookmark local-peer metadata drifted"
                ));
            };
            require_matching_peer_targets(action, local, remote)?;
            Ok(Some(local))
        }
        BookmarkLocalPeerState::Absent | BookmarkLocalPeerState::Unknown => Ok(None),
    }
}

fn tracking_target(
    name: &str,
    remote: &str,
    visible_state: String,
    local: Option<&BookmarkItem>,
) -> JjBookmarkTrackingTarget {
    match local {
        Some(local) => JjBookmarkTrackingTarget::local(
            local.bookmark_name(),
            name.to_owned(),
            remote.to_owned(),
            visible_state,
        ),
        None => {
            JjBookmarkTrackingTarget::remote_only(name.to_owned(), remote.to_owned(), visible_state)
        }
    }
}

fn local_peer_summary(local_peer: &BookmarkLocalPeerState) -> &'static str {
    match local_peer {
        BookmarkLocalPeerState::Present => "present",
        BookmarkLocalPeerState::Absent => "absent",
        BookmarkLocalPeerState::Unknown => "unknown in visible rows",
    }
}
