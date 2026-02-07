use std::collections::HashMap;
use std::io::{self, Stdout, Write};
use std::time::Duration;

use crate::commands::{SafetyTier, command_safety};
use crate::config::KeybindConfig;
use crate::error::JkError;
use crate::flows::{FlowAction, PromptKind, PromptRequest, plan_command};
use crate::jj;
use crate::keys::KeyBinding;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use crossterm::{execute, queue};

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
        let action = plan_command(command, self.selected_revision());
        self.apply_flow_action(action)
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

fn startup_action(startup_tokens: &[String]) -> FlowAction {
    if startup_tokens.is_empty() {
        FlowAction::Execute(vec!["log".to_string()])
    } else {
        let startup_command = startup_tokens.join(" ");
        plan_command(&startup_command, None)
    }
}

fn derive_row_revision_map(tokens: &[String], lines: &[String]) -> Vec<Option<String>> {
    if !matches!(tokens.first().map(String::as_str), Some("log")) {
        return vec![None; lines.len()];
    }

    let Some(metadata_tokens) = metadata_log_tokens(tokens) else {
        return vec![None; lines.len()];
    };

    let revisions = match jj::run(&metadata_tokens) {
        Ok(result) if result.success => parse_log_revisions(&result.output),
        _ => Vec::new(),
    };

    build_row_revision_map(lines, &revisions)
}

fn metadata_log_tokens(tokens: &[String]) -> Option<Vec<String>> {
    if !matches!(tokens.first().map(String::as_str), Some("log")) {
        return None;
    }

    let mut metadata_tokens = vec![
        "log".to_string(),
        "--no-graph".to_string(),
        "-T".to_string(),
        "change_id.short() ++ \" \" ++ commit_id.short()".to_string(),
    ];

    let mut skip_next_value = false;
    for token in tokens.iter().skip(1) {
        if skip_next_value {
            skip_next_value = false;
            continue;
        }

        match token.as_str() {
            "-T" | "--template" => {
                skip_next_value = true;
            }
            "--graph" | "--no-graph" | "-p" | "--patch" => {}
            value => metadata_tokens.push(value.to_string()),
        }
    }

    Some(metadata_tokens)
}

fn parse_log_revisions(lines: &[String]) -> Vec<String> {
    let mut revisions = Vec::new();
    for line in lines {
        let Some(token) = line.split_whitespace().next().map(trim_revision_token) else {
            continue;
        };
        if is_change_id(token) || is_commit_id(token) {
            revisions.push(token.to_string());
        }
    }
    revisions
}

fn build_row_revision_map(lines: &[String], ordered_revisions: &[String]) -> Vec<Option<String>> {
    let mut revision_positions = HashMap::new();
    for (index, revision) in ordered_revisions.iter().enumerate() {
        revision_positions.insert(revision.clone(), index);
    }

    let mut map = Vec::with_capacity(lines.len());
    let mut current: Option<String> = None;
    let mut next_ordinal = 0usize;

    for line in lines {
        if let Some(explicit) = extract_revision(line) {
            if let Some(position) = revision_positions.get(&explicit) {
                current = Some(explicit);
                next_ordinal = (*position + 1).max(next_ordinal);
            } else if ordered_revisions.is_empty() {
                current = Some(explicit);
            }
        } else if looks_like_graph_commit_row(line) && next_ordinal < ordered_revisions.len() {
            current = ordered_revisions.get(next_ordinal).cloned();
            next_ordinal += 1;
        }

        map.push(current.clone());
    }

    map
}

fn looks_like_graph_commit_row(line: &str) -> bool {
    for ch in line.chars() {
        if ch.is_whitespace()
            || matches!(
                ch,
                '│' | '┃'
                    | '┆'
                    | '┊'
                    | '┄'
                    | '┈'
                    | '─'
                    | '┬'
                    | '┴'
                    | '┼'
                    | '╭'
                    | '╮'
                    | '╯'
                    | '╰'
                    | '|'
                    | '/'
                    | '\\'
            )
        {
            continue;
        }
        return matches!(ch, '@' | '○' | '◉' | '●' | '◆' | '◌' | 'x' | 'X' | '*');
    }

    false
}

