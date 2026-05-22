//! Revision-scoped action-menu policy for graph and detail surfaces.
//!
//! The parent action-menu module owns shared action vocabulary and prompt
//! models. This module owns which revision actions are offered for graph and
//! detail selections, including multi-revision role prompts and mutation item
//! ordering.

mod context;
mod menu;

pub use context::ExactActionContext;
pub(super) use menu::build_action_menu;

#[cfg(test)]
mod tests;
