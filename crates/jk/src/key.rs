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

    /// Open the selected revision show/details view.
    OpenShow,

    /// Open the selected revision evolution log.
    OpenEvolog,

    /// Open repository status.
    OpenStatus,

    /// Open the workspace list.
    OpenWorkspaces,

    /// Open the command-history list.
    OpenCommandHistory,

    /// Open the operation log list.
    OpenOperationLog,

    /// Copy the current command line.
    CopyCommand,

    /// Preview and run `jj undo`.
    StartUndo,

    /// Preview and run `jj redo`.
    StartRedo,

    /// Start an inline describe mutation for the selected revision.
    StartDescribe,

    /// Open view-scoped display and template options.
    OpenViewOptions,

    /// Close the active mode or return to the previous view.
    Back,

    /// Start a search prompt in views that support search.
    StartSearch,

    /// Jump to the next search match.
    SearchNext,

    /// Jump to the previous search match.
    SearchPrevious,

    /// Leave the active view unchanged.
    Ignore,
}

impl AppKey {
    /// Converts a crossterm key event into the current action surface.
    pub const fn from_crossterm(key: KeyEvent) -> Self {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return action_for_control_key(key.code);
        }

        if let KeyCode::Char(character) = key.code
            && let Some(action) = action_for_character_key(character)
        {
            return action;
        }

        match key {
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => Self::Back,
            KeyEvent {
                code: KeyCode::Char('q') | KeyCode::Esc,
                ..
            } => Self::Action(LogAction::Quit),
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Self::OpenShow,
            KeyEvent {
                code: KeyCode::Right,
                ..
            } => Self::Action(LogAction::ToggleExpanded),
            KeyEvent {
                code: KeyCode::Up, ..
            } => Self::Action(LogAction::Previous),
            KeyEvent {
                code: KeyCode::Down,
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
            } => Self::Action(LogAction::PageNext),
            KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => Self::Action(LogAction::ToggleMark),
            KeyEvent {
                code: KeyCode::Home,
                ..
            } => Self::Action(LogAction::First),
            KeyEvent {
                code: KeyCode::End, ..
            } => Self::Action(LogAction::Last),
            KeyEvent {
                code: KeyCode::Left,
                ..
            } => Self::Action(LogAction::CollapseExpanded),
            _ => Self::Ignore,
        }
    }
}

/// Interprets unmodified character keys.
const fn action_for_character_key(character: char) -> Option<AppKey> {
    match character {
        'q' => Some(AppKey::Action(LogAction::Quit)),
        'r' => Some(AppKey::Action(LogAction::Refresh)),
        'H' => Some(AppKey::Action(LogAction::Home)),
        'L' => Some(AppKey::Action(LogAction::Log)),
        'V' => Some(AppKey::OpenViewOptions),
        'W' => Some(AppKey::OpenWorkspaces),
        'C' => Some(AppKey::OpenCommandHistory),
        'o' => Some(AppKey::OpenOperationLog),
        'y' => Some(AppKey::CopyCommand),
        'u' => Some(AppKey::StartUndo),
        'U' => Some(AppKey::StartRedo),
        'm' => Some(AppKey::StartDescribe),
        'v' => Some(AppKey::OpenEvolog),
        'l' => Some(AppKey::Action(LogAction::ToggleExpanded)),
        'd' => Some(AppKey::Action(LogAction::OpenDiff)),
        'c' => Some(AppKey::Action(LogAction::ClearMarks)),
        's' => Some(AppKey::OpenStatus),
        '?' => Some(AppKey::Action(LogAction::ToggleHelp)),
        '/' => Some(AppKey::StartSearch),
        'n' => Some(AppKey::SearchNext),
        'N' => Some(AppKey::SearchPrevious),
        'b' => Some(AppKey::Action(LogAction::PagePrevious)),
        'k' => Some(AppKey::Action(LogAction::Previous)),
        'j' => Some(AppKey::Action(LogAction::Next)),
        'g' => Some(AppKey::Action(LogAction::First)),
        'G' => Some(AppKey::Action(LogAction::Last)),
        '[' => Some(AppKey::Action(LogAction::PreviousFile)),
        ']' => Some(AppKey::Action(LogAction::NextFile)),
        '{' => Some(AppKey::Action(LogAction::PreviousHunk)),
        '}' => Some(AppKey::Action(LogAction::NextHunk)),
        '-' => Some(AppKey::Action(LogAction::FoldHunk)),
        '+' => Some(AppKey::Action(LogAction::UnfoldHunk)),
        '<' => Some(AppKey::Action(LogAction::HorizontalPrevious)),
        '>' => Some(AppKey::Action(LogAction::HorizontalNext)),
        'h' => Some(AppKey::Action(LogAction::CollapseExpanded)),
        _ => None,
    }
}

