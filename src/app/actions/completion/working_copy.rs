use ratatui::DefaultTerminal;

use crate::actions::{JjDuplicatePlan, JjNewPlan, JjSplitPlan, JjSplitTarget};
use crate::app::status_line::StatusLine;
use crate::jj::LogViewMode;
use crate::modes::InteractionMode;
use crate::view_state::ViewState;

use super::super::super::{App, current_viewport_width};
use super::super::ActionPane;

impl App {
    /// Run the new-change command, resolve the new working copy, and leave the result on the pane.
    pub fn confirm_new_change(
        &mut self,
        new_change: JjNewPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = new_change.command_label();
        let result_message = match self.services.run_new_change(&new_change) {
            Ok(output) => {
                let new_change_id = match self.services.resolve_revision("@") {
                    Ok(change_id) => change_id,
                    Err(error) => {
                        let message =
                            format!("{} | resolve @ failed: {error} | jj undo", output.trim());
                        self.status = StatusLine::error(&self.view, error.to_string());
                        self.mode = InteractionMode::NewPreview {
                            new_change,
                            output: ActionPane::finished(command_label, message, status_context),
                        };
                        return;
                    }
                };

                self.finish_successful_action_revealing_change(
                    output,
                    Some(new_change_id.as_str()),
                    viewport_height,
                    " | jj undo",
                )
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::NewPreview {
            new_change,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the duplicate command and leave the refresh/reveal result on the duplicate pane.
    pub fn confirm_duplicate(
        &mut self,
        duplicate: JjDuplicatePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = duplicate.command_label();
        let result_message = match self.services.run_duplicate(&duplicate) {
            Ok(output) => self.finish_successful_duplicate(&duplicate, output, viewport_height),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::DuplicatePreview {
            duplicate,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Refresh after duplicate and fall back to revealing the original source change when possible.
    fn finish_successful_duplicate(
        &mut self,
        duplicate: &JjDuplicatePlan,
        output: String,
        viewport_height: u16,
    ) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                let reveal_result = match &self.view {
                    ViewState::Log(_) => {
                        Some(self.reveal_log_change(duplicate.source(), LogViewMode::Recent))
                    }
                    _ => None,
                };

                match reveal_result {
                    Some(Ok(switched_modes)) => {
                        self.view.clamp(viewport_height, current_viewport_width());
                        let message = if switched_modes {
                            "duplicate completed | showing recent work fallback for source | jj undo | jj op show -p"
                        } else {
                            "duplicate completed | source selected as fallback | jj undo | jj op show -p"
                        };
                        self.status = StatusLine::with_message(&self.view, message);
                        format!(
                            "{}\nrefresh: active view refreshed\nreveal: selected original source {} because jk does not parse duplicate output for the new change id\nrecovery: jj undo\nreview: jj op show -p",
                            output.trim(),
                            duplicate.source()
                        )
                    }
                    Some(Err(error)) => {
                        self.status = StatusLine::error(&self.view, error.to_string());
                        format!(
                            "{}\nrefresh: active view refreshed\nreveal: source fallback failed: {error}\nrecovery: jj undo\nreview: jj op show -p",
                            output.trim()
                        )
                    }
                    None => {
                        let message = "duplicate completed | active view refreshed | source reveal unavailable | jj undo | jj op show -p";
                        self.status = StatusLine::with_message(&self.view, message);
                        format!(
                            "{}\nrefresh: active view refreshed\nreveal: source fallback not attempted because the active view cannot reveal log changes\nrecovery: jj undo\nreview: jj op show -p",
                            output.trim()
                        )
                    }
                }
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                format!(
                    "{}\nrefresh: failed: {error}\nrecovery: jj undo\nreview: jj op show -p",
                    output.trim()
                )
            }
        }
    }

    /// Run the split command, including inherited-stdio interactive mode when needed.
    pub fn confirm_split(
        &mut self,
        split: JjSplitPlan,
        status_context: Option<String>,
        viewport_height: u16,
        terminal: Option<&mut DefaultTerminal>,
    ) {
        let command_label = split.command_label();
        let result_message = match self.services.run_split(terminal, &split) {
            Ok(output) => self.finish_successful_split(&split, output, viewport_height),
            Err(error) => {
                let message = split.failure_result_message(&error.to_string());
                self.status = StatusLine::error(&self.view, message.clone());
                message
            }
        };

        self.mode = InteractionMode::SplitPreview {
            split,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Refresh after split and reveal the exact or current working-copy target when possible.
    fn finish_successful_split(
        &mut self,
        split: &JjSplitPlan,
        output: String,
        viewport_height: u16,
    ) -> String {
        let reveal_change_id = match split.target() {
            JjSplitTarget::ExactChange(change_id) => Some(change_id.clone()),
            JjSplitTarget::CurrentWorkingCopy => match self.services.resolve_revision("@") {
                Ok(change_id) => Some(change_id),
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    return format!(
                        "{output}\nrefresh: skipped because resolving @ failed: {error}\nrecovery: jj undo\nreview: jj op show -p"
                    );
                }
            },
        };

        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                let mut reveal_error = None;
                let revealed_in_recent = match reveal_change_id.as_deref() {
                    Some(change_id) => match self.reveal_log_change(change_id, LogViewMode::Recent)
                    {
                        Ok(switched_modes) => {
                            self.view.clamp(viewport_height, current_viewport_width());
                            Some(switched_modes)
                        }
                        Err(error) => {
                            self.status = StatusLine::error(&self.view, error.to_string());
                            reveal_error = Some(format!(
                                "{output}\nrefresh: active view refreshed\nreveal: failed: {error}\nrecovery: jj undo\nreview: jj op show -p"
                            ));
                            None
                        }
                    },
                    None => None,
                };

                let message = match revealed_in_recent {
                    Some(true) => "split completed | showing recent work | jj undo | jj op show -p",
                    Some(false) => "split completed | jj undo | jj op show -p",
                    None => match reveal_error.as_deref() {
                        Some(message) => return message.to_owned(),
                        None => "split completed | jj undo | jj op show -p",
                    },
                };
                self.status = StatusLine::with_message(&self.view, message);
                format!("{output}\nrefresh: {message}")
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                format!(
                    "{output}\nrefresh: failed: {error}\nrecovery: jj undo\nreview: jj op show -p"
                )
            }
        }
    }
}
