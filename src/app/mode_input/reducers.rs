//! Pure reducers for modal input.
//!
//! This module turns modal key presses and prompt state into small reducer outcomes. It does not
//! open previews, mutate app routing, update status text, or run commands; `mode_input.rs` owns
//! those side effects after interpreting the reducer result.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action_menu::{ActionKind, RolePrompt};
use crate::action_output::ActionOutput;
use crate::app_screen::view_menu_options;
use crate::jj_actions::{
    JjBookmarkMutationKind, JjBookmarkMutationPlan, JjBookmarkTarget, JjRebasePlan, JjSquashPlan,
};

pub(super) enum TextPromptKey {
    Cancel,
    Accept,
    Edited,
    Ignored,
}

pub(super) fn reduce_text_prompt_key(input: &mut String, code: KeyCode) -> TextPromptKey {
    match code {
        KeyCode::Esc => TextPromptKey::Cancel,
        KeyCode::Enter => TextPromptKey::Accept,
        KeyCode::Backspace => {
            input.pop();
            TextPromptKey::Edited
        }
        KeyCode::Char(character) => {
            input.push(character);
            TextPromptKey::Edited
        }
        _ => TextPromptKey::Ignored,
    }
}

pub(super) enum MenuKey {
    Cancel,
    Accept,
    Shortcut(char),
    Other,
}

pub(super) fn reduce_menu_key(selected: &mut usize, item_count: usize, code: KeyCode) -> MenuKey {
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

pub(super) fn reduce_view_menu_key(selected: &mut usize, code: KeyCode) -> MenuKey {
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

#[derive(Debug, Eq, PartialEq)]
pub(super) enum RolePromptDecision {
    Rebase(JjRebasePlan),
    Squash(JjSquashPlan),
    StatusMessage(String),
    StatusError(String),
}

pub(super) fn reduce_role_prompt_accept(
    action: ActionKind,
    prompt: &RolePrompt,
) -> RolePromptDecision {
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

pub(super) enum ConfirmationKey {
    Cancel,
    Accept,
    Handled,
    Ignored,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_menu::RolePromptOption;

    fn role_prompt(options: Vec<RolePromptOption>) -> RolePrompt {
        RolePrompt::new("confirm role assignment", options, "Preview required.")
    }

    #[test]
    fn role_prompt_accept_builds_rebase_plan() {
        let prompt = role_prompt(vec![
            RolePromptOption::new("source", "source-a"),
            RolePromptOption::new("destination", "dest"),
            RolePromptOption::new("source", "source-b"),
        ]);

        let RolePromptDecision::Rebase(rebase) =
            reduce_role_prompt_accept(ActionKind::Rebase, &prompt)
        else {
            panic!("expected rebase decision");
        };

        assert_eq!(rebase.sources(), &["source-a", "source-b"]);
        assert_eq!(rebase.destination(), "dest");
    }

    #[test]
    fn role_prompt_accept_builds_squash_plan() {
        let prompt = role_prompt(vec![
            RolePromptOption::new("source", "source-a"),
            RolePromptOption::new("source", "source-b"),
            RolePromptOption::new("destination", "dest"),
        ]);

        let RolePromptDecision::Squash(squash) =
            reduce_role_prompt_accept(ActionKind::Squash, &prompt)
        else {
            panic!("expected squash decision");
        };

        assert_eq!(squash.sources(), &["source-a", "source-b"]);
        assert_eq!(squash.destination(), "dest");
    }

    #[test]
    fn role_prompt_accept_reports_rewrite_prompt_error_without_a_plan() {
        let prompt = role_prompt(vec![RolePromptOption::new("source", "source-a")]);

        assert_eq!(
            reduce_role_prompt_accept(ActionKind::Rebase, &prompt),
            RolePromptDecision::StatusError("source: source-a\nPreview required.".to_owned())
        );
    }

    #[test]
    fn role_prompt_accept_reports_unsupported_action_as_status_message() {
        let prompt = role_prompt(vec![
            RolePromptOption::new("source", "source-a"),
            RolePromptOption::new("destination", "dest"),
        ]);

        assert_eq!(
            reduce_role_prompt_accept(ActionKind::Absorb, &prompt),
            RolePromptDecision::StatusMessage(
                "source: source-a\ndestination: dest\nPreview required.".to_owned(),
            )
        );
    }
}

pub(super) fn reduce_confirmation_key(
    input: &mut String,
    output: &mut ActionOutput,
    visible_lines: u16,
    code: KeyCode,
) -> ConfirmationKey {
    match code {
        KeyCode::Esc => ConfirmationKey::Cancel,
        KeyCode::Enter => ConfirmationKey::Accept,
        KeyCode::Backspace => {
            input.pop();
            ConfirmationKey::Handled
        }
        KeyCode::Char(character) => {
            input.push(character);
            ConfirmationKey::Handled
        }
        KeyCode::Down => {
            output.scroll_down(visible_lines);
            ConfirmationKey::Handled
        }
        KeyCode::Up => {
            output.scroll_up();
            ConfirmationKey::Handled
        }
        KeyCode::PageDown => {
            output.page_down(visible_lines);
            ConfirmationKey::Handled
        }
        KeyCode::PageUp => {
            output.page_up(visible_lines);
            ConfirmationKey::Handled
        }
        KeyCode::Home => {
            output.scroll_to_top();
            ConfirmationKey::Handled
        }
        KeyCode::End => {
            output.scroll_to_bottom(visible_lines);
            ConfirmationKey::Handled
        }
        _ => ConfirmationKey::Ignored,
    }
}

pub(in crate::app) fn rebase_plan_from_prompt(prompt: &RolePrompt) -> Option<JjRebasePlan> {
    let destination = prompt.destination_revision()?;
    let sources = prompt
        .source_revisions()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    (!sources.is_empty()).then(|| JjRebasePlan::new(sources, destination.to_owned()))
}

pub(in crate::app) fn squash_plan_from_prompt(prompt: &RolePrompt) -> Option<JjSquashPlan> {
    let destination = prompt.destination_revision()?;
    let sources = prompt
        .source_revisions()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    (!sources.is_empty()).then(|| JjSquashPlan::new(sources, destination.to_owned()))
}

pub(super) fn is_help_close_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => key.modifiers.is_empty(),
        KeyCode::Char('?') => key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT,
        _ => false,
    }
}

pub(super) fn is_help_scroll_key(key: KeyEvent) -> bool {
    key.modifiers.is_empty() && matches!(key.code, KeyCode::Down | KeyCode::Up)
}

pub(super) fn bookmark_mutation_plan(
    kind: JjBookmarkMutationKind,
    name: String,
    target: JjBookmarkTarget,
) -> JjBookmarkMutationPlan {
    match kind {
        JjBookmarkMutationKind::Create => JjBookmarkMutationPlan::create(name, target),
        JjBookmarkMutationKind::Set => JjBookmarkMutationPlan::set(name, target),
        JjBookmarkMutationKind::Move => JjBookmarkMutationPlan::move_to(name, target),
        JjBookmarkMutationKind::Rename => {
            unreachable!("bookmark rename uses the old-name prompt and has no revision target")
        }
        JjBookmarkMutationKind::Delete => JjBookmarkMutationPlan::delete(name),
        JjBookmarkMutationKind::Forget => {
            unreachable!("bookmark forget uses the selected bookmark row and has no prompt target")
        }
        JjBookmarkMutationKind::Track | JjBookmarkMutationKind::Untrack => {
            unreachable!(
                "bookmark track/untrack uses selected bookmark rows and has no prompt target"
            )
        }
    }
}
