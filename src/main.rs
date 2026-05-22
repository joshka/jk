#![warn(clippy::redundant_pub_crate)]

//! Binary entry point for `jk`.

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
mod operation_log;
mod rendered_rows;
mod resolve;
mod search;
mod selection;
mod show;
mod status;
mod terminal_process;
mod tui;
mod view_state;
mod workspaces;

use color_eyre::Result;

/// Install process-wide error reporting and transfer control to the app boundary.
fn main() -> Result<()> {
    color_eyre::install()?;
    app::run()
}
