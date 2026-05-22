//! The default/log graph feature root.
//!
//! Rows preserve rendered `jj` graph output while the view owns graph-local
//! selection, search, copy, mode switching, and action-menu behavior.

mod rows;
mod view;

pub use self::rows::{LogItem, load_compact_log_context, load_entries};
pub use self::view::LogView;

pub const BINDINGS: &[crate::command::Binding] = self::view::BINDINGS;

#[cfg(test)]
mod tests;
