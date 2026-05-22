//! Binary entry point for `jk`.
//!
//! This crate installs `color_eyre` and then hands control to `app::run`.
//! Process setup stays here; feature behavior starts in the app boundary.

mod actions;
mod app;
mod bookmarks;
mod clipboard;
mod command;
mod diff;
mod documents;
mod files;
mod help;
mod jj;
mod log;
mod menus;
mod modes;
mod rendered_rows;
mod terminal_process;

mod operation_log;
mod resolve;
mod search;
mod selection;
mod show;
mod status;
mod tui;
mod view_state;
mod workspaces;

use color_eyre::Result;

/// Install process-wide error reporting and transfer control to the app boundary.
fn main() -> Result<()> {
    color_eyre::install()?;
    app::run()
}
