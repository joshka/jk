//! Entrypoint for the `jk` TUI binary.
//!
//! This module is intentionally small: parse CLI arguments, load keybind configuration, decide the
//! startup command, then hand off to `App`.

mod alias;
mod app;
mod cli;
mod commands;
mod config;
mod error;
mod flow;
mod jj;
mod keys;

use clap::Parser;

use app::App;
use cli::Cli;
use config::KeybindConfig;
use error::JkError;

/// Parse arguments, load runtime configuration, and run the interactive TUI session.
fn main() -> Result<(), JkError> {
    let cli = Cli::parse();
    let keybinds = KeybindConfig::load()?;

    let startup_tokens = startup_command(cli);

    let mut app = App::new(keybinds);
    app.run(startup_tokens)
}

/// Translate optional startup CLI tokens into an initial in-app command.
///
/// An empty result means "start in default `log` flow" and is handled by `App` startup logic.
fn startup_command(cli: Cli) -> Vec<String> {
    let mut tokens = Vec::new();
    if let Some(command) = cli.command {
        tokens.push(command);
        tokens.extend(cli.args);
    }

    tokens
}
