//! Bookmark action-target resolution for selected rows.
//!
//! The resolver owns the fail-closed policy between rendered bookmark rows
//! and mutation plans. It only enables forget, track, and untrack when the
//! selected row and its visible peers carry enough trusted metadata to name an
//! exact local or remote bookmark target. Unknown, filtered, drifted, or
//! ambiguous metadata stays disabled with the existing user-facing wording.

use color_eyre::Result;
use color_eyre::eyre::eyre;

use super::actions::{JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkTrackingTarget};
use super::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState,
};

pub(super) struct BookmarkActionTargetResolver<'a> {
    /// Currently selected bookmark row, if any.
    selected: Option<&'a BookmarkItem>,
    /// All visible bookmark rows used to resolve peers and ambiguity.
    entries: &'a [BookmarkItem],
    /// View args used to detect whether all-remotes metadata is safely unfiltered.
    spec_args: &'a [String],
}

impl<'a> BookmarkActionTargetResolver<'a> {
    /// Builds the resolver from the selected row, visible rows, and view args.
    pub(super) fn new(
        selected: Option<&'a BookmarkItem>,
        entries: &'a [BookmarkItem],
        spec_args: &'a [String],
    ) -> Self {
        Self {
            selected,
            entries,
            spec_args,
        }
    }

