//! Status feature root.
//!
//! The status slice owns the rendered `jj status` screen, status-row path contracts, and
//! file-action availability derived from selected status rows. Treat this root as a table of
//! contents: `actions` names the feature-owned exact-path targets, `rows` derives those contracts
//! from rendered output, and `view` owns rendering, selection, search, copy, and command
//! dispatch for the status screen.

mod actions;
mod rows;
mod view;

pub use self::actions::StatusFileAction;
#[cfg(test)]
pub use self::view::BINDINGS;
pub use self::view::StatusView;
