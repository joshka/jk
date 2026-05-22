use crate::jj::{exact_change_id_revset, exact_string_pattern};

/// Bookmark revision target used by create/set/move plans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjBookmarkTarget {
    /// One exact selected revision from rendered metadata.
    ExactChange(String),
    /// The current working-copy change (`@`).
    CurrentWorkingCopy,
}

/// Forget-target scope derived from visible bookmark metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjBookmarkForgetTarget {
    /// Forget the local bookmark and include its tracked remote peer.
    Local { tracking: String },
    /// Forget one exact remote-only bookmark with no local peer.
    RemoteOnly { remote: String, tracking: String },
}

/// Exact remote bookmark scope for track/untrack plans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjBookmarkTrackingTarget {
    /// Matching local bookmark name when the action originates from a local row.
    local_bookmark: Option<String>,
    /// Exact remote bookmark name to track or untrack.
    remote_bookmark: String,
    /// Exact remote name that owns the remote bookmark.
    remote: String,
    /// User-facing description of the visible trusted metadata state.
    visible_state: String,
}

impl JjBookmarkTarget {
    /// Builds a bookmark target for one exact selected revision.
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    /// Builds a bookmark target for the current working-copy change.
    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    /// Returns the user-facing revision label for this target.
    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    pub(super) fn command_arg(&self) -> String {
        match self {
            Self::ExactChange(change_id) => exact_change_id_revset(change_id),
            Self::CurrentWorkingCopy => "@".to_owned(),
        }
    }

    pub(super) fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => format!("exact selected revision {change_id}"),
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }
}

impl JjBookmarkForgetTarget {
    pub fn local(tracking: impl Into<String>) -> Self {
        Self::Local {
            tracking: tracking.into(),
        }
    }

    pub fn remote_only(remote: impl Into<String>, tracking: impl Into<String>) -> Self {
        Self::RemoteOnly {
            remote: remote.into(),
            tracking: tracking.into(),
        }
    }

    pub(super) fn include_remotes(&self) -> bool {
        matches!(self, Self::RemoteOnly { .. })
    }

    pub(super) fn visible_state(&self) -> String {
        match self {
            Self::Local { tracking } => format!("local bookmark; {tracking}"),
            Self::RemoteOnly { remote, tracking } => {
                format!("remote-only bookmark on {remote}; {tracking}")
            }
        }
    }

    pub(super) fn scope_summary(&self) -> &'static str {
        match self {
            Self::Local { .. } => "local tracked bookmark or local bookmark with remote peer",
            Self::RemoteOnly { .. } => "one remote peer and no local peer; includes remotes",
        }
    }
}

impl JjBookmarkTrackingTarget {
    pub fn new(
        local_bookmark: Option<String>,
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self {
            local_bookmark,
            remote_bookmark: remote_bookmark.into(),
            remote: remote.into(),
            visible_state: visible_state.into(),
        }
    }

    pub fn local(
        local_bookmark: impl Into<String>,
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self::new(
            Some(local_bookmark.into()),
            remote_bookmark,
            remote,
            visible_state,
        )
    }

    pub fn remote_only(
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self::new(None, remote_bookmark, remote, visible_state)
    }

    pub fn remote_bookmark(&self) -> &str {
        &self.remote_bookmark
    }

    pub fn remote(&self) -> &str {
        &self.remote
    }

    pub fn visible_state(&self) -> &str {
        &self.visible_state
    }

    pub(super) fn local_bookmark_label(&self) -> &str {
        self.local_bookmark.as_deref().unwrap_or("absent")
    }

    pub(super) fn remote_pattern(&self) -> String {
        exact_string_pattern(self.remote())
    }

    pub(super) fn bookmark_pattern(&self) -> String {
        exact_string_pattern(self.remote_bookmark())
    }
}
