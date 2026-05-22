use crate::actions::{JjOperationRecovery, JjOperationTarget};
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;

use super::super::super::{App, current_viewport_width};
use super::super::ActionPane;

impl App {
    /// Run the undo/redo operation and leave its finished output on the recovery pane.
    pub(in crate::app) fn confirm_operation_recovery(
        &mut self,
        recovery: JjOperationRecovery,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = recovery.command_label().to_owned();
        let result_message = match self.services.run_operation_recovery(&recovery) {
            Ok(output) => self.finish_successful_action(
                output,
                viewport_height,
                &format!(" | {}", recovery.success_hint()),
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::OperationRecoveryPreview {
            recovery,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the operation restore/revert command and refresh current plus stacked repo views.
    pub(in crate::app) fn confirm_operation_target(
        &mut self,
        target: JjOperationTarget,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = target.command_label();
        let result_message = match self.services.run_operation_target(&target) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    match self.refresh_stacked_repo_views(viewport_height) {
                        Ok(()) => {
                            let message = format!("{} | jj undo", output.trim());
                            self.status = StatusLine::with_message(&self.view, message.as_str());
                            message
                        }
                        Err(error) => {
                            self.status = StatusLine::error(&self.view, error.to_string());
                            format!(
                                "{} | stacked view refresh failed: {error} | jj undo",
                                output.trim()
                            )
                        }
                    }
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!("{} | refresh failed: {error} | jj undo", output.trim())
                }
            },
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::OperationTargetPreview {
            target,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }
}
