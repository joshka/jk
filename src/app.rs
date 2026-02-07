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
        Some("new") => render_top_level_mutation_view("new", output),
        Some("describe") => render_top_level_mutation_view("describe", output),
        Some("commit") => render_top_level_mutation_view("commit", output),
        Some("edit") => render_top_level_mutation_view("edit", output),
        Some("next") => render_top_level_mutation_view("next", output),
        Some("prev") => render_top_level_mutation_view("prev", output),
        Some("rebase") => render_top_level_mutation_view("rebase", output),
        Some("squash") => render_top_level_mutation_view("squash", output),
        Some("split") => render_top_level_mutation_view("split", output),
        Some("abandon") => render_top_level_mutation_view("abandon", output),
        Some("undo") => render_top_level_mutation_view("undo", output),
        Some("redo") => render_top_level_mutation_view("redo", output),
        Some("restore") => render_top_level_mutation_view("restore", output),
        Some("revert") => render_top_level_mutation_view("revert", output),
        Some("root") => render_root_view(output),
        Some("resolve") if has_resolve_list_flag(command) => render_resolve_list_view(output),
        Some("file") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_file_list_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("show")) => {
            render_file_show_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("search")) => {
            render_file_search_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("annotate")) => {
            render_file_annotate_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("track")) => {
            render_file_track_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("untrack")) => {
            render_file_untrack_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("chmod")) => {
            render_file_chmod_view(output)
        }
        Some("tag") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_tag_list_view(output)
        }
        Some("tag") if matches!(command.get(1).map(String::as_str), Some("set")) => {
            render_tag_set_view(output)
        }
        Some("tag") if matches!(command.get(1).map(String::as_str), Some("delete")) => {
            render_tag_delete_view(output)
        }
        Some("workspace") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_workspace_list_view(output)
        }
        Some("workspace") if matches!(command.get(1).map(String::as_str), Some("root")) => {
            render_root_view(output)
        }
        Some("git") if matches!(command.get(1).map(String::as_str), Some("fetch")) => {
            render_git_fetch_view(output)
        }
        Some("git") if matches!(command.get(1).map(String::as_str), Some("push")) => {
            render_git_push_view(output)
        }
        Some("bookmark") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_bookmark_list_view(output)
        }
        Some("bookmark")
            if matches!(
                command.get(1).map(String::as_str),
                Some(
                    "create"
                        | "set"
                        | "move"
                        | "track"
                        | "untrack"
                        | "delete"
                        | "forget"
                        | "rename"
                )
            ) =>
        {
            render_bookmark_mutation_view(command.get(1).map(String::as_str), output)
        }
        Some("workspace")
            if matches!(
                command.get(1).map(String::as_str),
                Some("add" | "forget" | "rename" | "update-stale")
            ) =>
        {
            render_workspace_mutation_view(command.get(1).map(String::as_str), output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("diff")) => {
            render_operation_diff_view(output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("show")) => {
            render_operation_show_view(output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("restore")) => {
            render_operation_mutation_view("restore", output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("revert")) => {
            render_operation_mutation_view("revert", output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("log")) => {
            render_operation_log_view(output)
        }
        _ => output,
    }
}

fn render_status_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut section_lines: Vec<String> = Vec::new();
    let mut current_section: Option<String> = None;
    let mut has_working_copy_section = false;
    let mut has_conflicted_section = false;
    let mut working_copy_changes = 0usize;
    let mut conflicted_bookmarks = 0usize;

    for raw_line in lines {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.ends_with(':') {
            if matches!(section_lines.last(), Some(previous) if !previous.is_empty()) {
                section_lines.push(String::new());
            }

            current_section = Some(trimmed.to_string());
            if trimmed == "Working copy changes:" {
                has_working_copy_section = true;
            }
            if trimmed == "Conflicted bookmarks:" {
                has_conflicted_section = true;
            }
            section_lines.push(trimmed.to_string());
            continue;
        }

        match current_section.as_deref() {
            Some("Working copy changes:") => {
                if is_working_copy_change_line(trimmed) {
                    section_lines.push(format!("  {trimmed}"));
                    working_copy_changes += 1;
                } else {
                    current_section = None;
                    section_lines.push(trimmed.to_string());
                }
            }
            Some("Conflicted bookmarks:") => {
                section_lines.push(trimmed.to_string());
                conflicted_bookmarks += 1;
            }
            _ => {
                section_lines.push(trimmed.to_string());
            }
        }
    }

    while matches!(section_lines.last(), Some(previous) if previous.is_empty()) {
        section_lines.pop();
    }

    let mut summary_parts = Vec::new();
    if has_working_copy_section {
        summary_parts.push(format!(
            "{working_copy_changes} working-copy change{}",
            plural_suffix(working_copy_changes)
        ));
    }
    if has_conflicted_section {
        summary_parts.push(format!(
            "{conflicted_bookmarks} conflicted bookmark{}",
            plural_suffix(conflicted_bookmarks)
        ));
    }

    let summary = if summary_parts.is_empty() {
        "Summary: status captured".to_string()
    } else {
        format!("Summary: {}", summary_parts.join(", "))
    };

    let mut rendered = vec![
        "Status Overview".to_string(),
        "===============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(section_lines);
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
    rendered.extend(normalize_show_lines(lines));
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
    rendered.extend(normalize_diff_lines(lines));
    rendered.push(String::new());
    rendered.push("Shortcuts: d diff selected, Enter show selected, s status".to_string());
    rendered
}

fn normalize_show_lines(lines: Vec<String>) -> Vec<String> {
    let mut rendered: Vec<String> = Vec::new();

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.is_empty() {
            if matches!(rendered.last(), Some(previous) if !previous.is_empty()) {
                rendered.push(String::new());
            }
            continue;
        }

        if is_top_level_section_header(&line)
            && matches!(rendered.last(), Some(previous) if !previous.is_empty())
        {
            rendered.push(String::new());
        }

        rendered.push(line);
    }

    while matches!(rendered.last(), Some(previous) if previous.is_empty()) {
        rendered.pop();
    }

    rendered
}

fn normalize_diff_lines(lines: Vec<String>) -> Vec<String> {
    let mut rendered: Vec<String> = Vec::new();

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();

        if is_top_level_section_header(&line)
            && matches!(rendered.last(), Some(previous) if !previous.is_empty())
        {
            rendered.push(String::new());
        }

        rendered.push(line);
    }

    while matches!(rendered.last(), Some(previous) if previous.is_empty()) {
        rendered.pop();
    }

    rendered
}

fn is_top_level_section_header(line: &str) -> bool {
    !line.starts_with(' ') && line.ends_with(':')
}

fn keymap_overview_lines(config: &KeybindConfig, query: Option<&str>) -> Vec<String> {
    let filter = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    let entries: [(&str, &Vec<KeyBinding>); 47] = [
        ("normal.quit", &config.normal.quit),
        ("normal.refresh", &config.normal.refresh),
        ("normal.up", &config.normal.up),
        ("normal.down", &config.normal.down),
        ("normal.top", &config.normal.top),
        ("normal.bottom", &config.normal.bottom),
        ("normal.command_mode", &config.normal.command_mode),
        ("normal.help", &config.normal.help),
        ("normal.keymap", &config.normal.keymap),
        ("normal.aliases", &config.normal.aliases),
        ("normal.show", &config.normal.show),
        ("normal.diff", &config.normal.diff),
        ("normal.status", &config.normal.status),
        ("normal.operation_log", &config.normal.operation_log),
        ("normal.bookmark_list", &config.normal.bookmark_list),
        ("normal.resolve_list", &config.normal.resolve_list),
        ("normal.file_list", &config.normal.file_list),
        ("normal.tag_list", &config.normal.tag_list),
        ("normal.root", &config.normal.root),
        ("normal.repeat_last", &config.normal.repeat_last),
        ("normal.toggle_patch", &config.normal.toggle_patch),
        ("normal.fetch", &config.normal.fetch),
        ("normal.push", &config.normal.push),
        ("normal.rebase_main", &config.normal.rebase_main),
        ("normal.rebase_trunk", &config.normal.rebase_trunk),
        ("normal.new", &config.normal.new),
        ("normal.next", &config.normal.next),
        ("normal.prev", &config.normal.prev),
        ("normal.edit", &config.normal.edit),
        ("normal.commit", &config.normal.commit),
        ("normal.describe", &config.normal.describe),
        ("normal.bookmark_set", &config.normal.bookmark_set),
        ("normal.abandon", &config.normal.abandon),
        ("normal.rebase", &config.normal.rebase),
        ("normal.squash", &config.normal.squash),
        ("normal.split", &config.normal.split),
        ("normal.restore", &config.normal.restore),
        ("normal.revert", &config.normal.revert),
        ("normal.undo", &config.normal.undo),
        ("normal.redo", &config.normal.redo),
        ("command.submit", &config.command.submit),
        ("command.cancel", &config.command.cancel),
        ("command.backspace", &config.command.backspace),
        ("command.history_prev", &config.command.history_prev),
        ("command.history_next", &config.command.history_next),
        ("confirm.accept", &config.confirm.accept),
        ("confirm.reject", &config.confirm.reject),
    ];

    let mut lines = vec![
        "jk keymap".to_string(),
        format!("{:<24} {}", "action", "keys"),
        "-".repeat(44),
    ];

    for (action, bindings) in entries {
        let labels = key_binding_labels(bindings);
        if let Some(filter) = &filter
            && !action.to_ascii_lowercase().contains(filter)
            && !labels.to_ascii_lowercase().contains(filter)
        {
            continue;
        }

        lines.push(format!("{:<24} {}", action, labels));
    }

    lines
}

fn key_binding_labels(bindings: &[KeyBinding]) -> String {
    bindings
        .iter()
        .map(key_binding_label)
        .collect::<Vec<_>>()
        .join(", ")
}

fn key_binding_label(binding: &KeyBinding) -> String {
    match binding {
        KeyBinding::Char(value) => value.to_string(),
        KeyBinding::Enter => "Enter".to_string(),
        KeyBinding::Esc => "Esc".to_string(),
        KeyBinding::Backspace => "Backspace".to_string(),
        KeyBinding::Up => "Up".to_string(),
        KeyBinding::Down => "Down".to_string(),
        KeyBinding::Left => "Left".to_string(),
        KeyBinding::Right => "Right".to_string(),
        KeyBinding::Home => "Home".to_string(),
        KeyBinding::End => "End".to_string(),
    }
}

fn render_root_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Workspace Root".to_string(),
        "==============".to_string(),
        String::new(),
    ];

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        rendered.push(trimmed.to_string());
    }

    rendered.push(String::new());
    rendered.push("Tip: use jjrt/jk root to inspect current workspace path".to_string());
    rendered
}

