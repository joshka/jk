//! Key binding metadata and command effects.
//!
//! Bindings are static Rust data for now. The shape mirrors a command/key/when
//! model without introducing user-configurable keymaps before the app needs
//! them.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::copy::CopyOption;
use crate::jj::JjCommand;
use crate::search::SearchQuery;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Quit,
    Help,
    SearchPrompt,
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
    label: &'static str,
}

impl Binding {
    pub const fn new(key: KeyPattern, command: Command, label: &'static str) -> Self {
        Self {
            key,
            command,
            label,
        }
    }

    pub fn matches(self, key: KeyEvent) -> bool {
        self.key.matches(key)
    }

    pub fn command(self) -> Command {
        self.command
    }

    pub fn key(self) -> &'static str {
        self.key.label()
    }

    pub fn label(self) -> &'static str {
        self.label
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPattern {
    code: KeyCode,
    modifiers: KeyModifiers,
    label: &'static str,
}

impl KeyPattern {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers, label: &'static str) -> Self {
        Self {
            code,
            modifiers,
            label,
        }
    }

    pub const fn char(character: char, label: &'static str) -> Self {
        Self::new(KeyCode::Char(character), KeyModifiers::NONE, label)
    }

    pub const fn modified_char(
        character: char,
        modifiers: KeyModifiers,
        label: &'static str,
    ) -> Self {
        Self::new(KeyCode::Char(character), modifiers, label)
    }

    pub const fn code(code: KeyCode, label: &'static str) -> Self {
        Self::new(code, KeyModifiers::NONE, label)
    }

    fn matches(self, key: KeyEvent) -> bool {
        key.code == self.code && key.modifiers == self.modifiers
    }

    fn label(self) -> &'static str {
        self.label
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
            KeyPattern::modified_char('f', KeyModifiers::CONTROL, "C-f"),
            Command::View(ViewCommand::PageDown),
            "page",
        );

        assert!(binding.matches(key(KeyCode::Char('f'), KeyModifiers::CONTROL)));
        assert!(!binding.matches(key(KeyCode::Char('f'), KeyModifiers::NONE)));
    }

    #[test]
    fn find_binding_returns_first_matching_command() {
        let bindings = [
            Binding::new(
                KeyPattern::char('j', "j"),
                Command::View(ViewCommand::MoveDown),
                "move",
            ),
            Binding::new(KeyPattern::char('q', "q"), Command::Quit, "quit"),
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
