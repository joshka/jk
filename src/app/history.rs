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

    /// Record a high-level intent and update frequency + recency trackers.
    pub(super) fn record_intent(&mut self, intent: &str) {
        let trimmed = intent.trim();
        if trimmed.is_empty() {
            return;
        }

        *self.intent_counts.entry(trimmed.to_string()).or_insert(0) += 1;

        if let Some(index) = self.intent_recency.iter().position(|item| item == trimmed) {
            self.intent_recency.remove(index);
        }
        self.intent_recency.insert(0, trimmed.to_string());
        if self.intent_recency.len() > 20 {
            self.intent_recency.truncate(20);
        }

        if self.recent_intents.first().map(String::as_str) == Some(trimmed) {
            return;
        }
        self.recent_intents.insert(0, trimmed.to_string());
        if self.recent_intents.len() > 8 {
            self.recent_intents.truncate(8);
        }
    }

    /// Record intent from canonicalized command tokens.
    pub(super) fn record_intent_from_tokens(&mut self, tokens: &[String]) {
        let intent = match (
            tokens.first().map(String::as_str),
            tokens.get(1).map(String::as_str),
        ) {
            (Some("git"), Some("fetch")) => "git fetch",
            (Some("git"), Some("push")) => "git push",
            (Some("operation"), Some("log")) => "operation log",
            (Some("operation"), Some("show")) => "operation show",
            (Some("operation"), Some("diff")) => "operation diff",
            (Some("bookmark"), Some("list")) => "bookmark list",
            (Some("resolve"), Some("-l" | "--list")) => "resolve list",
            (Some("file"), Some("list")) => "file list",
            (Some("tag"), Some("list")) => "tag list",
            (Some("workspace"), Some("list")) => "workspace list",
            (Some("workspace"), Some("root")) => "workspace root",
            (Some(first), _) => first,
            _ => "",
        };

        self.record_intent(intent);
    }

    /// Return recent intents as display labels for footer/help surfaces.
    pub(super) fn recent_intent_labels(&self, max: usize) -> Vec<String> {
        self.recent_intents
            .iter()
            .take(max)
            .map(|intent| format!(":{intent}"))
            .collect()
    }

    /// Rank command suggestions for command mode by frequency then recency.
    pub(super) fn ranked_command_suggestions(&self, prefix: &str, max: usize) -> Vec<String> {
        let normalized = prefix.trim().to_ascii_lowercase();
        let mut candidates = command_palette_candidates();

        for entry in &self.command_history {
            if !candidates.contains(entry) {
                candidates.push(entry.clone());
            }
        }
        for entry in &self.recent_intents {
            if !candidates.contains(entry) {
                candidates.push(entry.clone());
            }
        }

        let mut ranked: Vec<(i64, i64, String)> = candidates
            .into_iter()
            .filter(|candidate| {
                normalized.is_empty()
                    || candidate.to_ascii_lowercase().starts_with(&normalized)
                    || candidate.to_ascii_lowercase().contains(&normalized)
            })
            .map(|candidate| {
                let frequency = *self.intent_counts.get(&candidate).unwrap_or(&0) as i64;
                let recency_rank = self
                    .intent_recency
                    .iter()
                    .position(|item| item == &candidate)
                    .map(|index| (1000_i64 - index as i64).max(0))
                    .unwrap_or(0);
                (frequency, recency_rank, candidate)
            })
            .collect();

        ranked.sort_by(|left, right| {
            right
                .0
                .cmp(&left.0)
                .then_with(|| right.1.cmp(&left.1))
                .then_with(|| left.2.cmp(&right.2))
        });

        ranked
            .into_iter()
            .take(max)
            .map(|(_, _, command)| command)
            .collect()
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

/// Return base command-palette candidates used for ranked suggestions.
fn command_palette_candidates() -> Vec<String> {
    [
        "log",
        "status",
        "show",
        "diff",
        "operation log",
        "bookmark list",
        "resolve -l",
        "file list",
        "tag list",
        "workspace root",
        "git fetch",
        "git push",
        "rebase",
        "squash",
        "split",
        "abandon",
        "undo",
        "redo",
        "commands",
        "help inspect",
        "help rewrite",
        "help sync",
        "help recover",
        "keys",
        "aliases",
    ]
    .iter()
    .map(|value| value.to_string())
    .collect()
}
