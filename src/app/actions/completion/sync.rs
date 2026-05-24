use crate::actions::{JjGitFetch, JjGitPush};
use crate::app::App;
use crate::app::actions::ActionPane;
use crate::app::actions::shared::fetch_status_message;
use crate::modes::InteractionMode;

impl App {
    /// Run the push command and leave its finished output on the push pane.
    pub fn confirm_push(
        &mut self,
        push: JjGitPush,
        status_context: Option<String>,
        _viewport_height: u16,
    ) {
        let command_label = push.command_label();
        let result_message = match self.services.run_push(&push) {
            Ok(output) => self.finish_successful_sync_action(output, str::to_owned),
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
        _viewport_height: u16,
    ) {
        let command_label = fetch.command_label();
        let result_message = match self.services.run_git_fetch(&fetch) {
            Ok(output) => self.finish_successful_sync_action(output, |output| {
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
