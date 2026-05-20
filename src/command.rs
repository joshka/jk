//! Key binding metadata and command effects.
//!
//! Bindings are static Rust data. Help and status text live in `tui.rs`, so
//! this module only owns the key-to-command mapping used by dispatch.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::copy::CopyOption;
use crate::jj::JjCommand;
use crate::search::SearchQuery;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Quit,
    Help,
    SearchPrompt,
    PromptLogRevset,
    Copy,
    ViewFormat,
    Refresh,
    Back,
    SwitchLog,
    SwitchDefault,
    View(ViewCommand),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewCommand {
    CycleMode,
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    MoveFirst,
    MoveLast,
    NextFile,
    PreviousFile,
    OpenShow,
    OpenDiff,
    StartSearch,
    NextSearchMatch,
    PreviousSearchMatch,
    Copy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Binding {
    key: KeyPattern,
    command: Command,
}

impl Binding {
    pub const fn new(key: KeyPattern, command: Command) -> Self {
        Self { key, command }
    }

    pub fn matches(self, key: KeyEvent) -> bool {
        self.key.matches(key)
    }

    pub fn command(self) -> Command {
        self.command
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPattern {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyPattern {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub const fn char(character: char) -> Self {
        Self::new(KeyCode::Char(character), KeyModifiers::NONE)
    }

    pub const fn modified_char(character: char, modifiers: KeyModifiers) -> Self {
        Self::new(KeyCode::Char(character), modifiers)
    }

    pub const fn code(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    fn matches(self, key: KeyEvent) -> bool {
        key.code == self.code && key.modifiers == self.modifiers
    }
}

pub struct CommandContext<'a> {
    pub viewport_height: u16,
    pub search: Option<&'a SearchQuery>,
}

impl CommandContext<'_> {
    pub fn page_size(&self) -> usize {
        usize::from(self.viewport_height.saturating_sub(1).max(1))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewEffect {
    Ignored,
    Handled,
    StatusMessage(String),
    StatusError(String),
    OpenDetail(JjCommand, String),
    SearchMoved,
    SearchStarted { matches: usize },
    CopyOptions(Vec<CopyOption>),
}

pub fn find_binding(bindings: &[Binding], key: KeyEvent) -> Option<Binding> {
    bindings
        .iter()
        .copied()
        .find(|binding| binding.matches(key))
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

    use super::*;

    #[test]
    fn binding_matches_key_code_and_modifiers() {
        let binding = Binding::new(
            KeyPattern::modified_char('f', KeyModifiers::CONTROL),
            Command::View(ViewCommand::PageDown),
        );

        assert!(binding.matches(key(KeyCode::Char('f'), KeyModifiers::CONTROL)));
        assert!(!binding.matches(key(KeyCode::Char('f'), KeyModifiers::NONE)));
    }

    #[test]
    fn find_binding_returns_first_matching_command() {
        let bindings = [
            Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
            Binding::new(KeyPattern::char('q'), Command::Quit),
        ];

        assert_eq!(
            find_binding(&bindings, key(KeyCode::Char('q'), KeyModifiers::NONE))
                .map(Binding::command),
            Some(Command::Quit)
        );
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}
