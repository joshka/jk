//! Binary entry point for `jk`.
//!
//! This crate installs `color_eyre` and then hands control to `app::run`.
//! Process setup stays here; feature behavior starts in the app boundary.

mod action_pane;
mod actions;
mod app;
mod bookmarks;
mod clipboard;
mod command;
mod copy;
mod diff;
mod documents;
mod files;
mod help;
mod jj;
mod log;
mod menus;
mod modes;
mod rendered_rows;
mod status_line;
mod terminal_process;

mod operation_log;
mod resolve;
mod search;
mod selection;
mod show;
mod status;
mod theme;
mod tui;
mod view_action_targets;
mod view_state;
mod workspaces;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    app::run()
}
