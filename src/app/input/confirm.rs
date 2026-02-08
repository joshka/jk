//! Confirmation-mode key handling.

use crate::error::JkError;
use crossterm::event::KeyEvent;

use super::super::selection::matches_any;
use super::{App, Mode};

impl App {
    /// Handle confirmation-mode accept/reject flows for dangerous commands.
    pub(super) fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches_any(&self.keybinds.confirm.reject, key) {
            self.pending_confirm = None;
            self.mode = Mode::Normal;
            self.status_line = "Command canceled".to_string();
            return Ok(());
        }

        if matches_any(&self.keybinds.confirm.accept, key)
            && let Some(tokens) = self.pending_confirm.take()
        {
            self.mode = Mode::Normal;
            self.execute_tokens(tokens)?;
        }

        Ok(())
    }
}
