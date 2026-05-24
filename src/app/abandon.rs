use color_eyre::Result;
use crossterm::event::KeyCode;

use crate::app::App;
use crate::app::actions::{ActionPaneKey, action_pane_visible_lines, handle_action_pane_key};
use crate::app::reducers::{ConfirmationKey, reduce_confirmation_key};
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;

impl App {
    pub fn handle_abandon_preview_key(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> Result<bool> {
        let InteractionMode::AbandonPreview {
            abandon,
            preview,
            output,
        } = &mut self.mode
        else {
            unreachable!("abandon preview key handler requires abandon preview mode");
        };

        let (abandon, preview, status_context, completed) = {
            (
                abandon.clone(),
                preview.clone(),
                output.status_context().cloned(),
                output.completed(),
            )
        };
        let visible_lines = action_pane_visible_lines(viewport_height);
        match handle_action_pane_key(code, output, visible_lines) {
            ActionPaneKey::Cancel => {
                self.mode = InteractionMode::Normal;
                if !completed {
                    self.status =
                        StatusLine::with_message(&self.view, "abandon cancelled".to_owned());
                }
            }
            ActionPaneKey::Primary => {
                if completed {
                    self.mode = InteractionMode::Normal;
                    return Ok(true);
                }

                if preview.is_empty_change() {
                    self.confirm_empty_abandon_after_recheck(
                        abandon,
                        status_context,
                        viewport_height,
                    );
                } else {
                    self.mode = InteractionMode::AbandonConfirm {
                        abandon,
                        input: String::new(),
                        output: output.clone(),
                    };
                }
            }
            ActionPaneKey::Handled | ActionPaneKey::Ignored => {}
        }
        Ok(true)
    }

    pub fn handle_abandon_confirm_key(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> Result<bool> {
        let InteractionMode::AbandonConfirm {
            abandon,
            input,
            output,
        } = &mut self.mode
        else {
            unreachable!("abandon confirm key handler requires abandon confirm mode");
        };

        let (abandon_plan, status_context) = (abandon.clone(), output.status_context().cloned());
        let visible_lines = action_pane_visible_lines(viewport_height);
        match reduce_confirmation_key(input, output, visible_lines, code) {
            ConfirmationKey::Cancel => {
                self.mode = InteractionMode::Normal;
                self.status = StatusLine::with_message(&self.view, "abandon cancelled".to_owned());
            }
            ConfirmationKey::Accept => {
                if input == abandon.revision() {
                    self.confirm_abandon(abandon_plan, status_context, viewport_height);
                } else {
                    self.status = StatusLine::error(
                        &self.view,
                        "confirmation did not match; abandon not run".to_owned(),
                    );
                }
            }
            ConfirmationKey::Handled | ConfirmationKey::Ignored => {}
        }
        Ok(true)
    }
}