    /// Returns the selected bookmark name only when the row is a trusted local bookmark.
    pub(super) fn selected_local_bookmark_name(&self) -> Option<&'a str> {
        self.selected
            .filter(|entry| matches!(entry.state(), BookmarkRowState::Local { .. }))
            .map(BookmarkItem::bookmark_name)
    }

    /// Resolves the exact forget target for the selected row or reports why it is unsafe.
    pub(super) fn selected_bookmark_forget_target(
        &self,
    ) -> Result<Option<(&'a str, JjBookmarkForgetTarget)>> {
        let Some(entry) = self.selected else {
            return Ok(None);
        };
        let name = entry.bookmark_name();

        match entry.state() {
            BookmarkRowState::Local { tracking } => {
                let target = local_forget_target(tracking)?;
                Ok(Some((name, target)))
            }
            BookmarkRowState::Remote {
                remote,
                tracking,
                local_peer,
            } => {
                let target = self.remote_forget_target(name, remote, tracking, local_peer)?;
                Ok(Some((name, target)))
            }
            BookmarkRowState::Unknown => Err(eyre!(
                "bookmark forget requires trusted bookmark metadata for the selected row"
            )),
        }
    }

    /// Resolves the exact track or untrack target for the selected row or reports why it is unsafe.
    pub(super) fn selected_bookmark_tracking_target(
        &self,
        kind: JjBookmarkMutationKind,
    ) -> Result<Option<(&'a str, JjBookmarkTrackingTarget)>> {
        let Some(entry) = self.selected else {
            return Ok(None);
        };
        match kind {
            JjBookmarkMutationKind::Track => self.selected_bookmark_track_target(entry),
            JjBookmarkMutationKind::Untrack => self.selected_bookmark_untrack_target(entry),
            JjBookmarkMutationKind::Create
            | JjBookmarkMutationKind::Set
            | JjBookmarkMutationKind::Move
            | JjBookmarkMutationKind::Rename
            | JjBookmarkMutationKind::Delete
            | JjBookmarkMutationKind::Forget => Err(eyre!(
                "bookmark tracking target requires track or untrack action"
            )),
        }
    }

    /// Resolves a remote-row forget target only when local-peer and remote uniqueness checks pass.
    fn remote_forget_target(
        &self,
        name: &str,
        remote: &str,
        tracking: &RemoteBookmarkTrackingState,
        local_peer: &BookmarkLocalPeerState,
    ) -> Result<JjBookmarkForgetTarget> {
        match local_peer {
            BookmarkLocalPeerState::Present => {
                return Err(eyre!(
                    "bookmark forget disabled: selected remote bookmark has a local peer"
                ));
            }
            BookmarkLocalPeerState::Unknown => {
                return Err(eyre!(
                    "bookmark forget disabled: selected remote bookmark has unknown local-peer metadata"
                ));
            }
            BookmarkLocalPeerState::Absent => {}
        }

        if matches!(
            tracking,
            RemoteBookmarkTrackingState::Tracked {
                local_present: true,
                ..
            }
        ) {
            return Err(eyre!(
                "bookmark forget disabled: selected remote bookmark tracking metadata says a local peer is present"
            ));
        }

        if self.has_local_bookmark_peer(name) {
            return Err(eyre!(
                "bookmark forget disabled: selected remote bookmark has a local peer"
            ));
        }

        let remote_siblings = self.remote_bookmark_peer_count(name);
        if remote_siblings != 1 {
            return Err(eyre!(
                "bookmark forget disabled: selected remote bookmark is not unique; found {remote_siblings} remote peers named '{name}'"
            ));
        }

        Ok(JjBookmarkForgetTarget::remote_only(
            remote,
            remote_tracking_summary(tracking),
        ))
    }

    /// Resolves an exact track target for the selected bookmark row.
    fn selected_bookmark_track_target(
        &self,
        entry: &'a BookmarkItem,
    ) -> Result<Option<(&'a str, JjBookmarkTrackingTarget)>> {
        let name = entry.bookmark_name();
        require_selected_target("track", entry)?;

        match entry.state() {
            BookmarkRowState::Local { tracking } => self
                .local_bookmark_track_target(name, entry, tracking)
                .map(Some),
            BookmarkRowState::Remote {
                remote,
                tracking,
                local_peer,
            } => remote_bookmark_track_target(name, remote, tracking, local_peer, entry, self)
                .map(|target| Some((name, target))),
            BookmarkRowState::Unknown => Err(eyre!(
                "bookmark track disabled: selected row has unknown bookmark metadata"
            )),
        }
    }

    /// Resolves an exact untrack target for the selected bookmark row.
    fn selected_bookmark_untrack_target(
        &self,
        entry: &'a BookmarkItem,
    ) -> Result<Option<(&'a str, JjBookmarkTrackingTarget)>> {
        let name = entry.bookmark_name();
        require_selected_target("untrack", entry)?;

        match entry.state() {
            BookmarkRowState::Local { tracking } => self
                .local_bookmark_untrack_target(name, entry, tracking)
                .map(Some),
            BookmarkRowState::Remote {
                remote,
                tracking,
                local_peer,
            } => remote_bookmark_untrack_target(name, remote, tracking, local_peer, entry, self)
                .map(|target| Some((name, target))),
            BookmarkRowState::Unknown => Err(eyre!(
                "bookmark untrack disabled: selected row has unknown bookmark metadata"
            )),
        }
    }

    /// Resolves a track target from a trusted local row with one eligible remote peer.
    fn local_bookmark_track_target(
        &self,
        name: &'a str,
        local: &BookmarkItem,
        tracking: &LocalBookmarkRemoteState,
    ) -> Result<(&'a str, JjBookmarkTrackingTarget)> {
        self.require_safe_local_tracking_context("track", name)?;

        match tracking {
            LocalBookmarkRemoteState::UntrackedRemotePresent => {
                let remote =
                    self.exactly_one_remote_peer(name, "track", remote_peer_is_untracked)?;
                require_matching_peer_targets("track", local, remote)?;
                let remote_name = remote_name(remote).expect("remote peer has remote state");
                Ok((
                    name,
                    JjBookmarkTrackingTarget::local(
                        name,
                        remote.bookmark_name(),
                        remote_name,
                        format!(
                            "local bookmark with one untracked remote peer on {}; {}",
                            remote_name,
                            remote_tracking_summary(
                                remote_tracking(remote).expect("remote peer has remote state")
                            )
                        ),
                    ),
                ))
            }
            LocalBookmarkRemoteState::LocalOnly => Err(eyre!(
                "bookmark track disabled: selected local bookmark is local-only"
            )),
            LocalBookmarkRemoteState::Tracked { .. } => Err(eyre!(
                "bookmark track disabled: selected local bookmark already has tracked remote metadata"
            )),
            LocalBookmarkRemoteState::Ambiguous => Err(eyre!(
                "bookmark track disabled: selected local bookmark has ambiguous remote metadata"
            )),
        }
    }

    /// Resolves an untrack target from a trusted local row with one eligible remote peer.
    fn local_bookmark_untrack_target(
        &self,
        name: &'a str,
        local: &BookmarkItem,
        tracking: &LocalBookmarkRemoteState,
    ) -> Result<(&'a str, JjBookmarkTrackingTarget)> {
        self.require_safe_local_tracking_context("untrack", name)?;

        match tracking {
            LocalBookmarkRemoteState::Tracked { .. } => {
                let remote =
                    self.exactly_one_remote_peer(name, "untrack", remote_peer_is_tracked)?;
                require_matching_peer_targets("untrack", local, remote)?;
                let remote_name = remote_name(remote).expect("remote peer has remote state");
                Ok((
                    name,
                    JjBookmarkTrackingTarget::local(
                        name,
                        remote.bookmark_name(),
                        remote_name,
                        format!(
                            "local bookmark with one tracked remote peer on {}; {}",
                            remote_name,
                            remote_tracking_summary(
                                remote_tracking(remote).expect("remote peer has remote state")
                            )
                        ),
                    ),
                ))
            }
            LocalBookmarkRemoteState::LocalOnly => Err(eyre!(
                "bookmark untrack disabled: selected local bookmark is local-only"
            )),
            LocalBookmarkRemoteState::UntrackedRemotePresent => Err(eyre!(
                "bookmark untrack disabled: selected local bookmark has only untracked remote metadata"
            )),
            LocalBookmarkRemoteState::Ambiguous => Err(eyre!(
                "bookmark untrack disabled: selected local bookmark has ambiguous remote metadata"
            )),
        }
    }

    /// Enforces the all-remotes and uniqueness preconditions for local tracking actions.
    fn require_safe_local_tracking_context(&self, action: &str, name: &str) -> Result<()> {
        if !self.has_unfiltered_all_remotes_metadata() {
            return Err(eyre!(
                "bookmark {action} disabled: selected local bookmark requires unfiltered all-remotes metadata"
            ));
        }

        let local_peers = self.local_bookmark_peer_count(name);
        if local_peers != 1 {
            return Err(eyre!(
                "bookmark {action} disabled: selected local bookmark is ambiguous; found {local_peers} local rows named '{name}'"
            ));
        }

        Ok(())
    }

    /// Returns one eligible remote peer or reports why the remote peer set is unsafe.
    fn exactly_one_remote_peer(
        &self,
        name: &str,
        action: &str,
        is_eligible: impl Fn(&BookmarkItem) -> bool,
    ) -> Result<&BookmarkItem> {
        let peers = self
            .entries
            .iter()
            .filter(|entry| entry.bookmark_name() == name && is_eligible(entry))
            .collect::<Vec<_>>();

        match peers.as_slice() {
            [peer] => Ok(peer),
            [] => Err(eyre!(
                "bookmark {action} disabled: selected local bookmark has no exactly typed eligible remote sibling"
            )),
            _ => Err(eyre!(
                "bookmark {action} disabled: selected local bookmark has ambiguous remote siblings; found {} eligible remote rows named '{name}'",
                peers.len()
            )),
        }
    }

    /// Returns the one visible local peer for a bookmark name or reports ambiguity.
    fn visible_local_peer_target(&self, action: &str, name: &str) -> Result<Option<&BookmarkItem>> {
        let peers = self
            .entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Local { .. })
            })
            .collect::<Vec<_>>();

        match peers.as_slice() {
            [] => Ok(None),
            [peer] => Ok(Some(peer)),
            _ => Err(eyre!(
                "bookmark {action} disabled: selected bookmark has ambiguous local peers; found {} local rows named '{name}'",
                peers.len()
            )),
        }
    }

    /// Returns whether any visible local peer exists for the bookmark name.
    fn has_local_bookmark_peer(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| {
            entry.bookmark_name() == name && matches!(entry.state(), BookmarkRowState::Local { .. })
        })
    }

    /// Counts visible local peers with the same exact bookmark name.
    fn local_bookmark_peer_count(&self, name: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Local { .. })
            })
            .count()
    }

    /// Counts visible remote peers with the same exact bookmark name.
    fn remote_bookmark_peer_count(&self, name: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Remote { .. })
            })
            .count()
    }

    /// Returns whether the view args prove unfiltered all-remotes metadata coverage.
    fn has_unfiltered_all_remotes_metadata(&self) -> bool {
        !self.spec_args.is_empty()
            && self
                .spec_args
                .iter()
                .all(|arg| matches!(arg.as_str(), "--all-remotes" | "-a"))
    }
}

