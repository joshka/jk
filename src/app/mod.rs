mod preview;
mod selection;
mod terminal;
mod view;

use std::io::{Stdout, Write};
use std::time::Duration;

use crate::config::KeybindConfig;
use crate::error::JkError;
use crate::flows::{FlowAction, PromptKind, PromptRequest, plan_command};
use crate::jj;
use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::queue;
use crossterm::style::{Print, ResetColor};
use crossterm::terminal::{Clear, ClearType, size};

use self::preview::{confirmation_preview_tokens, is_dangerous, toggle_patch_flag};
use self::selection::{
    derive_row_revision_map, extract_revision, matches_any, startup_action, trim_to_width,
};
use self::terminal::TerminalSession;
use self::view::{decorate_command_output, keymap_overview_lines};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Command,
    Confirm,
    Prompt,
}

pub struct App {
    keybinds: KeybindConfig,
    mode: Mode,
    lines: Vec<String>,
    cursor: usize,
    scroll: usize,
    status_line: String,
    command_input: String,
    command_history: Vec<String>,
    command_history_index: Option<usize>,
    command_history_draft: String,
    row_revision_map: Vec<Option<String>>,
    pending_confirm: Option<Vec<String>>,
    pending_prompt: Option<PromptState>,
    last_command: Vec<String>,
    last_log_tokens: Vec<String>,
    should_quit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptState {
    kind: PromptKind,
    label: String,
    allow_empty: bool,
    input: String,
}

impl App {
    pub fn new(keybinds: KeybindConfig) -> Self {
        Self {
            keybinds,
            mode: Mode::Normal,
            lines: vec!["Initializing jk...".to_string()],
            cursor: 0,
            scroll: 0,
            status_line: "Press : for commands, q to quit".to_string(),
            command_input: String::new(),
            command_history: Vec::new(),
            command_history_index: None,
            command_history_draft: String::new(),
            row_revision_map: Vec::new(),
            pending_confirm: None,
            pending_prompt: None,
            last_command: vec!["log".to_string()],
            last_log_tokens: vec!["log".to_string()],
            should_quit: false,
        }
    }

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

    fn handle_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
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

    fn handle_normal_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches_any(&self.keybinds.normal.quit, key) {
            self.should_quit = true;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.refresh, key) {
            let tokens = if self.last_command.is_empty() {
                vec!["log".to_string()]
            } else {
                self.last_command.clone()
            };
            self.execute_tokens(tokens)?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.command_mode, key) {
            self.mode = Mode::Command;
            self.command_input.clear();
            self.command_history_index = None;
            self.command_history_draft.clear();
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.help, key) {
            self.execute_command_line("commands")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.keymap, key) {
            self.execute_command_line("keys")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.aliases, key) {
            self.execute_command_line("aliases")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.repeat_last, key) {
            let tokens = if self.last_command.is_empty() {
                vec!["log".to_string()]
            } else {
                self.last_command.clone()
            };
            self.execute_tokens(tokens)?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.up, key) {
            self.move_cursor_up();
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.down, key) {
            self.move_cursor_down();
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.top, key) {
            self.cursor = 0;
            self.scroll = 0;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.bottom, key) {
            if !self.lines.is_empty() {
                self.cursor = self.lines.len() - 1;
                self.ensure_cursor_visible(20);
            }
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.show, key) {
            if let Some(revision) = self.selected_revision() {
                self.execute_with_confirmation(vec!["show".to_string(), revision])?;
            } else {
                self.status_line = "No revision selected on this line".to_string();
            }
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.diff, key) {
            if let Some(revision) = self.selected_revision() {
                self.execute_with_confirmation(vec![
                    "diff".to_string(),
                    "-r".to_string(),
                    revision,
                ])?;
            } else {
                self.status_line = "No revision selected on this line".to_string();
            }
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.status, key) {
            self.execute_command_line("status")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.log, key) {
            self.execute_command_line("log")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.operation_log, key) {
            self.execute_command_line("operation log")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.bookmark_list, key) {
            self.execute_command_line("bookmark list")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.resolve_list, key) {
            self.execute_command_line("resolve -l")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.file_list, key) {
            self.execute_command_line("file list")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.tag_list, key) {
            self.execute_command_line("tag list")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.root, key) {
            self.execute_command_line("root")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.toggle_patch, key) {
            if !matches!(
                self.last_log_tokens.first().map(String::as_str),
                Some("log")
            ) {
                self.status_line = "Patch toggle is available after running log".to_string();
                return Ok(());
            }

            let tokens = toggle_patch_flag(&self.last_log_tokens);
            self.execute_tokens(tokens)?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.fetch, key) {
            self.execute_command_line("gf")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.push, key) {
            self.execute_command_line("gp")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.rebase_main, key) {
            self.execute_command_line("rbm")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.rebase_trunk, key) {
            self.execute_command_line("rbt")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.new, key) {
            self.execute_command_line("new")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.next, key) {
            self.execute_command_line("next")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.prev, key) {
            self.execute_command_line("prev")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.edit, key) {
            self.execute_command_line("edit")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.commit, key) {
            self.execute_command_line("commit")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.describe, key) {
            self.execute_command_line("describe")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.bookmark_set, key) {
            self.execute_command_line("bookmark set")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.abandon, key) {
            self.execute_command_line("abandon")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.rebase, key) {
            self.execute_command_line("rebase")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.squash, key) {
            self.execute_command_line("squash")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.split, key) {
            self.execute_command_line("split")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.restore, key) {
            self.execute_command_line("restore")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.revert, key) {
            self.execute_command_line("revert")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.undo, key) {
            self.execute_command_line("undo")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.redo, key) {
            self.execute_command_line("redo")?;
            return Ok(());
        }

