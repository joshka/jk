mod app;
mod clipboard;
mod command;
mod copy;
mod diff;
mod graph;
mod jj;
mod rendered_jj;
mod search;
mod selection;
mod show;
mod sticky_file_view;
mod tui;
mod view_state;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    app::run()
}
