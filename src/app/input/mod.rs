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
use crate::{
    alias::{alias_overview_lines, alias_overview_lines_with_query},
    commands::{
        command_overview_lines_with_query_and_recent, command_overview_lines_with_recent,
        command_workflow_lines,
    },
};
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
        let trimmed = command.trim();
        if let Some(action) = self.local_view_action(command) {
            if matches!(action, FlowAction::Render { .. }) {
                self.record_view_visit(trimmed);
            }
            let intent = trimmed
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            self.record_intent(&intent);
            return self.apply_flow_action(action);
        }

        let action = plan_command(command, self.selected_revision());
        if matches!(action, FlowAction::Render { .. }) && !trimmed.is_empty() {
            self.record_view_visit(trimmed);
        }
        if !trimmed.is_empty() {
            let intent = trimmed
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            self.record_intent(&intent);
        }
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
        let tail = parts.collect::<Vec<_>>();
        let query = tail.join(" ");

        match head {
            "keys" | "keymap" => {
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
            "aliases" => {
                if query.is_empty() {
                    Some(FlowAction::Render {
                        lines: alias_overview_lines(),
                        status: "Showing alias catalog".to_string(),
                    })
                } else {
                    Some(FlowAction::Render {
                        lines: alias_overview_lines_with_query(Some(&query)),
                        status: format!("Showing alias catalog for `{query}`"),
                    })
                }
            }
            "commands" | "help" | "?" => {
                if matches!(
                    tail.first().copied(),
                    Some("inspect" | "rewrite" | "sync" | "recover")
                ) && tail.len() == 1
                {
                    let workflow = tail[0];
                    return command_workflow_lines(workflow, &self.recent_intent_labels(4)).map(
                        |lines| FlowAction::Render {
                            lines,
                            status: format!("Showing workflow help for `{workflow}`"),
                        },
                    );
                }

                if query.is_empty() {
                    Some(FlowAction::Render {
                        lines: command_overview_lines_with_recent(&self.recent_intent_labels(4)),
                        status: "Showing command registry".to_string(),
                    })
                } else {
                    Some(FlowAction::Render {
                        lines: command_overview_lines_with_query_and_recent(
                            Some(&query),
                            &self.recent_intent_labels(4),
                        ),
                        status: format!("Showing command registry for `{query}`"),
                    })
                }
            }
            _ => None,
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

    /// Return whether canonical command tokens represent a navigable read-only screen.
    pub(super) fn is_navigable_view_tokens(tokens: &[String]) -> bool {
        match tokens.first().map(String::as_str) {
            Some(
                "log" | "status" | "show" | "diff" | "root" | "version" | "evolog" | "interdiff",
            ) => true,
            Some("operation")
                if matches!(
                    tokens.get(1).map(String::as_str),
                    Some("log" | "show" | "diff")
                ) =>
            {
                true
            }
            Some("bookmark") if matches!(tokens.get(1).map(String::as_str), Some("list")) => true,
            Some("resolve")
                if tokens
                    .iter()
                    .any(|token| token == "-l" || token == "--list") =>
            {
                true
            }
            Some("file")
                if matches!(
                    tokens.get(1).map(String::as_str),
                    Some("list" | "show" | "search" | "annotate")
                ) =>
            {
                true
            }
            Some("tag") if matches!(tokens.get(1).map(String::as_str), Some("list")) => true,
            Some("workspace")
                if matches!(tokens.get(1).map(String::as_str), Some("list" | "root")) =>
            {
                true
            }
            _ => false,
        }
    }
}
