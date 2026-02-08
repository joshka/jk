//! Input handling and mode transitions.
//!
//! Key events are dispatched by mode and can trigger navigation, local view rendering, planning,
//! prompting, confirmation gating, or direct execution.

mod command;
mod confirm;
mod normal;
mod prompt;

use crate::error::JkError;
use crate::flow::{FlowAction, PromptRequest, plan_command};
use crate::jj;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::preview::{confirmation_preview_tokens, is_dangerous};
use super::view::keymap_overview_lines;
use super::{App, Mode, PromptState};

impl App {
    /// Dispatch one key event according to the current mode.
    ///
    /// `Ctrl+C` always exits immediately regardless of mode.
    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches!(key.code, KeyCode::Char('c')) && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }

        match self.mode {
            Mode::Normal => self.handle_normal_key(key),
            Mode::Command => self.handle_command_key(key),
            Mode::Confirm => self.handle_confirm_key(key),
            Mode::Prompt => self.handle_prompt_key(key),
        }
    }

    /// Execute tokens directly or enter confirmation mode when command tier is dangerous.
    ///
    /// Confirmation mode stores pending tokens and renders best-effort preview output without
    /// performing mutations.
    pub(super) fn execute_with_confirmation(&mut self, tokens: Vec<String>) -> Result<(), JkError> {
        if is_dangerous(&tokens) {
            self.pending_confirm = Some(tokens.clone());
            self.mode = Mode::Confirm;
            self.status_line = format!("Confirm dangerous command: jj {}", tokens.join(" "));
            self.render_confirmation_preview(&tokens);
            return Ok(());
        }

        self.execute_tokens(tokens)
    }

    /// Render preview lines shown while confirmation is pending.
    ///
    /// Preview failures are intentionally ignored so lack of preview does not block explicit user
    /// confirmation for the underlying command.
    fn render_confirmation_preview(&mut self, tokens: &[String]) {
        let mut lines = vec![format!("Confirm: jj {}", tokens.join(" "))];

        if let Some(preview_tokens) = confirmation_preview_tokens(tokens)
            && let Ok(preview) = jj::run(&preview_tokens)
        {
            lines.push(String::new());
            lines.push(format!("Preview: jj {}", preview_tokens.join(" ")));
            lines.extend(preview.output);
        }

        self.lines = lines;
        self.row_revision_map = vec![None; self.lines.len()];
        self.cursor = 0;
        self.scroll = 0;
    }

    /// Execute a raw command line by resolving local view actions or planner actions.
    pub(super) fn execute_command_line(&mut self, command: &str) -> Result<(), JkError> {
        if let Some(action) = self.local_view_action(command) {
            return self.apply_flow_action(action);
        }

        let action = plan_command(command, self.selected_revision());
        self.apply_flow_action(action)
    }

    /// Return a local render action for keymap views when command should not hit `jj`.
    pub(super) fn local_view_action(&self, command: &str) -> Option<FlowAction> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut parts = trimmed.split_whitespace();
        let head = parts.next()?;
        if head != "keys" && head != "keymap" {
            return None;
        }

        let query = parts.collect::<Vec<_>>().join(" ");
        if query.is_empty() {
            Some(FlowAction::Render {
                lines: keymap_overview_lines(&self.keybinds, None),
                status: "Showing keymap".to_string(),
            })
        } else {
            Some(FlowAction::Render {
                lines: keymap_overview_lines(&self.keybinds, Some(&query)),
                status: format!("Showing keymap for `{query}`"),
            })
        }
    }

    /// Enter prompt mode with empty prompt input and request metadata.
    pub(super) fn start_prompt(&mut self, request: PromptRequest) {
        self.pending_prompt = Some(PromptState {
            kind: request.kind,
            label: request.label.clone(),
            allow_empty: request.allow_empty,
            input: String::new(),
        });
        self.mode = Mode::Prompt;
        self.status_line = format!("Prompt: {}", request.label);
    }
}
