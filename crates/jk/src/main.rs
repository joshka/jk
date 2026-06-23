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
use jk_cli::{
    DiffFormat, DiffQuery, EvologQuery, JjCommandRunner, JjDiff, JjEvolog, JjLog, JjLogCommand,
    JjShow, JjStatus, JjWorkspaces, LogTemplateSelection, RecordingJjCommandRunner, ShowQuery,
    StatusQuery, SystemJjCommandRunner, WorkspaceInspectionQuery, WorkspaceListSnapshot,
    WorkspaceSummary,
};
use jk_core::{CommandHistory, CommandSource, SourceAction, SourceView};
use jk_tui::command_discovery::{BindingContext, discovery_lines, filtered_discovery_len};
use jk_tui::command_history_view::{
    CommandHistoryAction, CommandHistoryActionResult, CommandHistorySnapshot, CommandHistoryView,
};
use jk_tui::diff_view::{DiffAction, DiffActionResult, DiffView};
use jk_tui::log_view::{ActionResult, LogAction, LogView};
use jk_tui::rendered_view::{RenderedAction, RenderedActionResult, RenderedView};
use jk_tui::workspaces_view::{
    WorkspaceViewRow, WorkspaceViewSnapshot, WorkspacesAction, WorkspacesActionResult,
    WorkspacesView,
};

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

    /// Show repository status.
    Status(StatusArgs),
}

/// Options for the explicit `jk log` command.
#[derive(Debug, Parser)]
struct LogArgs {
    /// Maximum number of log entries to render.
    #[arg(short = 'n', long)]
    limit: Option<usize>,

    /// Rendered jj log template to pass to the explicit log command.
    #[arg(short = 'T', long = "template", value_name = "TEMPLATE")]
    template: Option<String>,
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

/// Options for the explicit `jk status` command.
#[derive(Debug, Parser)]
struct StatusArgs {
    /// Filesets to pass to jj status.
    #[arg(value_name = "FILESET")]
    filesets: Vec<String>,
}

impl StatusArgs {
    fn query(&self) -> StatusQuery {
        StatusQuery::new(self.filesets.clone())
    }
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
    let evolog_source = evolog_source(&args);
    let show_source = show_source(&args);
    let status_source = status_source(&args);
    let workspaces_source = workspaces_source(&args);
    let mut history = CommandHistory::default();
    let app = match &args.command {
        Some(Command::Diff(diff_args)) => {
            let query = diff_args.query();
            root_diff_view(&diff_source, query, &mut history)
        }
        Some(Command::Show(show_args)) => {
            let query = show_args.query();
            root_show_view(&show_source, query, &mut history)
        }
        Some(Command::Status(status_args)) => {
            let query = status_args.query();
            root_status_view(&status_source, query, &mut history)
        }
        Some(Command::Log(_)) | None => {
            let mut runner = recording_runner(
                &mut history,
                CommandSource::new(SourceView::Log, SourceAction::InitialLoad),
            );
            let entries = source.load_with_runner(&mut runner)?;
            AppView::Log(LogView::new(entries))
        }
    };

    run_terminal(
        app,
        source,
        &diff_source,
        &evolog_source,
        &show_source,
        &status_source,
        &workspaces_source,
        history,
    )?;
    Ok(())
}

/// Builds the log source that matches the requested command-line view.
///
/// Bare `jk` intentionally starts from jj's configured default command, while `jk log` forces the
/// explicit log command. The top-level limit applies to both forms unless the subcommand provides a
/// narrower value.
fn log_source(args: &Args) -> JjLog {
    let (command, limit, template) = match &args.command {
        Some(Command::Log(log_args)) => (
            JjLogCommand::Log,
            log_args.limit.or(args.limit),
            log_args
                .template
                .clone()
                .map(LogTemplateSelection::Custom)
                .unwrap_or(LogTemplateSelection::Configured),
        ),
        Some(Command::Diff(_) | Command::Show(_) | Command::Status(_)) | None => (
            JjLogCommand::ConfiguredDefault,
            args.limit,
            LogTemplateSelection::Configured,
        ),
    };

    let source = JjLog::default()
        .with_command(command)
        .with_limit(limit)
        .with_template(template);
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

/// Builds the evolog source for selected-change inspection.
fn evolog_source(args: &Args) -> JjEvolog {
    let source = JjEvolog::default();
    if let Some(repository) = &args.repository {
        source.with_repository(repository)
    } else {
        source
    }
}

/// Builds the status source for repository inspection.
fn status_source(args: &Args) -> JjStatus {
    let source = JjStatus::default();
    if let Some(repository) = &args.repository {
        source.with_repository(repository)
    } else {
        source
    }
}

/// Builds the workspace source for workspace list and selected-workspace inspection.
fn workspaces_source(args: &Args) -> JjWorkspaces {
    let source = JjWorkspaces::default();
    if let Some(repository) = &args.repository {
        source.with_repository(repository)
    } else {
        source
    }
}

fn root_diff_view(diff_source: &JjDiff, query: DiffQuery, history: &mut CommandHistory) -> AppView {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Diff, SourceAction::InitialLoad),
    );
    let snapshot = diff_source.load_query_with_runner(&query, &mut runner);
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

fn root_show_view(show_source: &JjShow, query: ShowQuery, history: &mut CommandHistory) -> AppView {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Show, SourceAction::InitialLoad),
    );
    let snapshot = show_source.load_query_with_runner(&query, &mut runner);
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

fn root_status_view(
    status_source: &JjStatus,
    query: StatusQuery,
    history: &mut CommandHistory,
) -> AppView {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Status, SourceAction::InitialLoad),
    );
    let snapshot = status_source.load_query_with_runner(&query, &mut runner);
    let status = match snapshot {
        Ok(snapshot) => RenderedView::new(snapshot),
        Err(error) => RenderedView::from_error(
            query.target_label(),
            status_source.spec_for(&query).title().to_owned(),
            error.to_string(),
        ),
    };
    AppView::Status {
        view: status,
        query,
    }
}

fn workspace_view_snapshot(snapshot: WorkspaceListSnapshot) -> WorkspaceViewSnapshot {
    let rows = snapshot
        .workspaces
        .into_iter()
        .map(workspace_view_row)
        .collect();
    WorkspaceViewSnapshot::new(rows).with_title(snapshot.title)
}

fn workspace_view_row(workspace: WorkspaceSummary) -> WorkspaceViewRow {
    let root_display = workspace
        .root
        .as_ref()
        .map(|root| root.display().to_string())
        .unwrap_or_else(|| "(no root)".to_owned());
    let mut row = WorkspaceViewRow::new(workspace.name, root_display, workspace.current);
    if let Some(root) = workspace.root {
        row = row.with_root(root);
    }
    if let Some(change_id) = workspace.change_id {
        row = row.with_change_id(change_id);
    }
    if let Some(commit_id) = workspace.commit_id {
        row = row.with_commit_id(commit_id);
    }
    row
}

fn recording_runner(
    history: &mut CommandHistory,
    source: CommandSource,
) -> RecordingJjCommandRunner<'_, SystemJjCommandRunner> {
    RecordingJjCommandRunner::new(SystemJjCommandRunner, history, source)
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
    Evolog {
        view: RenderedView,
        query: EvologQuery,
    },
    Status {
        view: RenderedView,
        query: StatusQuery,
    },
    Workspaces {
        view: WorkspacesView,
    },
    CommandHistory {
        view: CommandHistoryView,
    },
    WorkspaceStatus {
        view: RenderedView,
        query: WorkspaceInspectionQuery,
    },
    WorkspaceDiff {
        view: RenderedView,
        query: WorkspaceInspectionQuery,
    },
}

/// Application state owned by the terminal loop.
#[derive(Debug)]
struct AppState {
    views: ViewStack,
    modes: ModeStack,
    history: CommandHistory,
}

impl AppState {
    #[cfg(test)]
    fn new(root: AppView) -> Self {
        Self::with_history(root, CommandHistory::default())
    }

    fn with_history(root: AppView, history: CommandHistory) -> Self {
        Self {
            views: ViewStack::new(root),
            modes: ModeStack::default(),
            history,
        }
    }

