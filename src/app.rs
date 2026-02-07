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
            self.render_confirmation_preview(&tokens);
            return Ok(());
        }

        self.execute_tokens(tokens)
    }

    fn render_confirmation_preview(&mut self, tokens: &[String]) {
        let mut lines = vec![format!("Confirm: jj {}", tokens.join(" "))];

        if let Some(preview_tokens) = dry_run_preview_tokens(tokens)
            && let Ok(preview) = jj::run(&preview_tokens)
        {
            lines.push(String::new());
            lines.push(format!("Preview: jj {}", preview_tokens.join(" ")));
            lines.extend(preview.output);
        }

        self.lines = lines;
        self.cursor = 0;
        self.scroll = 0;
    }

    fn execute_command_line(&mut self, command: &str) -> Result<(), JkError> {
        let action = plan_command(command, self.selected_revision());
        self.apply_flow_action(action)
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

fn dry_run_preview_tokens(tokens: &[String]) -> Option<Vec<String>> {
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

    None
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

    use crate::flows::{FlowAction, PromptKind};

    use super::{
        App, Mode, dry_run_preview_tokens, extract_revision, is_change_id, is_commit_id,
        is_dangerous, startup_action,
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
        let preview = dry_run_preview_tokens(&["git".to_string(), "push".to_string()]);
        assert_eq!(
            preview,
            Some(vec![
                "git".to_string(),
                "push".to_string(),
                "--dry-run".to_string()
            ])
        );

        let existing = dry_run_preview_tokens(&[
            "git".to_string(),
            "push".to_string(),
            "--dry-run".to_string(),
        ]);
        assert_eq!(existing, None);
    }
}
