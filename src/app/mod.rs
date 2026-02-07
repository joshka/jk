mod history;
mod input;
mod preview;
mod runtime;
mod selection;
mod terminal;
mod view;

use crate::config::KeybindConfig;
use crate::flow::PromptKind;

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
}

#[cfg(test)]
mod tests;
