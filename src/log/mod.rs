//! The default/log graph feature root.
//!
//! Rows preserve rendered `jj` graph output while the view owns graph-local
//! selection, search, copy, mode switching, and action-menu behavior.
//!
//! The root module re-exports the row contract used by startup loading and the
//! `LogView` state machine used by app/view dispatch.

mod rows;
mod view;

/// Rendered log-row contract and row loaders used by startup and refresh paths.
pub use self::rows::{LogItem, load_compact_log_context, load_entries};
/// Log view state, rendering, and graph-local command handling.
pub use self::view::LogView;

/// View-local bindings for the default/log surface.
pub const BINDINGS: &[crate::command::Binding] = self::view::BINDINGS;

#[cfg(test)]
mod tests;
