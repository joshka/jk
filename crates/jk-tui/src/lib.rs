//! Ratatui views and interaction state for `jk`.
//!
//! The public surface is currently the log view in [`log_view`] and selected-change diff view in
//! [`diff_view`]. They accept rendered snapshots from `jk-cli`, apply input actions, and render
//! borderless views that keep `jj` output visually intact while adding title/status chrome and
//! selected-row highlighting.

pub mod diff_view;
pub mod log_view;
pub mod rendered_view;

mod ansi_text;
mod chrome;
mod diff_state;
mod keymap;
mod log_state;
mod rendered_log;
mod rendered_state;
mod selected_row;