fn trim_to_width(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    text.chars().take(width).collect()
}

fn extract_revision(line: &str) -> Option<String> {
    let tokens: Vec<&str> = line
        .split_whitespace()
        .map(trim_revision_token)
        .filter(|token| !token.is_empty())
        .collect();

    let commit_index = tokens.iter().position(|token| is_commit_id(token))?;

    for token in &tokens[..commit_index] {
        if is_change_id(token) {
            return Some((*token).to_string());
        }
    }

    tokens.get(commit_index).map(|token| (*token).to_string())
}

fn trim_revision_token(token: &str) -> &str {
    token.trim_matches(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '/'))
}

fn is_commit_id(value: &str) -> bool {
    value.len() >= 8 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn is_change_id(value: &str) -> bool {
    let Some((head, _counter)) = value.split_once('/') else {
        return value.len() >= 8 && value.chars().all(|ch| ch.is_ascii_lowercase());
    };

    !head.is_empty()
        && head.chars().all(|ch| ch.is_ascii_lowercase())
        && value.len() >= 8
        && value
            .rsplit_once('/')
            .map(|(_, suffix)| suffix.chars().all(|ch| ch.is_ascii_digit()))
            .unwrap_or(false)
}

fn matches_any(bindings: &[KeyBinding], key: KeyEvent) -> bool {
    bindings.iter().any(|binding| binding.matches(key))
}

fn is_dangerous(tokens: &[String]) -> bool {
    command_safety(tokens) == SafetyTier::C
}

fn confirmation_preview_tokens(tokens: &[String]) -> Option<Vec<String>> {
    if matches!(
        (
            tokens.first().map(String::as_str),
            tokens.get(1).map(String::as_str)
        ),
        (Some("git"), Some("push"))
    ) && !tokens.iter().any(|token| token == "--dry-run")
    {
        let mut preview = tokens.to_vec();
        preview.push("--dry-run".to_string());
        return Some(preview);
    }

    if matches!(
        (
            tokens.first().map(String::as_str),
            tokens.get(1).map(String::as_str)
        ),
        (Some("operation"), Some("restore" | "revert"))
    ) {
        let operation = tokens
            .get(2)
            .filter(|value| !value.starts_with('-'))
            .cloned()
            .unwrap_or_else(|| "@".to_string());
        return Some(vec![
            "operation".to_string(),
            "show".to_string(),
            operation,
            "--no-op-diff".to_string(),
        ]);
    }

    match tokens.first().map(String::as_str) {
        Some("rebase") => {
            let source = find_flag_value(tokens, &["-r", "--revision", "-b", "--branch"])
                .unwrap_or_else(|| "@".to_string());
            let destination = find_flag_value(tokens, &["-d", "--destination", "--onto"])?;
            Some(log_preview_tokens(&format!("{source} | {destination}")))
        }
        Some("squash") => {
            let from = find_flag_value(tokens, &["--from"]).unwrap_or_else(|| "@".to_string());
            let into = find_flag_value(tokens, &["--into"]).unwrap_or_else(|| "@-".to_string());
            Some(log_preview_tokens(&format!("{from} | {into}")))
        }
        Some("split") => {
            let revision =
                find_flag_value(tokens, &["-r", "--revision"]).unwrap_or_else(|| "@".to_string());
            Some(vec!["show".to_string(), revision])
        }
        Some("abandon") => {
            let revision = tokens.get(1).cloned().unwrap_or_else(|| "@".to_string());
            Some(log_preview_tokens(&revision))
        }
        Some("restore") => {
            let from = find_flag_value(tokens, &["--from"]).unwrap_or_else(|| "@-".to_string());
            let to = find_flag_value(tokens, &["--to"]).unwrap_or_else(|| "@".to_string());
            Some(log_preview_tokens(&format!("{from} | {to}")))
        }
        Some("revert") => {
            let revisions =
                find_flag_value(tokens, &["-r", "--revisions"]).unwrap_or_else(|| "@".to_string());
            let onto =
                find_flag_value(tokens, &["-o", "--onto"]).unwrap_or_else(|| "@".to_string());
            Some(log_preview_tokens(&format!("{revisions} | {onto}")))
        }
        Some("bookmark")
            if matches!(
                tokens.get(1).map(String::as_str),
                Some("set" | "move" | "delete" | "forget" | "rename")
            ) =>
        {
            Some(vec![
                "bookmark".to_string(),
                "list".to_string(),
                "--all".to_string(),
            ])
        }
        Some("git") if matches!(tokens.get(1).map(String::as_str), Some("push")) => None,
        Some("undo" | "redo") => Some(operation_log_preview_tokens()),
        _ if is_dangerous(tokens) => Some(operation_log_preview_tokens()),
        _ => None,
    }
}

fn find_flag_value(tokens: &[String], flags: &[&str]) -> Option<String> {
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        for flag in flags {
            if token == flag {
                if let Some(value) = tokens.get(index + 1) {
                    return Some(value.clone());
                }
            } else if let Some(value) = token.strip_prefix(&format!("{flag}=")) {
                return Some(value.to_string());
            }
        }
        index += 1;
    }

    None
}

