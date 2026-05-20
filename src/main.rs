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
mod jj;
mod operation_detail;
mod operation_log;
mod rendered_jj;
mod resolve;
mod search;
mod selection;
mod show;
mod status;
mod sticky_file_view;
mod tui;
mod view_state;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    app::run()
}
