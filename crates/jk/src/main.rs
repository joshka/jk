//! Binary entry point for the `jk` terminal UI.
//!
//! This crate owns command-line parsing, terminal lifecycle, and the bridge between crossterm input
//! events and backend-neutral TUI actions. The product behavior is intentionally delegated to
//! `jk-cli` and `jk-tui` so the binary stays a thin orchestration layer.

#![allow(
    clippy::large_enum_variant,
    clippy::nursery,
    clippy::pedantic,
    clippy::too_many_arguments
)]
#![cfg_attr(test, allow(clippy::expect_used))]

use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::force_color_output;
#[cfg(test)]
use jk_cli::RecoveryCommand;
use jk_cli::{
    AbandonQuery, DescribeQuery, DiffFormat, DiffQuery, EditQuery, EvologQuery, JjAbandon,
    JjCommandRunner, JjDescribe, JjDiff, JjEdit, JjEvolog, JjLog, JjLogCommand, JjNew, JjOperation,
    JjRecovery, JjShow, JjStatus, JjWorkspaces, LogTemplateSelection, NewQuery, OperationQuery,
    RecordingJjCommandRunner, ShowQuery, StatusQuery, SystemJjCommandRunner,
    WorkspaceInspectionQuery,
};
use jk_core::{CommandHistory, CommandSource, SourceAction, SourceView};
use jk_tui::command_discovery::{BindingContext, filtered_discovery_len};
use jk_tui::command_history_view::{CommandHistoryAction, CommandHistoryActionResult};
#[cfg(test)]
use jk_tui::command_history_view::{CommandHistorySnapshot, CommandHistoryView};
use jk_tui::diff_view::{DiffAction, DiffActionResult, DiffView};
#[cfg(test)]
use jk_tui::log_view::LogAction;
use jk_tui::log_view::{ActionResult, LogView};
use jk_tui::operation_log_view::{OperationLogAction, OperationLogActionResult, OperationLogView};
use jk_tui::rendered_view::{RenderedAction, RenderedActionResult, RenderedView};
#[cfg(test)]
use jk_tui::workspaces_view::WorkspaceViewSnapshot;
use jk_tui::workspaces_view::{WorkspacesActionResult, WorkspacesView};

mod actions;
mod cli;
mod clipboard;
mod command_history;
mod command_mode;
mod key;
mod menus;
mod mutation_preview;
mod mutations;
mod operation_log;
mod refresh;
mod rendering;
mod root_views;
mod runner;
mod state;
#[cfg(test)]
mod test_support;
mod workspace_routes;
mod workspaces;

use actions::{AppSources, DispatchResult, dispatch_app_key};
use cli::{Args, Command};
use clipboard::copy_command_line;
use command_history::{apply_command_history_action, open_command_history};
#[cfg(test)]
pub(crate) use command_history::{
    command_history_snapshot, open_command_history_operation_with_runner,
};
pub(crate) use command_history::{
    open_command_history_operation, open_operation_log, push_selected_command_history_details,
};
use command_mode::{command_mode_snapshot, command_mode_spec, parse_jj_command_args};
use key::AppKey;
use menus::{
    MenuDirection, ViewOptionRow, clamp_command_discovery_selection, view_option_rows,
    wrapped_selection,
};
#[cfg(test)]
use menus::{diff_file_list_lines, view_options_lines};
use mutation_preview::{PendingCommandPreview, selected_new_parents};
#[cfg(test)]
use mutations::{POST_MUTATION_RECOVERY_STATUS, confirm_command_preview_with_runner};
use mutations::{confirm_command_preview, open_recovery_preview};
#[cfg(test)]
use refresh::show_log_template_load_error;
use refresh::{
    OperationRenderedKind, apply_log_template_selection, operation_rendered_transition,
    refresh_diff, refresh_evolog, refresh_log, refresh_operation_log, refresh_operation_rendered,
    refresh_show, refresh_status, refresh_workspace_inspection, refresh_workspaces,
    switch_log_command,
};
use rendering::render_app;
use root_views::{
    root_diff_view, root_log_view, root_show_view, root_status_view, root_workspaces_view,
};
pub(crate) use runner::recording_runner;
#[cfg(test)]
use state::ViewStack;
use state::{AppState, AppView, InputMode, InputModeResult, ModeStack};
use workspace_routes::{
    WorkspaceInspectionKind, open_workspaces, push_selected_workspace_diff,
    push_selected_workspace_log, push_selected_workspace_status, push_status,
    update_selected_workspace_stale,
};
#[cfg(test)]
use workspace_routes::{
    open_workspaces_with_runner, push_selected_workspace_log_with_runner, push_status_with_runner,
    push_workspace_view,
};
use workspaces::{workspace_action_for_log_action, workspace_inspection_action_for_log_action};

fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let source = args.log_source();
    let diff_source = args.diff_source();
    let evolog_source = args.evolog_source();
    let show_source = args.show_source();
    let status_source = args.status_source();
    let describe_source = args.describe_source();
    let abandon_source = args.abandon_source();
    let new_source = args.new_source();
    let edit_source = args.edit_source();
    let operation_source = args.operation_source();
    let recovery_source = args.recovery_source();
    let workspaces_source = args.workspaces_source();
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
        Some(Command::Workspaces) => root_workspaces_view(&workspaces_source, &mut history),
        Some(Command::Log(_)) | None => root_log_view(&source, &mut history)?,
    };

    run_terminal(
        app,
        source,
        &diff_source,
        &evolog_source,
        &show_source,
        &status_source,
        &describe_source,
        &abandon_source,
        &new_source,
        &edit_source,
        &operation_source,
        &recovery_source,
        &workspaces_source,
        args.repository,
        history,
    )?;
    Ok(())
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
    describe_source: &JjDescribe,
    abandon_source: &JjAbandon,
    new_source: &JjNew,
    edit_source: &JjEdit,
    operation_source: &JjOperation,
    recovery_source: &JjRecovery,
    workspaces_source: &JjWorkspaces,
    command_repository: Option<PathBuf>,
    history: CommandHistory,
) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(eyre!("jk requires an interactive terminal"));
    }

    // jj should keep configured colors even when the parent process was run by an agent or tool
    // that exports NO_COLOR.
    force_color_output(true);
    let mut terminal = ratatui::try_init()?;
    let _terminal_restore = TerminalRestore;
    let mut needs_redraw = true;
    let mut state = AppState::with_history(app, history);

    loop {
        if needs_redraw {
            terminal.draw(|frame| render_app(frame, &mut state, source.template()))?;
            needs_redraw = false;
        }

        match event::read()? {
            Event::Key(key) => {
                if handle_input_mode(
                    &mut state,
                    &mut source,
                    diff_source,
                    describe_source,
                    command_repository.as_deref(),
                    key,
                ) == InputModeResult::Handled
                {
                    needs_redraw = true;
                    continue;
                }

                let app_key = AppKey::from_crossterm(key);
                let mut sources = AppSources {
                    log: &mut source,
                    diff: diff_source,
                    evolog: evolog_source,
                    show: show_source,
                    status: status_source,
                    abandon: abandon_source,
                    new_change: new_source,
                    edit: edit_source,
                    operation: operation_source,
                    recovery: recovery_source,
                    workspaces: workspaces_source,
                };
                if dispatch_app_key(&mut state, &mut sources, key, app_key) == DispatchResult::Quit
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

/// Handles key input while a prompt-like mode is active.
fn handle_input_mode(
    state: &mut AppState,
    source: &mut JjLog,
    diff_source: &JjDiff,
    describe_source: &JjDescribe,
    command_repository: Option<&Path>,
    key: KeyEvent,
) -> InputModeResult {
    if matches!(state.modes.active(), Some(InputMode::ViewOptions { .. })) {
        return handle_view_options_mode(state, source, diff_source, key);
    }
    if matches!(state.modes.active(), Some(InputMode::DiffFileList { .. })) {
        return handle_diff_file_list_mode(state, key);
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
    if matches!(state.modes.active(), Some(InputMode::CommandPreview { .. })) {
        return handle_command_preview_mode(state, source, key);
    }
    if matches!(state.modes.active(), Some(InputMode::JjCommand { .. })) {
        return handle_jj_command_mode(state, command_repository, key);
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
                InputMode::DescribeMessage { rev, message } => {
                    if message.trim().is_empty() {
                        return InputModeResult::Handled;
                    }
                    let preview = describe_source
                        .spec_for(&DescribeQuery::new(rev.clone(), message.clone()))
                        .command_preview();
                    state.modes.pop();
                    state.modes.push(InputMode::CommandPreview {
                        pending: PendingCommandPreview::describe(preview),
                    });
                    return InputModeResult::Handled;
                }
                InputMode::ViewOptions { .. } => unreachable!(),
                InputMode::DiffFileList { .. } => unreachable!(),
                InputMode::CommandDiscovery { .. } => unreachable!(),
                InputMode::CommandPreview { .. } => unreachable!(),
                InputMode::JjCommand { .. } => unreachable!(),
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
            if let InputMode::DescribeMessage { message, .. } = mode
                && !message.is_empty()
            {
                message.pop();
                return InputModeResult::Handled;
            }
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers,
            ..
        } if modifiers == KeyModifiers::CONTROL => {
            if let InputMode::DescribeMessage { message, .. } = mode {
                message.clear();
            }
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
                InputMode::DescribeMessage { message, .. } => {
                    message.push(character);
                }
                InputMode::ViewOptions { .. } => unreachable!(),
                InputMode::DiffFileList { .. } => unreachable!(),
                InputMode::CommandDiscovery { .. } => unreachable!(),
                InputMode::CommandPreview { .. } => unreachable!(),
                InputMode::JjCommand { .. } => unreachable!(),
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
            move_command_discovery_selection(state, MenuDirection::Previous);
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
            move_command_discovery_selection(state, MenuDirection::Next);
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

fn handle_command_preview_mode(
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
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            let Some(InputMode::CommandPreview { pending }) = state.modes.pop() else {
                return InputModeResult::Handled;
            };
            confirm_command_preview(state, source, pending);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Char('y'),
            modifiers: KeyModifiers::NONE,
            ..
        } => {
            copy_pending_command(state);
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

fn handle_jj_command_mode(
    state: &mut AppState,
    repository: Option<&Path>,
    key: KeyEvent,
) -> InputModeResult {
    match key {
        KeyEvent {
            code: KeyCode::Esc, ..
        } => {
            state.modes.pop();
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => {
            let should_close = match state.modes.active_mut() {
                Some(InputMode::JjCommand { input, error }) if input.is_empty() => {
                    *error = None;
                    true
                }
                Some(InputMode::JjCommand { input, error }) => {
                    input.pop();
                    *error = None;
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
            code: KeyCode::Enter,
            ..
        } => {
            submit_jj_command_mode(state, repository);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Char('u'),
            modifiers,
            ..
        } if modifiers == KeyModifiers::CONTROL => {
            if let Some(InputMode::JjCommand { input, error }) = state.modes.active_mut() {
                input.clear();
                *error = None;
            }
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Char(character),
            modifiers,
            ..
        } if !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            if let Some(InputMode::JjCommand { input, error }) = state.modes.active_mut() {
                input.push(character);
                *error = None;
            }
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

fn open_jj_command_mode(state: &mut AppState) {
    open_jj_command_mode_with_input(state, String::new());
}

fn open_jj_command_mode_with_input(state: &mut AppState, input: String) {
    state
        .modes
        .push(InputMode::JjCommand { input, error: None });
}

fn edit_command_output(state: &mut AppState) {
    let AppView::CommandOutput { input, .. } = state.views.active() else {
        return;
    };
    open_jj_command_mode_with_input(state, input.clone());
}

fn submit_jj_command_mode(state: &mut AppState, repository: Option<&Path>) {
    let input = match state.modes.active() {
        Some(InputMode::JjCommand { input, .. }) => input.clone(),
        _ => return,
    };

    match run_jj_command_mode_with_runner(state, repository, &input, SystemJjCommandRunner) {
        Ok(()) => {
            state.modes.pop();
        }
        Err(error) => {
            if let Some(InputMode::JjCommand {
                error: active_error,
                ..
            }) = state.modes.active_mut()
            {
                *active_error = Some(error);
            }
        }
    }
}

fn run_jj_command_mode_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    repository: Option<&Path>,
    input: &str,
    runner: R,
) -> std::result::Result<(), String> {
    let mut argv = parse_jj_command_args(input)?;
    if argv.first().is_some_and(|arg| arg == "jj") {
        argv.remove(0);
    }
    if argv.is_empty() {
        return Err("type a jj command after :".to_owned());
    }

    let spec = command_mode_spec(argv, repository);
    let command_line = spec.command_preview().command_line;
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        &mut state.history,
        CommandSource::new(
            SourceView::Other("command mode".to_owned()),
            SourceAction::UserJjCommand,
        )
        .with_key(":"),
    );
    let result = runner.run(&spec);
    let snapshot = command_mode_snapshot(&command_line, result.as_ref());
    state.views.push(AppView::CommandOutput {
        view: RenderedView::new(snapshot),
        input: input.trim().to_owned(),
    });
    Ok(())
}

fn open_describe_message(state: &mut AppState) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    let Some(rev) = log.selected_change_id().map(ToOwned::to_owned) else {
        log.show_error("No revision selected");
        return;
    };
    let message = log.selected_description().unwrap_or_default().to_owned();

    state
        .modes
        .push(InputMode::DescribeMessage { rev, message });
}

fn open_abandon_preview(state: &mut AppState, abandon_source: &JjAbandon) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    let Some(rev) = log.selected_change_id().map(ToOwned::to_owned) else {
        log.show_error("No revision selected");
        return;
    };

    let preview = abandon_source
        .spec_for(&AbandonQuery::new(rev))
        .command_preview();
    state.modes.push(InputMode::CommandPreview {
        pending: PendingCommandPreview::abandon(preview),
    });
}

fn open_new_preview(state: &mut AppState, new_source: &JjNew) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    let parents = selected_new_parents(log);
    if parents.is_empty() {
        log.show_error("No parent revision selected");
        return;
    }

    let preview = new_source
        .spec_for(&NewQuery::new(parents))
        .command_preview();
    state.modes.push(InputMode::CommandPreview {
        pending: PendingCommandPreview::new_change(preview),
    });
}

fn open_edit_preview(state: &mut AppState, edit_source: &JjEdit) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    let Some(rev) = log.selected_change_id().map(ToOwned::to_owned) else {
        log.show_error("No revision selected");
        return;
    };

    let preview = edit_source.spec_for(&EditQuery::new(rev)).command_preview();
    state.modes.push(InputMode::CommandPreview {
        pending: PendingCommandPreview::edit(preview),
    });
}

fn copy_pending_command(state: &mut AppState) {
    let Some(InputMode::CommandPreview { pending }) = state.modes.active_mut() else {
        return;
    };
    let status = copy_command_line(&pending.preview.command_line);
    pending.copy_status = Some(status);
}

fn copy_selected_command(state: &mut AppState) {
    let Some(action) = selected_command_copy_action(state) else {
        return;
    };

    let status = match action {
        CommandHistoryActionResult::CopyCommand { command_line } => {
            copy_command_line(&command_line)
        }
        _ => return,
    };
    if let AppView::CommandHistory { view } = state.views.active_mut() {
        view.show_status(status);
    }
}

fn selected_command_copy_action(state: &mut AppState) -> Option<CommandHistoryActionResult> {
    let AppView::CommandHistory { view } = state.views.active_mut() else {
        return None;
    };
    Some(view.apply(CommandHistoryAction::CopyCommand))
}

fn handle_view_options_mode(
    state: &mut AppState,
    source: &JjLog,
    diff_source: &JjDiff,
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
            move_view_options_selection(state, MenuDirection::Previous);
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
            move_view_options_selection(state, MenuDirection::Next);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            let selected = selected_view_option(state);
            state.modes.pop();
            match selected {
                Some(ViewOptionRow::LogTemplate) => {
                    open_template_selector(&mut state.modes, source);
                }
                Some(ViewOptionRow::DiffFormat(format)) => {
                    apply_diff_format_option(state, diff_source, format);
                }
                Some(ViewOptionRow::Placeholder) | None => {}
            }
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

fn handle_diff_file_list_mode(state: &mut AppState, key: KeyEvent) -> InputModeResult {
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
            move_diff_file_list_selection(state, MenuDirection::Previous);
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
            move_diff_file_list_selection(state, MenuDirection::Next);
            InputModeResult::Handled
        }
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => {
            apply_diff_file_list_selection(state);
            InputModeResult::Handled
        }
        _ => InputModeResult::Handled,
    }
}

fn open_diff_file_list(state: &mut AppState) {
    let AppView::Diff { view, .. } = state.views.active() else {
        return;
    };
    let selected = view.selected_file_index().unwrap_or_default();
    state.modes.push(InputMode::DiffFileList { selected });
}

fn move_diff_file_list_selection(state: &mut AppState, direction: MenuDirection) {
    let row_count = active_diff_file_count(state);
    let Some(InputMode::DiffFileList { selected }) = state.modes.active_mut() else {
        return;
    };
    if row_count == 0 {
        *selected = 0;
        return;
    }

    *selected = wrapped_selection(*selected, row_count, direction);
}

fn active_diff_file_count(state: &AppState) -> usize {
    match state.views.active() {
        AppView::Diff { view, .. } => view.file_count(),
        _ => 0,
    }
}

fn apply_diff_file_list_selection(state: &mut AppState) {
    let selected = match state.modes.active() {
        Some(InputMode::DiffFileList { selected }) => *selected,
        _ => return,
    };
    state.modes.pop();

    let AppView::Diff { view, .. } = state.views.active_mut() else {
        return;
    };
    view.select_file_index(selected);
}

fn apply_diff_format_option(state: &mut AppState, diff_source: &JjDiff, format: DiffFormat) {
    apply_diff_format_option_with_runner(state, diff_source, format, SystemJjCommandRunner);
}

fn apply_diff_format_option_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    diff_source: &JjDiff,
    format: DiffFormat,
    runner: R,
) {
    let new_query = match state.views.active() {
        AppView::Diff { query, .. } => query.with_format(format),
        _ => return,
    };

    let mut runner = RecordingJjCommandRunner::new(
        runner,
        &mut state.history,
        CommandSource::new(SourceView::Diff, SourceAction::Refresh),
    );
    let result = diff_source.load_query_with_runner(&new_query, &mut runner);

    let AppView::Diff { view, query } = state.views.active_mut() else {
        return;
    };
    match result {
        Ok(snapshot) => {
            *query = new_query;
            view.refresh(snapshot);
        }
        Err(error) => view.show_error(error.to_string()),
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
        selected: active_view_option_index(state),
    });
}

fn active_view_option_index(state: &AppState) -> usize {
    match state.views.active() {
        AppView::Diff { query, .. } => view_option_rows(BindingContext::Diff)
            .iter()
            .position(|row| *row == ViewOptionRow::DiffFormat(query.format()))
            .unwrap_or_default(),
        _ => 0,
    }
}

fn active_binding_context(state: &AppState) -> BindingContext {
    match state.views.active() {
        AppView::Log(_) => BindingContext::Log,
        AppView::Diff { .. } => BindingContext::Diff,
        AppView::Show { .. }
        | AppView::Evolog { .. }
        | AppView::Status { .. }
        | AppView::WorkspaceLog { .. }
        | AppView::WorkspaceStatus { .. }
        | AppView::WorkspaceDiff { .. }
        | AppView::OperationShow { .. }
        | AppView::OperationDiff { .. }
        | AppView::CommandOutput { .. }
        | AppView::CommandHistoryDetails { .. } => BindingContext::Inspection,
        AppView::Workspaces { .. } => BindingContext::Workspaces,
        AppView::CommandHistory { .. } => BindingContext::CommandHistory,
        AppView::OperationLog { .. } => BindingContext::OperationLog,
    }
}

fn move_command_discovery_selection(state: &mut AppState, direction: MenuDirection) {
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

    *selected = wrapped_selection(*selected, row_count, direction);
}

fn move_view_options_selection(state: &mut AppState, direction: MenuDirection) {
    let Some(InputMode::ViewOptions { context, selected }) = state.modes.active_mut() else {
        return;
    };
    let row_count = view_option_rows(*context).len();
    if row_count == 0 {
        *selected = 0;
        return;
    }

    *selected = wrapped_selection(*selected, row_count, direction);
}

fn selected_view_option(state: &AppState) -> Option<ViewOptionRow> {
    let Some(InputMode::ViewOptions { context, selected }) = state.modes.active() else {
        return None;
    };
    view_option_rows(*context).get(*selected).copied()
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
            move_template_selection(state, MenuDirection::Previous);
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
            move_template_selection(state, MenuDirection::Next);
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

fn move_template_selection(state: &mut AppState, direction: MenuDirection) {
    let Some(InputMode::LogTemplate { options, selected }) = state.modes.active_mut() else {
        return;
    };
    if options.is_empty() {
        *selected = 0;
        return;
    }

    *selected = wrapped_selection(*selected, options.len(), direction);
}

fn selected_template(state: &AppState) -> Option<LogTemplateSelection> {
    let Some(InputMode::LogTemplate { options, selected }) = state.modes.active() else {
        return None;
    };
    options.get(*selected).cloned()
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
        (
            AppView::WorkspaceStatus { view, .. }
            | AppView::WorkspaceLog { view, .. }
            | AppView::WorkspaceDiff { view, .. }
            | AppView::OperationShow { view, .. }
            | AppView::OperationDiff { view, .. }
            | AppView::CommandOutput { view, .. }
            | AppView::CommandHistoryDetails { view },
            SearchSubmit::Inspection(query),
        ) => {
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
        AppView::WorkspaceStatus { view, .. }
        | AppView::WorkspaceLog { view, .. }
        | AppView::WorkspaceDiff { view, .. }
        | AppView::OperationShow { view, .. }
        | AppView::OperationDiff { view, .. }
        | AppView::CommandOutput { view, .. }
        | AppView::CommandHistoryDetails { view } => {
            let action = match direction {
                SearchDirection::Next => RenderedAction::SearchNext,
                SearchDirection::Previous => RenderedAction::SearchPrevious,
            };
            let _ = view.apply(action);
        }
        AppView::Log(_)
        | AppView::Workspaces { .. }
        | AppView::CommandHistory { .. }
        | AppView::OperationLog { .. } => {}
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
    operation_source: &JjOperation,
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
            AppView::CommandHistoryDetails { view } => apply_static_rendered_action(view, action),
            AppView::CommandOutput { view, .. } => apply_static_rendered_action(view, action),
            AppView::OperationLog { view } => {
                apply_operation_log_action(view, history, operation_source, action)
            }
            AppView::WorkspaceStatus { view, query } => apply_workspace_inspection_action(
                view,
                query,
                history,
                workspaces_source,
                WorkspaceInspectionKind::Status,
                action,
            ),
            AppView::WorkspaceLog { view, query } => apply_workspace_inspection_action(
                view,
                query,
                history,
                workspaces_source,
                WorkspaceInspectionKind::Log,
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
            AppView::OperationShow { view, query } => apply_operation_rendered_action(
                view,
                query,
                history,
                operation_source,
                SourceView::OperationShow,
                action,
            ),
            AppView::OperationDiff { view, query } => apply_operation_rendered_action(
                view,
                query,
                history,
                operation_source,
                SourceView::OperationDiff,
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
        AppTransition::PushSelectedWorkspaceLog => {
            push_selected_workspace_log(state, workspaces_source);
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
        ActionResult::Refresh => {
            refresh_log(
                log,
                history,
                source,
                CommandSource::new(SourceView::Log, SourceAction::Refresh),
            );
        }
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
            rev: change_id,
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

fn push_selected_operation_show(state: &mut AppState, operation_source: &JjOperation) {
    let operation_id = {
        let AppView::OperationLog { view } = state.views.active_mut() else {
            return;
        };
        let Some(operation_id) = view.selected_operation_id().map(ToOwned::to_owned) else {
            return;
        };
        operation_id
    };
    let query = OperationQuery::show(operation_id);
    let transition = operation_rendered_transition(
        &mut state.history,
        operation_source,
        query,
        SourceView::OperationLog,
        SourceAction::OperationShow,
        OperationRenderedKind::Show,
    );
    if let AppTransition::Push(view) = transition {
        state.views.push(view);
    }
}

/// Applies an action while the operation log is active.
fn apply_operation_log_action(
    view: &mut OperationLogView,
    history: &mut CommandHistory,
    operation_source: &JjOperation,
    action: jk_tui::log_view::LogAction,
) -> AppTransition {
    let operation_action = match action {
        jk_tui::log_view::LogAction::Previous => OperationLogAction::Previous,
        jk_tui::log_view::LogAction::Next => OperationLogAction::Next,
        jk_tui::log_view::LogAction::ScrollPreviousLine => OperationLogAction::Previous,
        jk_tui::log_view::LogAction::ScrollNextLine => OperationLogAction::Next,
        jk_tui::log_view::LogAction::PagePrevious => OperationLogAction::PagePrevious,
        jk_tui::log_view::LogAction::PageNext | jk_tui::log_view::LogAction::ToggleMark => {
            OperationLogAction::PageNext
        }
        jk_tui::log_view::LogAction::First => OperationLogAction::First,
        jk_tui::log_view::LogAction::Last => OperationLogAction::Last,
        jk_tui::log_view::LogAction::Refresh => OperationLogAction::Refresh,
        jk_tui::log_view::LogAction::OpenDiff => OperationLogAction::OpenDiff,
        jk_tui::log_view::LogAction::ToggleHelp => OperationLogAction::ToggleHelp,
        jk_tui::log_view::LogAction::Quit => OperationLogAction::Quit,
        jk_tui::log_view::LogAction::Home
        | jk_tui::log_view::LogAction::Log
        | jk_tui::log_view::LogAction::CollapseExpanded => OperationLogAction::ReturnBack,
        _ => return AppTransition::Continue,
    };

    match view.apply(operation_action) {
        OperationLogActionResult::Refresh => refresh_operation_log(view, history, operation_source),
        OperationLogActionResult::OperationShow { operation_id } => {
            let query = OperationQuery::show(operation_id);
            return operation_rendered_transition(
                history,
                operation_source,
                query,
                SourceView::OperationLog,
                SourceAction::OperationShow,
                OperationRenderedKind::Show,
            );
        }
        OperationLogActionResult::OperationDiff { operation_id } => {
            let query = OperationQuery::diff(operation_id);
            return operation_rendered_transition(
                history,
                operation_source,
                query,
                SourceView::OperationLog,
                SourceAction::OperationDiff,
                OperationRenderedKind::Diff,
            );
        }
        OperationLogActionResult::ReturnBack => return AppTransition::PopView,
        OperationLogActionResult::Quit => return AppTransition::Quit,
        OperationLogActionResult::Continue => {}
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
    match view.apply(workspace_action_for_log_action(action)) {
        WorkspacesActionResult::Refresh => refresh_workspaces(view, history, workspaces_source),
        WorkspacesActionResult::OpenLog => {
            return AppTransition::PushSelectedWorkspaceLog;
        }
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
    match view.apply(workspace_inspection_action_for_log_action(action)) {
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

/// Applies an action while operation show or operation diff is active.
fn apply_operation_rendered_action(
    view: &mut RenderedView,
    query: &OperationQuery,
    history: &mut CommandHistory,
    operation_source: &JjOperation,
    source_view: SourceView,
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
            refresh_operation_rendered(view, query, history, operation_source, source_view);
        }
        RenderedActionResult::ReturnToLog => return AppTransition::PopView,
        RenderedActionResult::Quit => return AppTransition::Quit,
        RenderedActionResult::Continue => {}
        _ => {}
    }

    AppTransition::Continue
}

/// Applies an action while a static rendered in-memory view is active.
fn apply_static_rendered_action(
    view: &mut RenderedView,
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
            view.show_error("Command details are retained in memory; refresh Command History.");
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
pub(crate) enum AppTransition {
    Continue,
    Push(AppView),
    PopView,
    PushSelectedWorkspaceStatus,
    PushSelectedWorkspaceLog,
    PushSelectedWorkspaceDiff,
    OpenViewOptions,
    Quit,
}

fn open_template_selector(modes: &mut ModeStack, source: &JjLog) {
    let options = source.template_options();
    let selected = options
        .iter()
        .position(|template| template == source.template())
        .unwrap_or(0);
    modes.push(InputMode::LogTemplate { options, selected });
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
    use jk_tui::workspaces_view::WorkspaceViewRow;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::test_support::*;

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

        assert_eq!(state.views.len(), 2);
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

        assert_eq!(state.views.len(), 2);
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

        assert_eq!(state.views.len(), 2);
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
    fn command_history_enter_pushes_details_view() {
        let mut history = CommandHistory::new(4);
        append_history_record_with_operation_id(
            &mut history,
            jk_core::JjCommandSpec::render_read_only(["status"]),
            SourceView::Log,
            SourceAction::OpenStatus,
            Some("op-status"),
        );
        let mut state = AppState::with_history(
            AppView::CommandHistory {
                view: CommandHistoryView::new(command_history_snapshot(&history)),
            },
            history,
        );

        push_selected_command_history_details(&mut state);

        assert_eq!(state.views.len(), 2);
        assert!(matches!(
            state.views.active(),
            AppView::CommandHistoryDetails { .. }
        ));
    }

    #[test]
    fn static_command_details_refresh_shows_local_status() {
        let snapshot = jk_core::InspectionSnapshot::new("command 1", "Command: jj status\n")
            .with_title("Command 1");
        let mut view = RenderedView::new(snapshot);

        let transition =
            apply_static_rendered_action(&mut view, jk_tui::log_view::LogAction::Refresh);

        assert!(matches!(transition, AppTransition::Continue));
        let backend = TestBackend::new(92, 4);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };
        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());
        assert!(buffer_line(terminal.backend().buffer(), 3).contains("retained in memory"));
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
    fn command_history_operation_with_id_opens_operation_show() {
        let mut history = CommandHistory::new(4);
        append_history_record_with_operation_id(
            &mut history,
            jk_core::JjCommandSpec::render_read_only(["describe", "-m", "message", "@"]),
            SourceView::Log,
            SourceAction::DescribeRevision,
            Some("abc123op"),
        );
        let mut state = AppState::with_history(
            AppView::CommandHistory {
                view: CommandHistoryView::new(command_history_snapshot(&history)),
            },
            history,
        );
        let source = JjOperation::default();

        open_command_history_operation_with_runner(
            &mut state,
            &source,
            SequencedRunner::successes(vec![output(0, "operation details\n", "")]),
        );

        assert!(matches!(
            state.views.active(),
            AppView::OperationShow {
                query: OperationQuery::Show { operation },
                ..
            } if operation == "abc123op"
        ));
        let newest = state.command_history().records().last().expect("record");
        assert_eq!(newest.source.view, SourceView::CommandHistory);
        assert_eq!(newest.source.action, SourceAction::OperationShow);
        assert_eq!(newest.command.title, "jj op show abc123op");
    }

    #[test]
    fn command_history_operation_without_id_opens_operation_log() {
        let mut history = CommandHistory::new(4);
        append_history_record(
            &mut history,
            jk_core::JjCommandSpec::render_read_only(["log"]),
            SourceView::Log,
            SourceAction::InitialLoad,
        );
        let mut state = AppState::with_history(
            AppView::CommandHistory {
                view: CommandHistoryView::new(command_history_snapshot(&history)),
            },
            history,
        );
        let source = JjOperation::default();

        open_command_history_operation_with_runner(
            &mut state,
            &source,
            SequencedRunner::successes(vec![output(
                0,
                "@ abc123def456 user@example.test now\n│  current operation\n",
                "",
            )]),
        );

        assert!(matches!(state.views.active(), AppView::OperationLog { .. }));
        let newest = state.command_history().records().last().expect("record");
        assert_eq!(newest.source.view, SourceView::CommandHistory);
        assert_eq!(newest.source.action, SourceAction::OperationLog);
        assert_eq!(newest.command.title, "jj op log");
    }

    #[test]
    fn opening_workspaces_from_log_records_log_workspace_list() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let source = JjWorkspaces::default();
        let runner = SequencedRunner::successes(vec![
            output(0, "dogfood\t/repo/dogfood\tabc123\tdef456\n", ""),
            output(0, "/repo/dogfood\n", ""),
        ]);

        open_workspaces_with_runner(&mut state, &source, runner);

        assert_eq!(state.views.len(), 2);
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
            output(0, "dogfood\t/repo/dogfood\tabc123\tdef456\n", ""),
            output(0, "/repo/dogfood\n", ""),
        ]);

        open_workspaces_with_runner(&mut state, &source, runner);

        assert_eq!(state.views.len(), 1);
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

        assert_eq!(state.views.len(), 2);
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

        assert_eq!(state.views.len(), 2);
        assert_eq!(state.views.active(), &AppView::Evolog { view, query });
    }

    #[test]
    fn loaded_evolog_is_ignored_outside_log() {
        let root = diff_app_view("aaa");
        let mut state = AppState::new(root.clone());
        let query = EvologQuery::from("aaa".to_owned());
        let view = RenderedView::from_error("aaa", "jj evolog -r aaa", "fixture".to_owned());

        push_evolog_view(&mut state, query, view);

        assert_eq!(state.views.len(), 1);
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
    fn loaded_workspace_pushes_view() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let view = workspace_app_view();

        push_workspace_view(&mut state, view.clone());

        assert_eq!(state.views.len(), 2);
        assert_eq!(state.views.active(), &AppView::Workspaces { view });
    }

    #[test]
    fn w_on_active_workspace_list_does_not_stack_another_list() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        push_workspace_view(&mut state, workspace_app_view());
        let expected_depth = state.views.len();

        open_workspaces(
            &mut state,
            &JjWorkspaces::default().with_repository("/definitely/not-a-jj-repo"),
        );

        assert_eq!(state.views.len(), expected_depth);
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

        assert_eq!(state.views.len(), 1);
        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
    }

    #[test]
    fn workspace_missing_root_log_shows_error_without_pushing() {
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

        push_selected_workspace_log(&mut state, &source);

        assert_eq!(state.views.len(), 1);
        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
        );
    }

    #[test]
    fn selected_workspace_log_pushes_rendered_log_view() {
        let mut state = AppState::new(AppView::Workspaces {
            view: workspace_app_view(),
        });
        let source = JjWorkspaces::default();
        let runner =
            SequencedRunner::successes(vec![output(0, "@  abc123 selected workspace\n", "")]);

        push_selected_workspace_log_with_runner(&mut state, &source, runner);

        let AppView::WorkspaceLog { query, .. } = state.views.active() else {
            panic!("expected workspace log view");
        };
        assert_eq!(
            query.workspace_root(),
            std::path::Path::new("/repo/dogfood")
        );
        assert_eq!(state.command_history().records().count(), 1);
        let record = state.command_history().records().next().expect("record");
        assert_eq!(record.source.view, SourceView::Workspaces);
        assert_eq!(record.source.action, SourceAction::WorkspaceLog);
        assert_eq!(record.command.spec_preview, "jj log");
        assert_eq!(record.command.title, "jj -R /repo/dogfood log");
        assert_eq!(
            record.context.repository.as_deref(),
            Some(std::path::Path::new("/repo/dogfood"))
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

        assert_eq!(state.views.len(), 1);
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

        assert_eq!(state.views.len(), 1);
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

        assert_eq!(state.views.len(), 1);
        assert_eq!(
            state.views.active(),
            &AppView::Workspaces { view: expected }
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
            view: RenderedView::from_error("/repo/dogfood", "jj status", "fixture".to_owned()),
            query: WorkspaceInspectionQuery::new("/repo/dogfood"),
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
        assert_eq!(state.views.len(), 2);
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
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
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
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
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
    fn command_discovery_navigation_wraps_between_edges() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        state.modes.push(InputMode::CommandDiscovery {
            context: BindingContext::Log,
            query: String::new(),
            selected: 0,
        });
        let row_count = filtered_discovery_len(BindingContext::Log, "");

        move_command_discovery_selection(&mut state, MenuDirection::Previous);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::CommandDiscovery {
                context: BindingContext::Log,
                query: String::new(),
                selected: row_count - 1,
            })
        );

        move_command_discovery_selection(&mut state, MenuDirection::Next);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::CommandDiscovery {
                context: BindingContext::Log,
                query: String::new(),
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
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
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
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
    }

    #[test]
    fn describe_message_opens_for_selected_revision() {
        let mut state = AppState::new(log_app_view("abc123"));

        open_describe_message(&mut state);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::DescribeMessage {
                rev: "abc123".to_owned(),
                message: "abc123 summary".to_owned(),
            })
        );
    }

    #[test]
    fn describe_message_prefills_full_selected_description() {
        let mut state = AppState::new(log_app_view_with_description(
            "abc123",
            "Current summary\n\nCurrent body",
        ));

        open_describe_message(&mut state);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::DescribeMessage {
                rev: "abc123".to_owned(),
                message: "Current summary\n\nCurrent body".to_owned(),
            })
        );
    }

    #[test]
    fn describe_message_control_u_clears_prefilled_message() {
        let mut state = AppState::new(log_app_view("abc123"));
        open_describe_message(&mut state);
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(
            state.modes.active(),
            Some(&InputMode::DescribeMessage {
                rev: "abc123".to_owned(),
                message: String::new(),
            })
        );
    }

    #[test]
    fn describe_message_enter_opens_command_preview() {
        let mut state = AppState::new(log_app_view("abc123"));
        state.modes.push(InputMode::DescribeMessage {
            rev: "abc123".to_owned(),
            message: "New description".to_owned(),
        });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected command preview mode");
        };
        let preview = &pending.preview;
        assert_eq!(pending.source_action, SourceAction::DescribeRevision);
        assert_eq!(pending.source_key, "m");
        assert_eq!(preview.spec.argv()[0].to_string_lossy(), "describe");
        assert_eq!(
            preview.command_line,
            "jj --no-pager --color always describe -m 'New description' abc123"
        );
        assert_eq!(preview.safety, jk_core::SafetyClass::LocalRewrite);
    }

    #[test]
    fn describe_message_enter_ignores_empty_message() {
        let mut state = AppState::new(log_app_view("abc123"));
        state.modes.push(InputMode::DescribeMessage {
            rev: "abc123".to_owned(),
            message: "   ".to_owned(),
        });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(
            state.modes.active(),
            Some(&InputMode::DescribeMessage {
                rev: "abc123".to_owned(),
                message: "   ".to_owned(),
            })
        );
        assert_eq!(state.command_history().records().count(), 0);
    }

    #[test]
    fn abandon_preview_uses_selected_revision() {
        let mut state = AppState::new(log_app_view("abc123"));

        open_abandon_preview(&mut state, &JjAbandon::default());

        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected abandon command preview");
        };
        assert_eq!(pending.source_action, SourceAction::AbandonRevision);
        assert_eq!(pending.source_key, "a");
        assert_eq!(pending.failure_label, "jj abandon");
        assert_eq!(
            pending.preview.command_line,
            "jj --no-pager --color always abandon abc123"
        );
        assert_eq!(
            pending.preview.safety,
            jk_core::SafetyClass::DestructiveLocal
        );
        assert_eq!(
            pending.preview.warnings,
            vec![jk_core::CommandPreviewWarning::DestructiveLocal]
        );
    }

    #[test]
    fn new_preview_uses_selected_revision_as_parent() {
        let mut state = AppState::new(log_app_view("abc123"));

        open_new_preview(&mut state, &JjNew::default());

        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected new command preview");
        };
        assert_eq!(pending.source_action, SourceAction::NewRevision);
        assert_eq!(pending.source_key, "n");
        assert_eq!(pending.failure_label, "jj new");
        assert_eq!(
            pending.preview.command_line,
            "jj --no-pager --color always new abc123"
        );
        assert_eq!(pending.preview.safety, jk_core::SafetyClass::LocalRewrite);
        assert_eq!(
            pending.preview.warnings,
            vec![jk_core::CommandPreviewWarning::LocalRewrite]
        );
    }

    #[test]
    fn new_preview_uses_ordered_marks_as_parents() {
        let mut state = AppState::new(log_app_view_with_changes(["aaa", "bbb", "ccc"]));
        let AppView::Log(log) = state.views.active_mut() else {
            panic!("expected log");
        };
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::Next);
        let _ = log.apply(LogAction::ToggleMark);

        open_new_preview(&mut state, &JjNew::default());

        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected new command preview");
        };
        assert_eq!(
            pending.preview.command_line,
            "jj --no-pager --color always new aaa ccc"
        );
    }

    #[test]
    fn edit_preview_uses_selected_revision() {
        let mut state = AppState::new(log_app_view("abc123"));

        open_edit_preview(&mut state, &JjEdit::default());

        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected edit command preview");
        };
        assert_eq!(pending.source_action, SourceAction::EditRevision);
        assert_eq!(pending.source_key, "e");
        assert_eq!(pending.failure_label, "jj edit");
        assert_eq!(
            pending.preview.command_line,
            "jj --no-pager --color always edit abc123"
        );
        assert_eq!(pending.preview.safety, jk_core::SafetyClass::LocalRewrite);
        assert_eq!(
            pending.preview.warnings,
            vec![jk_core::CommandPreviewWarning::LocalRewrite]
        );
    }

    #[test]
    fn confirming_describe_preview_records_mutation_before_refresh() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut source = JjLog::default();
        let preview = JjDescribe::default()
            .spec_for(&DescribeQuery::new("abc123", "New description"))
            .command_preview();
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "Working copy now at: abc123\n", ""),
            output(0, "222222222222\n", ""),
            output(0, "refreshed rendered log\n", ""),
            output(0, "{}\n", ""),
        ]);

        confirm_command_preview_with_runner(
            &mut state,
            &mut source,
            PendingCommandPreview::describe(preview),
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(
            records[0].command.spec_preview,
            "jj describe -m 'New description' abc123"
        );
        assert_eq!(records[0].command.title, "jj describe abc123");
        assert_eq!(records[0].source.view, SourceView::Log);
        assert_eq!(records[0].source.action, SourceAction::DescribeRevision);
        assert_eq!(records[0].source.key.as_deref(), Some("m"));
        assert_eq!(records[0].safety, jk_core::SafetyClass::LocalRewrite);
        assert_eq!(
            records[0].execution_mode,
            jk_core::ExecutionMode::ConfirmMutation
        );
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        assert_eq!(records[0].refresh, jk_core::RefreshPlan::None);
        assert_eq!(records[1].source.action, SourceAction::Refresh);
        assert_eq!(records[2].source.action, SourceAction::Refresh);
        assert!(active_log_status(&mut state).contains(POST_MUTATION_RECOVERY_STATUS));
    }

    #[test]
    fn confirming_abandon_preview_records_destructive_mutation() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut source = JjLog::default();
        let preview = JjAbandon::default()
            .spec_for(&AbandonQuery::new("abc123"))
            .command_preview();
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "Abandoned 1 commits.\n", ""),
            output(0, "222222222222\n", ""),
            output(0, "refreshed rendered log\n", ""),
            output(0, "{}\n", ""),
        ]);

        confirm_command_preview_with_runner(
            &mut state,
            &mut source,
            PendingCommandPreview::abandon(preview),
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].command.spec_preview, "jj abandon abc123");
        assert_eq!(records[0].command.title, "jj abandon abc123");
        assert_eq!(records[0].source.view, SourceView::Log);
        assert_eq!(records[0].source.action, SourceAction::AbandonRevision);
        assert_eq!(records[0].source.key.as_deref(), Some("a"));
        assert_eq!(records[0].safety, jk_core::SafetyClass::DestructiveLocal);
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        assert_eq!(records[1].source.action, SourceAction::Refresh);
        assert_eq!(records[2].source.action, SourceAction::Refresh);
        assert!(active_log_status(&mut state).contains(POST_MUTATION_RECOVERY_STATUS));
    }

    #[test]
    fn confirming_new_preview_records_local_rewrite_mutation() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut source = JjLog::default();
        let preview = JjNew::default()
            .spec_for(&NewQuery::new(["abc123"]))
            .command_preview();
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "Working copy now at: def456\n", ""),
            output(0, "222222222222\n", ""),
            output(0, "refreshed rendered log\n", ""),
            output(0, "{}\n", ""),
        ]);

        confirm_command_preview_with_runner(
            &mut state,
            &mut source,
            PendingCommandPreview::new_change(preview),
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].command.spec_preview, "jj new abc123");
        assert_eq!(records[0].command.title, "jj new abc123");
        assert_eq!(records[0].source.view, SourceView::Log);
        assert_eq!(records[0].source.action, SourceAction::NewRevision);
        assert_eq!(records[0].source.key.as_deref(), Some("n"));
        assert_eq!(records[0].safety, jk_core::SafetyClass::LocalRewrite);
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        assert_eq!(records[1].source.action, SourceAction::Refresh);
        assert_eq!(records[2].source.action, SourceAction::Refresh);
        assert!(active_log_status(&mut state).contains(POST_MUTATION_RECOVERY_STATUS));
    }

    #[test]
    fn confirming_edit_preview_records_local_rewrite_mutation() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut source = JjLog::default();
        let preview = JjEdit::default()
            .spec_for(&EditQuery::new("abc123"))
            .command_preview();
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "Working copy now at: abc123\n", ""),
            output(0, "222222222222\n", ""),
            output(0, "refreshed rendered log\n", ""),
            output(0, "{}\n", ""),
        ]);

        confirm_command_preview_with_runner(
            &mut state,
            &mut source,
            PendingCommandPreview::edit(preview),
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].command.spec_preview, "jj edit abc123");
        assert_eq!(records[0].command.title, "jj edit abc123");
        assert_eq!(records[0].source.view, SourceView::Log);
        assert_eq!(records[0].source.action, SourceAction::EditRevision);
        assert_eq!(records[0].source.key.as_deref(), Some("e"));
        assert_eq!(records[0].safety, jk_core::SafetyClass::LocalRewrite);
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        assert_eq!(records[1].source.action, SourceAction::Refresh);
        assert_eq!(records[2].source.action, SourceAction::Refresh);
        assert!(active_log_status(&mut state).contains(POST_MUTATION_RECOVERY_STATUS));
    }

    #[test]
    fn command_history_after_confirmed_mutation_opens_resulting_operation() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut source = JjLog::default();
        let preview = JjDescribe::default()
            .spec_for(&DescribeQuery::new("abc123", "New description"))
            .command_preview();
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "Working copy now at: abc123\n", ""),
            output(0, "222222222222\n", ""),
            output(0, "refreshed rendered log\n", ""),
            output(0, "{}\n", ""),
        ]);

        confirm_command_preview_with_runner(
            &mut state,
            &mut source,
            PendingCommandPreview::describe(preview),
            runner,
        );
        open_command_history(&mut state);
        open_command_history_operation_with_runner(
            &mut state,
            &JjOperation::default(),
            SequencedRunner::successes(vec![output(0, "operation details\n", "")]),
        );

        assert!(matches!(
            state.views.active(),
            AppView::OperationShow {
                query: OperationQuery::Show { operation },
                ..
            } if operation == "222222222222"
        ));
    }

    #[test]
    fn undo_preview_uses_recovery_command_spec() {
        let mut state = AppState::new(log_app_view("abc123"));

        open_recovery_preview(&mut state, &JjRecovery::default(), RecoveryCommand::Undo);

        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected undo command preview");
        };
        assert_eq!(pending.source_action, SourceAction::Undo);
        assert_eq!(pending.source_key, "u");
        assert_eq!(
            pending.preview.command_line,
            "jj --no-pager --color always undo"
        );
        assert_eq!(pending.preview.safety, jk_core::SafetyClass::LocalRewrite);
    }

    #[test]
    fn redo_preview_uses_recovery_command_spec() {
        let mut state = AppState::new(log_app_view("abc123"));

        open_recovery_preview(&mut state, &JjRecovery::default(), RecoveryCommand::Redo);

        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected redo command preview");
        };
        assert_eq!(pending.source_action, SourceAction::Redo);
        assert_eq!(pending.source_key, "U");
        assert_eq!(
            pending.preview.command_line,
            "jj --no-pager --color always redo"
        );
    }

    #[test]
    fn command_mode_empty_enter_keeps_prompt_with_error() {
        let mut state = AppState::new(log_app_view("abc123"));
        open_jj_command_mode(&mut state);

        submit_jj_command_mode(&mut state, None);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::JjCommand {
                input: String::new(),
                error: Some("type a jj command after :".to_owned()),
            })
        );
        assert_eq!(state.views.len(), 1);
    }

    #[test]
    fn command_mode_runs_command_and_records_user_source() {
        let mut state = AppState::new(log_app_view("abc123"));

        run_jj_command_mode_with_runner(
            &mut state,
            Some(Path::new("/repo/dogfood")),
            "status",
            SequencedRunner::successes(vec![output(0, "clean\n", "")]),
        )
        .expect("command mode runs");

        assert!(matches!(
            state.views.active(),
            AppView::CommandOutput { .. }
        ));
        let record = state.command_history().records().next().expect("record");
        assert_eq!(
            record.command.command_family,
            jk_core::CommandFamily::UserJjCommand
        );
        assert_eq!(record.command.spec_preview, "jj status");
        assert_eq!(
            record.command.process_preview(),
            "jj --no-pager --color always --repository /repo/dogfood status"
        );
        assert_eq!(
            record.source.view,
            SourceView::Other("command mode".to_owned())
        );
        assert_eq!(record.source.action, SourceAction::UserJjCommand);
        assert_eq!(record.source.key.as_deref(), Some(":"));
        assert_eq!(record.execution_mode, jk_core::ExecutionMode::CommandMode);
    }

    #[test]
    fn command_output_edit_reopens_prompt_with_previous_input() {
        let mut state = AppState::new(log_app_view("abc123"));

        run_jj_command_mode_with_runner(
            &mut state,
            None,
            "jj log -r @",
            SequencedRunner::successes(vec![output(0, "rendered log\n", "")]),
        )
        .expect("command mode runs");
        edit_command_output(&mut state);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::JjCommand {
                input: "jj log -r @".to_owned(),
                error: None,
            })
        );
    }

    #[test]
    fn command_mode_accepts_optional_jj_prefix() {
        let mut state = AppState::new(log_app_view("abc123"));

        run_jj_command_mode_with_runner(
            &mut state,
            None,
            "jj status",
            SequencedRunner::successes(vec![output(0, "clean\n", "")]),
        )
        .expect("command mode runs");

        let record = state.command_history().records().next().expect("record");
        assert_eq!(record.command.spec_preview, "jj status");
    }

    #[test]
    fn confirming_undo_preview_records_recovery_action_before_refresh() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut source = JjLog::default();
        let preview = JjRecovery::default()
            .spec_for(RecoveryCommand::Undo)
            .command_preview();
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "undid\n", ""),
            output(0, "222222222222\n", ""),
            output(0, "refreshed rendered log\n", ""),
            output(0, "{}\n", ""),
        ]);

        confirm_command_preview_with_runner(
            &mut state,
            &mut source,
            PendingCommandPreview::undo(preview),
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].command.spec_preview, "jj undo");
        assert_eq!(records[0].source.action, SourceAction::Undo);
        assert_eq!(records[0].source.key.as_deref(), Some("u"));
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        assert_eq!(records[1].source.action, SourceAction::Refresh);
        assert!(active_log_status(&mut state).contains(POST_MUTATION_RECOVERY_STATUS));
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
    fn view_options_opens_on_active_diff_format() {
        let mut state = AppState::new(AppView::Diff {
            view: diff_view("aaa"),
            query: DiffQuery::Revision {
                rev: "aaa".to_owned(),
                format: DiffFormat::Summary,
            },
        });

        open_view_options(&mut state);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::ViewOptions {
                context: BindingContext::Diff,
                selected: 1,
            })
        );
    }

    #[test]
    fn view_options_navigation_wraps_between_edges() {
        let mut state = AppState::new(diff_app_view("aaa"));
        state.modes.push(InputMode::ViewOptions {
            context: BindingContext::Diff,
            selected: 0,
        });

        move_view_options_selection(&mut state, MenuDirection::Previous);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::ViewOptions {
                context: BindingContext::Diff,
                selected: view_option_rows(BindingContext::Diff).len() - 1,
            })
        );

        move_view_options_selection(&mut state, MenuDirection::Next);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::ViewOptions {
                context: BindingContext::Diff,
                selected: 0,
            })
        );
    }

    #[test]
    fn diff_file_list_opens_with_current_file_selected() {
        let mut view = real_diff_view("aaa");
        view.select_file_index(1);
        let mut state = AppState::new(AppView::Diff {
            view,
            query: diff_query("aaa"),
        });

        open_diff_file_list(&mut state);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::DiffFileList { selected: 1 })
        );
    }

    #[test]
    fn diff_file_list_navigation_wraps_between_edges() {
        let mut state = AppState::new(AppView::Diff {
            view: real_diff_view("aaa"),
            query: diff_query("aaa"),
        });
        state.modes.push(InputMode::DiffFileList { selected: 0 });

        move_diff_file_list_selection(&mut state, MenuDirection::Previous);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::DiffFileList { selected: 1 })
        );

        move_diff_file_list_selection(&mut state, MenuDirection::Next);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::DiffFileList { selected: 0 })
        );
    }

    #[test]
    fn diff_file_list_enter_jumps_to_selected_file() {
        let mut state = AppState::new(AppView::Diff {
            view: real_diff_view("aaa"),
            query: diff_query("aaa"),
        });
        state.modes.push(InputMode::DiffFileList { selected: 0 });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        assert_eq!(result, InputModeResult::Handled);

        let result = handle_input_mode(
            &mut state,
            &mut source,
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
        let AppView::Diff { view, .. } = state.views.active() else {
            panic!("expected diff view");
        };
        assert_eq!(view.selected_file_index(), Some(1));
    }

    #[test]
    fn diff_file_list_lines_mark_selected_path() {
        let view = real_diff_view("aaa");

        assert_eq!(
            diff_file_list_lines(&view, 1),
            vec![
                "   1/2 src/a.rs",
                ">  2/2 src/b.rs",
                "",
                "j/k or arrows move   enter jump   esc close",
            ]
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
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
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
    fn template_selection_navigation_wraps_between_edges() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let options = JjLog::default().template_options();
        state.modes.push(InputMode::LogTemplate {
            options: options.clone(),
            selected: 0,
        });

        move_template_selection(&mut state, MenuDirection::Previous);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::LogTemplate {
                options: options.clone(),
                selected: options.len() - 1,
            })
        );

        move_template_selection(&mut state, MenuDirection::Next);

        assert_eq!(
            state.modes.active(),
            Some(&InputMode::LogTemplate {
                options,
                selected: 0,
            })
        );
    }

    #[test]
    fn view_options_placeholder_enter_closes_inspection_overlay() {
        let snapshot =
            jk_core::InspectionSnapshot::new("show", "rendered show\n").with_title("jj show aaa");
        let mut state = AppState::new(AppView::Show {
            view: RenderedView::new(snapshot),
            query: ShowQuery::new(vec!["aaa".to_owned()]),
        });
        state.modes.push(InputMode::ViewOptions {
            context: BindingContext::Inspection,
            selected: 0,
        });
        let mut source = JjLog::default();

        let result = handle_input_mode(
            &mut state,
            &mut source,
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );

        assert_eq!(result, InputModeResult::Handled);
        assert_eq!(state.modes.active(), None);
    }

    #[test]
    fn diff_view_options_apply_selected_format() {
        let mut state = AppState::new(diff_app_view("aaa"));

        apply_diff_format_option_with_runner(
            &mut state,
            &JjDiff::default(),
            DiffFormat::Summary,
            SequencedRunner::successes(vec![
                output(0, "M src/a.rs\n", ""),
                output(0, "src/a.rs | 1 +\n", ""),
            ]),
        );

        assert!(matches!(
            state.views.active(),
            AppView::Diff {
                query: DiffQuery::Revision { rev, format },
                ..
            } if rev == "aaa" && *format == DiffFormat::Summary
        ));
        let newest = state.command_history().records().last().expect("record");
        assert_eq!(newest.command.title, "jj diff -r aaa --stat");
        assert_eq!(newest.source.view, SourceView::Diff);
        assert_eq!(newest.source.action, SourceAction::Refresh);
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
            &JjDiff::default(),
            &JjDescribe::default(),
            None,
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
                None,
            ),
            vec![
                "> Template           full description",
                "",
                "j/k or arrows move   enter open   esc close",
            ]
        );
        assert_eq!(
            view_options_lines(
                BindingContext::Diff,
                2,
                &LogTemplateSelection::Configured,
                Some(DiffFormat::Stat),
            ),
            vec![
                "    patch         ",
                "    summary       ",
                "> * stat          ",
                "    types         ",
                "    name only     ",
                "    git           ",
                "    color words   ",
                "",
                "j/k or arrows move   enter apply   esc close",
            ]
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
    fn workspaces_is_a_root_cli_command() {
        let args = Args::try_parse_from(["jk", "workspaces"]).expect("valid workspaces command");

        assert!(matches!(args.command, Some(Command::Workspaces)));
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
    fn diff_args_resolve_supported_format_flags() {
        for (flag, format) in [
            ("--summary", DiffFormat::Summary),
            ("--types", DiffFormat::Types),
            ("--name-only", DiffFormat::NameOnly),
            ("--git", DiffFormat::Git),
            ("--color-words", DiffFormat::ColorWords),
        ] {
            let args = Args::try_parse_from(["jk", "diff", "-r", "abc123", flag])
                .expect("valid diff args");
            let Some(Command::Diff(diff_args)) = args.command else {
                panic!("expected diff command");
            };

            assert_eq!(
                diff_args.query(),
                DiffQuery::Revision {
                    rev: "abc123".to_owned(),
                    format,
                }
            );
        }
    }

    #[test]
    fn diff_args_reject_multiple_format_flags() {
        assert!(Args::try_parse_from(["jk", "diff", "--stat", "--summary"]).is_err());
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
        let source = args.workspaces_source();
        let spec = source.list_spec();

        assert_eq!(spec.repository(), None);
    }
}
