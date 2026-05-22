//! Entry routing for app-owned action flows.
//!
//! This module owns the app lifecycle step after an accepted menu item or key action has already
//! been chosen: route it to a prompt, open the corresponding preview, or report a status. Feature
//! views and action menus own whether an action is available and the exact target values they
//! carry. [`super::preview`] owns preview pane construction and preview status contexts.
//! [`super::completion`] and [`super::shared`] own confirmed command result handling.
//! [`crate::actions`] owns command-plan argv, preview, and run contracts. When entry setup needs
//! jj/view side effects such as remote loading, it calls `AppServices` directly.

mod menu;
mod prompts;
mod remote;

#[cfg(test)]
use self::remote::{
    FetchRemotePromptDecision, PushRemotePromptDecision, decide_fetch_remote_prompt,
    decide_push_remote_prompt,
};

#[cfg(test)]
mod tests;
