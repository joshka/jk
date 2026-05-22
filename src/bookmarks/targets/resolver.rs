use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::bookmarks::actions::{
    JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkTrackingTarget,
};
use crate::bookmarks::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState,
};

use super::helpers::{
    local_forget_target, remote_bookmark_track_target, remote_bookmark_untrack_target, remote_name,
    remote_peer_is_tracked, remote_peer_is_untracked, remote_tracking, remote_tracking_summary,
    require_matching_peer_targets, require_selected_target,
};
use super::peers::VisibleBookmarkPeers;

pub struct BookmarkActionTargetResolver<'a> {
    /// Currently selected bookmark row, if any.
    selected: Option<&'a BookmarkItem>,
    /// Visible bookmark peers and metadata coverage used for exact action targeting.
    peers: VisibleBookmarkPeers<'a>,
}

impl<'a> BookmarkActionTargetResolver<'a> {
    /// Builds the resolver from the selected row, visible rows, and view args.
    pub fn new(
        selected: Option<&'a BookmarkItem>,
        entries: &'a [BookmarkItem],
        spec_args: &'a [String],
    ) -> Self {
        Self {
            selected,
            peers: VisibleBookmarkPeers::new(entries, spec_args),
        }
    }

    /// Returns the selected bookmark name only when the row is a trusted local bookmark.
    pub fn selected_local_bookmark_name(&self) -> Option<&'a str> {
        self.selected
            .filter(|entry| matches!(entry.state(), BookmarkRowState::Local { .. }))
            .map(BookmarkItem::bookmark_name)
    }

    /// Resolves the exact forget target for the selected row or reports why it is unsafe.
    pub fn selected_bookmark_forget_target(
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
    pub fn selected_bookmark_tracking_target(
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

    pub fn visible_local_peer_target(
        &self,
        action: &str,
        name: &str,
    ) -> Result<Option<&BookmarkItem>> {
        self.peers.visible_local_peer_target(action, name)
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

        if self.peers.has_local_bookmark_peer(name) {
            return Err(eyre!(
                "bookmark forget disabled: selected remote bookmark has a local peer"
            ));
        }

        let remote_siblings = self.peers.remote_bookmark_peer_count(name);
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
        self.peers
            .require_safe_local_tracking_context("track", name)?;

        match tracking {
            LocalBookmarkRemoteState::UntrackedRemotePresent => {
                let remote =
                    self.peers
                        .exactly_one_remote_peer(name, "track", remote_peer_is_untracked)?;
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
        self.peers
            .require_safe_local_tracking_context("untrack", name)?;

        match tracking {
            LocalBookmarkRemoteState::Tracked { .. } => {
                let remote =
                    self.peers
                        .exactly_one_remote_peer(name, "untrack", remote_peer_is_tracked)?;
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
}
