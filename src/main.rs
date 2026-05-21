//! Binary entry point for `jk`.
//!
//! This crate installs `color_eyre` and then hands control to `app::run`.

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
mod file_list;
mod file_show;
mod graph;
mod interactive_process;
mod jj;
mod jj_actions;
mod jj_rows;
mod jj_syntax;
mod operation_detail;
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
mod view_state;
mod workspaces;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    app::run()
}
