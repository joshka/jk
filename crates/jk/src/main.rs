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
use jk_cli::{DiffFormat, DiffQuery, JjDiff, JjLog, JjLogCommand, JjShow, ShowQuery};
use jk_tui::diff_view::{DiffAction, DiffActionResult, DiffView};
use jk_tui::log_view::{ActionResult, LogView};
use jk_tui::rendered_view::{RenderedAction, RenderedActionResult, RenderedView};

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

    /// Show one or more revisions.
    Show(ShowArgs),
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
    /// Compatibility sugar for `jk diff -r REV`.
    #[arg(value_name = "REV", conflicts_with_all = ["revision", "from", "to"])]
    compatibility_revision: Option<String>,

    /// Revision to diff against its parent.
    #[arg(short = 'r', long = "revision", value_name = "REV", conflicts_with_all = ["from", "to"])]
    revision: Option<String>,

    /// Starting revision for a two-revision diff.
    #[arg(
        long,
        value_name = "FROM",
        requires = "to",
        conflicts_with = "revision"
    )]
    from: Option<String>,

    /// Ending revision for a two-revision diff.
    #[arg(
        long,
        value_name = "TO",
        requires = "from",
        conflicts_with = "revision"
    )]
    to: Option<String>,

    /// Render `jj diff --stat`.
    #[arg(long)]
    stat: bool,
}

/// Options for the explicit `jk show` command.
#[derive(Debug, Parser)]
struct ShowArgs {
    /// Revisions to show.
    #[arg(value_name = "REV", required = true)]
    revs: Vec<String>,
}

impl ShowArgs {
    fn query(&self) -> ShowQuery {
        ShowQuery::new(self.revs.clone())
    }
}

