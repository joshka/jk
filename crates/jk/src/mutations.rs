use jk_cli::{
    AbandonQuery, JjAbandon, JjCommandRunner, JjLog, JjRecovery, RecordingJjCommandRunner,
    RecoveryCommand, SystemJjCommandRunner,
};
use jk_core::{CommandSource, SourceAction, SourceView};

use crate::mutation_preview::{
    PendingCommandPreview, command_failure_message, new_change_id_from_output,
};
use crate::state::{AppState, AppView, InputMode};

pub fn confirm_command_preview(
    state: &mut AppState,
    source: &mut JjLog,
    pending: PendingCommandPreview,
) {
    confirm_command_preview_with_runner(state, source, pending, SystemJjCommandRunner);
}

pub fn confirm_command_preview_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &mut JjLog,
    pending: PendingCommandPreview,
    runner: R,
) {
    execute_pending_command_with_runner(state, source, pending, runner);
}

pub(crate) fn execute_pending_command_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &mut JjLog,
    pending: PendingCommandPreview,
    runner: R,
) {
    let command_source = CommandSource::new(SourceView::Log, pending.source_action.clone())
        .with_key(pending.source_key);
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, command_source);
    let result = runner.run_confirmed_mutation(&pending.preview.spec);
    let runner = runner.into_inner();
    match result {
        Ok(output) if output.status.success() => {
            let new_change_id = (pending.source_action == SourceAction::NewRevision)
                .then(|| new_change_id_from_output(&output.stderr))
                .flatten();
            let refreshed =
                refresh_after_mutation_with_runner(state, source, runner, pending.success_message);
            if refreshed && pending.source_action == SourceAction::NewRevision {
                select_new_change(state, new_change_id.as_deref());
            }
        }
        Ok(output) => {
            let message =
                command_failure_message(pending.failure_label, &output.stderr, &output.stdout);
            show_log_error(state, message);
        }
        Err(error) => {
            show_log_error(
                state,
                format!("failed to run {}: {error}", pending.failure_label),
            );
        }
    }
}

/// Executes an empty revision's abandon immediately, or opens a destructive preview otherwise.
///
/// The emptiness probe snapshots working-copy edits before checking. If it fails, this takes the
/// conservative path and opens the same destructive preview used for a non-empty revision. The user
/// can inspect or cancel instead of having an uncertain probe turn into an immediate mutation.
pub fn abandon_or_preview(state: &mut AppState, source: &mut JjLog, abandon_source: &JjAbandon) {
    abandon_or_preview_with_runner(state, source, abandon_source, SystemJjCommandRunner);
}

pub(crate) fn abandon_or_preview_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &mut JjLog,
    abandon_source: &JjAbandon,
    mut runner: R,
) {
    let Some(rev) = selected_revision_id(state) else {
        show_log_error(state, "No revision selected".to_owned());
        return;
    };
    let query = AbandonQuery::new(rev);
    let pending = || InputMode::CommandPreview {
        pending: PendingCommandPreview::abandon(abandon_source.spec_for(&query).command_preview()),
    };
    match abandon_source.is_empty_with_runner(&query, &mut runner) {
        Ok(true) => execute_pending_command_with_runner(
            state,
            source,
            PendingCommandPreview::abandon(abandon_source.spec_for(&query).command_preview()),
            runner,
        ),
        Ok(false) => state.modes.push(pending()),
        Err(error) => {
            show_log_error(state, error.to_string());
            state.modes.push(pending());
        }
    }
}

/// Runs an operation recovery command through the recorded mutation path.
pub fn execute_recovery_action(
    state: &mut AppState,
    source: &mut JjLog,
    recovery_source: &JjRecovery,
    command: RecoveryCommand,
) {
    execute_recovery_action_with_runner(
        state,
        source,
        recovery_source,
        command,
        SystemJjCommandRunner,
    );
}

pub(crate) fn execute_recovery_action_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &mut JjLog,
    recovery_source: &JjRecovery,
    command: RecoveryCommand,
    runner: R,
) {
    if !matches!(state.views.active(), AppView::Log(_)) {
        return;
    }

    let preview = recovery_source.spec_for(command).command_preview();
    let pending = match command {
        RecoveryCommand::Undo => PendingCommandPreview::undo(preview),
        RecoveryCommand::Redo => PendingCommandPreview::redo(preview),
    };
    execute_pending_command_with_runner(state, source, pending, runner);
}

fn refresh_after_mutation_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &mut JjLog,
    runner: R,
    success_message: &'static str,
) -> bool {
    let AppView::Log(log) = state.views.active_mut() else {
        return false;
    };

    let refreshed = crate::refresh::refresh_log_with_runner(
        log,
        &mut state.history,
        source,
        CommandSource::new(SourceView::Log, SourceAction::Refresh),
        runner,
    );
    if refreshed {
        state.show_toast(success_message);
    }
    refreshed
}

fn select_new_change(state: &mut AppState, change_id: Option<&str>) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    if change_id.is_some_and(|change_id| log.select_change_id(change_id)) {
        return;
    }
    log.select_first();
}

fn selected_revision_id(state: &AppState) -> Option<String> {
    let AppView::Log(log) = state.views.active() else {
        return None;
    };
    log.selected_revision_id().map(ToOwned::to_owned)
}

fn show_log_error(state: &mut AppState, message: String) {
    if let AppView::Log(log) = state.views.active_mut() {
        log.show_error(message);
    }
}