fn remote_bookmark_track_target(
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

fn remote_bookmark_untrack_target(
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

fn require_selected_target(action: &str, entry: &BookmarkItem) -> Result<()> {
    if entry.target_change_id().is_none() {
        return Err(eyre!(
            "bookmark {action} disabled: selected bookmark row has no exact target metadata"
        ));
    }
    Ok(())
}

fn require_matching_peer_targets(
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

fn remote_peer_is_tracked(entry: &BookmarkItem) -> bool {
    matches!(
        entry.state(),
        BookmarkRowState::Remote {
            tracking: RemoteBookmarkTrackingState::Tracked { .. },
            ..
        }
    )
}

fn remote_peer_is_untracked(entry: &BookmarkItem) -> bool {
    matches!(
        entry.state(),
        BookmarkRowState::Remote {
            tracking: RemoteBookmarkTrackingState::Untracked { .. },
            ..
        }
    )
}

fn remote_name(entry: &BookmarkItem) -> Option<&str> {
    match entry.state() {
        BookmarkRowState::Remote { remote, .. } => Some(remote),
        BookmarkRowState::Local { .. } | BookmarkRowState::Unknown => None,
    }
}

fn remote_tracking(entry: &BookmarkItem) -> Option<&RemoteBookmarkTrackingState> {
    match entry.state() {
        BookmarkRowState::Remote { tracking, .. } => Some(tracking),
        BookmarkRowState::Local { .. } | BookmarkRowState::Unknown => None,
    }
}

fn local_peer_summary(local_peer: &BookmarkLocalPeerState) -> &'static str {
    match local_peer {
        BookmarkLocalPeerState::Present => "present",
        BookmarkLocalPeerState::Absent => "absent",
        BookmarkLocalPeerState::Unknown => "unknown in visible rows",
    }
}

fn local_forget_target(tracking: &LocalBookmarkRemoteState) -> Result<JjBookmarkForgetTarget> {
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

fn remote_tracking_summary(tracking: &RemoteBookmarkTrackingState) -> String {
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
