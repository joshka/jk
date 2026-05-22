use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::bookmarks::{BookmarkItem, BookmarkRowState};

/// Visible bookmark peer and metadata-coverage checks for fail-closed actions.
///
/// This owner answers the shared question "what can the currently visible
/// bookmark rows prove safely?" after the bookmarks feature has already decided
/// that an action needs exact local or remote peer state.
pub struct VisibleBookmarkPeers<'a> {
    entries: &'a [BookmarkItem],
    spec_args: &'a [String],
}

impl<'a> VisibleBookmarkPeers<'a> {
    pub fn new(entries: &'a [BookmarkItem], spec_args: &'a [String]) -> Self {
        Self { entries, spec_args }
    }

    pub fn visible_local_peer_target(
        &self,
        action: &str,
        name: &str,
    ) -> Result<Option<&BookmarkItem>> {
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

    pub fn has_local_bookmark_peer(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| {
            entry.bookmark_name() == name && matches!(entry.state(), BookmarkRowState::Local { .. })
        })
    }

    pub fn remote_bookmark_peer_count(&self, name: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Remote { .. })
            })
            .count()
    }

    pub fn require_safe_local_tracking_context(&self, action: &str, name: &str) -> Result<()> {
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

    pub fn exactly_one_remote_peer(
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

    fn local_bookmark_peer_count(&self, name: &str) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.bookmark_name() == name
                    && matches!(entry.state(), BookmarkRowState::Local { .. })
            })
            .count()
    }

    fn has_unfiltered_all_remotes_metadata(&self) -> bool {
        !self.spec_args.is_empty()
            && self
                .spec_args
                .iter()
                .all(|arg| matches!(arg.as_str(), "--all-remotes" | "-a"))
    }
}
