//! Status feature root.
//!
//! The status slice owns the rendered `jj status` screen, status-row path contracts, and
//! file-action availability derived from selected status rows.

mod actions;
mod rows;
mod view;

pub use self::actions::StatusFileAction;
#[cfg(test)]
pub use self::view::BINDINGS;
pub use self::view::StatusView;
