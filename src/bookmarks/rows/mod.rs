//! Rendered bookmark rows plus trusted metadata pairing.
//!
//! This module owns the bookmark-specific bridge between rendered `jj bookmark
//! list` rows and the template-driven metadata used for exact bookmark names,
//! target ids, and local/remote classification. Pairing stays fail-closed: if
//! the metadata drifts or row counts do not match the rendered rows, items keep
//! their rendered text but degrade to unknown bookmark state and empty targets.

mod metadata;
mod pairing;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;

use crate::jj::{ColorMode, ViewSpec, run_jj};
use crate::rendered_rows::line_text;

use metadata::{bookmark_metadata_coverage, run_jj_bookmark_metadata};
use pairing::pair_bookmark_lines;

/// One selectable bookmark item parsed from rendered bookmark output.
#[derive(Clone, Debug)]
pub struct BookmarkItem {
    /// Preserved rendered lines for one selectable bookmark row.
    lines: Vec<Line<'static>>,
    /// Exact bookmark name paired from metadata or degraded from rendered text.
    name: String,
    /// Exact target change id when trusted metadata still provides it.
    target_change_id: Option<String>,
    /// Exact target commit id when trusted metadata still provides it.
    target_commit_id: Option<String>,
    /// Local/remote/tracking classification derived from trusted metadata coverage.
    state: BookmarkRowState,
}

impl BookmarkItem {
    /// Builds one bookmark row from rendered lines and trusted target ids.
    pub fn new(
        lines: Vec<Line<'static>>,
        name: String,
        target_change_id: Option<String>,
        target_commit_id: Option<String>,
    ) -> Self {
        Self {
            lines,
            name,
            target_change_id,
            target_commit_id,
            state: BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Ambiguous,
            },
        }
    }

    /// Returns the preserved rendered lines for this bookmark row.
    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    /// Returns the number of rendered lines in this bookmark row.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the exact bookmark name for this row.
    pub fn bookmark_name(&self) -> &str {
        &self.name
    }

    /// Returns the exact target change id when metadata still proves it.
    pub fn target_change_id(&self) -> Option<&str> {
        self.target_change_id.as_deref()
    }

    /// Returns the exact target commit id when metadata still proves it.
    pub fn target_commit_id(&self) -> Option<&str> {
        self.target_commit_id.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn is_local(&self) -> bool {
        matches!(self.state, BookmarkRowState::Local { .. })
    }

    /// Returns the local/remote/tracking classification for this row.
    pub fn state(&self) -> &BookmarkRowState {
        &self.state
    }

    #[cfg(test)]
    pub(crate) fn with_local(mut self, local: bool) -> Self {
        self.state = if local {
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Ambiguous,
            }
        } else {
            BookmarkRowState::Unknown
        };
        self
    }

    #[cfg(test)]
    pub(crate) fn with_state(mut self, state: BookmarkRowState) -> Self {
        self.state = state;
        self
    }

    /// Returns plain rendered row text for copy and search surfaces.
    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BookmarkRowState {
    /// A local bookmark row plus its relationship to remote peers.
    Local {
        /// Remote-peer state derived from the trusted metadata coverage.
        tracking: LocalBookmarkRemoteState,
    },
    /// A remote bookmark row plus its tracking and local-peer status.
    Remote {
        /// Remote name shown for the remote bookmark row.
        remote: String,
        /// Tracking state for this exact remote bookmark.
        tracking: RemoteBookmarkTrackingState,
        /// Whether a local peer is known for the same bookmark name.
        local_peer: BookmarkLocalPeerState,
    },
    /// Metadata drifted or was incomplete, so only rendered text remains trusted.
    Unknown,
}

impl BookmarkRowState {
    pub(super) fn local(tracking: LocalBookmarkRemoteState) -> Self {
        Self::Local { tracking }
    }

    pub(super) fn remote(
        remote: String,
        tracking: RemoteBookmarkTrackingState,
        local_peer: BookmarkLocalPeerState,
    ) -> Self {
        Self::Remote {
            remote,
            tracking,
            local_peer,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalBookmarkRemoteState {
    /// No remote peer is known for this local bookmark.
    LocalOnly,
    /// A tracked remote peer exists, and another untracked peer may also be visible.
    Tracked { untracked_remote_present: bool },
    /// Only untracked remote peers are known for this local bookmark.
    UntrackedRemotePresent,
    /// Metadata coverage is not strong enough to classify the local bookmark safely.
    Ambiguous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteBookmarkTrackingState {
    /// The remote bookmark is tracked locally, with local presence and sync state.
    Tracked { local_present: bool, synced: bool },
    /// The remote bookmark is visible but not tracked locally.
    Untracked { synced: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BookmarkLocalPeerState {
    /// A local bookmark with the same exact name is known to exist.
    Present,
    /// No local bookmark with the same exact name is known to exist.
    Absent,
    /// Metadata coverage cannot prove whether a local peer exists.
    Unknown,
}

/// Loads rendered bookmark rows and pairs them with trusted metadata when coverage permits.
pub fn load_bookmark_entries(spec: &ViewSpec) -> Result<Vec<BookmarkItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;
    let metadata = run_jj_bookmark_metadata(spec)?;
    Ok(pair_bookmark_lines(
        lines,
        metadata,
        bookmark_metadata_coverage(spec),
    ))
}

#[cfg(test)]
mod tests;
