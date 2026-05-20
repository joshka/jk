//! Small shared wording helpers for action lifecycle modules.
//!
//! These helpers are kept separate because preview and completion code both need identical
//! status context language without owning command construction.

use crate::jj::{JjBookmarkMutationPlan, JjGitFetch, JjGitPushTarget};

pub(in crate::app::action_lifecycle) fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

pub(in crate::app::action_lifecycle) fn push_status_context(
    target: &JjGitPushTarget,
    remote: &str,
) -> String {
    match target {
        JjGitPushTarget::Bookmark(name) => {
            format!("bookmark push targets exact bookmark '{name}' on remote {remote}")
        }
        JjGitPushTarget::Revision(revision) => {
            format!("graph push targets exact selected revision '{revision}' on remote {remote}")
        }
        JjGitPushTarget::Status => {
            format!("status push uses jj default target resolution for remote {remote}")
        }
    }
}

pub(in crate::app::action_lifecycle) fn fetch_status_context(fetch: &JjGitFetch) -> String {
    match fetch.remote() {
        Some(remote) => {
            let pattern = fetch
                .exact_remote_pattern()
                .expect("remote-specific fetch has a remote pattern");
            format!("fetch targets exact remote '{remote}' with pattern {pattern}")
        }
        None => "default fetch uses jj git fetch remote resolution".to_owned(),
    }
}

pub(in crate::app::action_lifecycle) fn fetch_status_message(
    fetch: &JjGitFetch,
    output: &str,
) -> String {
    match fetch.remote() {
        Some(remote) => format!("fetch {remote}: {output}"),
        None => format!("fetch: {output}"),
    }
}

pub(in crate::app::action_lifecycle) fn bookmark_status_context(
    mutation: &JjBookmarkMutationPlan,
    view_label: &str,
) -> String {
    if let Some(new_name) = mutation.new_name() {
        return format!(
            "bookmark rename '{}' to '{}' from {}",
            mutation.name(),
            new_name,
            view_label
        );
    }

    match mutation.target() {
        Some(target) => format!(
            "bookmark {} '{}' targets {} from {}",
            mutation.kind().label(),
            mutation.name(),
            target.label(),
            view_label
        ),
        None => format!(
            "bookmark {} '{}' from {}",
            mutation.kind().label(),
            mutation.name(),
            view_label
        ),
    }
}
