//! Pure reducers for modal input.
//!
//! This module turns modal key presses and prompt state into small reducer outcomes. It does not
//! open previews, mutate app routing, update status text, or run commands; `input.rs` owns
//! those side effects after interpreting the reducer result.

mod confirmation;
mod menu;
mod prompts;

pub(super) use confirmation::{ConfirmationKey, reduce_confirmation_key};
pub(super) use menu::{
    MenuKey, RolePromptDecision, is_help_close_key, is_help_scroll_key, reduce_menu_key,
    reduce_role_prompt_accept, reduce_view_menu_key,
};
pub(super) use prompts::{
    PromptAcceptDecision, TextPromptKey, reduce_bookmark_name_prompt_accept,
    reduce_bookmark_rename_prompt_accept, reduce_commit_prompt_accept,
    reduce_describe_prompt_accept, reduce_text_prompt_key,
};
#[allow(unused_imports)]
pub(in crate::app) use prompts::{rebase_plan_from_prompt, squash_plan_from_prompt};

#[cfg(test)]
mod tests;
