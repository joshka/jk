//! Binary entry point for the `jk` terminal UI.
//!
//! This crate owns command-line parsing, terminal lifecycle, and the bridge between crossterm input
//! events and backend-neutral TUI actions. The product behavior is intentionally delegated to
//! `jk-cli` and `jk-tui` so the binary stays a thin orchestration layer.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
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
        let snapshot = diff_source.load(&diff_args.revision);
        let diff = match snapshot {
            Ok(snapshot) => DiffView::new(snapshot),
            Err(error) => DiffView::from_error(
                diff_args.revision.clone(),
                JjDiff::title(&diff_args.revision),
                error.to_string(),
            ),
        };
        AppView::Diff(diff)
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
#[derive(Clone, Debug, Eq, PartialEq)]
enum AppView {
    Log(LogView),
    Diff(DiffView),
}

/// Application state owned by the terminal loop.
#[derive(Debug)]
struct AppState {
    views: ViewStack,
    modes: ModeStack,
}

impl AppState {
    fn new(root: AppView) -> Self {
        Self {
            views: ViewStack::new(root),
            modes: ModeStack::default(),
        }
    }
}

/// Non-empty stack of top-level views.
#[derive(Debug)]
struct ViewStack {
    views: Vec<AppView>,
}

impl ViewStack {
    fn new(root: AppView) -> Self {
        Self { views: vec![root] }
    }

    fn active(&self) -> &AppView {
        self.views
            .last()
            .expect("view stack always keeps one root view")
    }

    fn active_mut(&mut self) -> &mut AppView {
        self.views
            .last_mut()
            .expect("view stack always keeps one root view")
    }

    fn push(&mut self, view: AppView) {
        self.views.push(view);
    }

    fn pop(&mut self) -> bool {
        if self.views.len() == 1 {
            return false;
        }

        self.views.pop();
        true
    }
}

/// Stack of transient prompt-like modes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ModeStack {
    modes: Vec<InputMode>,
}

impl ModeStack {
    fn active(&self) -> Option<&InputMode> {
        self.modes.last()
    }

    fn active_mut(&mut self) -> Option<&mut InputMode> {
        self.modes.last_mut()
    }

    fn push(&mut self, mode: InputMode) {
        self.modes.push(mode);
    }

    fn pop(&mut self) -> Option<InputMode> {
        self.modes.pop()
    }
}