    #[cfg(test)]
    fn command_history(&self) -> &CommandHistory {
        &self.history
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
    evolog_source: &JjEvolog,
    show_source: &JjShow,
    status_source: &JjStatus,
    workspaces_source: &JjWorkspaces,
    history: CommandHistory,
) -> Result<()> {
    // jj should keep configured colors even when the parent process was run by an agent or tool
    // that exports NO_COLOR.
    force_color_output(true);
    let mut terminal = ratatui::try_init().inspect_err(|_| ratatui::restore())?;
    let _terminal_restore = TerminalRestore;
    let mut needs_redraw = true;
    let mut state = AppState::with_history(app, history);

    loop {
        if needs_redraw {
            let mode = state.modes.active().cloned();
            terminal.draw(|frame| match state.views.active_mut() {
                AppView::Log(log) => match &mode {
                    Some(InputMode::ViewOptions { context, selected }) => {
                        let lines = view_options_lines(*context, *selected, source.template());
                        log.render_with_selector(frame, "View Options", &lines);
                    }
                    Some(InputMode::LogTemplate { options, selected }) => {
                        let lines = template_selector_lines(options, *selected);
                        log.render_with_selector(frame, "Log template", &lines);
                    }
                    Some(InputMode::CommandDiscovery {
                        context,
                        query,
                        selected,
                    }) => {
                        let lines = discovery_lines(*context, query, *selected);
                        log.render_with_selector(frame, "Command discovery", &lines);
                    }
                    _ => log.render(frame),
                },
                AppView::Diff { view, .. } => match &mode {
                    Some(InputMode::ViewOptions { context, selected }) => {
                        let lines = view_options_lines(*context, *selected, source.template());
                        view.render_with_overlay(frame, "View Options", &lines);
                    }
                    Some(InputMode::DiffSearch { query }) => {
                        let status = format!("/{query}");
                        view.render_with_status(frame, &status);
                    }
                    Some(InputMode::CommandDiscovery {
                        context,
                        query,
                        selected,
                    }) => {
                        let lines = discovery_lines(*context, query, *selected);
                        view.render_with_overlay(frame, "Command discovery", &lines);
                    }
                    _ => view.render(frame),
                },
                AppView::Show { view, .. } => match &mode {
                    Some(InputMode::ViewOptions { context, selected }) => {
                        let lines = view_options_lines(*context, *selected, source.template());
                        view.render_with_overlay(frame, "View Options", &lines);
                    }
                    Some(InputMode::InspectionSearch { query }) => {
                        let status = format!("/{query}");
                        view.render_with_status(frame, &status);
                    }
                    Some(InputMode::CommandDiscovery {
                        context,
                        query,
                        selected,
                    }) => {
                        let lines = discovery_lines(*context, query, *selected);
                        view.render_with_overlay(frame, "Command discovery", &lines);
                    }
                    _ => view.render(frame),
                },
                AppView::Evolog { view, .. } => match &mode {
                    Some(InputMode::ViewOptions { context, selected }) => {
                        let lines = view_options_lines(*context, *selected, source.template());
                        view.render_with_overlay(frame, "View Options", &lines);
                    }
                    Some(InputMode::InspectionSearch { query }) => {
                        let status = format!("/{query}");
                        view.render_with_status(frame, &status);
                    }
                    Some(InputMode::CommandDiscovery {
                        context,
                        query,
                        selected,
                    }) => {
                        let lines = discovery_lines(*context, query, *selected);
                        view.render_with_overlay(frame, "Command discovery", &lines);
                    }
                    _ => view.render(frame),
                },
                AppView::Status { view, .. } => match &mode {
                    Some(InputMode::ViewOptions { context, selected }) => {
                        let lines = view_options_lines(*context, *selected, source.template());
                        view.render_with_overlay(frame, "View Options", &lines);
                    }
                    Some(InputMode::InspectionSearch { query }) => {
                        let status = format!("/{query}");
                        view.render_with_status(frame, &status);
                    }
                    Some(InputMode::CommandDiscovery {
                        context,
                        query,
                        selected,
                    }) => {
                        let lines = discovery_lines(*context, query, *selected);
                        view.render_with_overlay(frame, "Command discovery", &lines);
                    }
                    _ => view.render(frame),
                },
                AppView::Workspaces { view } => match &mode {
                    Some(InputMode::ViewOptions { context, selected }) => {
                        let lines = view_options_lines(*context, *selected, source.template());
                        view.render(frame);
                        render_mode_overlay(frame, "View Options", &lines);
                    }
                    Some(InputMode::CommandDiscovery {
                        context,
                        query,
                        selected,
                    }) => {
                        let lines = discovery_lines(*context, query, *selected);
                        view.render(frame);
                        render_mode_overlay(frame, "Command discovery", &lines);
                    }
                    _ => view.render(frame),
                },
                AppView::CommandHistory { view } => match &mode {
                    Some(InputMode::CommandDiscovery {
                        context,
                        query,
                        selected,
                    }) => {
                        let lines = discovery_lines(*context, query, *selected);
                        view.render(frame);
                        render_mode_overlay(frame, "Command discovery", &lines);
                    }
                    _ => view.render(frame),
                },
                AppView::WorkspaceStatus { view, .. } | AppView::WorkspaceDiff { view, .. } => {
                    match &mode {
                        Some(InputMode::ViewOptions { context, selected }) => {
                            let lines = view_options_lines(*context, *selected, source.template());
                            view.render_with_overlay(frame, "View Options", &lines);
                        }
                        Some(InputMode::InspectionSearch { query }) => {
                            let status = format!("/{query}");
                            view.render_with_status(frame, &status);
                        }
                        Some(InputMode::CommandDiscovery {
                            context,
                            query,
                            selected,
                        }) => {
                            let lines = discovery_lines(*context, query, *selected);
                            view.render_with_overlay(frame, "Command discovery", &lines);
                        }
                        _ => view.render(frame),
                    }
                }
            })?;
            needs_redraw = false;
        }

        match event::read()? {
            Event::Key(key) => {
                if handle_input_mode(&mut state, &mut source, key) == InputModeResult::Handled {
                    needs_redraw = true;
                    continue;
                }

                let app_key = AppKey::from_crossterm(key);
                if matches!(
                    state.views.active(),
                    AppView::Workspaces { .. } | AppView::CommandHistory { .. }
                ) && matches!(key.code, KeyCode::Esc)
                {
                    handle_back(&mut state);
                    needs_redraw = true;
                    continue;
                }
                let AppKey::Action(action) = app_key else {
                    match app_key {
                        AppKey::Back => {
                            handle_back(&mut state);
                            needs_redraw = true;
                        }
                        AppKey::OpenShow => {
                            if matches!(state.views.active(), AppView::Workspaces { .. }) {
                                push_selected_workspace_status(&mut state, workspaces_source);
                            } else {
                                push_selected_show(&mut state, show_source);
                            }
                            needs_redraw = true;
                        }
                        AppKey::OpenEvolog => {
                            push_selected_evolog(&mut state, evolog_source);
                            needs_redraw = true;
                        }
                        AppKey::OpenStatus => {
                            if matches!(state.views.active(), AppView::Workspaces { .. }) {
                                push_selected_workspace_status(&mut state, workspaces_source);
                            } else {
                                push_status(&mut state, status_source);
                            }
                            needs_redraw = true;
                        }
                        AppKey::OpenWorkspaces => {
                            open_workspaces(&mut state, workspaces_source);
                            needs_redraw = true;
                        }
                        AppKey::OpenCommandHistory => {
                            open_command_history(&mut state);
                            needs_redraw = true;
                        }
                        AppKey::UpdateSelectedWorkspaceStale => {
                            update_selected_workspace_stale(&mut state, workspaces_source);
                            needs_redraw = true;
                        }
                        AppKey::OpenViewOptions => {
                            if !matches!(state.views.active(), AppView::CommandHistory { .. }) {
                                open_view_options(&mut state);
                                needs_redraw = true;
                            }
                        }
                        AppKey::StartSearch
                            if matches!(
                                state.views.active(),
                                AppView::Diff { .. }
                                    | AppView::Show { .. }
                                    | AppView::Evolog { .. }
                                    | AppView::Status { .. }
                                    | AppView::WorkspaceStatus { .. }
                                    | AppView::WorkspaceDiff { .. }
                            ) =>
                        {
                            let mode = match state.views.active() {
                                AppView::Diff { .. } => InputMode::DiffSearch {
                                    query: String::new(),
                                },
                                AppView::Show { .. } => InputMode::InspectionSearch {
                                    query: String::new(),
                                },
                                AppView::Evolog { .. } => InputMode::InspectionSearch {
                                    query: String::new(),
                                },
                                AppView::Status { .. } => InputMode::InspectionSearch {
                                    query: String::new(),
                                },
                                AppView::WorkspaceStatus { .. } => InputMode::InspectionSearch {
                                    query: String::new(),
                                },
                                AppView::WorkspaceDiff { .. } => InputMode::InspectionSearch {
                                    query: String::new(),
                                },
                                AppView::Log(_) | AppView::Workspaces { .. } => unreachable!(),
                                AppView::CommandHistory { .. } => unreachable!(),
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

                if matches!(app_key, AppKey::Action(LogAction::ToggleHelp)) {
                    open_command_discovery(&mut state);
                    needs_redraw = true;
                    continue;
                }

                if apply_action(
                    &mut state,
                    &mut source,
                    diff_source,
                    evolog_source,
                    show_source,
                    status_source,
                    workspaces_source,
                    action,
                ) == AppLoop::Quit
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
    ViewOptions {
        context: BindingContext,
        selected: usize,
    },
    DiffSearch {
        query: String,
    },
    InspectionSearch {
        query: String,
    },
    CommandDiscovery {
        context: BindingContext,
        query: String,
        selected: usize,
    },
    LogTemplate {
        options: Vec<LogTemplateSelection>,
        selected: usize,
    },
}

/// Whether an input-mode handler consumed a key event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InputModeResult {
    Handled,
    Unhandled,
}

/// Handles key input while a prompt-like mode is active.
fn handle_input_mode(state: &mut AppState, source: &mut JjLog, key: KeyEvent) -> InputModeResult {
    if matches!(state.modes.active(), Some(InputMode::ViewOptions { .. })) {
        return handle_view_options_mode(state, source, key);
    }
    if matches!(state.modes.active(), Some(InputMode::LogTemplate { .. })) {
        return handle_template_mode(state, source, key);
    }
    if matches!(
        state.modes.active(),
        Some(InputMode::CommandDiscovery { .. })
    ) {
        return handle_command_discovery_mode(state, key);
    }

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
                InputMode::ViewOptions { .. } => unreachable!(),
                InputMode::CommandDiscovery { .. } => unreachable!(),
                InputMode::LogTemplate { .. } => unreachable!(),
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
                InputMode::ViewOptions { .. } => unreachable!(),
                InputMode::CommandDiscovery { .. } => unreachable!(),
                InputMode::LogTemplate { .. } => unreachable!(),
            }
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

fn handle_command_discovery_mode(state: &mut AppState, key: KeyEvent) -> InputModeResult {
    match key {
        KeyEvent {
            code: KeyCode::Esc | KeyCode::Enter,
            ..
        } => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Char('q' | '?'),
            modifiers,
            ..
        } if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            let should_close = match state.modes.active_mut() {
                Some(InputMode::CommandDiscovery {
                    query, selected, ..
                }) if query.is_empty() => {
                    *selected = 0;
                    true
                }
                Some(InputMode::CommandDiscovery {
                    context,
                    query,
                    selected,
                }) => {
                    query.pop();
                    clamp_command_discovery_selection(*context, query, selected);
                    false
                }
                _ => false,
            };
            if should_close {
                state.modes.pop();
            }
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Up, ..
        }
        | KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            move_command_discovery_selection(state, DiscoveryMove::Previous);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            move_command_discovery_selection(state, DiscoveryMove::Next);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers,
            ..
        } if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            if let Some(InputMode::CommandDiscovery {
                context,
                query,
                selected,
            }) = state.modes.active_mut()
            {
                query.push(character);
                clamp_command_discovery_selection(*context, query, selected);
            }
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

fn handle_view_options_mode(
    state: &mut AppState,
    source: &JjLog,
    key: KeyEvent,
) -> InputModeResult {
    match key {
        KeyEvent {
            code: KeyCode::Esc | KeyCode::Backspace,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Up, ..
        }
        | KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            move_view_options_selection(state, ViewOptionsMove::Previous);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            move_view_options_selection(state, ViewOptionsMove::Next);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            let selected = selected_view_option(state);
            state.modes.pop();
            if selected == Some(ViewOptionRow::LogTemplate) {
                open_template_selector(&mut state.modes, source);
            }
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

fn open_command_discovery(state: &mut AppState) {
    let context = active_binding_context(state);
    state.modes.push(InputMode::CommandDiscovery {
        context,
        query: String::new(),
        selected: 0,
    });
}

fn open_view_options(state: &mut AppState) {
    if matches!(state.views.active(), AppView::CommandHistory { .. }) {
        return;
    }

    state.modes.push(InputMode::ViewOptions {
        context: active_binding_context(state),
        selected: 0,
    });
}

fn active_binding_context(state: &AppState) -> BindingContext {
    match state.views.active() {
        AppView::Log(_) => BindingContext::Log,
        AppView::Diff { .. } => BindingContext::Diff,
        AppView::Show { .. }
        | AppView::Evolog { .. }
        | AppView::Status { .. }
        | AppView::WorkspaceStatus { .. }
        | AppView::WorkspaceDiff { .. } => BindingContext::Inspection,
        AppView::Workspaces { .. } => BindingContext::Workspaces,
        AppView::CommandHistory { .. } => BindingContext::CommandHistory,
    }
}

fn render_mode_overlay(frame: &mut ratatui::Frame<'_>, title: &str, lines: &[String]) {
    use ratatui::layout::Rect;
    use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
    use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

    let area = frame.area();
    if area.is_empty() {
        return;
    }

    let content = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(2),
    };
    let width = 72_u16.min(content.width);
    let height = u16::try_from(lines.len().saturating_add(4))
        .unwrap_or(u16::MAX)
        .min(content.height);
    let overlay = Rect {
        x: content.x + content.width.saturating_sub(width) / 2,
        y: content.y + content.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, overlay);

    let text = Text::from(
        std::iter::once(Line::from(Span::styled(
            title,
            Style::new().add_modifier(Modifier::BOLD),
        )))
        .chain(std::iter::once(Line::from("")))
        .chain(lines.iter().map(|line| Line::from(line.as_str())))
        .collect::<Vec<_>>(),
    );
    let paragraph = Paragraph::new(text)
        .block(Block::bordered())
        .style(Style::new().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, overlay);
}

#[derive(Clone, Copy)]
enum DiscoveryMove {
    Previous,
    Next,
}

fn move_command_discovery_selection(state: &mut AppState, direction: DiscoveryMove) {
    let Some(InputMode::CommandDiscovery {
        context,
        query,
        selected,
    }) = state.modes.active_mut()
    else {
        return;
    };
    let row_count = filtered_discovery_len(*context, query);
    if row_count == 0 {
        *selected = 0;
        return;
    }

    match direction {
        DiscoveryMove::Previous => *selected = selected.saturating_sub(1),
        DiscoveryMove::Next => *selected = (*selected + 1).min(row_count - 1),
    }
}

fn clamp_command_discovery_selection(context: BindingContext, query: &str, selected: &mut usize) {
    let row_count = filtered_discovery_len(context, query);
    if row_count == 0 {
        *selected = 0;
    } else {
        *selected = (*selected).min(row_count - 1);
    }
}

#[derive(Clone, Copy)]
enum ViewOptionsMove {
    Previous,
    Next,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ViewOptionRow {
    LogTemplate,
    Placeholder,
}

fn move_view_options_selection(state: &mut AppState, direction: ViewOptionsMove) {
    let Some(InputMode::ViewOptions { context, selected }) = state.modes.active_mut() else {
        return;
    };
    let row_count = view_option_rows(*context).len();
    if row_count == 0 {
        *selected = 0;
        return;
    }

    match direction {
        ViewOptionsMove::Previous => *selected = selected.saturating_sub(1),
        ViewOptionsMove::Next => *selected = (*selected + 1).min(row_count - 1),
    }
}

fn selected_view_option(state: &AppState) -> Option<ViewOptionRow> {
    let Some(InputMode::ViewOptions { context, selected }) = state.modes.active() else {
        return None;
    };
    view_option_rows(*context).get(*selected).copied()
}

fn view_option_rows(context: BindingContext) -> &'static [ViewOptionRow] {
    match context {
        BindingContext::Log => &[ViewOptionRow::LogTemplate],
        BindingContext::Diff
        | BindingContext::Inspection
        | BindingContext::Workspaces
        | BindingContext::CommandHistory => &[ViewOptionRow::Placeholder],
    }
}

fn view_options_lines(
    context: BindingContext,
    selected: usize,
    template: &LogTemplateSelection,
) -> Vec<String> {
    match context {
        BindingContext::Log => {
            let marker = if selected == 0 { ">" } else { " " };
            vec![
                format!("{marker} {:<18} {}", "Template", template.label()),
                String::new(),
                "j/k or arrows move   enter open   esc close".to_owned(),
            ]
        }
        BindingContext::Diff | BindingContext::Inspection => vec![
            "No view options in this slice.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ],
        BindingContext::Workspaces => vec![
            "No workspace view options in this slice.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ],
        BindingContext::CommandHistory => vec![
            "No command history options in this slice.".to_owned(),
            String::new(),
            "esc close".to_owned(),
        ],
    }
}

fn handle_template_mode(
    state: &mut AppState,
    source: &mut JjLog,
    key: KeyEvent,
) -> InputModeResult {
    match key {
        KeyEvent {
            code: KeyCode::Esc | KeyCode::Backspace,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('q'),
            ..
        } => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Up, ..
        }
        | KeyEvent {
            code: KeyCode::Char('k'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            move_template_selection(state, TemplateMove::Previous);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Down,
            ..
        }
        | KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            move_template_selection(state, TemplateMove::Next);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            let template = selected_template(state);
            state.modes.pop();
            if let Some(template) = template {
                apply_log_template_selection(state, source, template);
            }
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

#[derive(Clone, Copy)]
enum TemplateMove {
    Previous,
    Next,
}

fn move_template_selection(state: &mut AppState, direction: TemplateMove) {
    let Some(InputMode::LogTemplate { options, selected }) = state.modes.active_mut() else {
        return;
    };
    if options.is_empty() {
        *selected = 0;
        return;
    }

    match direction {
        TemplateMove::Previous => *selected = selected.saturating_sub(1),
        TemplateMove::Next => *selected = (*selected + 1).min(options.len() - 1),
    }
}

fn selected_template(state: &AppState) -> Option<LogTemplateSelection> {
    let Some(InputMode::LogTemplate { options, selected }) = state.modes.active() else {
        return None;
    };
    options.get(*selected).cloned()
}

fn template_selector_lines(options: &[LogTemplateSelection], selected: usize) -> Vec<String> {
    options
        .iter()
        .enumerate()
        .map(|(index, template)| {
            let marker = if index == selected { ">" } else { " " };
            let name = template.template_name().unwrap_or("jj configured template");
            format!("{marker} {:<18} {name}", template.label())
        })
        .chain(std::iter::once(String::new()))
        .chain(std::iter::once(
            "j/k or arrows move   enter apply   esc cancel".to_owned(),
        ))
        .collect()
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
        (AppView::Evolog { view, .. }, SearchSubmit::Inspection(query)) => {
            let _ = view.apply(RenderedAction::Search(query));
        }
        (AppView::Status { view, .. }, SearchSubmit::Inspection(query)) => {
            let _ = view.apply(RenderedAction::Search(query));
        }
        (AppView::WorkspaceStatus { view, .. }, SearchSubmit::Inspection(query))
        | (AppView::WorkspaceDiff { view, .. }, SearchSubmit::Inspection(query)) => {
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
        AppView::Evolog { view, .. } => {
            let action = match direction {
                SearchDirection::Next => RenderedAction::SearchNext,
                SearchDirection::Previous => RenderedAction::SearchPrevious,
            };
            let _ = view.apply(action);
        }
        AppView::Status { view, .. } => {
            let action = match direction {
                SearchDirection::Next => RenderedAction::SearchNext,
                SearchDirection::Previous => RenderedAction::SearchPrevious,
            };
            let _ = view.apply(action);
        }
        AppView::WorkspaceStatus { view, .. } | AppView::WorkspaceDiff { view, .. } => {
            let action = match direction {
                SearchDirection::Next => RenderedAction::SearchNext,
                SearchDirection::Previous => RenderedAction::SearchPrevious,
            };
            let _ = view.apply(action);
        }
        AppView::Log(_) | AppView::Workspaces { .. } | AppView::CommandHistory { .. } => {}
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
    evolog_source: &JjEvolog,
    show_source: &JjShow,
    status_source: &JjStatus,
    workspaces_source: &JjWorkspaces,
    action: jk_tui::log_view::LogAction,
) -> AppLoop {
    let transition = {
        let AppState { views, history, .. } = state;
        match views.active_mut() {
            AppView::Log(log) => apply_log_action(log, history, source, diff_source, action),
            AppView::Diff { view, query } => {
                apply_diff_action(view, query, history, diff_source, action)
            }
            AppView::Show { view, query } => {
                apply_show_action(view, query, history, show_source, action)
            }
            AppView::Evolog { view, query } => {
                apply_evolog_action(view, query, history, evolog_source, action)
            }
            AppView::Status { view, query } => {
                apply_status_action(view, query, history, status_source, action)
            }
            AppView::Workspaces { view } => {
                apply_workspaces_action(view, history, workspaces_source, action)
            }
            AppView::CommandHistory { view } => apply_command_history_action(view, history, action),
            AppView::WorkspaceStatus { view, query } => apply_workspace_inspection_action(
                view,
                query,
                history,
                workspaces_source,
                WorkspaceInspectionKind::Status,
                action,
            ),
            AppView::WorkspaceDiff { view, query } => apply_workspace_inspection_action(
                view,
                query,
                history,
                workspaces_source,
                WorkspaceInspectionKind::Diff,
                action,
            ),
        }
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
        AppTransition::PushSelectedWorkspaceStatus => {
            push_selected_workspace_status(state, workspaces_source);
            AppLoop::Continue
        }
        AppTransition::PushSelectedWorkspaceDiff => {
            push_selected_workspace_diff(state, workspaces_source);
            AppLoop::Continue
        }
        AppTransition::OpenViewOptions => {
            open_view_options(state);
            AppLoop::Continue
        }
        AppTransition::Quit => AppLoop::Quit,
    }
}

/// Applies an action while the log view is active.
fn apply_log_action(
    log: &mut LogView,
    history: &mut CommandHistory,
    source: &mut JjLog,
    diff_source: &JjDiff,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    match log.apply(action) {
        ActionResult::Refresh => refresh_log(
            log,
            history,
            source,
            CommandSource::new(SourceView::Log, SourceAction::Refresh),
        ),
        ActionResult::SwitchHome => {
            switch_log_command(log, history, source, JjLogCommand::ConfiguredDefault);
        }
        ActionResult::SwitchLog => switch_log_command(log, history, source, JjLogCommand::Log),
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
        let mut runner = recording_runner(
            history,
            CommandSource::new(SourceView::Log, SourceAction::OpenDiff),
        );
        match diff_source.load_query_with_runner(&query, &mut runner) {
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
    let change_id = {
        let AppView::Log(log) = state.views.active_mut() else {
            return;
        };
        let Some(change_id) = log.selected_change_id().map(ToOwned::to_owned) else {
            return;
        };
        change_id
    };

    let query = ShowQuery::from(change_id);
    let mut runner = recording_runner(
        &mut state.history,
        CommandSource::new(SourceView::Log, SourceAction::OpenShow),
    );
    match show_source.load_query_with_runner(&query, &mut runner) {
        Ok(snapshot) => {
            state.views.push(AppView::Show {
                view: RenderedView::new(snapshot),
                query,
            });
        }
        Err(error) => {
            if let AppView::Log(log) = state.views.active_mut() {
                log.show_error(error.to_string());
            }
        }
    }
}

fn push_selected_evolog(state: &mut AppState, evolog_source: &JjEvolog) {
    let change_id = {
        let AppView::Log(log) = state.views.active_mut() else {
            return;
        };
        let Some(change_id) = log.selected_change_id().map(ToOwned::to_owned) else {
            return;
        };
        change_id
    };

    let query = EvologQuery::from(change_id);
    let mut runner = recording_runner(
        &mut state.history,
        CommandSource::new(SourceView::Log, SourceAction::OpenEvolog),
    );
    match evolog_source.load_query_with_runner(&query, &mut runner) {
        Ok(snapshot) => push_evolog_view(state, query, RenderedView::new(snapshot)),
        Err(error) => {
            if let AppView::Log(log) = state.views.active_mut() {
                log.show_error(error.to_string());
            }
        }
    }
}

fn push_evolog_view(state: &mut AppState, query: EvologQuery, view: RenderedView) {
    if !matches!(state.views.active(), AppView::Log(_)) {
        return;
    }

    state.views.push(AppView::Evolog { view, query });
}

fn push_status(state: &mut AppState, status_source: &JjStatus) {
    push_status_with_runner(state, status_source, SystemJjCommandRunner);
}

fn open_workspaces(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    open_workspaces_with_runner(state, workspaces_source, SystemJjCommandRunner);
}

fn open_command_history(state: &mut AppState) {
    let snapshot = command_history_snapshot(&state.history);
    if let AppView::CommandHistory { view } = state.views.active_mut() {
        view.refresh(snapshot);
        return;
    }

    let view = CommandHistoryView::new(snapshot);
    state.views.push(AppView::CommandHistory { view });
}

fn command_history_snapshot(history: &CommandHistory) -> CommandHistorySnapshot {
    CommandHistorySnapshot::from_records(history.records())
}

fn push_status_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    status_source: &JjStatus,
    runner: R,
) {
    if !matches!(state.views.active(), AppView::Log(_)) {
        return;
    }

    let query = StatusQuery::default();
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        &mut state.history,
        CommandSource::new(SourceView::Log, SourceAction::OpenStatus),
    );
    match status_source.load_query_with_runner(&query, &mut runner) {
        Ok(snapshot) => {
            state.views.push(AppView::Status {
                view: RenderedView::new(snapshot),
                query,
            });
        }
        Err(error) => {
            if let AppView::Log(log) = state.views.active_mut() {
                log.show_error(error.to_string());
            }
        }
    }
}

fn push_workspaces_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    workspaces_source: &JjWorkspaces,
    source: CommandSource,
    runner: R,
) {
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, source);
    let view = match workspaces_source.load_list_with_runner(&mut runner) {
        Ok(snapshot) => WorkspacesView::new(workspace_view_snapshot(snapshot)),
        Err(error) => {
            let mut view = WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new()));
            view.show_error(error.to_string());
            view
        }
    };
    push_workspace_view(state, view);
}

fn open_workspaces_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    workspaces_source: &JjWorkspaces,
    runner: R,
) {
    if matches!(state.views.active(), AppView::Workspaces { .. }) {
        let AppState { views, history, .. } = state;
        if let AppView::Workspaces { view } = views.active_mut() {
            refresh_workspaces_with_runner(view, history, workspaces_source, runner);
        }
        return;
    }

    push_workspaces_with_runner(
        state,
        workspaces_source,
        CommandSource::new(SourceView::Log, SourceAction::WorkspaceList),
        runner,
    );
}

fn push_workspace_view(state: &mut AppState, view: WorkspacesView) {
    state.views.push(AppView::Workspaces { view });
}

fn push_selected_workspace_status(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    push_selected_workspace_inspection(state, workspaces_source, WorkspaceInspectionKind::Status);
}

fn push_selected_workspace_diff(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    push_selected_workspace_inspection(state, workspaces_source, WorkspaceInspectionKind::Diff);
}

fn push_selected_workspace_inspection(
    state: &mut AppState,
    workspaces_source: &JjWorkspaces,
    kind: WorkspaceInspectionKind,
) {
    let query = {
        let AppView::Workspaces { view } = state.views.active_mut() else {
            return;
        };
        let Some(row) = view.selected_row() else {
            return;
        };
        let Some(root) = row.root().map(ToOwned::to_owned) else {
            view.show_error(format!("workspace `{}` has no root", row.name));
            return;
        };
        WorkspaceInspectionQuery::new(root)
    };

    let command_source = match kind {
        WorkspaceInspectionKind::Status => {
            CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceStatus)
        }
        WorkspaceInspectionKind::Diff => {
            CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceDiff)
        }
    };
    let mut runner = recording_runner(&mut state.history, command_source);
    let snapshot = match kind {
        WorkspaceInspectionKind::Status => {
            workspaces_source.load_status_with_runner(&query, &mut runner)
        }
        WorkspaceInspectionKind::Diff => {
            workspaces_source.load_diff_with_runner(&query, &mut runner)
        }
    };
    match snapshot {
        Ok(snapshot) => {
            let view = RenderedView::new(snapshot);
            let app_view = match kind {
                WorkspaceInspectionKind::Status => AppView::WorkspaceStatus { view, query },
                WorkspaceInspectionKind::Diff => AppView::WorkspaceDiff { view, query },
            };
            state.views.push(app_view);
        }
        Err(error) => {
            if let AppView::Workspaces { view } = state.views.active_mut() {
                view.show_error(error.to_string());
            }
        }
    }
}

fn update_selected_workspace_stale(state: &mut AppState, workspaces_source: &JjWorkspaces) {
    let (workspace_name, query) = {
        let AppView::Workspaces { view } = state.views.active_mut() else {
            return;
        };
        let Some(row) = view.selected_row() else {
            view.show_status("No workspace selected");
            return;
        };
        let workspace_name = row.name.clone();
        let Some(root) = row.root().map(ToOwned::to_owned) else {
            view.show_error(format!("workspace `{}` has no root", row.name));
            return;
        };
        (workspace_name, WorkspaceInspectionQuery::new(root))
    };

    let mut update_runner = recording_runner(
        &mut state.history,
        CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceUpdateStale),
    );
    match workspaces_source.update_stale_with_runner(&query, &mut update_runner) {
        Ok(outcome) => {
            let success = update_stale_success_message(
                &workspace_name,
                &outcome.title,
                &outcome.stderr,
                &outcome.stdout,
            );
            let refresh_result = {
                let mut refresh_runner = recording_runner(
                    &mut state.history,
                    CommandSource::new(SourceView::Workspaces, SourceAction::Refresh),
                );
                workspaces_source.load_list_with_runner(&mut refresh_runner)
            };
            if let AppView::Workspaces { view } = state.views.active_mut() {
                match refresh_result {
                    Ok(snapshot) => {
                        view.refresh(workspace_view_snapshot(snapshot));
                        view.show_status(success);
                    }
                    Err(error) => {
                        view.show_status(format!("{success}; refresh failed: {error}"));
                    }
                }
            }
        }
        Err(error) => {
            if let AppView::Workspaces { view } = state.views.active_mut() {
                view.show_error(error.to_string());
            }
        }
    }
}

fn update_stale_success_message(
    workspace_name: &str,
    command: &str,
    stderr: &str,
    stdout: &str,
) -> String {
    let command = compact_update_stale_command(command);
    let output = compact_command_output(stderr).or_else(|| compact_command_output(stdout));
    match output {
        Some(output) => format!("updated {workspace_name} via {command}: {output}"),
        None => format!("updated {workspace_name} via {command}"),
    }
}

fn compact_update_stale_command(command: &str) -> &str {
    command
        .strip_prefix("jj -R ")
        .and_then(|command| command.split_once(" workspace update-stale"))
        .map_or(command, |_| "workspace update-stale")
}

fn compact_command_output(output: &str) -> Option<String> {
    let first_line = output.lines().find(|line| !line.trim().is_empty())?.trim();
    const MAX_STATUS_CHARS: usize = 120;
    if first_line.chars().count() <= MAX_STATUS_CHARS {
        Some(first_line.to_owned())
    } else {
        let mut summary = first_line
            .chars()
            .take(MAX_STATUS_CHARS.saturating_sub(3))
            .collect::<String>();
        summary.push_str("...");
        Some(summary)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceInspectionKind {
    Status,
    Diff,
}

/// Applies an action while the command-history list is active.
fn apply_command_history_action(
    view: &mut CommandHistoryView,
    history: &CommandHistory,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let history_action = match action {
        jk_tui::log_view::LogAction::Previous => CommandHistoryAction::Previous,
        jk_tui::log_view::LogAction::Next => CommandHistoryAction::Next,
        jk_tui::log_view::LogAction::ScrollPreviousLine => CommandHistoryAction::Previous,
        jk_tui::log_view::LogAction::ScrollNextLine => CommandHistoryAction::Next,
        jk_tui::log_view::LogAction::PagePrevious => CommandHistoryAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext | jk_tui::log_view::LogAction::ToggleMark => {
            CommandHistoryAction::PageNext
        }
        jk_tui::log_view::LogAction::First => CommandHistoryAction::First,
        jk_tui::log_view::LogAction::Last => CommandHistoryAction::Last,
        jk_tui::log_view::LogAction::Refresh => CommandHistoryAction::Refresh,
        jk_tui::log_view::LogAction::ToggleHelp => CommandHistoryAction::ToggleHelp,
        jk_tui::log_view::LogAction::Quit => CommandHistoryAction::Quit,
        _ => return AppTransition::Continue,
    };

    match view.apply(history_action) {
        CommandHistoryActionResult::Refresh => view.refresh(command_history_snapshot(history)),
        CommandHistoryActionResult::ReturnBack => return AppTransition::PopView,
        CommandHistoryActionResult::Quit => return AppTransition::Quit,
        CommandHistoryActionResult::Continue => {}
        _ => {}
    }

    AppTransition::Continue
}

/// Applies an action while the workspace list is active.
fn apply_workspaces_action(
    view: &mut WorkspacesView,
    history: &mut CommandHistory,
    workspaces_source: &JjWorkspaces,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let workspaces_action = match action {
        jk_tui::log_view::LogAction::Previous => WorkspacesAction::Previous,
        jk_tui::log_view::LogAction::Next => WorkspacesAction::Next,
        jk_tui::log_view::LogAction::ScrollPreviousLine => WorkspacesAction::ScrollPreviousLine,
        jk_tui::log_view::LogAction::ScrollNextLine => WorkspacesAction::ScrollNextLine,
        jk_tui::log_view::LogAction::PagePrevious => WorkspacesAction::Previous,
        jk_tui::log_view::LogAction::PageNext => WorkspacesAction::Next,
        jk_tui::log_view::LogAction::First => WorkspacesAction::First,
        jk_tui::log_view::LogAction::Last => WorkspacesAction::Last,
        jk_tui::log_view::LogAction::Refresh => WorkspacesAction::Refresh,
        jk_tui::log_view::LogAction::OpenDiff => WorkspacesAction::OpenDiff,
        jk_tui::log_view::LogAction::ToggleHelp => WorkspacesAction::ToggleHelp,
        jk_tui::log_view::LogAction::Quit => WorkspacesAction::Quit,
        jk_tui::log_view::LogAction::Home
        | jk_tui::log_view::LogAction::Log
        | jk_tui::log_view::LogAction::CollapseExpanded => WorkspacesAction::ReturnBack,
        _ => WorkspacesAction::ReturnBack,
    };

    match view.apply(workspaces_action) {
        WorkspacesActionResult::Refresh => refresh_workspaces(view, history, workspaces_source),
        WorkspacesActionResult::OpenStatus => {
            return AppTransition::PushSelectedWorkspaceStatus;
        }
        WorkspacesActionResult::OpenDiff => {
            return AppTransition::PushSelectedWorkspaceDiff;
        }
        WorkspacesActionResult::ReturnBack => return AppTransition::PopView,
        WorkspacesActionResult::ViewOptions => return AppTransition::OpenViewOptions,
        WorkspacesActionResult::Quit => return AppTransition::Quit,
        WorkspacesActionResult::Continue => {}
        _ => {}
    }

    AppTransition::Continue
}

/// Applies an action while selected-workspace status or diff is active.
fn apply_workspace_inspection_action(
    view: &mut RenderedView,
    query: &WorkspaceInspectionQuery,
    history: &mut CommandHistory,
    workspaces_source: &JjWorkspaces,
    kind: WorkspaceInspectionKind,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let rendered_action = match action {
        jk_tui::log_view::LogAction::Previous => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::Next => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::ScrollPreviousLine => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::ScrollNextLine => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::PagePrevious => RenderedAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ToggleMark => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ClearMarks => RenderedAction::Ignore,
        jk_tui::log_view::LogAction::First => RenderedAction::First,
        jk_tui::log_view::LogAction::Last => RenderedAction::Last,
        jk_tui::log_view::LogAction::ToggleHelp => RenderedAction::ToggleHelp,
        jk_tui::log_view::LogAction::Refresh => RenderedAction::Refresh,
        jk_tui::log_view::LogAction::Quit => RenderedAction::Quit,
        _ => RenderedAction::ReturnToLog,
    };

    match view.apply(rendered_action) {
        RenderedActionResult::Refresh => {
            refresh_workspace_inspection(view, query, history, workspaces_source, kind);
        }
        RenderedActionResult::ReturnToLog => return AppTransition::PopView,
        RenderedActionResult::Quit => return AppTransition::Quit,
        RenderedActionResult::Continue => {}
        _ => {}
    }

    AppTransition::Continue
}

/// Applies an action while the diff view is active.
fn apply_diff_action(
    diff: &mut DiffView,
    query: &DiffQuery,
    history: &mut CommandHistory,
    diff_source: &JjDiff,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let diff_action = match action {
        jk_tui::log_view::LogAction::Previous => DiffAction::ScrollPrevious,
        jk_tui::log_view::LogAction::Next => DiffAction::ScrollNext,
        jk_tui::log_view::LogAction::ScrollPreviousLine => DiffAction::ScrollPrevious,
        jk_tui::log_view::LogAction::ScrollNextLine => DiffAction::ScrollNext,
        jk_tui::log_view::LogAction::PagePrevious => DiffAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext => DiffAction::PageNext,
        jk_tui::log_view::LogAction::ToggleMark => DiffAction::PageNext,
        jk_tui::log_view::LogAction::ClearMarks => DiffAction::Ignore,
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
        DiffActionResult::Refresh => refresh_diff(diff, query, history, diff_source),
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
    history: &mut CommandHistory,
    show_source: &JjShow,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let rendered_action = match action {
        jk_tui::log_view::LogAction::Previous => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::Next => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::ScrollPreviousLine => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::ScrollNextLine => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::PagePrevious => RenderedAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ToggleMark => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ClearMarks => RenderedAction::Ignore,
        jk_tui::log_view::LogAction::First => RenderedAction::First,
        jk_tui::log_view::LogAction::Last => RenderedAction::Last,
        jk_tui::log_view::LogAction::ToggleHelp => RenderedAction::ToggleHelp,
        jk_tui::log_view::LogAction::Refresh => RenderedAction::Refresh,
        jk_tui::log_view::LogAction::Quit => RenderedAction::Quit,
        _ => RenderedAction::ReturnToLog,
    };

    match view.apply(rendered_action) {
        RenderedActionResult::Refresh => refresh_show(view, query, history, show_source),
        RenderedActionResult::ReturnToLog => return AppTransition::PopView,
        RenderedActionResult::Quit => return AppTransition::Quit,
        _ => {}
    }

    AppTransition::Continue
}

/// Applies an action while an evolution-log view is active.
fn apply_evolog_action(
    view: &mut RenderedView,
    query: &EvologQuery,
    history: &mut CommandHistory,
    evolog_source: &JjEvolog,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let rendered_action = match action {
        jk_tui::log_view::LogAction::Previous => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::Next => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::ScrollPreviousLine => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::ScrollNextLine => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::PagePrevious => RenderedAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ToggleMark => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ClearMarks => RenderedAction::Ignore,
        jk_tui::log_view::LogAction::First => RenderedAction::First,
        jk_tui::log_view::LogAction::Last => RenderedAction::Last,
        jk_tui::log_view::LogAction::ToggleHelp => RenderedAction::ToggleHelp,
        jk_tui::log_view::LogAction::Refresh => RenderedAction::Refresh,
        jk_tui::log_view::LogAction::Quit => RenderedAction::Quit,
        _ => RenderedAction::ReturnToLog,
    };

    match view.apply(rendered_action) {
        RenderedActionResult::Refresh => refresh_evolog(view, query, history, evolog_source),
        RenderedActionResult::ReturnToLog => return AppTransition::PopView,
        RenderedActionResult::Quit => return AppTransition::Quit,
        _ => {}
    }

    AppTransition::Continue
}

/// Applies an action while a repository status view is active.
fn apply_status_action(
    view: &mut RenderedView,
    query: &StatusQuery,
    history: &mut CommandHistory,
    status_source: &JjStatus,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let rendered_action = match action {
        jk_tui::log_view::LogAction::Previous => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::Next => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::ScrollPreviousLine => RenderedAction::ScrollPrevious,
        jk_tui::log_view::LogAction::ScrollNextLine => RenderedAction::ScrollNext,
        jk_tui::log_view::LogAction::PagePrevious => RenderedAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ToggleMark => RenderedAction::PageNext,
        jk_tui::log_view::LogAction::ClearMarks => RenderedAction::Ignore,
        jk_tui::log_view::LogAction::First => RenderedAction::First,
        jk_tui::log_view::LogAction::Last => RenderedAction::Last,
        jk_tui::log_view::LogAction::ToggleHelp => RenderedAction::ToggleHelp,
        jk_tui::log_view::LogAction::Refresh => RenderedAction::Refresh,
        jk_tui::log_view::LogAction::Quit => RenderedAction::Quit,
        _ => RenderedAction::ReturnToLog,
    };

    match view.apply(rendered_action) {
        RenderedActionResult::Refresh => refresh_status(view, query, history, status_source),
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
    PushSelectedWorkspaceStatus,
    PushSelectedWorkspaceDiff,
    OpenViewOptions,
    Quit,
}

/// Reloads the current command without replacing the view on failure.
fn refresh_log(
    app: &mut LogView,
    history: &mut CommandHistory,
    source: &JjLog,
    command_source: CommandSource,
) {
    let mut runner = recording_runner(history, command_source);
    match source.load_with_runner(&mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active diff without replacing the view on failure.
fn refresh_diff(
    app: &mut DiffView,
    query: &DiffQuery,
    history: &mut CommandHistory,
    source: &JjDiff,
) {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Diff, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active show/details view without replacing it on failure.
fn refresh_show(
    app: &mut RenderedView,
    query: &ShowQuery,
    history: &mut CommandHistory,
    source: &JjShow,
) {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Show, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active evolog view without replacing it on failure.
fn refresh_evolog(
    app: &mut RenderedView,
    query: &EvologQuery,
    history: &mut CommandHistory,
    source: &JjEvolog,
) {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Evolog, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the active status view without replacing it on failure.
fn refresh_status(
    app: &mut RenderedView,
    query: &StatusQuery,
    history: &mut CommandHistory,
    source: &JjStatus,
) {
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Status, SourceAction::Refresh),
    );
    match source.load_query_with_runner(query, &mut runner) {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads the workspace list without replacing the view on failure.
fn refresh_workspaces(
    app: &mut WorkspacesView,
    history: &mut CommandHistory,
    source: &JjWorkspaces,
) {
    refresh_workspaces_with_runner(app, history, source, SystemJjCommandRunner);
}

fn refresh_workspaces_with_runner<R: JjCommandRunner>(
    app: &mut WorkspacesView,
    history: &mut CommandHistory,
    source: &JjWorkspaces,
    runner: R,
) {
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        history,
        CommandSource::new(SourceView::Workspaces, SourceAction::Refresh),
    );
    match source.load_list_with_runner(&mut runner) {
        Ok(snapshot) => app.refresh(workspace_view_snapshot(snapshot)),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Reloads a selected-workspace inspection view without changing its workspace root.
fn refresh_workspace_inspection(
    app: &mut RenderedView,
    query: &WorkspaceInspectionQuery,
    history: &mut CommandHistory,
    source: &JjWorkspaces,
    kind: WorkspaceInspectionKind,
) {
    let command_source = match kind {
        WorkspaceInspectionKind::Status => {
            CommandSource::new(SourceView::WorkspaceStatus, SourceAction::Refresh)
        }
        WorkspaceInspectionKind::Diff => {
            CommandSource::new(SourceView::WorkspaceDiff, SourceAction::Refresh)
        }
    };
    let mut runner = recording_runner(history, command_source);
    let snapshot = match kind {
        WorkspaceInspectionKind::Status => source.load_status_with_runner(query, &mut runner),
        WorkspaceInspectionKind::Diff => source.load_diff_with_runner(query, &mut runner),
    };
    match snapshot {
        Ok(snapshot) => app.refresh(snapshot),
        Err(error) => app.show_error(error.to_string()),
    }
}

/// Switches the command context only after the replacement log loads.
fn switch_log_command(
    app: &mut LogView,
    history: &mut CommandHistory,
    source: &mut JjLog,
    command: JjLogCommand,
) {
    let mut next_source = source.clone().with_command(command);
    if command == JjLogCommand::ConfiguredDefault {
        next_source = next_source.with_configured_template();
    }
    let mut runner = recording_runner(
        history,
        CommandSource::new(SourceView::Log, SourceAction::Refresh),
    );
    match next_source.load_with_runner(&mut runner) {
        Ok(snapshot) => {
            *source = next_source;
            app.refresh(snapshot);
        }
        Err(error) => app.show_error(error.to_string()),
    }
}

fn open_template_selector(modes: &mut ModeStack, source: &JjLog) {
    let options = source.template_options();
    let selected = options
        .iter()
        .position(|template| template == source.template())
        .unwrap_or(0);
    modes.push(InputMode::LogTemplate { options, selected });
}

/// Switches the rendered log template only after the replacement log loads.
fn apply_log_template_selection(
    state: &mut AppState,
    source: &mut JjLog,
    template: LogTemplateSelection,
) {
    if !matches!(state.views.active(), AppView::Log(_)) {
        return;
    }

    let next_source = source
        .clone()
        .with_command(JjLogCommand::Log)
        .with_template(template);
    let mut runner = recording_runner(
        &mut state.history,
        CommandSource::new(SourceView::Log, SourceAction::Refresh),
    );
    match next_source.load_with_runner(&mut runner) {
        Ok(snapshot) => {
            *source = next_source;
            if let AppView::Log(log) = state.views.active_mut() {
                log.refresh(snapshot);
            }
        }
        Err(error) => show_log_template_load_error(state, error.to_string()),
    }
}

fn show_log_template_load_error(state: &mut AppState, error: String) {
    if let AppView::Log(log) = state.views.active_mut() {
        log.show_error(error);
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
    use std::collections::VecDeque;
    use std::io;
    use std::process::Output;

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
    fn app_state_exposes_retained_command_history_for_tests() {
        let mut history = CommandHistory::new(4);
        let spec = jk_core::JjCommandSpec::render_read_only(["status"]);
        history.start(jk_core::CommandRecordStart::from_spec(
            &spec,
            CommandSource::new(SourceView::Status, SourceAction::InitialLoad),
        ));
        let state = AppState::with_history(AppView::Log(LogView::default()), history);

        assert_eq!(state.command_history().records().count(), 1);
    }

    #[test]
    fn opening_command_history_from_log_pushes_snapshot_without_recording() {
        let mut history = CommandHistory::new(4);
        append_history_record(
            &mut history,
            jk_core::JjCommandSpec::render_read_only(["log"]),
            SourceView::Log,
            SourceAction::InitialLoad,
        );
        let mut state = AppState::with_history(AppView::Log(LogView::default()), history);

        open_command_history(&mut state);

        assert_eq!(state.views.views.len(), 2);
        assert!(matches!(
            state.views.active(),
            AppView::CommandHistory { .. }
        ));
        assert_eq!(state.command_history().records().count(), 1);
    }

    #[test]
    fn opening_command_history_from_workspaces_preserves_previous_view() {
        let mut history = CommandHistory::new(4);
        append_history_record(
            &mut history,
            jk_core::JjCommandSpec::render_read_only(["workspace", "list"]),
            SourceView::Workspaces,
            SourceAction::WorkspaceList,
        );
        let workspace_view = workspace_app_view();
        let expected = workspace_view.clone();
        let mut state = AppState::with_history(
            AppView::Workspaces {
                view: workspace_view,
            },
            history,
        );

        open_command_history(&mut state);

        assert_eq!(state.views.views.len(), 2);
        assert!(matches!(
            state.views.active(),
            AppView::CommandHistory { .. }
        ));

        handle_back(&mut state);

        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
        assert_eq!(state.command_history().records().count(), 1);
    }

    #[test]
    fn opening_command_history_again_refreshes_without_stacking() {
        let mut history = CommandHistory::new(4);
        append_history_record(
            &mut history,
            jk_core::JjCommandSpec::render_read_only(["log"]),
            SourceView::Log,
            SourceAction::InitialLoad,
        );
        let mut state = AppState::with_history(AppView::Log(LogView::default()), history);

        open_command_history(&mut state);
        open_command_history(&mut state);

        assert_eq!(state.views.views.len(), 2);
        assert!(matches!(
            state.views.active(),
            AppView::CommandHistory { .. }
        ));
        assert_eq!(state.command_history().records().count(), 1);
    }

    #[test]
    fn refreshing_command_history_rebuilds_snapshot_without_recording() {
        let mut history = CommandHistory::new(4);
        append_history_record(
            &mut history,
            jk_core::JjCommandSpec::render_read_only(["log"]),
            SourceView::Log,
            SourceAction::InitialLoad,
        );
        let mut view = CommandHistoryView::new(CommandHistorySnapshot::new(Vec::new()));

        let transition =
            apply_command_history_action(&mut view, &history, jk_tui::log_view::LogAction::Refresh);

        assert!(matches!(transition, AppTransition::Continue));
        assert_eq!(
            view.selected_row().map(|row| row.command.as_str()),
            Some("jj log")
        );
        assert_eq!(history.records().count(), 1);
    }

    #[test]
    fn unsupported_command_history_actions_do_not_pop_view() {
        let history = CommandHistory::new(4);
        let mut view = CommandHistoryView::new(CommandHistorySnapshot::new(Vec::new()));

        let transition = apply_command_history_action(
            &mut view,
            &history,
            jk_tui::log_view::LogAction::OpenDiff,
        );

        assert!(matches!(transition, AppTransition::Continue));
    }

    #[test]
    fn view_options_are_ignored_from_command_history() {
        let mut state = AppState::new(AppView::CommandHistory {
            view: CommandHistoryView::new(CommandHistorySnapshot::new(Vec::new())),
        });

        open_view_options(&mut state);

        assert_eq!(state.modes.active(), None);
    }

    #[test]
    fn opening_workspaces_from_log_records_log_workspace_list() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let source = JjWorkspaces::default();
        let runner = SequencedRunner::successes(vec![
            output(0, "vibe\t/repo/vibe\tabc123\tdef456\n", ""),
            output(0, "/repo/vibe\n", ""),
        ]);

        open_workspaces_with_runner(&mut state, &source, runner);

        assert_eq!(state.views.views.len(), 2);
        assert!(matches!(state.views.active(), AppView::Workspaces { .. }));
        assert_eq!(state.command_history().records().count(), 2);

        let records = state.command_history().records().collect::<Vec<_>>();
        assert!(
            records[0]
                .command
                .spec_preview
                .starts_with("jj workspace list")
        );
        assert!(records[1].command.spec_preview.starts_with("jj root"));
        assert!(
            records
                .iter()
                .all(|record| record.source.view == SourceView::Log)
        );
        assert!(
            records
                .iter()
                .all(|record| record.source.action == SourceAction::WorkspaceList)
        );
    }

    #[test]
    fn refreshing_workspaces_keeps_workspace_source_and_refresh_action() {
        let mut state = AppState::new(AppView::Workspaces {
            view: WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new())),
        });
        let source = JjWorkspaces::default();
        let runner = SequencedRunner::successes(vec![
            output(0, "vibe\t/repo/vibe\tabc123\tdef456\n", ""),
            output(0, "/repo/vibe\n", ""),
        ]);

        open_workspaces_with_runner(&mut state, &source, runner);

        assert_eq!(state.views.views.len(), 1);
        assert!(matches!(state.views.active(), AppView::Workspaces { .. }));
        assert_eq!(state.command_history().records().count(), 2);

        let records = state.command_history().records().collect::<Vec<_>>();
        assert!(
            records[0]
                .command
                .spec_preview
                .starts_with("jj workspace list")
        );
        assert!(records[1].command.spec_preview.starts_with("jj root"));
        assert!(
            records
                .iter()
                .all(|record| record.source.view == SourceView::Workspaces)
        );
        assert!(
            records
                .iter()
                .all(|record| record.source.action == SourceAction::Refresh)
        );
    }

    #[test]
    fn opening_status_from_log_records_log_open_status() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let source = JjStatus::default();
        let runner = SequencedRunner::successes(vec![output(0, "status output\n", "")]);

        push_status_with_runner(&mut state, &source, runner);

        assert_eq!(state.views.views.len(), 2);
        assert!(matches!(state.views.active(), AppView::Status { .. }));
        assert_eq!(state.command_history().records().count(), 1);

        let record = state.command_history().records().next().expect("record");
        assert_eq!(record.command.spec_preview, "jj status");
        assert_eq!(record.source.view, SourceView::Log);
        assert_eq!(record.source.action, SourceAction::OpenStatus);
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
    fn loaded_evolog_pushes_from_log() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let query = EvologQuery::from("aaa".to_owned());
        let view = RenderedView::from_error("aaa", "jj evolog -r aaa", "fixture".to_owned());

        push_evolog_view(&mut state, query.clone(), view.clone());

        assert_eq!(state.views.views.len(), 2);
        assert_eq!(state.views.active(), &AppView::Evolog { view, query });
    }

    #[test]
    fn loaded_evolog_is_ignored_outside_log() {
        let root = diff_app_view("aaa");
        let mut state = AppState::new(root.clone());
        let query = EvologQuery::from("aaa".to_owned());
        let view = RenderedView::from_error("aaa", "jj evolog -r aaa", "fixture".to_owned());

        push_evolog_view(&mut state, query, view);

        assert_eq!(state.views.views.len(), 1);
        assert_eq!(state.views.active(), &root);
    }

    #[test]
    fn back_from_evolog_returns_to_preserved_log() {
        let mut log = LogView::default();
        log.show_error("preserved log state");
        let expected_log = log.clone();
        let mut state = AppState::new(AppView::Log(log));
        let query = EvologQuery::from("aaa".to_owned());
        let view = RenderedView::from_error("aaa", "jj evolog -r aaa", "fixture".to_owned());
        push_evolog_view(&mut state, query, view);

        handle_back(&mut state);

        assert_eq!(state.views.active(), &AppView::Log(expected_log));
    }

    #[test]
    fn workspace_snapshot_mapping_preserves_row_fields() {
        let snapshot = WorkspaceListSnapshot {
            title: "jj workspace list".to_owned(),
            workspaces: vec![WorkspaceSummary {
                name: "vibe".to_owned(),
                root: Some(PathBuf::from("/repo/vibe")),
                current: true,
                change_id: Some("abc123".to_owned()),
                commit_id: Some("def456".to_owned()),
            }],
        };

        let snapshot = workspace_view_snapshot(snapshot);

        assert_eq!(snapshot.title(), "jj workspace list");
        assert_eq!(
            snapshot.rows(),
            &[WorkspaceViewRow::new("vibe", "/repo/vibe", true)
                .with_root("/repo/vibe")
                .with_change_id("abc123")
                .with_commit_id("def456")]
        );
    }

    #[test]
    fn loaded_workspace_pushes_view() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let view = workspace_app_view();

        push_workspace_view(&mut state, view.clone());

        assert_eq!(state.views.views.len(), 2);
        assert_eq!(state.views.active(), &AppView::Workspaces { view });
    }

    #[test]
    fn w_on_active_workspace_list_does_not_stack_another_list() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        push_workspace_view(&mut state, workspace_app_view());
        let expected_depth = state.views.views.len();

        open_workspaces(
            &mut state,
            &JjWorkspaces::default().with_repository("/definitely/not-a-jj-repo"),
        );

        assert_eq!(state.views.views.len(), expected_depth);
        assert!(matches!(state.views.active(), AppView::Workspaces { .. }));
    }

    #[test]
    fn workspace_missing_root_status_shows_error_without_pushing() {
        let mut state = AppState::new(AppView::Workspaces {
            view: WorkspacesView::new(WorkspaceViewSnapshot::new(vec![WorkspaceViewRow::new(
                "detached",
                "(no root)",
                true,
            )])),
        });
        let source = JjWorkspaces::default();
        let mut expected =
            WorkspacesView::new(WorkspaceViewSnapshot::new(vec![WorkspaceViewRow::new(
                "detached",
                "(no root)",
                true,
            )]));
        expected.show_error("workspace `detached` has no root");

        push_selected_workspace_status(&mut state, &source);

        assert_eq!(state.views.views.len(), 1);
        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
    }

    #[test]
    fn workspace_missing_root_diff_shows_error_without_pushing() {
        let mut state = AppState::new(AppView::Workspaces {
            view: WorkspacesView::new(WorkspaceViewSnapshot::new(vec![WorkspaceViewRow::new(
                "detached",
                "(no root)",
                true,
            )])),
        });
        let source = JjWorkspaces::default();
        let mut expected =
            WorkspacesView::new(WorkspaceViewSnapshot::new(vec![WorkspaceViewRow::new(
                "detached",
                "(no root)",
                true,
            )]));
        expected.show_error("workspace `detached` has no root");

        push_selected_workspace_diff(&mut state, &source);

        assert_eq!(state.views.views.len(), 1);
        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
    }

    #[test]
    fn workspace_missing_root_update_stale_shows_error_without_pushing() {
        let mut state = AppState::new(AppView::Workspaces {
            view: WorkspacesView::new(WorkspaceViewSnapshot::new(vec![WorkspaceViewRow::new(
                "detached",
                "(no root)",
                true,
            )])),
        });
        let source = JjWorkspaces::default();
        let mut expected =
            WorkspacesView::new(WorkspaceViewSnapshot::new(vec![WorkspaceViewRow::new(
                "detached",
                "(no root)",
                true,
            )]));
        expected.show_error("workspace `detached` has no root");

        update_selected_workspace_stale(&mut state, &source);

        assert_eq!(state.views.views.len(), 1);
        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
    }

    #[test]
    fn workspace_update_stale_without_selection_keeps_list_visible() {
        let mut state = AppState::new(AppView::Workspaces {
            view: WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new())),
        });
        let source = JjWorkspaces::default();
        let mut expected = WorkspacesView::new(WorkspaceViewSnapshot::new(Vec::new()));
        expected.show_status("No workspace selected");

        update_selected_workspace_stale(&mut state, &source);

        assert_eq!(state.views.views.len(), 1);
        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
    }

    #[test]
    fn workspace_update_stale_success_message_prefers_stderr_then_stdout() {
        assert_eq!(
            update_stale_success_message(
                "vibe",
                "jj -R /repo/vibe workspace update-stale",
                "warning line",
                "stdout line",
            ),
            "updated vibe via workspace update-stale: warning line"
        );

        assert_eq!(
            update_stale_success_message(
                "vibe",
                "jj -R /repo/vibe workspace update-stale",
                "",
                "stdout line",
            ),
            "updated vibe via workspace update-stale: stdout line"
        );

        assert_eq!(
            update_stale_success_message("vibe", "jj workspace update-stale", "", ""),
            "updated vibe via jj workspace update-stale"
        );
    }

    #[test]
    fn back_from_workspace_status_returns_to_workspaces() {
        let workspace_view = workspace_app_view();
        let expected = workspace_view.clone();
        let mut state = AppState::new(AppView::Workspaces {
            view: workspace_view,
        });
        state.views.push(AppView::WorkspaceStatus {
            view: RenderedView::from_error("/repo/vibe", "jj status", "fixture".to_owned()),
            query: WorkspaceInspectionQuery::new("/repo/vibe"),
        });

        handle_back(&mut state);

        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
    }

    #[test]
    fn active_binding_context_reports_workspaces() {
        let state = AppState::new(AppView::Workspaces {
            view: workspace_app_view(),
        });

        assert_eq!(active_binding_context(&state), BindingContext::Workspaces);
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

        let mut source = JjLog::default();
        let result = handle_input_mode(
            &mut state,
            &mut source,
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
    fn command_discovery_opens_for_active_context() {
        let mut state = AppState::new(diff_app_view("aaa"));

        open_command_discovery(&mut state);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::CommandDiscovery {
                context: BindingContext::Diff,
                query: String::new(),
                selected: 0,
            })
        );
    }

    #[test]
    fn command_discovery_filters_and_clamps_selection() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.modes.push(InputMode::CommandDiscovery {
            context: BindingContext::Log,
            query: "jj sho".to_owned(),
            selected: 12,
        });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(
            state.modes.active(),
            Some(&InputMode::CommandDiscovery {
                context: BindingContext::Log,
                query: "jj show".to_owned(),
                selected: 0,
            })
        );
    }

    #[test]
    fn command_discovery_navigation_clamps_to_visible_rows() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.modes.push(InputMode::CommandDiscovery {
            context: BindingContext::Log,
            query: "jj show".to_owned(),
            selected: 0,
        });

        move_command_discovery_selection(&mut state, DiscoveryMove::Next);
        move_command_discovery_selection(&mut state, DiscoveryMove::Previous);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::CommandDiscovery {
                context: BindingContext::Log,
                query: "jj show".to_owned(),
                selected: 0,
            })
        );
    }

    #[test]
    fn command_discovery_enter_closes_without_executing() {
        let mut state = AppState::new(diff_app_view("aaa"));
        let expected_view = state.views.active().clone();
        state.modes.push(InputMode::CommandDiscovery {
            context: BindingContext::Diff,
            query: "refresh".to_owned(),
            selected: 0,
        });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
        assert_eq!(state.views.active(), &expected_view);
    }

    #[test]
    fn command_discovery_backspace_closes_when_query_empty() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.modes.push(InputMode::CommandDiscovery {
            context: BindingContext::Log,
            query: String::new(),
            selected: 0,
        });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
    }

    #[test]
    fn view_options_opens_for_active_context() {
        let mut state = AppState::new(diff_app_view("aaa"));

        open_view_options(&mut state);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::ViewOptions {
                context: BindingContext::Diff,
                selected: 0,
            })
        );
    }

