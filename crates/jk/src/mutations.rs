use jk_cli::{
    JjCommandRunner, JjLog, JjRecovery, RecordingJjCommandRunner, RecoveryCommand,
    SystemJjCommandRunner,
};
use jk_core::{CommandSource, SourceAction, SourceView};

use crate::mutation_preview::{PendingCommandPreview, command_failure_message};
use crate::state::{AppState, AppView, InputMode};

pub const POST_MUTATION_RECOVERY_STATUS: &str = "u undo  U redo  o operation  C history";

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
    let command_source = CommandSource::new(SourceView::Log, pending.source_action.clone())
        .with_key(pending.source_key);
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, command_source);
    let result = runner.run_confirmed_mutation(&pending.preview.spec);
    let runner = runner.into_inner();
    match result {
        Ok(output) if output.status.success() => {
            refresh_after_mutation_with_runner(state, source, runner)
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

pub fn open_recovery_preview(
    state: &mut AppState,
    recovery_source: &JjRecovery,
    command: RecoveryCommand,
) {
    if !matches!(state.views.active(), AppView::Log(_)) {
        return;
    }

    let preview = recovery_source.spec_for(command).command_preview();
    let pending = match command {
        RecoveryCommand::Undo => PendingCommandPreview::undo(preview),
        RecoveryCommand::Redo => PendingCommandPreview::redo(preview),
    };
    state.modes.push(InputMode::CommandPreview { pending });
}

fn refresh_after_mutation_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &mut JjLog,
    runner: R,
) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };

    let refreshed = crate::refresh::refresh_log_with_runner(
        log,
        &mut state.history,
        source,
        CommandSource::new(SourceView::Log, SourceAction::Refresh),
        runner,
    );
    if refreshed {
        log.show_status(POST_MUTATION_RECOVERY_STATUS);
    }
}

fn show_log_error(state: &mut AppState, message: String) {
    if let AppView::Log(log) = state.views.active_mut() {
        log.show_error(message);
    }
}