fn render_resolve_list_view(lines: Vec<String>) -> Vec<String> {
    let mut body_lines = Vec::new();
    let mut conflict_count = 0usize;
    let mut saw_no_conflicts = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "(no output)" {
            continue;
        }

        if trimmed.contains("No conflicts found") {
            saw_no_conflicts = true;
        } else if !trimmed.starts_with("Error:") && !trimmed.starts_with("Hint:") {
            conflict_count += 1;
        }

        body_lines.push(trimmed.to_string());
    }

    let summary = if saw_no_conflicts || conflict_count == 0 {
        "Summary: no conflicts listed".to_string()
    } else {
        format!(
            "Summary: {conflict_count} conflicted path{} listed",
            plural_suffix(conflict_count)
        )
    };

    let mut rendered = vec![
        "Resolve List".to_string(),
        "============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if body_lines.is_empty() {
        rendered.push("(no conflicts found)".to_string());
    } else {
        rendered.extend(body_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: run `resolve <path>` to open a merge tool for specific conflicts".to_string());
    rendered
}

fn has_resolve_list_flag(command: &[String]) -> bool {
    command
        .iter()
        .any(|token| token == "-l" || token == "--list")
}

fn render_file_list_view(lines: Vec<String>) -> Vec<String> {
    let file_lines: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed != "(no output)"
        })
        .collect();
    let file_count = file_lines.len();

    let mut rendered = vec![
        "File List".to_string(),
        "=========".to_string(),
        String::new(),
        format!(
            "Summary: {file_count} file{} listed",
            plural_suffix(file_count)
        ),
        String::new(),
    ];

    if file_count == 0 {
        rendered.push("(no files listed)".to_string());
    } else {
        rendered.extend(file_lines);
    }

    rendered.push(String::new());
    rendered.push(
        "Tip: use `show`/`diff` with selection to inspect file-affecting revisions".to_string(),
    );
    rendered
}

fn render_file_show_view(lines: Vec<String>) -> Vec<String> {
    let mut content_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .collect();
    if content_lines.len() == 1 && content_lines[0].trim() == "(no output)" {
        content_lines.clear();
    }

    let line_count = content_lines.len();
    let mut rendered = vec![
        "File Show".to_string(),
        "=========".to_string(),
        String::new(),
        format!(
            "Summary: {line_count} content line{}",
            plural_suffix(line_count)
        ),
        String::new(),
    ];

    if content_lines.is_empty() {
        rendered.push("(no file content shown)".to_string());
    } else {
        rendered.extend(content_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: use `show`/`diff -r <rev>` to inspect surrounding change context".to_string());
    rendered
}

fn render_file_search_view(lines: Vec<String>) -> Vec<String> {
    let match_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let match_count = match_lines.len();

    let mut rendered = vec![
        "File Search".to_string(),
        "===========".to_string(),
        String::new(),
        format!(
            "Summary: {match_count} match line{}",
            plural_suffix(match_count)
        ),
        String::new(),
    ];

    if match_lines.is_empty() {
        rendered.push("(no matches found)".to_string());
    } else {
        rendered.extend(match_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: refine search patterns with additional terms or regex options".to_string());
    rendered
}

fn render_file_annotate_view(lines: Vec<String>) -> Vec<String> {
    let annotation_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let annotation_count = annotation_lines.len();

    let mut rendered = vec![
        "File Annotate".to_string(),
        "=============".to_string(),
        String::new(),
        format!(
            "Summary: {annotation_count} annotated line{}",
            plural_suffix(annotation_count)
        ),
        String::new(),
    ];

    if annotation_lines.is_empty() {
        rendered.push("(no annotation output)".to_string());
    } else {
        rendered.extend(annotation_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: pair with `show <rev>` to inspect the source revision details".to_string());
    rendered
}

fn render_file_track_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Track",
        "==========",
        lines,
        "Tip: review tracked paths with `file list` and verify with `status`",
    )
}

fn render_file_untrack_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Untrack",
        "============",
        lines,
        "Tip: ensure paths are ignored before untracking and confirm with `status`",
    )
}

fn render_file_chmod_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Chmod",
        "==========",
        lines,
        "Tip: run `file show` or `diff` to verify executable-bit updates",
    )
}

fn render_file_mutation_view(
    title: &str,
    underline: &str,
    lines: Vec<String>,
    tip: &str,
) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let detail_count = detail_lines.len();

    let summary = if detail_count == 0 {
        "Summary: command completed with no output".to_string()
    } else {
        format!(
            "Summary: {detail_count} output line{}",
            plural_suffix(detail_count)
        )
    };

    let mut rendered = vec![
        title.to_string(),
        underline.to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push(tip.to_string());
    rendered
}

fn render_tag_list_view(lines: Vec<String>) -> Vec<String> {
    let tag_lines: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed != "(no output)"
        })
        .collect();
    let tag_count = tag_lines.len();

    let mut rendered = vec![
        "Tag List".to_string(),
        "========".to_string(),
        String::new(),
        format!(
            "Summary: {tag_count} tag{} listed",
            plural_suffix(tag_count)
        ),
        String::new(),
    ];

    if tag_count == 0 {
        rendered.push("(no tags listed)".to_string());
    } else {
        rendered.extend(tag_lines);
    }

    rendered.push(String::new());
    rendered.push(
        "Tip: use `tag create` and `tag forget` from command mode for tag updates".to_string(),
    );
    rendered
}

