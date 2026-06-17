//! Binary entry point for the `jk` terminal UI.
//!
//! This crate owns command-line parsing, terminal lifecycle, and the bridge between crossterm input
//! events and backend-neutral TUI actions. The product behavior is intentionally delegated to
//! `jk-cli` and `jk-tui` so the binary stays a thin orchestration layer.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use crossterm::event::{self, Event};
use crossterm::style::force_color_output;
use jk_cli::{JjLog, JjLogCommand};
use jk_tui::log_view::{ActionResult, LogView};

mod key;

use key::AppKey;

/// Command-line options for the first log-oriented `jk` surface.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Repository path to pass to jj.
    #[arg(short = 'R', long = "repository")]
    repository: Option<PathBuf>,

    /// Maximum number of log entries to render for the default command.
    #[arg(short = 'n', long)]
    limit: Option<usize>,

    /// View to open. If omitted, jk follows jj's configured default command.
    #[command(subcommand)]
    command: Option<Command>,
}

/// Top-level view commands supported by the binary.
#[derive(Debug, Subcommand)]
enum Command {
    /// Show the jj log view.
    Log(LogArgs),
}

/// Options for the explicit `jk log` command.
#[derive(Debug, Parser)]
struct LogArgs {
    /// Maximum number of log entries to render.
    #[arg(short = 'n', long)]
    limit: Option<usize>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let source = log_source(&args);
    let entries = source.load()?;
    let app = LogView::new(entries);

    run_terminal(app, source)?;
    Ok(())
}

/// Builds the log source that matches the requested command-line view.
///
/// Bare `jk` intentionally starts from jj's configured default command, while `jk log` forces the
/// explicit log command. The top-level limit applies to both forms unless the subcommand provides a
/// narrower value.
fn log_source(args: &Args) -> JjLog {
    let (command, limit) = match &args.command {
        Some(Command::Log(log_args)) => (JjLogCommand::Log, log_args.limit.or(args.limit)),
        None => (JjLogCommand::ConfiguredDefault, args.limit),
    };

    let source = JjLog::default().with_command(command).with_limit(limit);
    if let Some(repository) = &args.repository {
        source.with_repository(repository)
    } else {
        source
    }
}

/// Owns the terminal event loop for the current log-first application.
///
/// The view remains responsible for state transitions and rendering. This loop only translates
/// terminal events, performs I/O requested by the view, and redraws when input or terminal resize
/// events can change the screen.
fn run_terminal(mut app: LogView, mut source: JjLog) -> Result<()> {
    // jj should keep configured colors even when the parent process was run by an agent or tool
    // that exports NO_COLOR.
    force_color_output(true);
    let mut terminal = ratatui::try_init().inspect_err(|_| ratatui::restore())?;
    let _terminal_restore = TerminalRestore;
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            terminal.draw(|frame| app.render(frame))?;
            needs_redraw = false;
        }

        match event::read()? {
            Event::Key(key) => {
                let AppKey::Action(action) = AppKey::from_crossterm(key) else {
                    continue;
                };

                match app.apply(action) {
                    ActionResult::Refresh => refresh_log(&mut app, &source),
                    ActionResult::SwitchHome => {
                        switch_log_command(&mut app, &mut source, JjLogCommand::ConfiguredDefault);
                    }
                    ActionResult::SwitchLog => {
                        switch_log_command(&mut app, &mut source, JjLogCommand::Log);
                    }
                    ActionResult::Quit => break,
                    _ => {}
                }
                needs_redraw = true;
            }
            Event::Resize(_, _) => {
                needs_redraw = true;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Reloads the current command without replacing the view on failure.
fn refresh_log(app: &mut LogView, source: &JjLog) {
    match source.load() {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Switches the command context only after the replacement log loads.
fn switch_log_command(app: &mut LogView, source: &mut JjLog, command: JjLogCommand) {
    let next_source = source.clone().with_command(command);
    match next_source.load() {
        Ok(snapshot) => {
            *source = next_source;
            app.refresh(snapshot);
        }
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Restores terminal mode even when the event loop returns through an error.
struct TerminalRestore;

impl Drop for TerminalRestore {
    fn drop(&mut self) {
        ratatui::restore();
    }
}
