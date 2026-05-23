use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjRebasePlan, JjRestorePlan, JjRevertPlan,
    JjSquashPlan, JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan,
};
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;

use super::super::super::App;
use super::super::ActionPane;

impl App {
    /// Run the working-copy navigation command and reveal the resulting active change.
    pub fn confirm_working_copy_navigation(
        &mut self,
        navigation: JjWorkingCopyNavigationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = navigation.command_label();
        let result_message = match self.services.run_working_copy_navigation(&navigation) {
            Ok(output) => {
                let reveal_change_id = match navigation.kind() {
                    JjWorkingCopyNavigationKind::Edit => navigation
                        .target_change_id()
                        .expect("edit navigation requires exact target change id")
                        .to_owned(),
                    JjWorkingCopyNavigationKind::Next | JjWorkingCopyNavigationKind::Prev => {
                        match self.services.resolve_revision("@") {
                            Ok(change_id) => change_id,
                            Err(error) => {
                                let message = format!(
                                    "{} | resolve @ failed: {error} | jj undo",
                                    output.trim()
                                );
                                self.status = StatusLine::error(&self.view, error.to_string());
                                self.mode = InteractionMode::WorkingCopyNavigationPreview {
                                    navigation,
                                    output: ActionPane::finished(
                                        command_label,
                                        message,
                                        status_context,
                                    ),
                                };
                                return;
                            }
                        }
                    }
                };

                self.finish_successful_action_revealing_change(
                    output,
                    Some(reveal_change_id.as_str()),
                    viewport_height,
                    " | jj undo",
                )
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::WorkingCopyNavigationPreview {
            navigation,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run abandon and leave its finished output on the abandon pane.
    pub fn confirm_abandon(
        &mut self,
        abandon: JjAbandonPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = abandon.command_label();
        let result_message = match self.services.run_abandon(&abandon) {
            Ok(output) => self.finish_successful_action(output, viewport_height, " | jj undo"),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::AbandonPreview {
            abandon,
            preview: JjAbandonPreview::new(String::new(), None, String::new()),
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Re-check emptiness before abandoning so the fast path only applies to still-empty changes.
    pub fn confirm_empty_abandon_after_recheck(
        &mut self,
        abandon: JjAbandonPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        match self.services.load_abandon_preview(&abandon) {
            Ok(preview) if preview.is_empty_change() => {
                self.confirm_abandon(abandon, status_context, viewport_height);
            }
            Ok(preview) => {
                let message = "change is no longer empty; type exact revision to confirm abandon";
                self.status = StatusLine::error(&self.view, message.to_owned());
                let command_label = abandon.command_label();
                let output = format!("{message}\n\n{}", preview.preview_text());
                self.mode = InteractionMode::AbandonConfirm {
                    abandon,
                    input: String::new(),
                    output: ActionPane::pending(command_label, output, status_context),
                };
            }
            Err(error) => {
                let message = error.to_string();
                self.status = StatusLine::error(&self.view, message.clone());
                let command_label = abandon.diff_summary_label();
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    preview: JjAbandonPreview::new(String::new(), None, String::new()),
                    output: ActionPane::finished(command_label, message, status_context),
                };
            }
        }
    }

    /// Run restore and leave its finished output on the restore pane.
    pub fn confirm_restore(
        &mut self,
        restore: JjRestorePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = restore.command_label();
        let result_message = match self.services.run_restore(&restore) {
            Ok(output) => self.finish_successful_action(output, viewport_height, " | jj undo"),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::RestorePreview {
            restore,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run revert and leave its finished output on the revert pane.
    pub fn confirm_revert(
        &mut self,
        revert: JjRevertPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = revert.command_label();
        let result_message = match self.services.run_revert(&revert) {
            Ok(output) => self.finish_successful_action(output, viewport_height, " | jj undo"),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::RevertPreview {
            revert,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run rebase and reveal the first source change after refresh when possible.
    pub fn confirm_rebase(
        &mut self,
        rebase: JjRebasePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = rebase.command_label();
        let primary_source = rebase.sources().first().cloned();
        let result_message = match self.services.run_rebase(&rebase) {
            Ok(output) => self.finish_successful_action_revealing_change(
                output,
                primary_source.as_deref(),
                viewport_height,
                " | jj undo | jj op show -p",
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::RebasePreview {
            rebase,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run squash and reveal the destination change after refresh when possible.
    pub fn confirm_squash(
        &mut self,
        squash: JjSquashPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = squash.command_label();
        let destination = squash.destination().to_owned();
        let result_message = match self.services.run_squash(&squash) {
            Ok(output) => self.finish_successful_action_revealing_change(
                output,
                Some(destination.as_str()),
                viewport_height,
                " | jj undo",
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::SquashPreview {
            squash,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run absorb and leave its finished output on the absorb pane.
    pub fn confirm_absorb(
        &mut self,
        absorb: JjAbsorbPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = absorb.command_label();
        let result_message = match self.services.run_absorb(&absorb) {
            Ok(output) => {
                self.finish_successful_action(output, viewport_height, " | jj undo | jj op show -p")
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::AbsorbPreview {
            absorb,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }
}