        Ok(())
    }

    fn handle_command_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches_any(&self.keybinds.command.cancel, key) {
            self.mode = Mode::Normal;
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
            self.mode = Mode::Normal;
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

    fn handle_prompt_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches_any(&self.keybinds.command.cancel, key) {
            self.pending_prompt = None;
            self.mode = Mode::Normal;
            self.status_line = "Prompt canceled".to_string();
            return Ok(());
        }

        if matches_any(&self.keybinds.command.backspace, key) {
            if let Some(prompt) = self.pending_prompt.as_mut() {
                prompt.input.pop();
            }
            return Ok(());
        }

        if matches_any(&self.keybinds.command.submit, key) {
            if let Some(prompt) = self.pending_prompt.take() {
                let input = prompt.input.trim();
                if !prompt.allow_empty && input.is_empty() {
                    self.pending_prompt = Some(prompt);
                    self.status_line = "Input required for this flow".to_string();
                    return Ok(());
                }

                match prompt.kind.to_tokens(input) {
                    Ok(tokens) => {
                        self.mode = Mode::Normal;
                        self.execute_with_confirmation(tokens)?;
                    }
                    Err(message) => {
                        self.pending_prompt = Some(prompt);
                        self.mode = Mode::Prompt;
                        self.status_line = message;
                    }
                }
            } else {
                self.mode = Mode::Normal;
                self.status_line = "Prompt unavailable".to_string();
            }
            return Ok(());
        }

        if let KeyCode::Char(ch) = key.code
            && let Some(prompt) = self.pending_prompt.as_mut()
        {
            prompt.input.push(ch);
        }

        Ok(())
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
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

    fn execute_with_confirmation(&mut self, tokens: Vec<String>) -> Result<(), JkError> {
        if is_dangerous(&tokens) {
            self.pending_confirm = Some(tokens.clone());
            self.mode = Mode::Confirm;
            self.status_line = format!("Confirm dangerous command: jj {}", tokens.join(" "));
            self.render_confirmation_preview(&tokens);
            return Ok(());
        }

        self.execute_tokens(tokens)
    }

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

    fn execute_command_line(&mut self, command: &str) -> Result<(), JkError> {
        if let Some(action) = self.local_view_action(command) {
            return self.apply_flow_action(action);
        }

        let action = plan_command(command, self.selected_revision());
        self.apply_flow_action(action)
    }

    fn local_view_action(&self, command: &str) -> Option<FlowAction> {
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

    fn record_command_history(&mut self, command: &str) {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return;
        }

        if self.command_history.last().map(String::as_str) == Some(trimmed) {
            return;
        }

        self.command_history.push(trimmed.to_string());
    }

    fn navigate_command_history_prev(&mut self) {
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

    fn navigate_command_history_next(&mut self) {
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

    fn start_prompt(&mut self, request: PromptRequest) {
        self.pending_prompt = Some(PromptState {
            kind: request.kind,
            label: request.label.clone(),
            allow_empty: request.allow_empty,
            input: String::new(),
        });
        self.mode = Mode::Prompt;
        self.status_line = format!("Prompt: {}", request.label);
    }

    fn apply_startup_tokens(&mut self, startup_tokens: Vec<String>) -> Result<(), JkError> {
        if !startup_tokens.is_empty() {
            let startup_command = startup_tokens.join(" ");
            if let Some(action) = self.local_view_action(&startup_command) {
                return self.apply_flow_action(action);
            }
        }

        let action = startup_action(&startup_tokens);
        self.apply_flow_action(action)
    }

    fn apply_flow_action(&mut self, action: FlowAction) -> Result<(), JkError> {
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

    fn execute_tokens(&mut self, tokens: Vec<String>) -> Result<(), JkError> {
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

    fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.ensure_cursor_visible(20);
    }

    fn move_cursor_down(&mut self) {
        if self.cursor + 1 < self.lines.len() {
            self.cursor += 1;
        }
        self.ensure_cursor_visible(20);
    }

    fn ensure_cursor_visible(&mut self, content_height: usize) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
            return;
        }

        if self.cursor >= self.scroll.saturating_add(content_height) {
            self.scroll = self.cursor.saturating_sub(content_height.saturating_sub(1));
        }
    }

    fn selected_revision(&self) -> Option<String> {
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

    fn mode_label(&self) -> &'static str {
        match self.mode {
            Mode::Normal => "normal",
            Mode::Command => "command",
            Mode::Confirm => "confirm",
            Mode::Prompt => "prompt",
        }
    }

    #[cfg(test)]
    pub fn render_for_snapshot(&mut self, width: usize, height: usize) -> String {
        self.render_for_display(width, height).join("\n")
    }
}

#[cfg(test)]
mod tests;
