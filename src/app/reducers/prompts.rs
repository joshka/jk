use crossterm::event::KeyCode;

use crate::actions::{
    JjBookmarkMutationKind, JjBookmarkMutationPlan, JjBookmarkTarget, JjCommitPlan, JjDescribePlan,
    JjDescribeTarget, JjRebasePlan, JjSquashPlan, validate_bookmark_rename_new_name,
};
use crate::menus::RolePrompt;

/// Outcome of one text-prompt key in a pure reducer context.
pub enum TextPromptKey {
    Cancel,
    Accept,
    Edited,
    Ignored,
}

/// Pure outcome produced when a text prompt is accepted.
#[derive(Debug, Eq, PartialEq)]
pub enum PromptAcceptDecision<T> {
    Preview(T),
    StatusMessage(String),
}

/// Reduce a text-entry prompt key without performing any app-side effects.
pub fn reduce_text_prompt_key(input: &mut String, code: KeyCode) -> TextPromptKey {
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

/// Decide whether a describe prompt should open preview or stop with a status message.
pub fn reduce_describe_prompt_accept(
    target: &JjDescribeTarget,
    input: &str,
) -> PromptAcceptDecision<JjDescribePlan> {
    let message = input.trim().to_owned();

    if message.is_empty() {
        PromptAcceptDecision::StatusMessage("describe cancelled: empty description".to_owned())
    } else {
        PromptAcceptDecision::Preview(JjDescribePlan::new(target.clone(), message))
    }
}

/// Decide whether a commit prompt should open preview or stop with a status message.
pub fn reduce_commit_prompt_accept(input: &str) -> PromptAcceptDecision<JjCommitPlan> {
    let message = input.trim().to_owned();

    if message.is_empty() {
        PromptAcceptDecision::StatusMessage("commit cancelled: empty description".to_owned())
    } else {
        PromptAcceptDecision::Preview(JjCommitPlan::new(message))
    }
}

/// Decide whether a bookmark-name prompt should open preview or stop with a status message.
pub fn reduce_bookmark_name_prompt_accept(
    kind: JjBookmarkMutationKind,
    target: &JjBookmarkTarget,
    input: &str,
) -> PromptAcceptDecision<JjBookmarkMutationPlan> {
    let name = input.trim().to_owned();

    if name.is_empty() {
        PromptAcceptDecision::StatusMessage(format!(
            "bookmark {} cancelled: empty bookmark name",
            kind.label()
        ))
    } else {
        PromptAcceptDecision::Preview(bookmark_mutation_plan(kind, name, target.clone()))
    }
}

/// Decide whether a bookmark-rename prompt should open preview or stop with a status message.
pub fn reduce_bookmark_rename_prompt_accept(
    old_name: &str,
    input: &str,
) -> PromptAcceptDecision<JjBookmarkMutationPlan> {
    let new_name = input.to_owned();

    match validate_bookmark_rename_new_name(old_name, &new_name) {
        Ok(()) => PromptAcceptDecision::Preview(JjBookmarkMutationPlan::rename(
            old_name.to_owned(),
            new_name,
        )),
        Err(reason) => {
            PromptAcceptDecision::StatusMessage(format!("bookmark rename cancelled: {reason}"))
        }
    }
}

/// Build a rebase plan from the explicit source/destination roles selected in a role prompt.
pub fn rebase_plan_from_prompt(prompt: &RolePrompt) -> Option<JjRebasePlan> {
    let destination = prompt.destination_revision()?;
    let sources = prompt
        .source_revisions()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    (!sources.is_empty()).then(|| JjRebasePlan::new(sources, destination.to_owned()))
}

/// Build a squash plan from the explicit source/destination roles selected in a role prompt.
pub fn squash_plan_from_prompt(prompt: &RolePrompt) -> Option<JjSquashPlan> {
    let destination = prompt.destination_revision()?;
    let sources = prompt
        .source_revisions()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    (!sources.is_empty()).then(|| JjSquashPlan::new(sources, destination.to_owned()))
}

/// Build the bookmark mutation plan implied by a prompt-confirmed bookmark name.
pub fn bookmark_mutation_plan(
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