fn render_tag_set_view(lines: Vec<String>) -> Vec<String> {
    render_tag_mutation_view(
        "Tag Set",
        "=======",
        lines,
        "Tip: run `tag list` to confirm updated tag targets",
    )
}

fn render_tag_delete_view(lines: Vec<String>) -> Vec<String> {
    render_tag_mutation_view(
        "Tag Delete",
        "==========",
        lines,
        "Tip: run `tag list` to confirm removed tags",
    )
}

fn render_tag_mutation_view(
    title: &str,
    underline: &str,
    lines: Vec<String>,
    tip: &str,
) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let detail_count = detail_lines.len();

    let summary = if detail_count == 0 {
        "Summary: command completed with no output".to_string()
    } else {
        format!(
            "Summary: {detail_count} output line{}",
            plural_suffix(detail_count)
        )
    };

    let mut rendered = vec![
        title.to_string(),
        underline.to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push(tip.to_string());
    rendered
}

fn render_workspace_list_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut workspace_lines = Vec::new();
    let mut workspace_count = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if line.contains(':') {
            workspace_count += 1;
        }
        workspace_lines.push(line);
    }

    let mut rendered = vec![
        "Workspace List".to_string(),
        "==============".to_string(),
        String::new(),
        format!(
            "Summary: {workspace_count} workspace{} listed",
            plural_suffix(workspace_count)
        ),
        String::new(),
    ];
    rendered.extend(workspace_lines);
    rendered.push(String::new());
    rendered
        .push("Tip: use `workspace add/forget/rename` flows from normal mode or `:`".to_string());
    rendered
}

fn render_git_fetch_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = if detail_lines
        .iter()
        .any(|line| line.contains("Nothing changed"))
    {
        "Summary: no remote updates fetched".to_string()
    } else if detail_lines.is_empty() {
        "Summary: fetch completed with no output".to_string()
    } else {
        format!(
            "Summary: {} output line{}",
            detail_lines.len(),
            plural_suffix(detail_lines.len())
        )
    };

    let mut rendered = vec![
        "Git Fetch".to_string(),
        "=========".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `log` or `status` to inspect fetched changes".to_string());
    rendered
}

fn render_git_push_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = if detail_lines
        .iter()
        .any(|line| line.contains("Nothing changed"))
    {
        "Summary: no bookmark updates pushed".to_string()
    } else if detail_lines.is_empty() {
        "Summary: push completed with no output".to_string()
    } else {
        format!(
            "Summary: {} output line{}",
            detail_lines.len(),
            plural_suffix(detail_lines.len())
        )
    };

    let mut rendered = vec![
        "Git Push".to_string(),
        "========".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: push stays confirm-gated with a dry-run preview when available".to_string());
    rendered
}

fn render_top_level_mutation_view(command_name: &str, lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = top_level_mutation_summary(command_name, &detail_lines);
    let title = format!("{} Result", capitalize_word(command_name));
    let mut rendered = vec![
        title.clone(),
        "=".repeat(title.len()),
        String::new(),
        summary,
    ];
    rendered.push(String::new());

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push(top_level_mutation_tip(command_name).to_string());
    rendered
}

fn top_level_mutation_summary(command_name: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: `{command_name}` completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_top_level_mutation_signal(command_name, line))
    {
        return format!("Summary: {}", signal.trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

fn is_top_level_mutation_signal(command_name: &str, line: &str) -> bool {
    let trimmed = line.trim();
    match command_name {
        "new" | "describe" | "commit" | "edit" | "next" | "prev" => {
            trimmed.starts_with("Working copy now at:")
                || trimmed.starts_with("Working copy  (@) :")
        }
        "undo" => trimmed.starts_with("Undid operation"),
        "redo" => trimmed.starts_with("Redid operation"),
        "rebase" | "squash" | "split" => trimmed.starts_with("Rebased "),
        "abandon" => trimmed.starts_with("Abandoned "),
        "restore" => trimmed.starts_with("Restored "),
        "revert" => trimmed.starts_with("Reverted "),
        _ => false,
    }
}

fn top_level_mutation_tip(command_name: &str) -> &'static str {
    match command_name {
        "new" | "describe" | "commit" => "Tip: run `show` or `log` to inspect the updated revision",
        "edit" | "next" | "prev" => "Tip: run `show` or `diff` to inspect the selected revision",
        "undo" | "redo" => "Tip: run `operation log` to inspect operation history",
        "rebase" | "squash" | "split" | "abandon" | "restore" | "revert" => {
            "Tip: run `log`, `status`, or `diff` to verify the resulting history"
        }
        _ => "Tip: run `log` and `status` to verify repository state",
    }
}

fn render_bookmark_mutation_view(subcommand: Option<&str>, lines: Vec<String>) -> Vec<String> {
    let subcommand = subcommand.unwrap_or("update");
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = if detail_lines.is_empty() {
        format!("Summary: bookmark {subcommand} completed with no output")
    } else {
        format!(
            "Summary: {} output line{}",
            detail_lines.len(),
            plural_suffix(detail_lines.len())
        )
    };

    let mut rendered = vec![
        format!("Bookmark {}", capitalize_word(subcommand)),
        "================".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `bookmark list` to verify bookmark state".to_string());
    rendered
}

fn render_bookmark_list_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Bookmark List".to_string(),
        "=============".to_string(),
        String::new(),
    ];
    rendered.extend(lines);
    rendered.push(String::new());
    rendered.push("Tip: use `bookmark set/move/track` flows from normal mode or `:`".to_string());
    rendered
}

fn render_workspace_mutation_view(subcommand: Option<&str>, lines: Vec<String>) -> Vec<String> {
    let subcommand = subcommand.unwrap_or("update");
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = if detail_lines.is_empty() {
        format!("Summary: workspace {subcommand} completed with no output")
    } else {
        format!(
            "Summary: {} output line{}",
            detail_lines.len(),
            plural_suffix(detail_lines.len())
        )
    };

    let mut rendered = vec![
        format!("Workspace {}", capitalize_word(subcommand)),
        "=================".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `workspace list` to inspect workspace state".to_string());
    rendered
}

fn render_operation_show_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut detail_lines: Vec<String> = Vec::new();
    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if line.ends_with(':')
            && matches!(detail_lines.last(), Some(previous) if !previous.is_empty())
        {
            detail_lines.push(String::new());
        }

        detail_lines.push(line);
    }

    while matches!(detail_lines.last(), Some(previous) if previous.is_empty()) {
        detail_lines.pop();
    }

    let operation_id = detail_lines
        .first()
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or("@");

    let mut rendered = vec![
        "Operation Details".to_string(),
        "=================".to_string(),
        String::new(),
        format!("Summary: operation {operation_id}"),
        String::new(),
    ];
    rendered.extend(detail_lines);
    rendered.push(String::new());
    rendered.push("Tip: operation restore/revert remain confirm-gated with previews".to_string());
    rendered
}

fn render_operation_mutation_view(subcommand: &str, lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = if detail_lines.is_empty() {
        format!("Summary: operation {subcommand} completed with no output")
    } else {
        format!(
            "Summary: {} output line{}",
            detail_lines.len(),
            plural_suffix(detail_lines.len())
        )
    };

    let mut rendered = vec![
        format!("Operation {}", capitalize_word(subcommand)),
        "=================".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `operation log` and `status` to validate repository state".to_string());
    rendered
}

fn render_operation_diff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut detail_lines: Vec<String> = Vec::new();
    let mut commit_count = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if is_operation_entry_header(&line) {
            commit_count += 1;
        }

        if line.ends_with(':')
            && matches!(detail_lines.last(), Some(previous) if !previous.is_empty())
        {
            detail_lines.push(String::new());
        }

        detail_lines.push(line);
    }

    while matches!(detail_lines.last(), Some(previous) if previous.is_empty()) {
        detail_lines.pop();
    }

    let summary = if commit_count == 0 {
        "Summary: operation delta shown".to_string()
    } else {
        format!(
            "Summary: {commit_count} changed commit entr{} shown",
            if commit_count == 1 { "y" } else { "ies" }
        )
    };

    let mut rendered = vec![
        "Operation Diff".to_string(),
        "==============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(detail_lines);
    rendered.push(String::new());
    rendered
        .push("Tip: use operation show/restore/revert for deeper operation workflows".to_string());
    rendered
}

