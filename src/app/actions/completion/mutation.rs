use crate::actions::{JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, JjFileMutationPlan};
use crate::modes::InteractionMode;

use super::super::super::App;
use super::super::ActionPane;

impl App {
    /// Run the describe command and leave its finished output on the describe pane.
    pub fn confirm_describe(
        &mut self,
        describe: JjDescribePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = describe.command_label();
        let reveal_change_id = describe.target().exact_change_id().map(str::to_owned);
        let result_message = match self.services.run_describe(&describe) {
            Ok(output) => self.finish_successful_action_revealing_change(
                output,
                reveal_change_id.as_deref(),
                viewport_height,
                " | jj undo",
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::DescribePreview {
            describe,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the commit command and leave its finished output on the commit pane.
    pub fn confirm_commit(
        &mut self,
        commit: JjCommitPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = commit.command_label();
        let result_message = match self.services.run_commit(&commit) {
            Ok(output) => self.finish_successful_action(
                output,
                viewport_height,
                " | new working-copy change created on top | jj undo",
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::CommitPreview {
            commit,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the bookmark mutation and leave its finished output on the bookmark pane.
    pub fn confirm_bookmark_mutation(
        &mut self,
        mutation: JjBookmarkMutationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = mutation.command_label();
        let result_message = match self.services.run_bookmark_mutation(&mutation) {
            Ok(output) => self.finish_successful_action(output, viewport_height, " | jj undo"),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::BookmarkMutationPreview {
            mutation,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the file mutation and leave its finished output on the file-mutation pane.
    pub fn confirm_file_mutation(
        &mut self,
        mutation: JjFileMutationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = mutation.command_label();
        let result_message = match self.services.run_file_mutation(&mutation) {
            Ok(output) => {
                self.finish_successful_action(output, viewport_height, " | jj undo | jj op show -p")
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::FileMutationPreview {
            mutation,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }
}
