use crate::actions::{JjGitFetch, JjGitPush, JjGitPushTarget};
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;

use super::super::super::{App, current_viewport_width};
use super::super::ActionPane;
use super::super::shared::{fetch_status_context, fetch_status_message, push_status_context};

impl App {
    /// Open the fetch preview for one chosen remote.
    pub(in crate::app) fn open_fetch_preview(&mut self, remote: String) {
        let fetch = JjGitFetch::for_remote(remote);
        let status_context = Some(fetch_status_context(&fetch));

        let command_label = fetch.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            fetch.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::FetchPreview { fetch, output };
    }

    /// Run default fetch immediately and keep its output on the shared fetch-preview surface.
    pub(in crate::app) fn fetch(&mut self, viewport_height: u16) {
        let fetch = JjGitFetch::default_remotes();
        let command_label = fetch.command_label();
        let status_context = Some(fetch_status_context(&fetch));
        let result_message = match self.services.run_git_fetch(&fetch) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    self.status =
                        StatusLine::with_message(&self.view, fetch_status_message(&fetch, &output));
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
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::FetchPreview {
            fetch,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Open the push preview for one chosen target/remote pair.
    pub(in crate::app) fn open_push_preview(&mut self, target: JjGitPushTarget, remote: String) {
        let status_context = Some(push_status_context(&target, remote.as_str()));
        let push = match target {
            JjGitPushTarget::Bookmark(name) => JjGitPush::for_bookmark(name).with_remote(remote),
            JjGitPushTarget::Revision(name) => JjGitPush::for_revision(name).with_remote(remote),
            JjGitPushTarget::Status => JjGitPush::for_status().with_remote(remote),
        };

        let command_label = push.command_label(true);
        let output = self.preview_output_with_error_status(
            command_label,
            self.services.load_push_preview(&push),
            std::convert::identity,
            status_context,
        );
        self.mode = InteractionMode::PushPreview { push, output };
    }
}
