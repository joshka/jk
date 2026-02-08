//! Keybinding primitives used by configuration parsing and runtime input handling.

use crossterm::event::{KeyCode, KeyEvent};

/// Supported key tokens in config files and rendered keymap output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyBinding {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Up,
    Down,
    PageUp,
    PageDown,
    Left,
    Right,
    Home,
    End,
}

impl KeyBinding {
    /// Parse one configured key token from TOML.
    ///
    /// Single-character values map to [`Self::Char`]; named values map to non-character keys.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "Enter" => Some(Self::Enter),
            "Esc" => Some(Self::Esc),
            "Backspace" => Some(Self::Backspace),
            "Up" => Some(Self::Up),
            "Down" => Some(Self::Down),
            "PageUp" => Some(Self::PageUp),
            "PageDown" => Some(Self::PageDown),
            "Left" => Some(Self::Left),
            "Right" => Some(Self::Right),
            "Home" => Some(Self::Home),
            "End" => Some(Self::End),
            single if single.chars().count() == 1 => single.chars().next().map(Self::Char),
            _ => None,
        }
    }

    /// Return whether this binding matches an incoming key event.
    ///
    /// Matching is exact on character code and does not currently model modifiers.
    pub fn matches(&self, event: KeyEvent) -> bool {
        match self {
            Self::Char(expected) => {
                matches!(event.code, KeyCode::Char(actual) if actual == *expected)
            }
            Self::Enter => matches!(event.code, KeyCode::Enter),
            Self::Esc => matches!(event.code, KeyCode::Esc),
            Self::Backspace => matches!(event.code, KeyCode::Backspace),
            Self::Up => matches!(event.code, KeyCode::Up),
            Self::Down => matches!(event.code, KeyCode::Down),
            Self::PageUp => matches!(event.code, KeyCode::PageUp),
            Self::PageDown => matches!(event.code, KeyCode::PageDown),
            Self::Left => matches!(event.code, KeyCode::Left),
            Self::Right => matches!(event.code, KeyCode::Right),
            Self::Home => matches!(event.code, KeyCode::Home),
            Self::End => matches!(event.code, KeyCode::End),
        }
    }
}
