//! Command history storage and navigation for command mode.

use crate::error::JkError;

use super::App;

impl App {
    /// Append a command to history unless it is empty or duplicates the latest entry.
    pub(super) fn record_command_history(&mut self, command: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return;
        }

        if self.command_history.last().map(String::as_str) == Some(trimmed) {
            return;
        }

        self.command_history.push(trimmed.to_string());
    }

    /// Move one step backward in command history.
    ///
    /// On first entry, current input is captured as a draft so forward navigation can restore it.
    pub(super) fn navigate_command_history_prev(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        let next_index = match self.command_history_index {
            Some(index) if index > 0 => index - 1,
            Some(index) => index,
            None => {
                self.command_history_draft = self.command_input.clone();
                self.command_history.len() - 1
            }
        };

        self.command_history_index = Some(next_index);
        self.command_input = self.command_history[next_index].clone();
    }

    /// Move one step forward in command history or restore draft input at the end.
    pub(super) fn navigate_command_history_next(&mut self) {
        let Some(index) = self.command_history_index else {
            return;
        };

        if index + 1 < self.command_history.len() {
            let next_index = index + 1;
            self.command_history_index = Some(next_index);
            self.command_input = self.command_history[next_index].clone();
            return;
        }

        self.command_history_index = None;
        self.command_input = self.command_history_draft.clone();
    }

    /// Record navigable screen command for back/forward traversal.
    pub(super) fn record_view_visit(&mut self, command: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return;
        }

        if self.navigating_view_stack {
            self.current_view_command = trimmed.to_string();
            return;
        }

        if self.current_view_command == trimmed {
            return;
        }

        if !self.current_view_command.is_empty() {
            self.view_back_stack.push(self.current_view_command.clone());
        }
        self.current_view_command = trimmed.to_string();
        self.view_forward_stack.clear();
    }

    /// Navigate to previous screen command if available.
    pub(super) fn navigate_view_back(&mut self) -> Result<(), JkError> {
        let Some(previous) = self.view_back_stack.pop() else {
            self.status_line = "No previous screen".to_string();
            return Ok(());
        };

        if !self.current_view_command.is_empty() {
            self.view_forward_stack
                .push(self.current_view_command.clone());
        }

        self.navigating_view_stack = true;
        let result = self.execute_command_line(&previous);
        self.navigating_view_stack = false;

        if result.is_err() {
            if let Some(current) = self.view_forward_stack.pop() {
                self.current_view_command = current;
            }
            self.view_back_stack.push(previous);
            return result;
        }

        self.status_line = format!("back: {}", self.current_view_command);
        Ok(())
    }

    /// Navigate to next screen command if available.
    pub(super) fn navigate_view_forward(&mut self) -> Result<(), JkError> {
        let Some(next) = self.view_forward_stack.pop() else {
            self.status_line = "No next screen".to_string();
            return Ok(());
        };

        if !self.current_view_command.is_empty() {
            self.view_back_stack.push(self.current_view_command.clone());
        }

        self.navigating_view_stack = true;
        let result = self.execute_command_line(&next);
        self.navigating_view_stack = false;

        if result.is_err() {
            if let Some(current) = self.view_back_stack.pop() {
                self.current_view_command = current;
            }
            self.view_forward_stack.push(next);
            return result;
        }

        self.status_line = format!("forward: {}", self.current_view_command);
        Ok(())
    }
}