fn log_preview_tokens(revset: &str) -> Vec<String> {
    vec![
        "log".to_string(),
        "-r".to_string(),
        revset.to_string(),
        "-n".to_string(),
        "20".to_string(),
    ]
}

fn toggle_patch_flag(tokens: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(tokens.len() + 1);
    let mut has_patch = false;

    for token in tokens {
        if token == "-p" || token == "--patch" {
            has_patch = true;
            continue;
        }
        result.push(token.clone());
    }

    if !has_patch {
        result.push("--patch".to_string());
    }

    result
}

fn operation_log_preview_tokens() -> Vec<String> {
    vec![
        "operation".to_string(),
        "log".to_string(),
        "-n".to_string(),
        "5".to_string(),
    ]
}

fn decorate_command_output(command: &[String], output: Vec<String>) -> Vec<String> {
    match command.first().map(String::as_str) {
        Some("status") => render_status_view(output),
        Some("show") => render_show_view(output),
        Some("diff") => render_diff_view(output),
        _ => output,
    }
}

fn render_status_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Status Overview".to_string(),
        "===============".to_string(),
        String::new(),
    ];

    let mut section_has_items = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.ends_with(':') {
            if section_has_items {
                rendered.push(String::new());
            }
            rendered.push(trimmed.to_string());
            section_has_items = false;
            continue;
        }

        if matches!(
            rendered.last().map(String::as_str),
            Some("Working copy changes:")
        ) {
            rendered.push(format!("  {trimmed}"));
            section_has_items = true;
            continue;
        }

        rendered.push(trimmed.to_string());
        section_has_items = true;
    }

    rendered.push(String::new());
    rendered.push("Shortcuts: s status, F fetch, P push, B rebase, :commands".to_string());
    rendered
}

fn render_show_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Show View".to_string(),
        "=========".to_string(),
        String::new(),
    ];
    rendered.extend(lines);
    rendered.push(String::new());
    rendered.push("Shortcuts: Enter show selected, d diff selected, s status".to_string());
    rendered
}

