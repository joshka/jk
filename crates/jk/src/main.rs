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
use jk_cli::{JjDiff, JjLog, JjLogCommand};
use jk_tui::diff_view::{DiffAction, DiffActionResult, DiffView};
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

    /// Show the jj diff view for a revision.
    Diff(DiffArgs),
}

/// Options for the explicit `jk log` command.
#[derive(Debug, Parser)]
struct LogArgs {
    /// Maximum number of log entries to render.
    #[arg(short = 'n', long)]
    limit: Option<usize>,
}

/// Options for the explicit `jk diff` command.
#[derive(Debug, Parser)]
struct DiffArgs {
    /// Revision to diff against its parent.
    #[arg(default_value = "@")]
    revision: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let source = log_source(&args);
    let diff_source = diff_source(&args);
    let app = if let Some(Command::Diff(diff_args)) = &args.command {
        let snapshot = diff_source.load(&diff_args.revision)?;
        AppView::Diff {
            log: None,
            diff: DiffView::new(snapshot),
        }
    } else {
        let entries = source.load()?;
        AppView::Log(LogView::new(entries))
    };

    run_terminal(app, source, &diff_source)?;
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
        Some(Command::Diff(_)) | None => (JjLogCommand::ConfiguredDefault, args.limit),
    };

    let source = JjLog::default().with_command(command).with_limit(limit);
    if let Some(repository) = &args.repository {
        source.with_repository(repository)
    } else {
        source
    }
}

/// Builds the diff source for selected-change inspection.
fn diff_source(args: &Args) -> JjDiff {
    let source = JjDiff::default();
    if let Some(repository) = &args.repository {
        source.with_repository(repository)
    } else {
        source
    }
}

/// Active top-level application view.
#[derive(Debug)]
enum AppView {
    Log(LogView),
    Diff {
        log: Option<LogView>,
        diff: DiffView,
    },
}

/// Owns the terminal event loop for the current log-first application.
///
/// The view remains responsible for state transitions and rendering. This loop only translates
/// terminal events, performs I/O requested by the view, and redraws when input or terminal resize
/// events can change the screen.
fn run_terminal(mut app: AppView, mut source: JjLog, diff_source: &JjDiff) -> Result<()> {
    // jj should keep configured colors even when the parent process was run by an agent or tool
    // that exports NO_COLOR.
    force_color_output(true);
    let mut terminal = ratatui::try_init().inspect_err(|_| ratatui::restore())?;
    let _terminal_restore = TerminalRestore;
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            terminal.draw(|frame| match &mut app {
                AppView::Log(log) => log.render(frame),
                AppView::Diff { diff, .. } => diff.render(frame),
            })?;
            needs_redraw = false;
        }

        match event::read()? {
            Event::Key(key) => {
                let AppKey::Action(action) = AppKey::from_crossterm(key) else {
                    continue;
                };

                if apply_action(&mut app, &mut source, diff_source, action) == AppLoop::Quit {
                    break;
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

/// Applies an action to the active view and performs any requested I/O.
fn apply_action(
    app: &mut AppView,
    source: &mut JjLog,
    diff_source: &JjDiff,
    action: jk_tui::log_view::LogAction,
) -> AppLoop {
    let transition = match app {
        AppView::Log(log) => apply_log_action(log, source, diff_source, action),
        AppView::Diff { diff, .. } => apply_diff_action(diff, diff_source, action),
    };

    match transition {
        AppTransition::Continue => AppLoop::Continue,
        AppTransition::OpenDiff(diff) => {
            let previous = std::mem::replace(app, AppView::Log(LogView::default()));
            if let AppView::Log(log) = previous {
                *app = AppView::Diff {
                    log: Some(log),
                    diff,
                };
            }
            AppLoop::Continue
        }
        AppTransition::ReturnToLog => {
            let previous = std::mem::replace(app, AppView::Log(LogView::default()));
            if let AppView::Diff { log, diff } = previous {
                if let Some(log) = log {
                    *app = AppView::Log(log);
                } else {
                    *app = AppView::Diff { log: None, diff };
                }
            }
            AppLoop::Continue
        }
        AppTransition::Quit => AppLoop::Quit,
    }
}

/// Applies an action while the log view is active.
fn apply_log_action(
    log: &mut LogView,
    source: &mut JjLog,
    diff_source: &JjDiff,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    match log.apply(action) {
        ActionResult::Refresh => refresh_log(log, source),
        ActionResult::SwitchHome => {
            switch_log_command(log, source, JjLogCommand::ConfiguredDefault);
        }
        ActionResult::SwitchLog => switch_log_command(log, source, JjLogCommand::Log),
        ActionResult::Quit => return AppTransition::Quit,
        _ => {}
    }

    if action == jk_tui::log_view::LogAction::OpenDiff {
        let Some(change_id) = log.selected_change_id().map(ToOwned::to_owned) else {
            return AppTransition::Continue;
        };

        match diff_source.load(&change_id) {
            Ok(snapshot) => {
                let diff = DiffView::new(snapshot);
                return AppTransition::OpenDiff(diff);
            }
            Err(error) => log.show_error(error.to_string()),
        }
    }

    AppTransition::Continue
}

/// Applies an action while the diff view is active.
fn apply_diff_action(
    diff: &mut DiffView,
    diff_source: &JjDiff,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let diff_action = match action {
        jk_tui::log_view::LogAction::Previous => DiffAction::ScrollPrevious,
        jk_tui::log_view::LogAction::Next => DiffAction::ScrollNext,
        jk_tui::log_view::LogAction::PagePrevious => DiffAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext => DiffAction::PageNext,
        jk_tui::log_view::LogAction::First => DiffAction::First,
        jk_tui::log_view::LogAction::Last => DiffAction::Last,
        jk_tui::log_view::LogAction::PreviousFile => DiffAction::PreviousFile,
        jk_tui::log_view::LogAction::NextFile => DiffAction::NextFile,
        jk_tui::log_view::LogAction::ToggleExpanded => DiffAction::UnfoldFile,
        jk_tui::log_view::LogAction::CollapseExpanded => DiffAction::FoldFile,
        jk_tui::log_view::LogAction::FoldAll => DiffAction::FoldAll,
        jk_tui::log_view::LogAction::UnfoldAll => DiffAction::UnfoldAll,
        jk_tui::log_view::LogAction::Refresh => DiffAction::Refresh,
        jk_tui::log_view::LogAction::OpenDiff => DiffAction::Ignore,
        jk_tui::log_view::LogAction::Quit => DiffAction::Quit,
        _ => DiffAction::ReturnToLog,
    };

    match diff.apply(diff_action) {
        DiffActionResult::Refresh => refresh_diff(diff, diff_source),
        DiffActionResult::ReturnToLog => return AppTransition::ReturnToLog,
        DiffActionResult::Quit => return AppTransition::Quit,
        _ => {}
    }

    AppTransition::Continue
}

/// Whether the terminal event loop should continue running.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AppLoop {
    Continue,
    Quit,
}

/// View transition requested after an action is applied.
#[derive(Debug)]
enum AppTransition {
    Continue,
    OpenDiff(DiffView),
    ReturnToLog,
    Quit,
}

/// Reloads the current command without replacing the view on failure.
fn refresh_log(app: &mut LogView, source: &JjLog) {
    match source.load() {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active diff without replacing the view on failure.
fn refresh_diff(app: &mut DiffView, source: &JjDiff) {
    let change_id = app.change_id().to_owned();
    match source.load(&change_id) {
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
