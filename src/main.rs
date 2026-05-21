//! Binary entry point for `jk`.
//!
//! This crate installs `color_eyre` and then hands control to `app::run`.
//! Process setup stays here; feature behavior starts in the app boundary.

mod action_menu;
mod action_output;
mod app;
mod app_screen;
mod app_status;
mod bookmarks;
mod clipboard;
mod command;
mod copy;
mod diff;
mod files;
mod graph;
mod help;
mod interactive_process;
mod jj;
mod jj_actions;
mod jj_rows;
mod jj_syntax;
mod operation_log;
mod rendered_jj;
mod resolve;
mod search;
mod selection;
mod show;
mod status;
mod sticky_file_view;
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
