//! Shared menu models for log, status, file, bookmark, sync, and operation surfaces.
//!
//! This module owns only shared menu contracts: the stable action vocabulary, safety marker text,
//! role-prompt presentation state, and follow-up payloads handed back after a selection. Feature
//! roots and their builders decide which actions are available for the current row or path context.
//! The app action lifecycle and `actions` own preview construction, process execution, and any
//! refresh or reveal behavior after a command completes.

mod model;
mod path_actions;
mod revision_actions;

pub(in crate::menus) use model::PREVIEW_REQUIRED_MARKER;
pub use model::{
    ActionKind, ActionMenu, ActionMenuItem, FollowUp, RolePrompt, RolePromptOption, SafetyTier,
};
pub use revision_actions::ExactActionContext;

/// Build the shared revision action menu for an exact log/detail context.
///
/// This facade exists so callers do not depend on the current staging module. New
/// feature-specific action policy should move toward that feature owner instead of growing this
/// shared wrapper.
pub fn build_action_menu(context: &ExactActionContext) -> ActionMenu {
    revision_actions::build_action_menu(context)
}

/// Truncates an exact id for compact menu labels.
fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}
