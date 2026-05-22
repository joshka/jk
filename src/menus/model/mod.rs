//! Shared action-menu presentation models.
//!
//! This module owns the stable action vocabulary, safety marker text,
//! role-prompt presentation state, copy-menu options, and follow-up payloads
//! handed back after a selection. Feature roots and their builders decide
//! which actions are available for the current row or path context.

mod action;
mod copy;
mod menu;
mod prompt;

pub(in crate::menus) use action::PREVIEW_REQUIRED_MARKER;
pub use action::{ActionKind, SafetyTier};
pub use copy::CopyOption;
pub use menu::{ActionMenu, ActionMenuItem, FollowUp};
pub use prompt::{RolePrompt, RolePromptOption};