/// Owns the terminal event loop for the current jj-native application.
///
/// The view remains responsible for state transitions and rendering. This loop only translates
/// terminal events, performs I/O requested by the view, and redraws when input or terminal resize
/// events can change the screen.
fn run_terminal(app: AppView, mut source: JjLog, diff_source: &JjDiff) -> Result<()> {
    // jj should keep configured colors even when the parent process was run by an agent or tool
    // that exports NO_COLOR.
    force_color_output(true);
    let mut terminal = ratatui::try_init().inspect_err(|_| ratatui::restore())?;
    let _terminal_restore = TerminalRestore;
    let mut needs_redraw = true;
    let mut state = AppState::new(app);

    loop {
        if needs_redraw {
            let mode = state.modes.active().cloned();
            terminal.draw(|frame| match state.views.active_mut() {
                AppView::Log(log) => log.render(frame),
                AppView::Diff(diff) => match &mode {
                    Some(InputMode::DiffSearch { query }) => {
                        let status = format!("/{query}");
                        diff.render_with_status(frame, &status);
                    }
                    None => diff.render(frame),
                },
            })?;
            needs_redraw = false;
        }

        match event::read()? {
            Event::Key(key) => {
                if handle_input_mode(&mut state, key) == InputModeResult::Handled {
                    needs_redraw = true;
                    continue;
                }

                let AppKey::Action(action) = AppKey::from_crossterm(key) else {
                    match AppKey::from_crossterm(key) {
                        AppKey::Back => {
                            handle_back(&mut state);
                            needs_redraw = true;
                        }
                        AppKey::StartSearch if matches!(state.views.active(), AppView::Diff(_)) => {
                            state.modes.push(InputMode::DiffSearch {
                                query: String::new(),
                            });
                            needs_redraw = true;
                        }
                        AppKey::SearchNext => {
                            apply_diff_search_action(&mut state, DiffAction::SearchNext);
                            needs_redraw = true;
                        }
                        AppKey::SearchPrevious => {
                            apply_diff_search_action(&mut state, DiffAction::SearchPrevious);
                            needs_redraw = true;
                        }
                        _ => {}
                    }
                    continue;
                };

                if apply_action(&mut state, &mut source, diff_source, action) == AppLoop::Quit {
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

/// Transient input modes owned by the terminal loop.
#[derive(Clone, Debug, Eq, PartialEq)]
enum InputMode {
    DiffSearch { query: String },
}

/// Whether an input-mode handler consumed a key event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputModeResult {
    Handled,
    Unhandled,
}

/// Handles key input while a prompt-like mode is active.
fn handle_input_mode(state: &mut AppState, key: KeyEvent) -> InputModeResult {
    let Some(InputMode::DiffSearch { query }) = state.modes.active_mut() else {
        return InputModeResult::Unhandled;
    };

    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            let query = query.clone();
            apply_diff_search_action(state, DiffAction::Search(query));
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers,
            ..
        } if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            query.push(character);
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

/// Applies a search-only diff action when the active view supports it.
fn apply_diff_search_action(state: &mut AppState, action: DiffAction) {
    if let AppView::Diff(diff) = state.views.active_mut() {
        let _ = diff.apply(action);
    }
}

/// Closes the active mode, or returns to the previous view when possible.
fn handle_back(state: &mut AppState) {
    if state.modes.pop().is_some() {
        return;
    }

    state.views.pop();
}

/// Applies an action to the active view and performs any requested I/O.
fn apply_action(
    state: &mut AppState,
    source: &mut JjLog,
    diff_source: &JjDiff,
    action: jk_tui::log_view::LogAction,
) -> AppLoop {
    let transition = match state.views.active_mut() {
        AppView::Log(log) => apply_log_action(log, source, diff_source, action),
        AppView::Diff(diff) => apply_diff_action(diff, diff_source, action),
    };

    match transition {
        AppTransition::Continue => AppLoop::Continue,
        AppTransition::Push(view) => {
            state.views.push(view);
            AppLoop::Continue
        }
        AppTransition::PopView => {
            state.views.pop();
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
                return AppTransition::Push(AppView::Diff(diff));
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
        jk_tui::log_view::LogAction::PreviousHunk => DiffAction::PreviousHunk,
        jk_tui::log_view::LogAction::NextHunk => DiffAction::NextHunk,
        jk_tui::log_view::LogAction::FoldHunk => DiffAction::FoldHunk,
        jk_tui::log_view::LogAction::UnfoldHunk => DiffAction::UnfoldHunk,
        jk_tui::log_view::LogAction::HorizontalPrevious => DiffAction::ScrollLeft,
        jk_tui::log_view::LogAction::HorizontalNext => DiffAction::ScrollRight,
        jk_tui::log_view::LogAction::ToggleHelp => DiffAction::ToggleHelp,
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
        DiffActionResult::ReturnToLog => return AppTransition::PopView,
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
    Push(AppView),
    PopView,
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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    #[test]
    fn view_stack_keeps_root_when_popped() {
        let root = AppView::Diff(diff_view("aaa"));
        let mut stack = ViewStack::new(root.clone());

        assert!(!stack.pop());
        assert_eq!(stack.active(), &root);
    }

    #[test]
    fn view_stack_returns_to_previous_log_state() {
        let mut log = LogView::default();
        log.show_error("preserved log state");
        let previous_log = log.clone();
        let mut stack = ViewStack::new(AppView::Log(log));

        stack.push(AppView::Diff(diff_view("bbb")));

        assert!(stack.pop());
        assert_eq!(stack.active(), &AppView::Log(previous_log));
    }

    #[test]
    fn mode_stack_closes_search_before_popping_view() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.views.push(AppView::Diff(diff_view("aaa")));
        state.modes.push(InputMode::DiffSearch {
            query: "alpha".to_owned(),
        });

        handle_back(&mut state);

        assert_eq!(state.modes.active(), None);
        assert!(matches!(state.views.active(), AppView::Diff(_)));
        assert_eq!(state.views.views.len(), 2);
    }

    #[test]
    fn mode_stack_submit_search_restores_normal_mode() {
        let mut state = AppState::new(AppView::Diff(diff_view("aaa")));
        state.modes.push(InputMode::DiffSearch {
            query: "alpha".to_owned(),
        });
        let mut expected = diff_view("aaa");
        let _ = expected.apply(DiffAction::Search("alpha".to_owned()));

        let result = handle_input_mode(
            &mut state,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
        assert_eq!(state.views.active(), &AppView::Diff(expected));
    }

    fn diff_view(change_id: &str) -> DiffView {
        DiffView::from_error(
            change_id,
            format!("jj diff -r {change_id}"),
            "synthetic diff fixture".to_owned(),
        )
    }
}
