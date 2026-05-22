//! Shared terminal chrome and modal presentation.
//!
//! View slices render their own main content and own their selection, scroll, search, and command
//! policy. `modes.rs` projects app interaction state into borrowed [`Overlay`] values; this
//! module sizes, clears, styles, and renders those values as shared chrome around the active view.
//! Future behavior belongs in the view, app screen, action lifecycle, or action output owner that
//! holds the state being changed, not in this presentation layer. This module should only adjust
//! presentation geometry and styling.

mod chrome;
mod overlays;
mod status_hints;

pub use chrome::{Areas, areas, render_chrome};
pub use overlays::{Overlay, render_overlay};
pub use status_hints::StatusHints;

#[cfg(test)]
use overlays::{
    action_menu, help_overlay, help_overlay_text, render_abandon_confirm, render_action_pane,
    role_prompt,
};

#[cfg(test)]
mod tests;
