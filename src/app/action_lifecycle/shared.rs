//! Small shared completion and wording helpers for action lifecycle modules.
//!
//! These helpers are kept separate because preview and completion code both need identical status
//! context language, and confirmed actions share the same refresh/reveal/status outcome policy
//! without owning command construction.

use std::fmt::Display;

use crate::app_status::StatusLine;
use crate::jj::LogViewMode;
use crate::jj_actions::{JjBookmarkMutationPlan, JjGitFetch, JjGitPushTarget};

use super::super::{App, current_viewport_width};

impl App {
    pub(in crate::app::action_lifecycle) fn finish_failed_action(
        &mut self,
        error: impl Display,
    ) -> String {
        let message = error.to_string();
        self.status = StatusLine::error(&self.view, message.clone());
        message
    }

    pub(in crate::app::action_lifecycle) fn finish_successful_action(
        &mut self,
        output: String,
        viewport_height: u16,
        success_suffix: &str,
    ) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                let message = format!("{}{}", output.trim(), success_suffix);
                self.status = StatusLine::with_message(&self.view, message.as_str());
                message
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                format!(
                    "{} | refresh failed: {error}{success_suffix}",
                    output.trim()
                )
            }
        }
    }

    pub(in crate::app::action_lifecycle) fn finish_successful_action_revealing_change(
        &mut self,
        output: String,
        reveal_change_id: Option<&str>,
        viewport_height: u16,
        success_suffix: &str,
    ) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                let revealed_in_recent = match reveal_change_id {
                    Some(change_id) => {
                        match self.reveal_graph_change(change_id, LogViewMode::Recent) {
                            Ok(switched_modes) => {
                                self.view.clamp(viewport_height, current_viewport_width());
                                Some(switched_modes)
                            }
                            Err(error) => {
                                self.status = StatusLine::error(&self.view, error.to_string());
                                return format!(
                                    "{} | reveal failed: {error}{success_suffix}",
                                    output.trim()
                                );
                            }
                        }
                    }
                    None => None,
                };

                let message = match revealed_in_recent {
                    Some(true) => {
                        format!("{} | showing recent work{success_suffix}", output.trim())
                    }
                    Some(false) | None => format!("{}{}", output.trim(), success_suffix),
                };
                self.status = StatusLine::with_message(&self.view, message.as_str());
                message
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                format!(
                    "{} | refresh failed: {error}{success_suffix}",
                    output.trim()
                )
            }
        }
    }

    /// Finish sync commands that preserve raw command output as the result-pane body.
    pub(in crate::app::action_lifecycle) fn finish_successful_sync_action(
        &mut self,
        output: String,
        viewport_height: u16,
        status_message: impl FnOnce(&str) -> String,
    ) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                self.status = StatusLine::with_message(&self.view, status_message(output.as_str()));
                output
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                if output.is_empty() {
                    format!("refresh failed: {error}")
                } else {
                    format!("{output}\nrefresh failed: {error}")
                }
            }
        }
    }
}

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
