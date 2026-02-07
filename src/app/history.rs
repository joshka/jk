use super::App;

impl App {
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
}
