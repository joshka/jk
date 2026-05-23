//! Shared terminal chrome and modal presentation.
//!
//! View slices render their own main content and own their selection, scroll, search, and command
//! policy. `modes.rs` projects app interaction state into borrowed [`Overlay`] values; this
//! module sizes, clears, styles, and renders those values as shared chrome around the active view.
//! Future behavior belongs in the view, app screen, action lifecycle, or action output owner that
//! holds the state being changed, not in this presentation layer. Treat this root as a table of
//! contents: `chrome` owns frame layout and status/title rows, `status_hints` owns width-fit hint
//! projection, and `overlays` owns shared modal rendering for help, menus, prompts, and action
//! panes.

mod chrome;
mod overlays;
mod status_hints;
pub mod theme;

pub use chrome::{areas, render_chrome};
pub use overlays::{Overlay, render_overlay};
#[cfg(test)]
use overlays::{
    action_menu, help_overlay, help_overlay_text, render_abandon_confirm, render_action_pane,
    role_prompt,
};
pub use status_hints::StatusHints;

#[cfg(test)]
mod tests;
