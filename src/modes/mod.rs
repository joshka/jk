//! App-level modal and prompt screen contracts.
//!
//! `app.rs` owns dispatch and side effects. This root owns the transient screen
//! state and the projection from that state to status-line text and shared TUI
//! overlays. It should stay free of command execution and feature-specific
//! availability rules.

mod projection;
mod state;
mod view_menu;

pub(crate) use self::state::InteractionMode;
pub use self::view_menu::{ViewMenuAction, ViewMenuOption, view_menu_options};

#[cfg(test)]
mod tests;
