//! Command-mode key handling.

use crate::error::JkError;
use crossterm::event::{KeyCode, KeyEvent};

use super::super::selection::matches_any;
use super::App;

impl App {
    /// Handle command-mode editing and submission behavior.
    ///
    /// Submitting records history, exits command mode, clears editing state, then executes the
    /// captured command line.
    pub(super) fn handle_command_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches_any(&self.keybinds.command.cancel, key) {
            self.mode = super::Mode::Normal;
            self.status_line = "Command canceled".to_string();
            return Ok(());
        }

        if matches_any(&self.keybinds.command.history_prev, key) {
            self.navigate_command_history_prev();
            return Ok(());
        }

        if matches_any(&self.keybinds.command.history_next, key) {
            self.navigate_command_history_next();
            return Ok(());
        }

        if matches_any(&self.keybinds.command.backspace, key) {
            self.command_history_index = None;
            self.command_input.pop();
            return Ok(());
        }

        if matches_any(&self.keybinds.command.submit, key) {
            let command = self.command_input.clone();
            self.record_command_history(&command);
            self.mode = super::Mode::Normal;
            self.command_input.clear();
            self.command_history_index = None;
            self.command_history_draft.clear();
            self.execute_command_line(&command)?;
            return Ok(());
        }

        if let KeyCode::Char(ch) = key.code {
            self.command_history_index = None;
            self.command_input.push(ch);
        }

        Ok(())
    }
}
