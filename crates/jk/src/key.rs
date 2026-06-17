//! Keyboard input adapter for the binary crate.
//!
//! The TUI crate owns semantic actions, while this module owns the terminal keymap that turns
//! crossterm events into those actions. Keeping the adapter here lets `jk-tui` stay backend-neutral
//! and keeps key binding tests close to the binary surface users exercise.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use jk_tui::log_view::LogAction;

/// Result of interpreting one terminal key event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppKey {
    /// Dispatch this action to the active view.
    Action(LogAction),

    /// Leave the active view unchanged.
    Ignore,
}

impl AppKey {
    /// Converts a crossterm key event into the log-first action surface.
    pub const fn from_crossterm(key: KeyEvent) -> Self {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return action_for_control_key(key.code);
        }

        match key {
            KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Esc,
                ..
            } => Self::Action(LogAction::Quit),
            KeyEvent {
                code: KeyCode::Char('r'),
                ..
            } => Self::Action(LogAction::Refresh),
            KeyEvent {
                code: KeyCode::Char('H'),
                ..
            } => Self::Action(LogAction::Home),
            KeyEvent {
                code: KeyCode::Char('L'),
                ..
            } => Self::Action(LogAction::Log),
            KeyEvent {
                code: KeyCode::Char('k') | KeyCode::Up,
                ..
            } => Self::Action(LogAction::Previous),
            KeyEvent {
                code: KeyCode::Char('j') | KeyCode::Down,
                ..
            } => Self::Action(LogAction::Next),
            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => Self::Action(LogAction::PagePrevious),
            KeyEvent {
                code: KeyCode::Char(' '),
                modifiers,
                ..
            } if modifiers.contains(KeyModifiers::SHIFT) => Self::Action(LogAction::PagePrevious),
            KeyEvent {
                code: KeyCode::PageDown,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => Self::Action(LogAction::PageNext),
            KeyEvent {
                code: KeyCode::Char('g') | KeyCode::Home,
                ..
            } => Self::Action(LogAction::First),
            KeyEvent {
                code: KeyCode::Char('G') | KeyCode::End,
                ..
            } => Self::Action(LogAction::Last),
            KeyEvent {
                code: KeyCode::Enter | KeyCode::Right,
                ..
            } => Self::Action(LogAction::ToggleExpanded),
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => Self::Action(LogAction::CollapseExpanded),
            _ => Self::Ignore,
        }
    }
}

/// Interprets Ctrl-key bindings that should override ordinary character keys.
const fn action_for_control_key(code: KeyCode) -> AppKey {
    match code {
        KeyCode::Char('b') => AppKey::Action(LogAction::PagePrevious),
        KeyCode::Char('f') => AppKey::Action(LogAction::PageNext),
        _ => AppKey::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_and_right_toggle_expanded_details() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            AppKey::Action(LogAction::ToggleExpanded)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            AppKey::Action(LogAction::ToggleExpanded)
        );
    }

    #[test]
    fn left_collapses_expanded_details() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            AppKey::Action(LogAction::CollapseExpanded)
        );
    }

    #[test]
    fn uppercase_h_and_l_switch_log_views() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::Home)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('L'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::Log)
        );
    }

    #[test]
    fn page_and_vimish_navigation_keys_move_by_larger_steps() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
            AppKey::Action(LogAction::PagePrevious)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
            AppKey::Action(LogAction::PageNext)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL)),
            AppKey::Action(LogAction::PagePrevious)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)),
            AppKey::Action(LogAction::PageNext)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            AppKey::Action(LogAction::PageNext)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::SHIFT)),
            AppKey::Action(LogAction::PagePrevious)
        );
    }

    #[test]
    fn home_end_and_vimish_navigation_keys_move_to_edges() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)),
            AppKey::Action(LogAction::First)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
            AppKey::Action(LogAction::Last)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::First)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::Last)
        );
    }
}