    #[test]
    fn view_options_enter_opens_log_template_selector() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.modes.push(InputMode::ViewOptions {
            context: BindingContext::Log,
            selected: 0,
        });
        let mut source = JjLog::default().with_template(LogTemplateSelection::Detailed);

        let result = handle_input_mode(
            &mut state,
            &mut source,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        let options = source.template_options();
        let selected = options
            .iter()
            .position(|template| template == source.template())
            .unwrap_or_default();
        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(
            state.modes.active(),
            Some(&InputMode::LogTemplate { options, selected })
        );
    }

    #[test]
    fn view_options_placeholder_enter_closes_non_log_overlay() {
        let mut state = AppState::new(diff_app_view("aaa"));
        state.modes.push(InputMode::ViewOptions {
            context: BindingContext::Diff,
            selected: 0,
        });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
    }

    #[test]
    fn view_options_close_without_changing_source() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.modes.push(InputMode::ViewOptions {
            context: BindingContext::Log,
            selected: 0,
        });
        let mut source = JjLog::default().with_template(LogTemplateSelection::Compact);
        let expected = source.template().clone();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
        assert_eq!(source.template(), &expected);
    }

    #[test]
    fn view_options_lines_show_template_or_placeholder() {
        assert_eq!(
            view_options_lines(
                BindingContext::Log,
                0,
                &LogTemplateSelection::CompactFullDescription,
            ),
            vec![
                "> Template           full description",
                "",
                "j/k or arrows move   enter open   esc close",
            ]
        );
        assert_eq!(
            view_options_lines(BindingContext::Diff, 0, &LogTemplateSelection::Configured),
            vec!["No view options in this slice.", "", "esc close"]
        );
    }

    #[test]
    fn template_selection_failure_preserves_previous_log_and_source() {
        let previous_log = LogView::default();
        let mut state = AppState::new(AppView::Log(previous_log.clone()));
        let source = JjLog::default().with_template(LogTemplateSelection::Detailed);
        let expected_template = source.template().clone();

        show_log_template_load_error(&mut state, "template reload failed".to_owned());

        let mut expected_log = previous_log;
        expected_log.show_error("template reload failed");
        assert_eq!(source.template(), &expected_template);
        assert_eq!(state.views.active(), &AppView::Log(expected_log));
    }

    #[test]
    fn mark_action_pages_diff_view() {
        let query = diff_query("aaa");
        let source = JjDiff::default();
        let mut mark_view = diff_view("aaa");
        let mut page_view = diff_view("aaa");
        let mut mark_history = CommandHistory::default();
        let mut page_history = CommandHistory::default();

        let mark_transition = apply_diff_action(
            &mut mark_view,
            &query,
            &mut mark_history,
            &source,
            LogAction::ToggleMark,
        );
        let page_transition = apply_diff_action(
            &mut page_view,
            &query,
            &mut page_history,
            &source,
            LogAction::PageNext,
        );

        assert!(matches!(mark_transition, AppTransition::Continue));
        assert!(matches!(page_transition, AppTransition::Continue));
        assert_eq!(mark_view, page_view);
    }

    #[test]
    fn mark_action_pages_inspection_views() {
        let show_query = ShowQuery::from("aaa".to_owned());
        let show_source = JjShow::default();
        let mut mark_show = RenderedView::from_error("aaa", "jj show aaa", "fixture".to_owned());
        let mut page_show = mark_show.clone();
        let mut mark_history = CommandHistory::default();
        let mut page_history = CommandHistory::default();

        let mark_transition = apply_show_action(
            &mut mark_show,
            &show_query,
            &mut mark_history,
            &show_source,
            LogAction::ToggleMark,
        );
        let page_transition = apply_show_action(
            &mut page_show,
            &show_query,
            &mut page_history,
            &show_source,
            LogAction::PageNext,
        );

        assert!(matches!(mark_transition, AppTransition::Continue));
        assert!(matches!(page_transition, AppTransition::Continue));
        assert_eq!(mark_show, page_show);

        let evolog_query = EvologQuery::from("aaa".to_owned());
        let evolog_source = JjEvolog::default();
        let mut mark_evolog =
            RenderedView::from_error("aaa", "jj evolog -r aaa", "fixture".to_owned());
        let mut page_evolog = mark_evolog.clone();
        let mut mark_history = CommandHistory::default();
        let mut page_history = CommandHistory::default();

        let mark_transition = apply_evolog_action(
            &mut mark_evolog,
            &evolog_query,
            &mut mark_history,
            &evolog_source,
            LogAction::ToggleMark,
        );
        let page_transition = apply_evolog_action(
            &mut page_evolog,
            &evolog_query,
            &mut page_history,
            &evolog_source,
            LogAction::PageNext,
        );

        assert!(matches!(mark_transition, AppTransition::Continue));
        assert!(matches!(page_transition, AppTransition::Continue));
        assert_eq!(mark_evolog, page_evolog);

        let status_query = StatusQuery::default();
        let status_source = JjStatus::default();
        let mut mark_status = RenderedView::from_error("status", "jj status", "fixture".to_owned());
        let mut page_status = mark_status.clone();
        let mut mark_history = CommandHistory::default();
        let mut page_history = CommandHistory::default();

        let mark_transition = apply_status_action(
            &mut mark_status,
            &status_query,
            &mut mark_history,
            &status_source,
            LogAction::ToggleMark,
        );
        let page_transition = apply_status_action(
            &mut page_status,
            &status_query,
            &mut page_history,
            &status_source,
            LogAction::PageNext,
        );

        assert!(matches!(mark_transition, AppTransition::Continue));
        assert!(matches!(page_transition, AppTransition::Continue));
        assert_eq!(mark_status, page_status);
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
    fn evolog_is_not_a_root_cli_command() {
        assert!(Args::try_parse_from(["jk", "evolog", "-r", "abc123"]).is_err());
    }

    #[test]
    fn log_args_preserve_startup_template() {
        let args =
            Args::try_parse_from(["jk", "log", "-T", "description"]).expect("valid log args");
        let Some(Command::Log(log_args)) = args.command else {
            panic!("expected log command");
        };

        assert_eq!(log_args.template, Some("description".to_owned()));
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

    #[test]
    fn status_args_default_to_repository_status() {
        let args = Args::try_parse_from(["jk", "status"]).expect("valid status args");
        let Some(Command::Status(status_args)) = args.command else {
            panic!("expected status command");
        };

        assert_eq!(status_args.query(), StatusQuery::default());
    }

    #[test]
    fn status_args_preserve_filesets() {
        let args =
            Args::try_parse_from(["jk", "status", "crates/jk", "docs"]).expect("valid status args");
        let Some(Command::Status(status_args)) = args.command else {
            panic!("expected status command");
        };

        assert_eq!(
            status_args.query(),
            StatusQuery::new(vec!["crates/jk".to_owned(), "docs".to_owned()])
        );
    }

    #[test]
    fn workspace_source_defaults_to_cwd_discovery() {
        let args = Args::try_parse_from(["jk"]).expect("valid args");
        let source = workspaces_source(&args);
        let spec = source.list_spec();

        assert_eq!(spec.repository(), None);
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

    fn workspace_app_view() -> WorkspacesView {
        WorkspacesView::new(WorkspaceViewSnapshot::new(vec![
            WorkspaceViewRow::new("default", "/repo/default", false).with_root("/repo/default"),
            WorkspaceViewRow::new("vibe", "/repo/vibe", true).with_root("/repo/vibe"),
        ]))
    }

    fn append_history_record(
        history: &mut CommandHistory,
        spec: jk_core::JjCommandSpec,
        view: SourceView,
        action: SourceAction,
    ) {
        let start = jk_core::CommandRecordStart::from_spec(&spec, CommandSource::new(view, action))
            .with_started_at(std::time::SystemTime::UNIX_EPOCH);
        let finish = jk_core::CommandRecordFinish::from_exit_code(
            0,
            "",
            "",
            std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(1),
        );
        history.append(start, finish);
    }

    fn output(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: exit_status(code),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    struct SequencedRunner {
        outputs: VecDeque<io::Result<Output>>,
    }

    impl SequencedRunner {
        fn successes(outputs: Vec<Output>) -> Self {
            Self {
                outputs: outputs.into_iter().map(Ok).collect(),
            }
        }
    }

    impl JjCommandRunner for SequencedRunner {
        fn run(&mut self, _spec: &jk_core::JjCommandSpec) -> io::Result<Output> {
            self.outputs
                .pop_front()
                .expect("runner called too many times")
        }
    }

    #[cfg(unix)]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;

        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(not(unix))]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
            .args(if cfg!(windows) {
                vec!["/C".into(), format!("exit {code}").into()]
            } else {
                vec!["-c".into(), format!("exit {code}").into()]
            })
            .status()
            .unwrap()
    }
}