fn render_operation_log_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut operation_lines: Vec<String> = Vec::new();
    let mut operation_count = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if is_operation_entry_header(&line) {
            if matches!(operation_lines.last(), Some(previous) if !previous.is_empty()) {
                operation_lines.push(String::new());
            }
            operation_count += 1;
        }

        operation_lines.push(line);
    }

    while matches!(operation_lines.last(), Some(previous) if previous.is_empty()) {
        operation_lines.pop();
    }

    let mut rendered = vec![
        "Operation Log".to_string(),
        "=============".to_string(),
        String::new(),
        format!(
            "Summary: {operation_count} operation entr{} shown",
            if operation_count == 1 { "y" } else { "ies" }
        ),
        String::new(),
    ];
    rendered.extend(operation_lines);
    rendered.push(String::new());
    rendered.push("Tip: restore/revert operations stay confirm-gated with previews".to_string());
    rendered
}

fn is_operation_entry_header(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('@') || trimmed.starts_with('○')
}

fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn is_working_copy_change_line(line: &str) -> bool {
    let mut chars = line.chars();
    match (chars.next(), chars.next()) {
        (Some(status), Some(' ')) => matches!(status, 'M' | 'A' | 'D' | 'R' | 'C' | '?' | 'U'),
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
    use crossterm::event::{KeyCode, KeyEvent};

    use crate::config::KeybindConfig;

    use crate::flows::{FlowAction, PromptKind};

    use super::{
        App, Mode, build_row_revision_map, confirmation_preview_tokens, decorate_command_output,
        extract_revision, is_change_id, is_commit_id, is_dangerous, keymap_overview_lines,
        looks_like_graph_commit_row, metadata_log_tokens, render_bookmark_list_view,
        render_bookmark_mutation_view, render_diff_view, render_file_annotate_view,
        render_file_chmod_view, render_file_list_view, render_file_search_view,
        render_file_show_view, render_file_track_view, render_file_untrack_view,
        render_git_fetch_view, render_git_push_view, render_operation_diff_view,
        render_operation_log_view, render_operation_mutation_view, render_operation_show_view,
        render_resolve_list_view, render_root_view, render_show_view, render_status_view,
        render_tag_delete_view, render_tag_list_view, render_tag_set_view,
        render_top_level_mutation_view, render_workspace_list_view, render_workspace_mutation_view,
        startup_action, toggle_patch_flag, top_level_mutation_summary,
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
    fn startup_keys_view_renders_without_running_jj() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.apply_startup_tokens(vec!["keys".to_string()])
            .expect("startup action should succeed");

        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.status_line, "Showing keymap".to_string());
        assert!(app.lines.iter().any(|line| line.contains("jk keymap")));
        assert!(app.lines.iter().any(|line| line.contains("normal.push")));
        assert!(
            app.lines
                .iter()
                .any(|line| line.contains("normal.file_list"))
        );
        assert!(
            app.lines
                .iter()
                .any(|line| line.contains("normal.resolve_list"))
        );
        assert!(
            app.lines
                .iter()
                .any(|line| line.contains("normal.tag_list"))
        );
    }

    #[test]
    fn filters_keymap_view_by_query() {
        let lines = keymap_overview_lines(
            &KeybindConfig::load().expect("keybind config should parse"),
            Some("push"),
        );

        assert!(lines.iter().any(|line| line.contains("normal.push")));
        assert!(!lines.iter().any(|line| line.contains("normal.quit")));
    }

    #[test]
    fn command_keys_view_renders_and_filters() {
        let mut app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        app.execute_command_line("keys push")
            .expect("keys command should render");

        assert_eq!(app.status_line, "Showing keymap for `push`".to_string());
        assert!(app.lines.iter().any(|line| line.contains("normal.push")));
        assert!(!app.lines.iter().any(|line| line.contains("normal.quit")));
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
                .any(|line| line.contains("Summary: 2 working-copy changes"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("1 conflicted bookmark"))
        );
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
    fn inserts_section_spacing_in_show_view() {
        let rendered = render_show_view(vec![
            "Commit ID: abcdef0123456789".to_string(),
            "Change ID: abcdefghijklmnop".to_string(),
            "Modified regular file src/app.rs:".to_string(),
            "  1: old".to_string(),
            "Modified regular file src/config.rs:".to_string(),
            "  1: new".to_string(),
        ]);

        let second_section_index = rendered
            .iter()
            .position(|line| line == "Modified regular file src/config.rs:")
            .expect("second show section should exist");
        assert_eq!(rendered.get(second_section_index - 1), Some(&String::new()));
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
    fn inserts_file_spacing_in_diff_view() {
        let rendered = render_diff_view(vec![
            "Modified regular file src/app.rs:".to_string(),
            "  1  1: use std::collections::HashMap;".to_string(),
            "Modified regular file src/config.rs:".to_string(),
            "  1  1: use std::env;".to_string(),
        ]);

        let second_file_index = rendered
            .iter()
            .position(|line| line == "Modified regular file src/config.rs:")
            .expect("second diff file should exist");
        assert_eq!(rendered.get(second_file_index - 1), Some(&String::new()));
    }

    #[test]
    fn renders_root_view_with_header_and_tip() {
        let rendered = render_root_view(vec!["/Users/joshka/local/jk".to_string()]);

        assert_eq!(rendered.first(), Some(&"Workspace Root".to_string()));
        assert!(rendered.iter().any(|line| line == "/Users/joshka/local/jk"));
        assert!(rendered.iter().any(|line| line.contains("jjrt/jk root")));
    }

    #[test]
    fn renders_resolve_list_view_with_no_conflicts_summary() {
        let rendered = render_resolve_list_view(vec![
            "Error: No conflicts found at this revision".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Resolve List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: no conflicts listed"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("No conflicts found at this revision"))
        );
    }

    #[test]
    fn renders_resolve_list_view_with_conflict_count() {
        let rendered =
            render_resolve_list_view(vec!["src/app.rs".to_string(), "src/flows.rs".to_string()]);

        assert_eq!(rendered.first(), Some(&"Resolve List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 conflicted paths listed"))
        );
        assert!(rendered.iter().any(|line| line == "src/app.rs"));
    }

    #[test]
    fn decorates_resolve_list_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["resolve".to_string(), "-l".to_string()],
            vec!["Error: No conflicts found at this revision".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Resolve List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: no conflicts listed"))
        );
    }

    #[test]
    fn renders_file_list_view_with_summary_and_tip() {
        let rendered =
            render_file_list_view(vec!["src/app.rs".to_string(), "src/flows.rs".to_string()]);

        assert_eq!(rendered.first(), Some(&"File List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 files listed"))
        );
        assert!(rendered.iter().any(|line| line == "src/app.rs"));
        assert!(rendered.iter().any(|line| line.contains("show`/`diff")));
    }

    #[test]
    fn renders_file_show_view_with_summary_and_tip() {
        let rendered = render_file_show_view(vec![
            "fn main() {".to_string(),
            String::new(),
            "}".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"File Show".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 3 content lines"))
        );
        assert!(rendered.iter().any(|line| line == "fn main() {"));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("surrounding change context"))
        );
    }

    #[test]
    fn renders_file_search_view_with_summary_and_tip() {
        let rendered = render_file_search_view(vec![
            "src/app.rs:120:render_status_view".to_string(),
            "src/flows.rs:88:plan_command".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"File Search".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 match lines"))
        );
        assert!(rendered.iter().any(|line| line.contains("src/app.rs:120")));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("refine search patterns"))
        );
    }

    #[test]
    fn renders_file_annotate_view_with_summary_and_tip() {
        let rendered = render_file_annotate_view(vec![
            "uxqqtlkq src/app.rs:1 use std::io;".to_string(),
            "qtswpusn src/app.rs:2 fn main() {}".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"File Annotate".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 annotated lines"))
        );
        assert!(rendered.iter().any(|line| line.contains("src/app.rs:1")));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("source revision details"))
        );
    }

    #[test]
    fn renders_file_track_view_with_summary_and_tip() {
        let rendered = render_file_track_view(vec![
            "Started tracking 2 paths".to_string(),
            "src/new.rs".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"File Track".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 output lines"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("review tracked paths"))
        );
    }

    #[test]
    fn renders_file_untrack_view_with_summary_and_tip() {
        let rendered =
            render_file_untrack_view(vec!["Stopped tracking target/generated.txt".to_string()]);

        assert_eq!(rendered.first(), Some(&"File Untrack".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("ensure paths are ignored"))
        );
    }

    #[test]
    fn renders_file_chmod_view_with_summary_and_tip() {
        let rendered = render_file_chmod_view(vec![
            "Updated mode to executable for scripts/deploy.sh".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"File Chmod".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("verify executable-bit updates"))
        );
    }

    #[test]
    fn renders_tag_list_view_with_empty_state() {
        let rendered = render_tag_list_view(vec!["(no output)".to_string()]);

        assert_eq!(rendered.first(), Some(&"Tag List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 0 tags listed"))
        );
        assert!(rendered.iter().any(|line| line == "(no tags listed)"));
    }

    #[test]
    fn renders_tag_set_view_with_summary_and_tip() {
        let rendered =
            render_tag_set_view(vec!["Created tag v0.2.0 pointing to abc12345".to_string()]);

        assert_eq!(rendered.first(), Some(&"Tag Set".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("confirm updated tag targets"))
        );
    }

    #[test]
    fn renders_tag_delete_view_with_summary_and_tip() {
        let rendered = render_tag_delete_view(vec!["Deleted tag v0.1.0".to_string()]);

        assert_eq!(rendered.first(), Some(&"Tag Delete".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("confirm removed tags"))
        );
    }

    #[test]
    fn decorates_file_list_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["file".to_string(), "list".to_string()],
            vec!["src/main.rs".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"File List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 file listed"))
        );
    }

    #[test]
    fn decorates_file_show_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["file".to_string(), "show".to_string()],
            vec!["fn main() {}".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"File Show".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 content line"))
        );
    }

    #[test]
    fn decorates_file_search_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["file".to_string(), "search".to_string()],
            vec!["src/app.rs:1:fn main()".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"File Search".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 match line"))
        );
    }

    #[test]
    fn decorates_file_annotate_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["file".to_string(), "annotate".to_string()],
            vec!["uxqqtlkq src/app.rs:1 use std::io;".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"File Annotate".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 annotated line"))
        );
    }

    #[test]
    fn decorates_file_track_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["file".to_string(), "track".to_string()],
            vec!["Started tracking src/new.rs".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"File Track".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_file_untrack_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["file".to_string(), "untrack".to_string()],
            vec!["Stopped tracking target/generated.txt".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"File Untrack".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_file_chmod_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["file".to_string(), "chmod".to_string()],
            vec!["Updated mode for scripts/deploy.sh".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"File Chmod".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_tag_list_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["tag".to_string(), "list".to_string()],
            vec!["v0.1.0: abcdef12".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Tag List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 tag listed"))
        );
    }

    #[test]
    fn decorates_tag_set_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["tag".to_string(), "set".to_string()],
            vec!["Created tag v0.2.0".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Tag Set".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_tag_delete_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["tag".to_string(), "delete".to_string()],
            vec!["Deleted tag v0.1.0".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Tag Delete".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_workspace_root_output_with_root_wrapper() {
        let rendered = decorate_command_output(
            &["workspace".to_string(), "root".to_string()],
            vec!["/Users/joshka/local/jk".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Workspace Root".to_string()));
        assert!(rendered.iter().any(|line| line == "/Users/joshka/local/jk"));
    }

    #[test]
    fn renders_workspace_list_view_with_summary_and_tip() {
        let rendered = render_workspace_list_view(vec![
            "default: abcdef12 main workspace".to_string(),
            "staging: 0123abcd staging workspace".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Workspace List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 workspaces listed"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line == "default: abcdef12 main workspace")
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("workspace add/forget/rename"))
        );
    }

    #[test]
    fn decorates_workspace_list_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["workspace".to_string(), "list".to_string()],
            vec!["default: abcdef12 main workspace".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Workspace List".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 workspace listed"))
        );
    }

    #[test]
    fn renders_git_fetch_view_with_summary_and_tip() {
        let rendered = render_git_fetch_view(vec![
            "Nothing changed.".to_string(),
            "Hint: use -b to fetch bookmarks".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Git Fetch".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: no remote updates fetched"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("run `log` or `status`"))
        );
    }

    #[test]
    fn renders_git_push_view_with_summary_and_tip() {
        let rendered = render_git_push_view(vec![
            "Pushed bookmark main to origin".to_string(),
            "Done.".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Git Push".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 output lines"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("confirm-gated with a dry-run preview"))
        );
    }

    #[test]
    fn decorates_git_fetch_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["git".to_string(), "fetch".to_string()],
            vec!["Nothing changed.".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Git Fetch".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: no remote updates fetched"))
        );
    }

    #[test]
    fn decorates_git_push_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["git".to_string(), "push".to_string()],
            vec!["Pushed bookmark main to origin".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Git Push".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn renders_bookmark_list_view_with_header_and_tip() {
        let rendered = render_bookmark_list_view(vec![
            "main: abcdef12".to_string(),
            "feature: 0123abcd".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Bookmark List".to_string()));
        assert!(rendered.iter().any(|line| line == "main: abcdef12"));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("bookmark set/move/track"))
        );
    }

    #[test]
    fn renders_bookmark_mutation_view_with_summary_and_tip() {
        let rendered = render_bookmark_mutation_view(
            Some("set"),
            vec!["Moved bookmark main to abcdef12".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Bookmark Set".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line == "Moved bookmark main to abcdef12")
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("run `bookmark list`"))
        );
    }

    #[test]
    fn decorates_bookmark_set_output_with_wrapper() {
        let rendered = decorate_command_output(
            &[
                "bookmark".to_string(),
                "set".to_string(),
                "main".to_string(),
            ],
            vec!["Moved bookmark main to abcdef12".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Bookmark Set".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_all_bookmark_mutation_subcommands_with_wrappers() {
        let cases = [
            ("create", "Bookmark Create"),
            ("set", "Bookmark Set"),
            ("move", "Bookmark Move"),
            ("track", "Bookmark Track"),
            ("untrack", "Bookmark Untrack"),
            ("delete", "Bookmark Delete"),
            ("forget", "Bookmark Forget"),
            ("rename", "Bookmark Rename"),
        ];

        for (subcommand, expected_header) in cases {
            let rendered = decorate_command_output(
                &[
                    "bookmark".to_string(),
                    subcommand.to_string(),
                    "feature".to_string(),
                ],
                vec![format!("{subcommand} bookmark output")],
            );

            assert_eq!(
                rendered.first(),
                Some(&expected_header.to_string()),
                "expected wrapper header for bookmark {subcommand}",
            );
            assert!(
                rendered
                    .iter()
                    .any(|line| line.contains("Summary: 1 output line")),
                "expected summary line for bookmark {subcommand}",
            );
        }
    }

    #[test]
    fn renders_workspace_mutation_view_with_summary_and_tip() {
        let rendered = render_workspace_mutation_view(
            Some("add"),
            vec!["Created workspace docs at ../jk-docs".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Workspace Add".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("run `workspace list`"))
        );
    }

    #[test]
    fn decorates_workspace_add_output_with_wrapper() {
        let rendered = decorate_command_output(
            &[
                "workspace".to_string(),
                "add".to_string(),
                "../jk-docs".to_string(),
            ],
            vec!["Created workspace docs at ../jk-docs".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Workspace Add".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_all_workspace_mutation_subcommands_with_wrappers() {
        let cases = [
            ("add", "Workspace Add"),
            ("forget", "Workspace Forget"),
            ("rename", "Workspace Rename"),
            ("update-stale", "Workspace Update-stale"),
        ];

        for (subcommand, expected_header) in cases {
            let rendered = decorate_command_output(
                &[
                    "workspace".to_string(),
                    subcommand.to_string(),
                    "demo".to_string(),
                ],
                vec![format!("{subcommand} workspace output")],
            );

            assert_eq!(
                rendered.first(),
                Some(&expected_header.to_string()),
                "expected wrapper header for workspace {subcommand}",
            );
            assert!(
                rendered
                    .iter()
                    .any(|line| line.contains("Summary: 1 output line")),
                "expected summary line for workspace {subcommand}",
            );
        }
    }

    #[test]
    fn renders_operation_restore_view_with_summary_and_tip() {
        let rendered = render_operation_mutation_view(
            "restore",
            vec!["Restored to operation 7699d9773e37".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Operation Restore".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("operation log` and `status"))
        );
    }

    #[test]
    fn decorates_operation_restore_output_with_wrapper() {
        let rendered = decorate_command_output(
            &[
                "operation".to_string(),
                "restore".to_string(),
                "7699d9773e37".to_string(),
            ],
            vec!["Restored to operation 7699d9773e37".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Operation Restore".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn decorates_operation_revert_output_with_wrapper() {
        let rendered = decorate_command_output(
            &[
                "operation".to_string(),
                "revert".to_string(),
                "7699d9773e37".to_string(),
            ],
            vec!["Reverted operation 7699d9773e37".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Operation Revert".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 output line"))
        );
    }

    #[test]
    fn renders_top_level_mutation_view_with_summary_and_tip() {
        let rendered = render_top_level_mutation_view(
            "commit",
            vec!["Working copy now at: abcdef12 commit message".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Commit Result".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: Working copy now at: abcdef12"))
        );
        assert!(rendered.iter().any(|line| line.contains("show` or `log")));
    }

    #[test]
    fn decorates_commit_output_with_wrapper() {
        let rendered = decorate_command_output(
            &[
                "commit".to_string(),
                "-m".to_string(),
                "message".to_string(),
            ],
            vec!["Working copy now at: abcdef12 commit message".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Commit Result".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: Working copy now at: abcdef12"))
        );
    }

    #[test]
    fn decorates_rebase_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["rebase".to_string(), "-d".to_string(), "main".to_string()],
            vec!["Rebased 3 commits onto main".to_string()],
        );

        assert_eq!(rendered.first(), Some(&"Rebase Result".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: Rebased 3 commits onto main"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("log`, `status`, or `diff`"))
        );
    }

    #[test]
    fn mutation_summary_uses_signal_line_when_available() {
        let summary =
            top_level_mutation_summary("undo", &[String::from("Undid operation 67d547b627fb")]);
        assert_eq!(summary, "Summary: Undid operation 67d547b627fb");
    }

    #[test]
    fn mutation_summary_falls_back_to_line_count() {
        let summary = top_level_mutation_summary(
            "abandon",
            &[String::from("Random line"), String::from("Second line")],
        );
        assert_eq!(summary, "Summary: 2 output lines");
    }

    #[test]
    fn decorates_all_top_level_mutation_commands_with_wrappers() {
        let cases = [
            ("new", "New Result"),
            ("describe", "Describe Result"),
            ("commit", "Commit Result"),
            ("edit", "Edit Result"),
            ("next", "Next Result"),
            ("prev", "Prev Result"),
            ("rebase", "Rebase Result"),
            ("squash", "Squash Result"),
            ("split", "Split Result"),
            ("abandon", "Abandon Result"),
            ("undo", "Undo Result"),
            ("redo", "Redo Result"),
            ("restore", "Restore Result"),
            ("revert", "Revert Result"),
        ];

        for (command, expected_header) in cases {
            let rendered = decorate_command_output(
                &[command.to_string()],
                vec![format!("{command} output line")],
            );

            assert_eq!(
                rendered.first(),
                Some(&expected_header.to_string()),
                "expected wrapper header for command `{command}`",
            );
            assert!(
                rendered
                    .iter()
                    .any(|line| line.contains("Summary: 1 output line")),
                "expected summary line for command `{command}`",
            );
        }
    }

    #[test]
    fn top_level_mutation_wrappers_use_command_specific_tips() {
        let commit_rendered = render_top_level_mutation_view(
            "commit",
            vec!["Working copy now at: abcdef12 commit message".to_string()],
        );
        assert!(
            commit_rendered
                .iter()
                .any(|line| line.contains("show` or `log"))
        );

        let undo_rendered = render_top_level_mutation_view(
            "undo",
            vec!["Undid operation 67d547b627fb".to_string()],
        );
        assert!(
            undo_rendered
                .iter()
                .any(|line| line.contains("operation log"))
        );

        let rebase_rendered = render_top_level_mutation_view(
            "rebase",
            vec!["Rebased 3 commits onto main".to_string()],
        );
        assert!(
            rebase_rendered
                .iter()
                .any(|line| line.contains("log`, `status`, or `diff`"))
        );

        let next_rendered = render_top_level_mutation_view(
            "next",
            vec!["Working copy now at: abcdef12 next change".to_string()],
        );
        assert!(
            next_rendered
                .iter()
                .any(|line| line.contains("show` or `diff"))
        );
    }

    #[test]
    fn decorates_gold_command_set_with_native_wrapper_headers() {
        let cases = vec![
            (
                vec!["status"],
                vec!["Working copy changes:", "M src/app.rs"],
                "Status Overview",
            ),
            (
                vec!["show", "-r", "abc12345"],
                vec!["Commit ID: abc12345", "Change ID: qtswpusn"],
                "Show View",
            ),
            (
                vec!["diff", "-r", "abc12345"],
                vec!["Modified regular file src/app.rs:", "@@ -1,1 +1,2 @@"],
                "Diff View",
            ),
            (
                vec!["new"],
                vec!["Working copy now at: abcdef12 new change"],
                "New Result",
            ),
            (
                vec!["describe"],
                vec!["Working copy now at: abcdef12 described change"],
                "Describe Result",
            ),
            (
                vec!["commit"],
                vec!["Working copy now at: abcdef12 commit change"],
                "Commit Result",
            ),
            (
                vec!["next"],
                vec!["Working copy now at: abcdef12 next change"],
                "Next Result",
            ),
            (
                vec!["prev"],
                vec!["Working copy now at: abcdef12 prev change"],
                "Prev Result",
            ),
            (
                vec!["edit"],
                vec!["Working copy now at: abcdef12 edit change"],
                "Edit Result",
            ),
            (
                vec!["rebase", "-d", "main"],
                vec!["Rebased 3 commits onto main"],
                "Rebase Result",
            ),
            (
                vec!["squash", "--into", "main"],
                vec!["Rebased 1 commits onto main"],
                "Squash Result",
            ),
            (
                vec!["split", "-r", "abc12345"],
                vec!["Rebased 1 commits onto abc12345"],
                "Split Result",
            ),
            (
                vec!["abandon", "abc12345"],
                vec!["Abandoned 1 commits."],
                "Abandon Result",
            ),
            (
                vec!["undo"],
                vec!["Undid operation 67d547b627fb"],
                "Undo Result",
            ),
            (
                vec!["redo"],
                vec!["Redid operation 67d547b627fb"],
                "Redo Result",
            ),
            (
                vec!["bookmark", "list"],
                vec!["main: abcdef12", "feature: 0123abcd"],
                "Bookmark List",
            ),
            (
                vec!["bookmark", "create", "feature"],
                vec!["Created bookmark feature at abcdef12"],
                "Bookmark Create",
            ),
            (
                vec!["bookmark", "set", "main"],
                vec!["Moved bookmark main to abcdef12"],
                "Bookmark Set",
            ),
            (
                vec!["bookmark", "move", "main"],
                vec!["Moved bookmark main to abcdef12"],
                "Bookmark Move",
            ),
            (
                vec!["bookmark", "track", "main"],
                vec!["Started tracking bookmark main@origin"],
                "Bookmark Track",
            ),
            (
                vec!["bookmark", "untrack", "main"],
                vec!["Stopped tracking bookmark main@origin"],
                "Bookmark Untrack",
            ),
            (
                vec!["git", "fetch"],
                vec!["Fetched 2 commits from origin"],
                "Git Fetch",
            ),
            (
                vec!["git", "push"],
                vec!["Pushed bookmark main to origin"],
                "Git Push",
            ),
        ];

        for (command, output, expected_header) in cases {
            let command_tokens = command
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>();
            let output_lines = output
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>();

            let rendered = decorate_command_output(&command_tokens, output_lines);
            assert_eq!(
                rendered.first(),
                Some(&expected_header.to_string()),
                "expected native wrapper header for command `{}`",
                command.join(" "),
            );
        }
    }

    #[test]
    fn renders_operation_log_view_with_header_and_tip() {
        let rendered = render_operation_log_view(vec![
            "@  fac974146f86 user 5 seconds ago".to_string(),
            "│  snapshot working copy".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Operation Log".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line == "@  fac974146f86 user 5 seconds ago")
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 operation entry shown"))
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("restore/revert operations"))
        );
    }

    #[test]
    fn renders_operation_show_view_with_header_and_tip() {
        let rendered = render_operation_show_view(vec![
            "7699d9773e37 user 41 seconds ago, lasted 19 milliseconds".to_string(),
            "snapshot working copy".to_string(),
            "Changed commits:".to_string(),
            "○  + qqrxwkpt c245343e feat(help): surface local views".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Operation Details".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: operation 7699d9773e37"))
        );
        assert!(rendered.iter().any(|line| line == "Changed commits:"));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("operation restore/revert remain confirm-gated"))
        );
    }

    #[test]
    fn renders_operation_diff_view_with_header_and_summary() {
        let rendered = render_operation_diff_view(vec![
            "From operation: abc123 (2026-02-07) describe commit old".to_string(),
            "  To operation: def456 (2026-02-07) describe commit new".to_string(),
            "Changed commits:".to_string(),
            "○  + uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
            "Changed working copy default@:".to_string(),
            "+ uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
        ]);

        assert_eq!(rendered.first(), Some(&"Operation Diff".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 changed commit entry shown"))
        );
        assert!(rendered.iter().any(|line| line == "Changed commits:"));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("operation show/restore/revert"))
        );
    }

    #[test]
    fn decorates_operation_diff_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["operation".to_string(), "diff".to_string()],
            vec![
                "From operation: abc123 (2026-02-07) describe commit old".to_string(),
                "Changed commits:".to_string(),
                "○  + uxqqtlkq 722c112d feature".to_string(),
            ],
        );

        assert_eq!(rendered.first(), Some(&"Operation Diff".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 1 changed commit entry shown"))
        );
    }

    #[test]
    fn decorates_operation_show_output_with_wrapper() {
        let rendered = decorate_command_output(
            &["operation".to_string(), "show".to_string()],
            vec![
                "7699d9773e37 user 41 seconds ago, lasted 19 milliseconds".to_string(),
                "snapshot working copy".to_string(),
            ],
        );

        assert_eq!(rendered.first(), Some(&"Operation Details".to_string()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: operation 7699d9773e37"))
        );
    }

    #[test]
    fn inserts_spacing_between_operation_entries() {
        let rendered = render_operation_log_view(vec![
            "@  fac974146f86 user 5 seconds ago".to_string(),
            "│  snapshot working copy".to_string(),
            "○  4a8a95e95f6f user 22 seconds ago".to_string(),
            "│  snapshot working copy".to_string(),
        ]);

        let second_entry_index = rendered
            .iter()
            .position(|line| line == "○  4a8a95e95f6f user 22 seconds ago")
            .expect("second operation entry should exist");
        assert_eq!(rendered.get(second_entry_index - 1), Some(&String::new()));
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("Summary: 2 operation entries shown"))
        );
    }

    #[test]
    fn snapshot_renders_bookmark_list_wrapper_view() {
        let rendered = render_bookmark_list_view(vec![
            "main: abcdef12".to_string(),
            "feature: 0123abcd".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_set_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("set"),
            vec!["Moved bookmark main to abcdef12".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_create_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("create"),
            vec!["Created bookmark feature at abcdef12".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_move_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("move"),
            vec!["Moved bookmark main to abcdef12".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_track_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("track"),
            vec!["Started tracking bookmark main@origin".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_untrack_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("untrack"),
            vec!["Stopped tracking bookmark main@origin".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_delete_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("delete"),
            vec!["Deleted bookmark stale-feature".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_forget_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("forget"),
            vec!["Forgot bookmark stale-feature@origin".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_bookmark_rename_wrapper_view() {
        let rendered = render_bookmark_mutation_view(
            Some("rename"),
            vec!["Renamed bookmark main to primary".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_file_list_wrapper_view() {
        let rendered = render_file_list_view(vec![
            "src/app.rs".to_string(),
            "src/flows.rs".to_string(),
            "README.md".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_file_show_wrapper_view() {
        let rendered = render_file_show_view(vec![
            "fn main() {".to_string(),
            "    println!(\"hi\");".to_string(),
            "}".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_file_search_wrapper_view() {
        let rendered = render_file_search_view(vec![
            "src/app.rs:627:decorate_command_output".to_string(),
            "src/flows.rs:264:plan_command".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_file_annotate_wrapper_view() {
        let rendered = render_file_annotate_view(vec![
            "uxqqtlkq src/app.rs:1 use std::io;".to_string(),
            "qtswpusn src/app.rs:2 fn main() {}".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_file_track_wrapper_view() {
        let rendered = render_file_track_view(vec![
            "Started tracking 2 paths".to_string(),
            "src/new.rs".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_file_untrack_wrapper_view() {
        let rendered =
            render_file_untrack_view(vec!["Stopped tracking target/generated.txt".to_string()]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_file_chmod_wrapper_view() {
        let rendered = render_file_chmod_view(vec![
            "Updated mode to executable for scripts/deploy.sh".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_resolve_list_wrapper_view() {
        let rendered = render_resolve_list_view(vec![
            "Error: No conflicts found at this revision".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_workspace_list_wrapper_view() {
        let rendered = render_workspace_list_view(vec![
            "default: qqrxwkpt c245343e feature workspace".to_string(),
            "staging: abcdef12 release workspace".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_workspace_add_wrapper_view() {
        let rendered = render_workspace_mutation_view(
            Some("add"),
            vec!["Created workspace docs at ../jk-docs".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_workspace_forget_wrapper_view() {
        let rendered = render_workspace_mutation_view(
            Some("forget"),
            vec!["Forgot workspace docs".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_workspace_rename_wrapper_view() {
        let rendered = render_workspace_mutation_view(
            Some("rename"),
            vec!["Renamed workspace docs to docs-v2".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_workspace_update_stale_wrapper_view() {
        let rendered = render_workspace_mutation_view(
            Some("update-stale"),
            vec!["Updated 2 stale workspaces".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_status_wrapper_view() {
        let rendered = render_status_view(vec![
            "Working copy changes:".to_string(),
            "M src/app.rs".to_string(),
            "A src/new.rs".to_string(),
            "Working copy  (@) : abcdefgh 0123abcd summary".to_string(),
            "Parent commit (@-): hgfedcba 89abcdef parent".to_string(),
            "Conflicted bookmarks:".to_string(),
            "feature".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_operation_log_wrapper_view() {
        let rendered = render_operation_log_view(vec![
            "@  fac974146f86 user 5 seconds ago".to_string(),
            "│  snapshot working copy".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_operation_show_wrapper_view() {
        let rendered = render_operation_show_view(vec![
            "7699d9773e37 user 41 seconds ago, lasted 19 milliseconds".to_string(),
            "snapshot working copy".to_string(),
            "Changed commits:".to_string(),
            "○  + qqrxwkpt c245343e feature".to_string(),
            "Changed working copy default@:".to_string(),
            "+ qqrxwkpt c245343e feature".to_string(),
            "- qqrxwkpt/1 2fb0ae09 (hidden) feature".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_operation_restore_wrapper_view() {
        let rendered = render_operation_mutation_view(
            "restore",
            vec![
                "Restored to operation 7699d9773e37".to_string(),
                "Working copy now matches restored operation".to_string(),
            ],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_operation_revert_wrapper_view() {
        let rendered = render_operation_mutation_view(
            "revert",
            vec![
                "Reverted operation 7699d9773e37".to_string(),
                "Created undo operation 89abcdef0123".to_string(),
            ],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_operation_diff_wrapper_view() {
        let rendered = render_operation_diff_view(vec![
            "From operation: 67d547b627fb (2026-02-07) describe commit old".to_string(),
            "  To operation: 3c63d5e89db3 (2026-02-07) describe commit new".to_string(),
            "Changed commits:".to_string(),
            "○  + uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
            "   - uxqqtlkq/1 f8cbf93c (hidden) feat(ux): expand read-mode wrappers and shortcuts"
                .to_string(),
            "Changed working copy default@:".to_string(),
            "+ uxqqtlkq 722c112d feat(ux): expand read-mode wrappers and shortcuts".to_string(),
            "- uxqqtlkq/1 f8cbf93c (hidden) feat(ux): expand read-mode wrappers and shortcuts"
                .to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_git_fetch_wrapper_view() {
        let rendered = render_git_fetch_view(vec![
            "Nothing changed.".to_string(),
            "Hint: use -b to fetch bookmarks".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_git_push_wrapper_view() {
        let rendered = render_git_push_view(vec![
            "Pushed bookmark main to origin".to_string(),
            "Done.".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_commit_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "commit",
            vec!["Working copy now at: abcdef12 commit message".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_new_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "new",
            vec!["Working copy now at: abcdef12 new change".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_describe_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "describe",
            vec!["Working copy now at: abcdef12 updated description".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_edit_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "edit",
            vec!["Working copy now at: abcdef12 edit target".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_next_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "next",
            vec!["Working copy now at: abcdef12 next revision".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_prev_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "prev",
            vec!["Working copy now at: abcdef12 previous revision".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_undo_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "undo",
            vec!["Undid operation 67d547b627fb".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_abandon_wrapper_view() {
        let rendered =
            render_top_level_mutation_view("abandon", vec!["Abandoned 1 commits.".to_string()]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_restore_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "restore",
            vec!["Restored 2 paths from revision abcdef12".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_redo_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "redo",
            vec!["Redid operation 67d547b627fb".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_revert_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "revert",
            vec!["Reverted 2 paths from revision abcdef12".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_rebase_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "rebase",
            vec!["Rebased 3 commits onto 0123abcd".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_squash_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "squash",
            vec!["Rebased 1 commits onto abcdef12".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_split_wrapper_view() {
        let rendered = render_top_level_mutation_view(
            "split",
            vec!["Rebased 1 commits onto abcdef12".to_string()],
        );
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_tag_list_wrapper_view() {
        let rendered = render_tag_list_view(vec!["(no output)".to_string()]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_tag_set_wrapper_view() {
        let rendered =
            render_tag_set_view(vec!["Created tag v0.2.0 pointing to abc12345".to_string()]);
        insta::assert_snapshot!(rendered.join("\n"));
    }

    #[test]
    fn snapshot_renders_tag_delete_wrapper_view() {
        let rendered = render_tag_delete_view(vec![
            "Deleted tag v0.1.0".to_string(),
            "Deleted tag release-1".to_string(),
        ]);
        insta::assert_snapshot!(rendered.join("\n"));
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

        let mut operation_log_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        operation_log_app
            .handle_key(KeyEvent::from(KeyCode::Char('o')))
            .expect("operation-log shortcut should be handled");
        assert_eq!(operation_log_app.mode, Mode::Normal);
        assert_eq!(
            operation_log_app.last_command,
            vec!["operation".to_string(), "log".to_string()]
        );
        assert!(
            operation_log_app
                .lines
                .iter()
                .any(|line| line.contains("Operation Log"))
        );

        let mut bookmark_list_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        bookmark_list_app
            .handle_key(KeyEvent::from(KeyCode::Char('L')))
            .expect("bookmark-list shortcut should be handled");
        assert_eq!(bookmark_list_app.mode, Mode::Normal);
        assert_eq!(
            bookmark_list_app.last_command,
            vec!["bookmark".to_string(), "list".to_string()]
        );
        assert!(
            bookmark_list_app
                .lines
                .iter()
                .any(|line| line.contains("Bookmark List"))
                || bookmark_list_app.lines == vec!["(no output)".to_string()]
        );

        let mut resolve_list_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        resolve_list_app
            .handle_key(KeyEvent::from(KeyCode::Char('v')))
            .expect("resolve-list shortcut should be handled");
        assert_eq!(resolve_list_app.mode, Mode::Normal);
        assert_eq!(
            resolve_list_app.last_command,
            vec!["resolve".to_string(), "-l".to_string()]
        );
        assert!(
            resolve_list_app
                .lines
                .iter()
                .any(|line| line.contains("Resolve List"))
        );

        let mut file_list_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        file_list_app
            .handle_key(KeyEvent::from(KeyCode::Char('f')))
            .expect("file-list shortcut should be handled");
        assert_eq!(file_list_app.mode, Mode::Normal);
        assert_eq!(
            file_list_app.last_command,
            vec!["file".to_string(), "list".to_string()]
        );
        assert!(
            file_list_app
                .lines
                .iter()
                .any(|line| line.contains("File List"))
        );

        let mut tag_list_app =
            App::new(KeybindConfig::load().expect("keybind config should parse"));
        tag_list_app
            .handle_key(KeyEvent::from(KeyCode::Char('t')))
            .expect("tag-list shortcut should be handled");
        assert_eq!(tag_list_app.mode, Mode::Normal);
        assert_eq!(
            tag_list_app.last_command,
            vec!["tag".to_string(), "list".to_string()]
        );
        assert!(
            tag_list_app
                .lines
                .iter()
                .any(|line| line.contains("Tag List"))
        );

        let mut root_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        root_app
            .handle_key(KeyEvent::from(KeyCode::Char('w')))
            .expect("root shortcut should be handled");
        assert_eq!(root_app.mode, Mode::Normal);
        assert_eq!(root_app.last_command, vec!["root".to_string()]);
        assert!(
            root_app
                .lines
                .iter()
                .any(|line| line.contains("Workspace Root"))
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

        let mut keymap_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        keymap_app
            .handle_key(KeyEvent::from(KeyCode::Char('K')))
            .expect("keymap shortcut should be handled");
        assert_eq!(keymap_app.mode, Mode::Normal);
        assert_eq!(keymap_app.status_line, "Showing keymap".to_string());
        assert!(
            keymap_app
                .lines
                .iter()
                .any(|line| line.contains("jk keymap"))
        );

        let mut aliases_app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        aliases_app
            .handle_key(KeyEvent::from(KeyCode::Char('A')))
            .expect("aliases shortcut should be handled");
        assert_eq!(aliases_app.mode, Mode::Normal);
        assert_eq!(aliases_app.status_line, "Showing alias catalog".to_string());
        assert!(
            aliases_app
                .lines
                .iter()
                .any(|line| line.contains("jk alias catalog"))
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
