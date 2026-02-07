mod alias;
mod app;
mod cli;
mod commands;
mod config;
mod error;
mod flows;
mod jj;
mod keys;

use clap::Parser;

use alias::normalize_alias;
use app::App;
use cli::Cli;
use config::KeybindConfig;
use error::JkError;

fn main() -> Result<(), JkError> {
    let cli = Cli::parse();
    let keybinds = KeybindConfig::load()?;

    let startup_tokens = startup_command(cli);

    let mut app = App::new(keybinds);
    app.run(startup_tokens)
}

fn startup_command(cli: Cli) -> Vec<String> {
    let mut tokens = Vec::new();
    if let Some(command) = cli.command {
        tokens.push(command);
        tokens.extend(cli.args);
    }

    normalize_alias(&tokens)
}
