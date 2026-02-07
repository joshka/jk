use std::io::{self, Stdout, Write};
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use crossterm::{execute, queue};
use regex::Regex;

use crate::config::KeybindConfig;
use crate::error::JkError;
use crate::flows::{FlowAction, PromptKind, PromptRequest, plan_command};
use crate::jj;
use crate::keys::KeyBinding;

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
    pending_confirm: Option<Vec<String>>,
    pending_prompt: Option<PromptState>,
    last_command: Vec<String>,
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
            pending_confirm: None,
            pending_prompt: None,
            last_command: vec!["log".to_string()],
            should_quit: false,
        }
    }

    pub fn run(&mut self, startup_tokens: Vec<String>) -> Result<(), JkError> {
        self.execute_tokens(startup_tokens)?;

        let mut terminal = TerminalSession::enter()?;

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
            self.execute_tokens(vec!["log".to_string()])?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.command_mode, key) {
            self.mode = Mode::Command;
            self.command_input.clear();
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

        Ok(())
    }

    fn handle_command_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches_any(&self.keybinds.command.cancel, key) {
            self.mode = Mode::Normal;
            self.status_line = "Command canceled".to_string();
            return Ok(());
        }

        if matches_any(&self.keybinds.command.backspace, key) {
            self.command_input.pop();
            return Ok(());
        }

        if matches_any(&self.keybinds.command.submit, key) {
            let command = self.command_input.clone();
            self.mode = Mode::Normal;
            self.command_input.clear();
            self.execute_command_line(&command)?;
            return Ok(());
        }

        if let KeyCode::Char(ch) = key.code {
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
            return Ok(());
        }

        self.execute_tokens(tokens)
    }

    fn execute_command_line(&mut self, command: &str) -> Result<(), JkError> {
        match plan_command(command, self.selected_revision()) {
            FlowAction::Quit => {
                self.should_quit = true;
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

    fn execute_tokens(&mut self, tokens: Vec<String>) -> Result<(), JkError> {
        let result = jj::run(&tokens)?;
        self.lines = result.output;
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
        let line = self.lines.get(self.cursor)?;
        extract_revision(line)
    }

    fn draw(&mut self, stdout: &mut Stdout) -> Result<(), JkError> {
        let (width, height) = terminal::size()?;
        let width = width as usize;
        let height = height as usize;

        let frame = self.render_for_display(width, height);
        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        for (index, line) in frame.into_iter().enumerate() {
            queue!(stdout, MoveTo(0, index as u16), Print(line))?;
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

fn trim_to_width(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    text.chars().take(width).collect()
}

fn extract_revision(line: &str) -> Option<String> {
    let change_re = Regex::new(r"\b[a-z]{8,}(?:/[0-9]+)?\b").ok()?;
    if let Some(found) = change_re.find(line) {
        return Some(found.as_str().to_string());
    }

    let commit_re = Regex::new(r"\b[0-9a-f]{8,}\b").ok()?;
    commit_re.find(line).map(|found| found.as_str().to_string())
}

fn matches_any(bindings: &[KeyBinding], key: KeyEvent) -> bool {
    bindings.iter().any(|binding| binding.matches(key))
}

fn is_dangerous(tokens: &[String]) -> bool {
    let Some(first) = tokens.first().map(String::as_str) else {
        return false;
    };

    match first {
        "rebase" | "squash" | "split" | "abandon" | "undo" | "redo" | "revert" | "restore" => true,
        "git" => matches!(tokens.get(1).map(String::as_str), Some("push")),
        "bookmark" => matches!(
            tokens.get(1).map(String::as_str),
            Some("set" | "move" | "delete" | "forget" | "rename")
        ),
        _ => false,
    }
}

struct TerminalSession {
    stdout: Stdout,
}

impl TerminalSession {
    fn enter() -> Result<Self, JkError> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self { stdout })
    }

    fn stdout_mut(&mut self) -> &mut Stdout {
        &mut self.stdout
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(
            self.stdout,
            Show,
            LeaveAlternateScreen,
            Clear(ClearType::All)
        );
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use crate::config::KeybindConfig;

    use super::{App, extract_revision};

    #[test]
    fn extracts_change_id_from_log_line() {
        let line = "@  abcdefgh joshka@example.com 2026-02-07 0123abcd";
        assert_eq!(extract_revision(line), Some("abcdefgh".to_string()));
    }

    #[test]
    fn extracts_commit_id_when_change_missing() {
        let line = "Commit hash 0123abcd updated";
        assert_eq!(extract_revision(line), Some("0123abcd".to_string()));
    }

    #[test]
    fn snapshot_renders_basic_frame() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.lines = vec![
            "@  abcdefgh 0123abcd message".to_string(),
            "○  hgfedcba 89abcdef parent".to_string(),
        ];
        app.status_line = "Ready".to_string();
        insta::assert_snapshot!(app.render_for_snapshot(60, 6));
    }
}
