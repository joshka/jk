//! Interactive TUI application runtime.
//!
//! `App` owns UI state, input mode transitions, and rendered output lines while delegating command
//! planning to `flow` and subprocess execution to `jj`.

mod history;
mod input;
mod preview;
mod runtime;
mod selection;
mod terminal;
mod view;

use crate::config::KeybindConfig;
use crate::flow::PromptKind;

/// Current input mode for the footer interaction model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Command,
    Confirm,
    Prompt,
}

/// Mutable runtime state for one `jk` session.
pub struct App {
    keybinds: KeybindConfig,
    mode: Mode,
    lines: Vec<String>,
    cursor: usize,
    scroll: usize,
    viewport_rows: usize,
    status_line: String,
    current_view_command: String,
    view_back_stack: Vec<String>,
    view_forward_stack: Vec<String>,
    navigating_view_stack: bool,
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

/// Active prompt state while in [`Mode::Prompt`].
#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptState {
    kind: PromptKind,
    label: String,
    allow_empty: bool,
    input: String,
}

impl App {
    /// Construct a new app with default state and provided keybindings.
    ///
    /// The initial screen is placeholder content until startup command execution populates lines.
    pub fn new(keybinds: KeybindConfig) -> Self {
        Self {
            keybinds,
            mode: Mode::Normal,
            lines: vec!["Initializing jk...".to_string()],
            cursor: 0,
            scroll: 0,
            viewport_rows: 20,
            status_line: "Press : for commands, q to quit".to_string(),
            current_view_command: "log".to_string(),
            view_back_stack: Vec::new(),
            view_forward_stack: Vec::new(),
            navigating_view_stack: false,
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
