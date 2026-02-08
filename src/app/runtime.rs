//! Runtime loop, rendering, and state transitions.

use std::io::{Stdout, Write};
use std::time::Duration;

use crate::error::JkError;
use crate::flow::FlowAction;
use crate::jj;
use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event};
use crossterm::queue;
use crossterm::style::{Print, ResetColor};
use crossterm::terminal::{Clear, ClearType, size};

use super::selection::{derive_row_revision_map, extract_revision, startup_action, trim_to_width};
use super::terminal::TerminalSession;
use super::view::decorate_command_output;
use super::{App, Mode};

impl App {
    /// Enter terminal session, run startup action, then drive draw/input loop until quit.
    pub fn run(&mut self, startup_tokens: Vec<String>) -> Result<(), JkError> {
        let mut terminal = TerminalSession::enter()?;
        self.apply_startup_tokens(startup_tokens)?;

        while !self.should_quit {
            self.draw(terminal.stdout_mut())?;

            if event::poll(Duration::from_millis(120))?
                && let Event::Key(key) = event::read()?
            {
                self.handle_key(key)?;
            }
        }

        Ok(())
    }

    /// Execute startup command tokens or default startup flow.
    ///
    /// Local view commands are handled without invoking `jj`.
    pub(super) fn apply_startup_tokens(
        &mut self,
        startup_tokens: Vec<String>,
    ) -> Result<(), JkError> {
        if !startup_tokens.is_empty() {
            let startup_command = startup_tokens.join(" ");
            if let Some(action) = self.local_view_action(&startup_command) {
                return self.apply_flow_action(action);
            }
        }

        let action = startup_action(&startup_tokens);
        self.apply_flow_action(action)
    }

    /// Apply a planned action and enforce mode/state transition invariants.
    ///
    /// Render actions reset cursor/scroll and clear row metadata so selection cannot reference stale
    /// rows. Prompt actions enter prompt mode without side effects. Execute actions are routed
    /// through confirmation policy before subprocess execution.
    pub(super) fn apply_flow_action(&mut self, action: FlowAction) -> Result<(), JkError> {
        match action {
            FlowAction::Quit => {
                self.should_quit = true;
                Ok(())
            }
            FlowAction::Render { lines, status } => {
                self.lines = lines;
                self.row_revision_map = vec![None; self.lines.len()];
                self.cursor = 0;
                self.scroll = 0;
                self.status_line = status;
                Ok(())
            }
            FlowAction::Status(message) => {
                self.status_line = message;
                Ok(())
            }
            FlowAction::Execute(tokens) => self.execute_with_confirmation(tokens),
            FlowAction::Prompt(request) => {
                self.start_prompt(request);
                Ok(())
            }
        }
    }

    /// Execute a concrete `jj` command and replace rendered output state.
    ///
    /// Successful `log` runs refresh `last_log_tokens` so patch toggle can re-run with the same
    /// base arguments. This path also rebuilds row-revision metadata so selection-based shortcuts
    /// stay aligned with decorated output.
    pub(super) fn execute_tokens(&mut self, tokens: Vec<String>) -> Result<(), JkError> {
        let result = jj::run(&tokens)?;
        if matches!(result.command.first().map(String::as_str), Some("log")) {
            self.last_log_tokens = result.command.clone();
        }
        self.row_revision_map = derive_row_revision_map(&result.command, &result.output);
        self.lines = decorate_command_output(&result.command, result.output);
        self.cursor = 0;
        self.scroll = 0;
        self.last_command = result.command;
        self.status_line = if result.success {
            format!("ok: jj {}", self.last_command.join(" "))
        } else {
            format!("error: jj {}", self.last_command.join(" "))
        };
        Ok(())
    }

    /// Move selection cursor up and keep viewport aligned.
    pub(super) fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.ensure_cursor_visible(20);
    }

    /// Move selection cursor down and keep viewport aligned.
    pub(super) fn move_cursor_down(&mut self) {
        if self.cursor + 1 < self.lines.len() {
            self.cursor += 1;
        }
        self.ensure_cursor_visible(20);
    }

    /// Adjust scroll so selected row stays within the visible content window.
    pub(super) fn ensure_cursor_visible(&mut self, content_height: usize) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
            return;
        }

        if self.cursor >= self.scroll.saturating_add(content_height) {
            self.scroll = self.cursor.saturating_sub(content_height.saturating_sub(1));
        }
    }

    /// Resolve selected revision nearest to cursor.
    ///
    /// Metadata row mapping is preferred and plain-text parsing is used as a fallback.
    pub(super) fn selected_revision(&self) -> Option<String> {
        if !self.row_revision_map.is_empty() {
            for line_index in (0..=self.cursor).rev() {
                if let Some(Some(revision)) = self.row_revision_map.get(line_index) {
                    return Some(revision.clone());
                }
            }
        }

        for line_index in (0..=self.cursor).rev() {
            if let Some(line) = self.lines.get(line_index)
                && let Some(revision) = extract_revision(line)
            {
                return Some(revision);
            }
        }

        None
    }

    /// Draw one frame into the terminal alternate screen.
    fn draw(&mut self, stdout: &mut Stdout) -> Result<(), JkError> {
        let (width, height) = size()?;
        let width = width as usize;
        let height = height as usize;

        let frame = self.render_for_display(width, height);
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        for (index, line) in frame.into_iter().enumerate() {
            queue!(stdout, MoveTo(0, index as u16), Print(line), ResetColor)?;
        }

        stdout.flush()?;
        Ok(())
    }

    /// Build current frame rows including header, visible content slice, and mode-specific footer.
    fn render_for_display(&mut self, width: usize, height: usize) -> Vec<String> {
        let header = format!(
            "jk [{}] :: jj {}",
            self.mode_label(),
            self.last_command.join(" ")
        );

        let content_height = height.saturating_sub(2);
        self.ensure_cursor_visible(content_height.max(1));

        let mut rows = Vec::with_capacity(height.max(1));
        rows.push(trim_to_width(&header, width));

        for idx in 0..content_height {
            let line_index = self.scroll + idx;
            let content = if let Some(line) = self.lines.get(line_index) {
                let marker = if line_index == self.cursor && self.mode == Mode::Normal {
                    ">"
                } else {
                    " "
                };
                format!("{marker} {}", line)
            } else {
                String::new()
            };
            rows.push(trim_to_width(&content, width));
        }

        let footer = match self.mode {
            Mode::Normal => self.status_line.clone(),
            Mode::Command => format!(":{}", self.command_input),
            Mode::Confirm => {
                let pending = self.pending_confirm.clone().unwrap_or_default();
                format!("Run `jj {}` ? [y/n]", pending.join(" "))
            }
            Mode::Prompt => {
                if let Some(prompt) = &self.pending_prompt {
                    format!("{} > {}", prompt.label, prompt.input)
                } else {
                    "prompt unavailable".to_string()
                }
            }
        };
        rows.push(trim_to_width(&footer, width));

        rows
    }

    /// Return short label used in header for current input mode.
    fn mode_label(&self) -> &'static str {
        match self.mode {
            Mode::Normal => "normal",
            Mode::Command => "command",
            Mode::Confirm => "confirm",
            Mode::Prompt => "prompt",
        }
    }

    #[cfg(test)]
    /// Render deterministic frame text for snapshot tests.
    pub fn render_for_snapshot(&mut self, width: usize, height: usize) -> String {
        self.render_for_display(width, height).join("\n")
    }
}