/// Interprets Ctrl-key bindings that should override ordinary character keys.
const fn action_for_control_key(code: KeyCode) -> AppKey {
    match code {
        KeyCode::Char('b') => AppKey::Action(LogAction::PagePrevious),
        KeyCode::Char('f') => AppKey::Action(LogAction::PageNext),
        KeyCode::Char('k') => AppKey::Action(LogAction::ScrollPreviousLine),
        KeyCode::Char('j') => AppKey::Action(LogAction::ScrollNextLine),
        KeyCode::Left => AppKey::Action(LogAction::FoldAll),
        KeyCode::Right => AppKey::Action(LogAction::UnfoldAll),
        _ => AppKey::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_opens_show_details() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            AppKey::OpenShow
        );
    }

    #[test]
    fn left_collapses_expanded_details() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            AppKey::Action(LogAction::CollapseExpanded)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::CollapseExpanded)
        );
    }

    #[test]
    fn right_and_l_toggle_expanded_details() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            AppKey::Action(LogAction::ToggleExpanded)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::ToggleExpanded)
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
    fn uppercase_v_opens_view_options() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('V'), KeyModifiers::NONE)),
            AppKey::OpenViewOptions
        );
    }

    #[test]
    fn uppercase_w_opens_workspaces() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('W'), KeyModifiers::NONE)),
            AppKey::OpenWorkspaces
        );
    }

    #[test]
    fn uppercase_c_opens_command_history() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('C'), KeyModifiers::NONE)),
            AppKey::OpenCommandHistory
        );
    }

    #[test]
    fn lowercase_o_opens_operation_log() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE)),
            AppKey::OpenOperationLog
        );
    }

    #[test]
    fn lowercase_y_copies_command() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
            AppKey::CopyCommand
        );
    }

    #[test]
    fn lowercase_u_starts_undo_preview() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::NONE)),
            AppKey::StartUndo
        );
    }

    #[test]
    fn uppercase_u_starts_redo_preview() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('U'), KeyModifiers::NONE)),
            AppKey::StartRedo
        );
    }

    #[test]
    fn uppercase_t_is_unbound_after_view_options_migration() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE)),
            AppKey::Ignore
        );
    }

    #[test]
    fn lowercase_v_opens_evolog() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::NONE)),
            AppKey::OpenEvolog
        );
    }

    #[test]
    fn lowercase_m_starts_describe() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE)),
            AppKey::StartDescribe
        );
    }

    #[test]
    fn lowercase_d_opens_selected_diff() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::OpenDiff)
        );
    }

    #[test]
    fn lowercase_c_clears_log_marks() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::ClearMarks)
        );
    }

    #[test]
    fn lowercase_s_opens_status() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE)),
            AppKey::OpenStatus
        );
    }

    #[test]
    fn question_mark_toggles_mode_help() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::ToggleHelp)
        );
    }

    #[test]
    fn slash_and_navigate_search_matches() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE)),
            AppKey::StartSearch
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
            AppKey::SearchNext
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('N'), KeyModifiers::NONE)),
            AppKey::SearchPrevious
        );
    }

    #[test]
    fn backspace_maps_to_back() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            AppKey::Back
        );
    }

    #[test]
    fn brackets_jump_between_diff_files() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('['), KeyModifiers::NONE)),
            AppKey::Action(LogAction::PreviousFile)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::NextFile)
        );
    }

    #[test]
    fn angle_brackets_scroll_horizontally() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::HorizontalPrevious)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('>'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::HorizontalNext)
        );
    }

    #[test]
    fn braces_jump_between_diff_hunks() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('{'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::PreviousHunk)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('}'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::NextHunk)
        );
    }

    #[test]
    fn minus_and_plus_fold_and_unfold_diff_hunks() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::FoldHunk)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::UnfoldHunk)
        );
    }

    #[test]
    fn control_arrows_fold_and_unfold_diff_files() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL)),
            AppKey::Action(LogAction::FoldAll)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL)),
            AppKey::Action(LogAction::UnfoldAll)
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
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE)),
            AppKey::Action(LogAction::PagePrevious)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL)),
            AppKey::Action(LogAction::PageNext)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::SHIFT)),
            AppKey::Action(LogAction::PagePrevious)
        );
    }

    #[test]
    fn space_toggles_log_mark() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            AppKey::Action(LogAction::ToggleMark)
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

    #[test]
    fn control_j_and_k_scroll_log_by_line() {
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL)),
            AppKey::Action(LogAction::ScrollNextLine)
        );
        assert_eq!(
            AppKey::from_crossterm(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL)),
            AppKey::Action(LogAction::ScrollPreviousLine)
        );
    }
}