impl DiffArgs {
    fn query(&self) -> DiffQuery {
        let format = if self.stat {
            DiffFormat::Stat
        } else {
            DiffFormat::Patch
        };

        if let (Some(from), Some(to)) = (&self.from, &self.to) {
            return DiffQuery::FromTo {
                from: from.clone(),
                to: to.clone(),
                format,
            };
        }

        let rev = self
            .revision
            .as_ref()
            .or(self.compatibility_revision.as_ref())
            .cloned()
            .unwrap_or_else(|| "@".to_owned());
        DiffQuery::Revision { rev, format }
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let source = log_source(&args);
    let diff_source = diff_source(&args);
    let show_source = show_source(&args);
    let app = match &args.command {
        Some(Command::Diff(diff_args)) => {
            let query = diff_args.query();
            root_diff_view(&diff_source, query)
        }
        Some(Command::Show(show_args)) => {
            let query = show_args.query();
            root_show_view(&show_source, query)
        }
        Some(Command::Log(_)) | None => {
            let entries = source.load()?;
            AppView::Log(LogView::new(entries))
        }
    };

    run_terminal(app, source, &diff_source, &show_source)?;
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
        Some(Command::Diff(_) | Command::Show(_)) | None => {
            (JjLogCommand::ConfiguredDefault, args.limit)
        }
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

/// Builds the show source for selected-change inspection.
fn show_source(args: &Args) -> JjShow {
    let source = JjShow::default();
    if let Some(repository) = &args.repository {
        source.with_repository(repository)
    } else {
        source
    }
}

fn root_diff_view(diff_source: &JjDiff, query: DiffQuery) -> AppView {
    let snapshot = diff_source.load_query(&query);
    let diff = match snapshot {
        Ok(snapshot) => DiffView::new(snapshot),
        Err(error) => DiffView::from_error(
            query.target_label(),
            diff_source.spec_for(&query).title().to_owned(),
            error.to_string(),
        ),
    };
    AppView::Diff { view: diff, query }
}

fn root_show_view(show_source: &JjShow, query: ShowQuery) -> AppView {
    let snapshot = show_source.load_query(&query);
    let show = match snapshot {
        Ok(snapshot) => RenderedView::new(snapshot),
        Err(error) => RenderedView::from_error(
            query.target_label(),
            show_source.spec_for(&query).title().to_owned(),
            error.to_string(),
        ),
    };
    AppView::Show { view: show, query }
}

/// Active top-level application view.
#[derive(Clone, Debug, Eq, PartialEq)]
enum AppView {
    Log(LogView),
    Diff {
        view: DiffView,
        query: DiffQuery,
    },
    Show {
        view: RenderedView,
        query: ShowQuery,
    },
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
fn run_terminal(
    app: AppView,
    mut source: JjLog,
    diff_source: &JjDiff,
    show_source: &JjShow,
) -> Result<()> {
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
                AppView::Diff { view, .. } => match &mode {
                    Some(InputMode::DiffSearch { query }) => {
                        let status = format!("/{query}");
                        view.render_with_status(frame, &status);
                    }
                    _ => view.render(frame),
                },
                AppView::Show { view, .. } => match &mode {
                    Some(InputMode::InspectionSearch { query }) => {
                        let status = format!("/{query}");
                        view.render_with_status(frame, &status);
                    }
                    _ => view.render(frame),
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
                        AppKey::OpenShow => {
                            push_selected_show(&mut state, show_source);
                            needs_redraw = true;
                        }
                        AppKey::StartSearch
                            if matches!(
                                state.views.active(),
                                AppView::Diff { .. } | AppView::Show { .. }
                            ) =>
                        {
                            let mode = match state.views.active() {
                                AppView::Diff { .. } => InputMode::DiffSearch {
                                    query: String::new(),
                                },
                                AppView::Show { .. } => InputMode::InspectionSearch {
                                    query: String::new(),
                                },
                                AppView::Log(_) => unreachable!(),
                            };
                            state.modes.push(mode);
                            needs_redraw = true;
                        }
                        AppKey::SearchNext => {
                            apply_search_action(&mut state, SearchDirection::Next);
                            needs_redraw = true;
                        }
                        AppKey::SearchPrevious => {
                            apply_search_action(&mut state, SearchDirection::Previous);
                            needs_redraw = true;
                        }
                        _ => {}
                    }
                    continue;
                };

                if apply_action(&mut state, &mut source, diff_source, show_source, action)
                    == AppLoop::Quit
                {
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
    InspectionSearch { query: String },
}

/// Whether an input-mode handler consumed a key event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputModeResult {
    Handled,
    Unhandled,
}

/// Handles key input while a prompt-like mode is active.
fn handle_input_mode(state: &mut AppState, key: KeyEvent) -> InputModeResult {
    let Some(mode) = state.modes.active_mut() else {
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
            let action = match mode {
                InputMode::DiffSearch { query } => SearchSubmit::Diff(query.clone()),
                InputMode::InspectionSearch { query } => SearchSubmit::Inspection(query.clone()),
            };
            state.modes.pop();
            apply_search_submit(state, action);
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
            match mode {
                InputMode::DiffSearch { query } | InputMode::InspectionSearch { query } => {
                    query.push(character);
                }
            }
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

enum SearchSubmit {
    Diff(String),
    Inspection(String),
}

#[derive(Clone, Copy)]
enum SearchDirection {
    Next,
    Previous,
}

fn apply_search_submit(state: &mut AppState, action: SearchSubmit) {
    match (state.views.active_mut(), action) {
        (AppView::Diff { view, .. }, SearchSubmit::Diff(query)) => {
            let _ = view.apply(DiffAction::Search(query));
        }
        (AppView::Show { view, .. }, SearchSubmit::Inspection(query)) => {
            let _ = view.apply(RenderedAction::Search(query));
        }
        _ => {}
    }
}

/// Applies a search navigation action when the active view supports it.
fn apply_search_action(state: &mut AppState, direction: SearchDirection) {
    match state.views.active_mut() {
        AppView::Diff { view, .. } => {
            let action = match direction {
                SearchDirection::Next => DiffAction::SearchNext,
                SearchDirection::Previous => DiffAction::SearchPrevious,
            };
            let _ = view.apply(action);
        }
        AppView::Show { view, .. } => {
            let action = match direction {
                SearchDirection::Next => RenderedAction::SearchNext,
                SearchDirection::Previous => RenderedAction::SearchPrevious,
            };
            let _ = view.apply(action);
        }
        AppView::Log(_) => {}
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
    show_source: &JjShow,
    action: jk_tui::log_view::LogAction,
) -> AppLoop {
    let transition = match state.views.active_mut() {
        AppView::Log(log) => apply_log_action(log, source, diff_source, action),
        AppView::Diff { view, query } => apply_diff_action(view, query, diff_source, action),
        AppView::Show { view, query } => apply_show_action(view, query, show_source, action),
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

        let query = DiffQuery::Revision {
            rev: change_id.clone(),
            format: DiffFormat::Patch,
        };
        match diff_source.load_query(&query) {
            Ok(snapshot) => {
                let diff = DiffView::new(snapshot);
                return AppTransition::Push(AppView::Diff { view: diff, query });
            }
            Err(error) => log.show_error(error.to_string()),
        }
    }

    AppTransition::Continue
}

fn push_selected_show(state: &mut AppState, show_source: &JjShow) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    let Some(change_id) = log.selected_change_id().map(ToOwned::to_owned) else {
        return;
    };

    let query = ShowQuery::from(change_id);
    match show_source.load_query(&query) {
        Ok(snapshot) => {
            state.views.push(AppView::Show {
                view: RenderedView::new(snapshot),
                query,
            });
        }
        Err(error) => log.show_error(error.to_string()),
    }
}

/// Applies an action while the diff view is active.
fn apply_diff_action(
    diff: &mut DiffView,
    query: &DiffQuery,
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
        DiffActionResult::Refresh => refresh_diff(diff, query, diff_source),
        DiffActionResult::ReturnToLog => return AppTransition::PopView,
        DiffActionResult::Quit => return AppTransition::Quit,
        _ => {}
    }

    AppTransition::Continue
}

/// Applies an action while a rendered inspection view is active.
fn apply_show_action(
    view: &mut RenderedView,
    query: &ShowQuery,
    show_source: &JjShow,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let rendered_action = match action {
        jk_tui::log_view::LogAction::Previous => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::Next => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::PagePrevious => RenderedAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::First => RenderedAction::First,
        jk_tui::log_view::LogAction::Last => RenderedAction::Last,
        jk_tui::log_view::LogAction::ToggleHelp => RenderedAction::ToggleHelp,
        jk_tui::log_view::LogAction::Refresh => RenderedAction::Refresh,
        jk_tui::log_view::LogAction::Quit => RenderedAction::Quit,
        _ => RenderedAction::ReturnToLog,
    };

    match view.apply(rendered_action) {
        RenderedActionResult::Refresh => refresh_show(view, query, show_source),
        RenderedActionResult::ReturnToLog => return AppTransition::PopView,
        RenderedActionResult::Quit => return AppTransition::Quit,
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
fn refresh_diff(app: &mut DiffView, query: &DiffQuery, source: &JjDiff) {
    match source.load_query(query) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active show/details view without replacing it on failure.
fn refresh_show(app: &mut RenderedView, query: &ShowQuery, source: &JjShow) {
    match source.load_query(query) {
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
        let root = diff_app_view("aaa");
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

        stack.push(diff_app_view("bbb"));

        assert!(stack.pop());
        assert_eq!(stack.active(), &AppView::Log(previous_log));
    }

    #[test]
    fn mode_stack_closes_search_before_popping_view() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.views.push(diff_app_view("aaa"));
        state.modes.push(InputMode::DiffSearch {
            query: "alpha".to_owned(),
        });

        handle_back(&mut state);

        assert_eq!(state.modes.active(), None);
        assert!(matches!(state.views.active(), AppView::Diff { .. }));
        assert_eq!(state.views.views.len(), 2);
    }

    #[test]
    fn mode_stack_submit_search_restores_normal_mode() {
        let mut state = AppState::new(diff_app_view("aaa"));
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
        assert_eq!(
            state.views.active(),
            &AppView::Diff {
                view: expected,
                query: diff_query("aaa")
            }
        );
    }

    #[test]
    fn diff_args_default_to_revision_at() {
        let args = Args::try_parse_from(["jk", "diff"]).expect("valid diff args");
        let Some(Command::Diff(diff_args)) = args.command else {
            panic!("expected diff command");
        };

        assert_eq!(
            diff_args.query(),
            DiffQuery::Revision {
                rev: "@".to_owned(),
                format: DiffFormat::Patch,
            }
        );
    }

    #[test]
    fn diff_args_keep_positional_revision_as_sugar() {
        let args =
            Args::try_parse_from(["jk", "diff", "abc123", "--stat"]).expect("valid diff args");
        let Some(Command::Diff(diff_args)) = args.command else {
            panic!("expected diff command");
        };

        assert_eq!(
            diff_args.query(),
            DiffQuery::Revision {
                rev: "abc123".to_owned(),
                format: DiffFormat::Stat,
            }
        );
    }

    #[test]
    fn diff_args_resolve_explicit_revision_query() {
        let args = Args::try_parse_from(["jk", "diff", "-r", "abc123"]).expect("valid diff args");
        let Some(Command::Diff(diff_args)) = args.command else {
            panic!("expected diff command");
        };

        assert_eq!(
            diff_args.query(),
            DiffQuery::Revision {
                rev: "abc123".to_owned(),
                format: DiffFormat::Patch,
            }
        );
    }

    #[test]
    fn diff_args_resolve_from_to_query() {
        let args = Args::try_parse_from(["jk", "diff", "--from", "main", "--to", "@"])
            .expect("valid diff args");
        let Some(Command::Diff(diff_args)) = args.command else {
            panic!("expected diff command");
        };

        assert_eq!(
            diff_args.query(),
            DiffQuery::FromTo {
                from: "main".to_owned(),
                to: "@".to_owned(),
                format: DiffFormat::Patch,
            }
        );
    }

    #[test]
    fn diff_args_reject_mixed_revision_and_from_to() {
        assert!(
            Args::try_parse_from(["jk", "diff", "-r", "@", "--from", "main", "--to", "@"]).is_err()
        );
    }

    #[test]
    fn diff_args_require_from_and_to_together() {
        assert!(Args::try_parse_from(["jk", "diff", "--from", "main"]).is_err());
        assert!(Args::try_parse_from(["jk", "diff", "--to", "@"]).is_err());
    }

    fn diff_app_view(change_id: &str) -> AppView {
        AppView::Diff {
            view: diff_view(change_id),
            query: diff_query(change_id),
        }
    }

    fn diff_query(change_id: &str) -> DiffQuery {
        DiffQuery::Revision {
            rev: change_id.to_owned(),
            format: DiffFormat::Patch,
        }
    }

    fn diff_view(change_id: &str) -> DiffView {
        DiffView::from_error(
            change_id,
            format!("jj diff -r {change_id}"),
            "synthetic diff fixture".to_owned(),
        )
    }
}
