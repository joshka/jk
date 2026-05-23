//! Small shared completion and wording helpers for action lifecycle modules.
//!
//! These helpers are kept separate because preview and completion code both need identical status
//! context language, and confirmed actions share the same refresh/reveal/status outcome policy
//! without owning command construction.
//!
//! Keep this module limited to lifecycle-completion wording and refresh/reveal helpers. It is not a
//! generic action-policy bucket: action availability, target selection, preview planning, and jj
//! argv construction belong in feature owners, action families, or the app action entry flow.

use std::fmt::Display;

use super::super::{App, clamp_view_to_current_viewport};
use crate::actions::{JjBookmarkMutationPlan, JjGitFetch, JjGitPushTarget};
use crate::app::status_line::StatusLine;
use crate::jj::LogViewMode;

impl App {
    /// Finish a failed action by surfacing the error on the app status line and result pane.
    pub fn finish_failed_action(&mut self, error: impl Display) -> String {
        let message = error.to_string();
        self.status = StatusLine::error(&self.view, message.clone());
        message
    }

    /// Refresh the active view after a successful action and return the result-pane message.
    ///
    /// The post-action clamp uses the live terminal size because command execution can outlive the
    /// frame whose geometry started the action.
    pub fn finish_successful_action(&mut self, output: String, success_suffix: &str) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                clamp_view_to_current_viewport(&mut self.view);
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

    /// Refresh and reveal a change after success when the action has an exact reveal target.
    pub fn finish_successful_action_revealing_change(
        &mut self,
        output: String,
        reveal_change_id: Option<&str>,
        success_suffix: &str,
    ) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                clamp_view_to_current_viewport(&mut self.view);
                let revealed_in_recent = match reveal_change_id {
                    Some(change_id) => {
                        match self.reveal_log_change(change_id, LogViewMode::Recent) {
                            Ok(switched_modes) => {
                                clamp_view_to_current_viewport(&mut self.view);
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
    pub fn finish_successful_sync_action(
        &mut self,
        output: String,
        status_message: impl FnOnce(&str) -> String,
    ) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                clamp_view_to_current_viewport(&mut self.view);
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

/// Return the short display id used in action status context wording.
pub fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

/// Build the push preview status context for the selected push target and remote.
pub fn push_status_context(target: &JjGitPushTarget, remote: &str) -> String {
    match target {
        JjGitPushTarget::Bookmark(name) => {
            format!("bookmark push targets exact bookmark '{name}' on remote {remote}")
        }
        JjGitPushTarget::Revision(revision) => {
            format!("log push targets exact selected revision '{revision}' on remote {remote}")
        }
        JjGitPushTarget::Status => {
            format!("status push uses jj default target resolution for remote {remote}")
        }
    }
}

/// Build the fetch preview status context for the current fetch configuration.
pub fn fetch_status_context(fetch: &JjGitFetch) -> String {
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

/// Build the short status-line message shown after a successful fetch.
pub fn fetch_status_message(fetch: &JjGitFetch, output: &str) -> String {
    match fetch.remote() {
        Some(remote) => format!("fetch {remote}: {output}"),
        None => format!("fetch: {output}"),
    }
}

/// Build bookmark preview status context from the selected mutation and current view label.
pub fn bookmark_status_context(mutation: &JjBookmarkMutationPlan, view_label: &str) -> String {
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
