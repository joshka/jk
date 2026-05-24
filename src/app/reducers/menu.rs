use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::actions::{JjRebasePlan, JjSquashPlan};
use crate::app::reducers::prompts::{rebase_plan_from_prompt, squash_plan_from_prompt};
use crate::menus::{ActionKind, RolePrompt};
use crate::modes::view_menu_options;

/// Outcome of one menu key in a pure reducer context.
pub enum MenuKey {
    Cancel,
    Accept,
    Shortcut(char),
    Other,
}

/// Pure decision produced when a role-prompt selection is accepted.
#[derive(Debug, Eq, PartialEq)]
pub enum RolePromptDecision {
    Rebase(JjRebasePlan),
    Squash(JjSquashPlan),
    StatusMessage(String),
    StatusError(String),
}

/// Reduce a menu-navigation key, updating the selected index in place when needed.
pub fn reduce_menu_key(selected: &mut usize, item_count: usize, code: KeyCode) -> MenuKey {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => MenuKey::Cancel,
        KeyCode::Char('j') | KeyCode::Down if *selected + 1 < item_count => {
            *selected += 1;
            MenuKey::Other
        }
        KeyCode::Char('k') | KeyCode::Up => {
            *selected = selected.saturating_sub(1);
            MenuKey::Other
        }
        KeyCode::Enter => MenuKey::Accept,
        KeyCode::Char(shortcut) => MenuKey::Shortcut(shortcut),
        _ => MenuKey::Other,
    }
}

/// Reduce the view-menu key set, including `v` as an explicit close key.
pub fn reduce_view_menu_key(selected: &mut usize, code: KeyCode) -> MenuKey {
    match code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => MenuKey::Cancel,
        KeyCode::Char('j') | KeyCode::Down => {
            *selected = (*selected + 1).min(view_menu_options().len().saturating_sub(1));
            MenuKey::Other
        }
        KeyCode::Char('k') | KeyCode::Up => {
            *selected = selected.saturating_sub(1);
            MenuKey::Other
        }
        KeyCode::Enter => MenuKey::Accept,
        _ => MenuKey::Other,
    }
}

/// Turn an accepted role prompt into either a rewrite plan or a status result.
pub fn reduce_role_prompt_accept(action: ActionKind, prompt: &RolePrompt) -> RolePromptDecision {
    match action {
        ActionKind::Rebase => match rebase_plan_from_prompt(prompt) {
            Some(rebase) => RolePromptDecision::Rebase(rebase),
            None => RolePromptDecision::StatusError(prompt.status_message()),
        },
        ActionKind::Squash => match squash_plan_from_prompt(prompt) {
            Some(squash) => RolePromptDecision::Squash(squash),
            None => RolePromptDecision::StatusError(prompt.status_message()),
        },
        ActionKind::Edit
        | ActionKind::New
        | ActionKind::Split
        | ActionKind::Duplicate
        | ActionKind::Restore
        | ActionKind::Revert
        | ActionKind::Abandon
        | ActionKind::Absorb
        | ActionKind::FileTrack
        | ActionKind::FileUntrack
        | ActionKind::FileChmodExecutable
        | ActionKind::FileChmodNormal => RolePromptDecision::StatusMessage(prompt.status_message()),
    }
}

/// Report whether a key closes the help overlay without executing a command.
pub fn is_help_close_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => key.modifiers.is_empty(),
        KeyCode::Char('?') => key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT,
        _ => false,
    }
}

/// Report whether a key should be treated as local help-menu scrolling only.
pub fn is_help_scroll_key(key: KeyEvent) -> bool {
    key.modifiers.is_empty() && matches!(key.code, KeyCode::Down | KeyCode::Up)
}
