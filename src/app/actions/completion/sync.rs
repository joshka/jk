use crate::actions::{JjGitFetch, JjGitPush};
use crate::modes::InteractionMode;

use super::super::super::App;
use super::super::ActionPane;
use super::super::shared::fetch_status_message;

impl App {
    /// Run the push command and leave its finished output on the push pane.
    pub fn confirm_push(
        &mut self,
        push: JjGitPush,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = push.command_label(false);
        let result_message = match self.services.run_push(&push) {
            Ok(output) => {
                self.finish_successful_sync_action(output, viewport_height, str::to_owned)
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::PushPreview {
            push,
            output: ActionPane::finished(command_label, result_message, status_context),
        }
    }

    /// Run the fetch command and leave its finished output on the fetch pane.
    pub fn confirm_fetch(
        &mut self,
        fetch: JjGitFetch,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = fetch.command_label();
        let result_message = match self.services.run_git_fetch(&fetch) {
            Ok(output) => self.finish_successful_sync_action(output, viewport_height, |output| {
                fetch_status_message(&fetch, output)
            }),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::FetchPreview {
            fetch,
            output: ActionPane::finished(command_label, result_message, status_context),
        }
    }
}
