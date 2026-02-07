use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyBinding {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
}

impl KeyBinding {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "Enter" => Some(Self::Enter),
            "Esc" => Some(Self::Esc),
            "Backspace" => Some(Self::Backspace),
            "Up" => Some(Self::Up),
            "Down" => Some(Self::Down),
            "Left" => Some(Self::Left),
            "Right" => Some(Self::Right),
            "Home" => Some(Self::Home),
            "End" => Some(Self::End),
            single if single.chars().count() == 1 => single.chars().next().map(Self::Char),
            _ => None,
        }
    }

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
            Self::Left => matches!(event.code, KeyCode::Left),
            Self::Right => matches!(event.code, KeyCode::Right),
            Self::Home => matches!(event.code, KeyCode::Home),
            Self::End => matches!(event.code, KeyCode::End),
        }
    }
}
