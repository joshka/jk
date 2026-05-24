use std::collections::HashSet;

use color_eyre::Result;
use serde_json::Value;

use crate::bookmarks::{
    BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState, RemoteBookmarkTrackingState,
};
use crate::jj::{ViewSpec, run_jj_template_lines};
use crate::rendered_rows::{
    boolean_field, non_empty_string_field, optional_string_field, string_field,
};

pub const BOOKMARK_METADATA_TEMPLATE: &str = concat!(
    r#""{\"name\":" ++ json(name)"#,
    r#" ++ ",\"remote\":" ++ json(remote)"#,
    r#" ++ ",\"tracked\":" ++ json(tracked)"#,
    r#" ++ ",\"tracking_present\":" ++ json(tracking_present)"#,
    r#" ++ ",\"synced\":" ++ json(synced)"#,
    r#" ++ ",\"target_change_id\":" ++ json(if(normal_target, normal_target.change_id(), ""))"#,
    r#" ++ ",\"target_commit_id\":" ++ json(if(normal_target, normal_target.commit_id(), ""))"#,
    r#" ++ "}\n""#,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BookmarkMetadataCoverage {
    VisibleRowsOnly,
    UnfilteredAllRemotes,
}

/// Returns whether bookmark metadata covers only visible rows or all remotes without filtering.
pub fn bookmark_metadata_coverage(spec: &ViewSpec) -> BookmarkMetadataCoverage {
    if !spec.args().is_empty()
        && spec
            .args()
            .iter()
            .all(|arg| matches!(arg.as_str(), "--all-remotes" | "-a"))
    {
        BookmarkMetadataCoverage::UnfilteredAllRemotes
    } else {
        BookmarkMetadataCoverage::VisibleRowsOnly
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookmarkMetadata {
    /// Exact bookmark name from metadata.
    pub name: String,
    /// Remote name for remote rows, or `None` for local rows.
    pub remote: Option<String>,
    /// Whether the row is tracked according to `jj` metadata.
    pub tracked: bool,
    /// Whether tracking metadata was present for this row.
    pub tracking_present: bool,
    /// Whether the local and remote targets are already synced.
    pub synced: bool,
    /// Exact target change id when available.
    pub target_change_id: Option<String>,
    /// Exact target commit id when available.
    pub target_commit_id: Option<String>,
}

impl BookmarkMetadata {
    /// Classifies one metadata row into the user-visible bookmark row state.
    pub fn row_state(
        &self,
        coverage: BookmarkMetadataCoverage,
        local_names: &HashSet<String>,
        tracked_remote_names: &HashSet<String>,
        untracked_remote_names: &HashSet<String>,
    ) -> BookmarkRowState {
        match &self.remote {
            None => BookmarkRowState::local(local_bookmark_remote_state(
                self,
                coverage,
                tracked_remote_names,
                untracked_remote_names,
            )),
            Some(remote) => BookmarkRowState::remote(
                remote.clone(),
                remote_bookmark_tracking_state(self),
                local_peer_state(self, coverage, local_names),
            ),
        }
    }

    #[cfg(test)]
    pub fn with_tracking(mut self, tracked: bool, tracking_present: bool, synced: bool) -> Self {
        self.tracked = tracked;
        self.tracking_present = tracking_present;
        self.synced = synced;
        self
    }
}

/// Loads bookmark metadata rows through the bookmark-specific template side channel.
pub fn run_jj_bookmark_metadata(spec: &ViewSpec) -> Result<Vec<BookmarkMetadata>> {
    Ok(run_jj_template_lines(spec, BOOKMARK_METADATA_TEMPLATE)?
        .into_iter()
        .filter_map(|line| parse_bookmark_metadata_line(&line))
        .collect())
}

/// Parses one metadata line and rejects rows that do not match the exact schema.
pub fn parse_bookmark_metadata_line(line: &str) -> Option<BookmarkMetadata> {
    if line.is_empty() {
        return None;
    }

    let Value::Object(fields) = serde_json::from_str::<Value>(line).ok()? else {
        return None;
    };

    let name = string_field(&fields, "name")?;
    if name.is_empty() {
        return None;
    }

    Some(BookmarkMetadata {
        name,
        remote: optional_string_field(&fields, "remote")?,
        tracked: boolean_field(&fields, "tracked")?,
        tracking_present: boolean_field(&fields, "tracking_present")?,
        synced: boolean_field(&fields, "synced")?,
        target_change_id: non_empty_string_field(&fields, "target_change_id"),
        target_commit_id: non_empty_string_field(&fields, "target_commit_id"),
    })
}

fn local_bookmark_remote_state(
    metadata: &BookmarkMetadata,
    coverage: BookmarkMetadataCoverage,
    tracked_remote_names: &HashSet<String>,
    untracked_remote_names: &HashSet<String>,
) -> LocalBookmarkRemoteState {
    let tracked_remote_present = tracked_remote_names.contains(metadata.name.as_str());
    let untracked_remote_present = untracked_remote_names.contains(metadata.name.as_str());

    if tracked_remote_present {
        LocalBookmarkRemoteState::Tracked {
            untracked_remote_present,
        }
    } else if untracked_remote_present {
        LocalBookmarkRemoteState::UntrackedRemotePresent
    } else if coverage == BookmarkMetadataCoverage::UnfilteredAllRemotes {
        LocalBookmarkRemoteState::LocalOnly
    } else {
        LocalBookmarkRemoteState::Ambiguous
    }
}

fn remote_bookmark_tracking_state(metadata: &BookmarkMetadata) -> RemoteBookmarkTrackingState {
    if metadata.tracked {
        RemoteBookmarkTrackingState::Tracked {
            local_present: metadata.tracking_present,
            synced: metadata.synced,
        }
    } else {
        RemoteBookmarkTrackingState::Untracked {
            synced: metadata.synced,
        }
    }
}

fn local_peer_state(
    metadata: &BookmarkMetadata,
    coverage: BookmarkMetadataCoverage,
    local_names: &HashSet<String>,
) -> BookmarkLocalPeerState {
    if local_names.contains(metadata.name.as_str()) {
        BookmarkLocalPeerState::Present
    } else if coverage == BookmarkMetadataCoverage::UnfilteredAllRemotes {
        BookmarkLocalPeerState::Absent
    } else {
        BookmarkLocalPeerState::Unknown
    }
}