fn render_diff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Diff View".to_string(),
        "=========".to_string(),
        String::new(),
    ];
    rendered.extend(lines);
    rendered.push(String::new());
    rendered.push("Shortcuts: d diff selected, Enter show selected, s status".to_string());
    rendered
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
    use crossterm::event::{KeyCode, KeyEvent};

    use crate::config::KeybindConfig;

    use crate::flows::{FlowAction, PromptKind};

    use super::{
        App, Mode, build_row_revision_map, confirmation_preview_tokens, extract_revision,
        is_change_id, is_commit_id, is_dangerous, looks_like_graph_commit_row, metadata_log_tokens,
        render_diff_view, render_show_view, render_status_view, startup_action, toggle_patch_flag,
    };

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

    #[test]
    fn marks_tier_c_commands_as_dangerous() {
        assert!(is_dangerous(&["rebase".to_string()]));
        assert!(is_dangerous(&["squash".to_string()]));
        assert!(is_dangerous(&["split".to_string()]));
        assert!(is_dangerous(&["abandon".to_string()]));
        assert!(is_dangerous(&["undo".to_string()]));
        assert!(is_dangerous(&["redo".to_string()]));
        assert!(is_dangerous(&["restore".to_string()]));
        assert!(is_dangerous(&["revert".to_string()]));
        assert!(is_dangerous(&["git".to_string(), "push".to_string()]));
        assert!(is_dangerous(&["bookmark".to_string(), "set".to_string()]));
        assert!(is_dangerous(&["bookmark".to_string(), "move".to_string()]));
        assert!(is_dangerous(&[
            "bookmark".to_string(),
            "delete".to_string()
        ]));
        assert!(is_dangerous(&[
            "bookmark".to_string(),
            "forget".to_string()
        ]));
        assert!(is_dangerous(&[
            "bookmark".to_string(),
            "rename".to_string()
        ]));
    }

    #[test]
    fn leaves_read_and_low_risk_commands_unguarded() {
        assert!(!is_dangerous(&["log".to_string()]));
        assert!(!is_dangerous(&["status".to_string()]));
        assert!(!is_dangerous(&["show".to_string()]));
        assert!(!is_dangerous(&["diff".to_string()]));
        assert!(!is_dangerous(&["git".to_string(), "fetch".to_string()]));
        assert!(!is_dangerous(&["bookmark".to_string(), "list".to_string()]));
        assert!(!is_dangerous(&[
            "bookmark".to_string(),
            "create".to_string()
        ]));
        assert!(!is_dangerous(&["next".to_string()]));
        assert!(!is_dangerous(&["prev".to_string()]));
        assert!(!is_dangerous(&["edit".to_string()]));
    }

    #[test]
    fn selected_revision_falls_back_to_previous_revision_line() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.lines = vec![
            "@  abcdefgh 0123abcd top commit".to_string(),
            "│  detailed message line without ids".to_string(),
            "○  hgfedcba 89abcdef parent commit".to_string(),
        ];
        app.cursor = 1;

        assert_eq!(app.selected_revision(), Some("abcdefgh".to_string()));
    }

    #[test]
    fn does_not_extract_revision_from_message_line_without_commit_id() {
        let line = "│  detailed message line without ids";
        assert_eq!(extract_revision(line), None);
    }

    #[test]
    fn recognizes_change_and_commit_id_formats() {
        assert!(is_change_id("abcdefgh"));
        assert!(is_change_id("abcdefgh/12"));
        assert!(!is_change_id("abc-defgh"));

        assert!(is_commit_id("0123abcd"));
        assert!(!is_commit_id("abcdefgh"));
    }

    #[test]
    fn startup_action_defaults_to_log() {
        assert_eq!(
            startup_action(&[]),
            FlowAction::Execute(vec!["log".to_string()])
        );
    }

    #[test]
    fn startup_action_uses_guided_planner() {
        let tokens = vec!["new".to_string()];
        match startup_action(&tokens) {
            FlowAction::Prompt(request) => {
                assert_eq!(request.kind, PromptKind::NewMessage);
            }
            other => panic!("expected prompt, got {other:?}"),
        }
    }

    #[test]
    fn startup_dangerous_command_requires_confirmation() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.apply_startup_tokens(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "main".to_string(),
        ])
        .expect("startup action should succeed");

        assert_eq!(app.mode, Mode::Confirm);
        assert_eq!(
            app.pending_confirm,
            Some(vec![
                "rebase".to_string(),
                "-d".to_string(),
                "main".to_string()
            ])
        );
    }

    #[test]
    fn startup_commands_view_renders_without_running_jj() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.apply_startup_tokens(vec!["commands".to_string()])
            .expect("startup action should succeed");

        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.status_line, "Showing command registry".to_string());
        assert!(
            app.lines
                .iter()
                .any(|line| line.contains("jj top-level coverage"))
        );
    }

    #[test]
    fn confirm_preview_renders_header_for_tier_c_commands() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.execute_with_confirmation(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "main".to_string(),
        ])
        .expect("confirmation setup should succeed");

        assert_eq!(app.mode, Mode::Confirm);
        assert!(
            app.lines
                .iter()
                .any(|line| line.contains("Confirm: jj rebase -d main"))
        );
    }

    #[test]
    fn builds_dry_run_preview_for_git_push() {
        let preview = confirmation_preview_tokens(&["git".to_string(), "push".to_string()]);
        assert_eq!(
            preview,
            Some(vec![
                "git".to_string(),
                "push".to_string(),
                "--dry-run".to_string()
            ])
        );

        let existing = confirmation_preview_tokens(&[
            "git".to_string(),
            "push".to_string(),
            "--dry-run".to_string(),
        ]);
        assert_eq!(existing, None);
    }

    #[test]
    fn builds_operation_preview_for_restore_and_revert() {
        let restore = confirmation_preview_tokens(&[
            "operation".to_string(),
            "restore".to_string(),
            "abc123".to_string(),
        ]);
        assert_eq!(
            restore,
            Some(vec![
                "operation".to_string(),
                "show".to_string(),
                "abc123".to_string(),
                "--no-op-diff".to_string()
            ])
        );

        let revert_default =
            confirmation_preview_tokens(&["operation".to_string(), "revert".to_string()]);
        assert_eq!(
            revert_default,
            Some(vec![
                "operation".to_string(),
                "show".to_string(),
                "@".to_string(),
                "--no-op-diff".to_string()
            ])
        );
    }

    #[test]
    fn builds_rebase_and_squash_preview_revsets() {
        let rebase = confirmation_preview_tokens(&[
            "rebase".to_string(),
            "-d".to_string(),
            "main".to_string(),
        ]);
        assert_eq!(
            rebase,
            Some(vec![
                "log".to_string(),
                "-r".to_string(),
                "@ | main".to_string(),
                "-n".to_string(),
                "20".to_string()
            ])
        );

        let squash = confirmation_preview_tokens(&[
            "squash".to_string(),
            "--from".to_string(),
            "abc123".to_string(),
            "--into".to_string(),
            "@-".to_string(),
        ]);
        assert_eq!(
            squash,
            Some(vec![
                "log".to_string(),
                "-r".to_string(),
                "abc123 | @-".to_string(),
                "-n".to_string(),
                "20".to_string()
            ])
        );
    }

    #[test]
    fn builds_split_and_abandon_preview_commands() {
        let split = confirmation_preview_tokens(&[
            "split".to_string(),
            "-r".to_string(),
            "abc123".to_string(),
            "src/main.rs".to_string(),
        ]);
        assert_eq!(split, Some(vec!["show".to_string(), "abc123".to_string()]));

        let abandon = confirmation_preview_tokens(&["abandon".to_string(), "abc123".to_string()]);
        assert_eq!(
            abandon,
            Some(vec![
                "log".to_string(),
                "-r".to_string(),
                "abc123".to_string(),
                "-n".to_string(),
                "20".to_string()
            ])
        );
    }

    #[test]
    fn builds_restore_revert_bookmark_and_operation_log_previews() {
        let restore = confirmation_preview_tokens(&[
            "restore".to_string(),
            "--from".to_string(),
            "@-".to_string(),
            "--to".to_string(),
            "@".to_string(),
        ]);
        assert_eq!(
            restore,
            Some(vec![
                "log".to_string(),
                "-r".to_string(),
                "@- | @".to_string(),
                "-n".to_string(),
                "20".to_string()
            ])
        );

        let revert = confirmation_preview_tokens(&[
            "revert".to_string(),
            "-r".to_string(),
            "abc123".to_string(),
            "-o".to_string(),
            "@".to_string(),
        ]);
        assert_eq!(
            revert,
            Some(vec![
                "log".to_string(),
                "-r".to_string(),
                "abc123 | @".to_string(),
                "-n".to_string(),
                "20".to_string()
            ])
        );

        let bookmark = confirmation_preview_tokens(&[
            "bookmark".to_string(),
            "set".to_string(),
            "feature".to_string(),
            "-r".to_string(),
            "@".to_string(),
        ]);
        assert_eq!(
            bookmark,
            Some(vec![
                "bookmark".to_string(),
                "list".to_string(),
                "--all".to_string()
            ])
        );

        let undo = confirmation_preview_tokens(&["undo".to_string()]);
        assert_eq!(
            undo,
            Some(vec![
                "operation".to_string(),
                "log".to_string(),
                "-n".to_string(),
                "5".to_string()
            ])
        );
    }

    #[test]
    fn falls_back_to_operation_log_for_unknown_tier_c_commands() {
        let preview = confirmation_preview_tokens(&[
            "simplify-parents".to_string(),
            "-r".to_string(),
            "@".to_string(),
        ]);
        assert_eq!(
            preview,
            Some(vec![
                "operation".to_string(),
                "log".to_string(),
                "-n".to_string(),
                "5".to_string()
            ])
        );

        let safe = confirmation_preview_tokens(&["status".to_string()]);
        assert_eq!(safe, None);
    }

    #[test]
    fn metadata_log_tokens_strip_template_and_patch_options() {
        let tokens = vec![
            "log".to_string(),
            "-r".to_string(),
            "all()".to_string(),
            "-T".to_string(),
            "user_template".to_string(),
            "--patch".to_string(),
        ];
        assert_eq!(
            metadata_log_tokens(&tokens),
            Some(vec![
                "log".to_string(),
                "--no-graph".to_string(),
                "-T".to_string(),
                "change_id.short() ++ \" \" ++ commit_id.short()".to_string(),
                "-r".to_string(),
                "all()".to_string(),
            ])
        );
    }

    #[test]
    fn graph_row_detection_handles_connectors() {
        assert!(looks_like_graph_commit_row("@  abcdefgh 0123abcd message"));
        assert!(looks_like_graph_commit_row("│ ○ hgfedcba 89abcdef parent"));
        assert!(!looks_like_graph_commit_row("│ detailed message line"));
    }

    #[test]
    fn row_revision_map_falls_back_to_metadata_order() {
        let lines = vec![
            "@  no explicit ids".to_string(),
            "│ detail row".to_string(),
            "○ also missing ids".to_string(),
            "│ detail row two".to_string(),
        ];
        let metadata = vec!["abcdefgh".to_string(), "hgfedcba".to_string()];
        let map = build_row_revision_map(&lines, &metadata);

        assert_eq!(
            map,
            vec![
                Some("abcdefgh".to_string()),
                Some("abcdefgh".to_string()),
                Some("hgfedcba".to_string()),
                Some("hgfedcba".to_string()),
            ]
        );
    }

    #[test]
    fn renders_status_output_as_scannable_sections() {
        let rendered = render_status_view(vec![
            "Working copy changes:".to_string(),
            "M src/app.rs".to_string(),
            "A src/new.rs".to_string(),
            "Working copy  (@) : abcdefgh 0123abcd summary".to_string(),
            "Parent commit (@-): hgfedcba 89abcdef parent".to_string(),
            "Conflicted bookmarks:".to_string(),
            "  feature".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Status Overview".to_string()));
        assert!(rendered.iter().any(|line| line == "Working copy changes:"));
        assert!(rendered.iter().any(|line| line == "  M src/app.rs"));
        assert!(rendered.iter().any(|line| line == "Conflicted bookmarks:"));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Shortcuts: s status"))
        );
    }

    #[test]
    fn renders_show_view_with_header_and_shortcuts() {
        let rendered = render_show_view(vec![
            "Commit ID: abcdef0123456789".to_string(),
            "Change ID: abcdefghijklmnop".to_string(),
            "Modified regular file src/app.rs:".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Show View".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line == "Commit ID: abcdef0123456789")
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Shortcuts: Enter show selected"))
        );
    }

    #[test]
    fn renders_diff_view_with_header_and_shortcuts() {
        let rendered = render_diff_view(vec![
            "Modified regular file src/app.rs:".to_string(),
            "  1  1: use std::collections::HashMap;".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Diff View".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line == "Modified regular file src/app.rs:")
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Shortcuts: d diff selected"))
        );
    }

    #[test]
    fn toggles_patch_flag_for_log_commands() {
        assert_eq!(
            toggle_patch_flag(&["log".to_string(), "-r".to_string(), "all()".to_string()]),
            vec![
                "log".to_string(),
                "-r".to_string(),
                "all()".to_string(),
                "--patch".to_string()
            ]
        );

        assert_eq!(
            toggle_patch_flag(&[
                "log".to_string(),
                "--patch".to_string(),
                "-r".to_string(),
                "all()".to_string()
            ]),
            vec!["log".to_string(), "-r".to_string(), "all()".to_string()]
        );
    }

    #[test]
    fn normal_mode_shortcuts_route_to_expected_flows() {
        let mut fetch_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        fetch_app
            .handle_key(KeyEvent::from(KeyCode::Char('F')))
            .expect("fetch shortcut should be handled");
        assert_eq!(fetch_app.mode, Mode::Prompt);
        assert_eq!(
            fetch_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::GitFetchRemote)
        );

        let mut push_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        push_app
            .handle_key(KeyEvent::from(KeyCode::Char('P')))
            .expect("push shortcut should be handled");
        assert_eq!(push_app.mode, Mode::Prompt);
        assert_eq!(
            push_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::GitPushBookmark)
        );

        let mut rebase_main_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        rebase_main_app
            .handle_key(KeyEvent::from(KeyCode::Char('M')))
            .expect("rebase-main shortcut should be handled");
        assert_eq!(rebase_main_app.mode, Mode::Confirm);
        assert_eq!(
            rebase_main_app.pending_confirm,
            Some(vec![
                "rebase".to_string(),
                "-d".to_string(),
                "main".to_string()
            ])
        );

        let mut rebase_trunk_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        rebase_trunk_app
            .handle_key(KeyEvent::from(KeyCode::Char('T')))
            .expect("rebase-trunk shortcut should be handled");
        assert_eq!(rebase_trunk_app.mode, Mode::Confirm);
        assert_eq!(
            rebase_trunk_app.pending_confirm,
            Some(vec![
                "rebase".to_string(),
                "-d".to_string(),
                "trunk()".to_string()
            ])
        );

        let mut new_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        new_app
            .handle_key(KeyEvent::from(KeyCode::Char('n')))
            .expect("new shortcut should be handled");
        assert_eq!(new_app.mode, Mode::Prompt);
        assert_eq!(
            new_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::NewMessage)
        );

        let mut commit_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        commit_app
            .handle_key(KeyEvent::from(KeyCode::Char('c')))
            .expect("commit shortcut should be handled");
        assert_eq!(commit_app.mode, Mode::Prompt);
        assert_eq!(
            commit_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::CommitMessage)
        );

        let mut describe_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        describe_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        describe_app
            .handle_key(KeyEvent::from(KeyCode::Char('D')))
            .expect("describe shortcut should be handled");
        assert_eq!(describe_app.mode, Mode::Prompt);
        assert_eq!(
            describe_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::DescribeMessage {
                revision: "abcdefgh".to_string()
            })
        );

        let mut bookmark_set_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        bookmark_set_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        bookmark_set_app
            .handle_key(KeyEvent::from(KeyCode::Char('b')))
            .expect("bookmark-set shortcut should be handled");
        assert_eq!(bookmark_set_app.mode, Mode::Prompt);
        assert_eq!(
            bookmark_set_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::BookmarkSet {
                target_revision: "abcdefgh".to_string()
            })
        );

        let mut abandon_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        abandon_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        abandon_app
            .handle_key(KeyEvent::from(KeyCode::Char('a')))
            .expect("abandon shortcut should be handled");
        assert_eq!(abandon_app.mode, Mode::Confirm);
        assert_eq!(
            abandon_app.pending_confirm,
            Some(vec!["abandon".to_string(), "abcdefgh".to_string()])
        );

        let mut rebase_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        rebase_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        rebase_app
            .handle_key(KeyEvent::from(KeyCode::Char('B')))
            .expect("rebase shortcut should be handled");
        assert_eq!(rebase_app.mode, Mode::Prompt);
        assert_eq!(
            rebase_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::RebaseDestination {
                source_revision: "abcdefgh".to_string()
            })
        );

        let mut squash_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        squash_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        squash_app
            .handle_key(KeyEvent::from(KeyCode::Char('S')))
            .expect("squash shortcut should be handled");
        assert_eq!(squash_app.mode, Mode::Prompt);
        assert_eq!(
            squash_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::SquashInto {
                from_revision: "abcdefgh".to_string()
            })
        );

        let mut split_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        split_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        split_app
            .handle_key(KeyEvent::from(KeyCode::Char('X')))
            .expect("split shortcut should be handled");
        assert_eq!(split_app.mode, Mode::Prompt);
        assert_eq!(
            split_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::SplitFileset {
                revision: "abcdefgh".to_string()
            })
        );

        let mut restore_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        restore_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        restore_app
            .handle_key(KeyEvent::from(KeyCode::Char('O')))
            .expect("restore shortcut should be handled");
        assert_eq!(restore_app.mode, Mode::Prompt);
        assert_eq!(
            restore_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::RestoreFrom {
                target_revision: "abcdefgh".to_string()
            })
        );

        let mut revert_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        revert_app.lines = vec!["@  abcdefgh 0123abcd message".to_string()];
        revert_app
            .handle_key(KeyEvent::from(KeyCode::Char('R')))
            .expect("revert shortcut should be handled");
        assert_eq!(revert_app.mode, Mode::Prompt);
        assert_eq!(
            revert_app.pending_prompt.map(|prompt| prompt.kind),
            Some(PromptKind::RevertRevisions {
                default_revisions: "abcdefgh".to_string(),
                onto_revision: "@".to_string()
            })
        );

        let mut undo_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        undo_app
            .handle_key(KeyEvent::from(KeyCode::Char('u')))
            .expect("undo shortcut should be handled");
        assert_eq!(undo_app.mode, Mode::Confirm);
        assert_eq!(undo_app.pending_confirm, Some(vec!["undo".to_string()]));

        let mut redo_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        redo_app
            .handle_key(KeyEvent::from(KeyCode::Char('U')))
            .expect("redo shortcut should be handled");
        assert_eq!(redo_app.mode, Mode::Confirm);
        assert_eq!(redo_app.pending_confirm, Some(vec!["redo".to_string()]));

        let mut status_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        status_app
            .handle_key(KeyEvent::from(KeyCode::Char('s')))
            .expect("status shortcut should be handled");
        assert_eq!(status_app.mode, Mode::Normal);
        assert_eq!(status_app.last_command, vec!["status".to_string()]);
        assert!(
            status_app
                .lines
                .iter()
                .any(|line| line.contains("Status Overview"))
        );

        let mut help_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        help_app
            .handle_key(KeyEvent::from(KeyCode::Char('?')))
            .expect("help shortcut should be handled");
        assert_eq!(help_app.mode, Mode::Normal);
        assert_eq!(help_app.status_line, "Showing command registry".to_string());
        assert!(
            help_app
                .lines
                .iter()
                .any(|line| line.contains("jj top-level coverage"))
        );

        let mut repeat_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        repeat_app.last_command = vec!["status".to_string()];
        repeat_app
            .handle_key(KeyEvent::from(KeyCode::Char('.')))
            .expect("repeat-last shortcut should be handled");
        assert_eq!(repeat_app.mode, Mode::Normal);
        assert_eq!(repeat_app.last_command, vec!["status".to_string()]);
        assert!(
            repeat_app
                .lines
                .iter()
                .any(|line| line.contains("Status Overview"))
        );
    }

    #[test]
    fn command_history_navigates_previous_and_next_entries() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.mode = Mode::Command;
        app.command_history = vec!["status".to_string(), "log -n 5".to_string()];

        app.handle_key(KeyEvent::from(KeyCode::Up))
            .expect("history previous should succeed");
        assert_eq!(app.command_input, "log -n 5".to_string());

        app.handle_key(KeyEvent::from(KeyCode::Up))
            .expect("history previous should stay at oldest");
        assert_eq!(app.command_input, "status".to_string());

        app.handle_key(KeyEvent::from(KeyCode::Down))
            .expect("history next should succeed");
        assert_eq!(app.command_input, "log -n 5".to_string());
    }

    #[test]
    fn command_history_restores_draft_after_navigation() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.mode = Mode::Command;
        app.command_history = vec!["status".to_string(), "log -n 5".to_string()];
        app.command_input = "boo".to_string();

        app.handle_key(KeyEvent::from(KeyCode::Up))
            .expect("history previous should succeed");
        assert_eq!(app.command_input, "log -n 5".to_string());

        app.handle_key(KeyEvent::from(KeyCode::Down))
            .expect("history next should restore draft");
        assert_eq!(app.command_input, "boo".to_string());
    }
}
