use jk_cli::{
    AbandonQuery, JjAbandon, JjBookmarks, JjCommandRunner, JjLog, JjRecovery, JjRestore,
    RecordingJjCommandRunner, RecoveryCommand, RestoreQuery,
};
use jk_core::{CommandSource, OperationLoadPolicy, SourceAction, SourceView, WorkingCopyPolicy};

use crate::abandon_confirmation::AbandonConfirmation;
use crate::mutation_preview::{
    PendingCommandPreview, command_failure_message, new_change_id_from_output,
};
use crate::state::{AppState, AppView, InputMode};

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
    state.cancel_log_refresh();
    let command_source = CommandSource::new(SourceView::Log, pending.source_action.clone())
        .with_key(pending.source_key);
    let reselect_change_id = pending.reselect_change_id.clone();
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, command_source);
    let result = runner.run_confirmed_mutation(&pending.preview.spec);
    let runner = runner.into_inner();
    match result {
        Ok(output) if output.status.success() => {
            let new_change_id = (pending.source_action == SourceAction::NewRevision)
                .then(|| new_change_id_from_output(&output.stderr))
                .flatten();
            let message =
                crate::run_options::success_message(&pending.preview.spec, pending.success_message);
            let options = pending.preview.spec.global_options();
            let mut refresh_source = source.clone();
            if options.working_copy() == WorkingCopyPolicy::Ignore
                || matches!(options.operation(), OperationLoadPolicy::AtOperation(_))
            {
                refresh_source = refresh_source.with_working_copy(WorkingCopyPolicy::Ignore);
            }
            let refreshed =
                refresh_after_mutation_with_runner(state, &refresh_source, runner, message);
            if refreshed && pending.source_action == SourceAction::NewRevision {
                select_new_change(state, new_change_id.as_deref());
            }
            if refreshed && let Some(change_id) = reselect_change_id.as_deref() {
                select_change(state, change_id);
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

/// Opens a destructive preview that restores all paths from one exact commit into `@`.
pub fn open_restore_preview(state: &mut AppState, restore_source: &JjRestore) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    if log.has_marks() {
        log.show_error("Restore needs one source revision; clear revision marks first");
        return;
    }
    let Some(source) = log.selected_commit_id().map(ToOwned::to_owned) else {
        log.show_error("No source revision selected");
        return;
    };

    let query = RestoreQuery::all_paths(source.clone(), "@");
    let preview = restore_source.spec_for(&query).command_preview();
    state.modes.push(InputMode::CommandPreview {
        pending: PendingCommandPreview::restore(preview, &source),
    });
}

/// Executes a confirmed local bookmark mutation and refreshes the bookmark list.
pub fn confirm_bookmark_command_preview(
    state: &mut AppState,
    bookmarks_source: &JjBookmarks,
    pending: PendingCommandPreview,
) {
    confirm_bookmark_command_preview_with_runner(
        state,
        bookmarks_source,
        pending,
        crate::runner::system_runner(),
    );
}

fn confirm_bookmark_command_preview_with_runner(
    state: &mut AppState,
    bookmarks_source: &JjBookmarks,
    pending: PendingCommandPreview,
    runner: impl JjCommandRunner,
) {
    state.cancel_log_refresh();
    let command_source = CommandSource::new(SourceView::Bookmarks, pending.source_action.clone())
        .with_key(pending.source_key);
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, command_source);
    let result = runner.run_confirmed_mutation(&pending.preview.spec);
    let runner = runner.into_inner();
    match result {
        Ok(output) if output.status.success() => {
            if matches!(
                state.modes.active(),
                Some(InputMode::BookmarkMutation { .. })
            ) {
                state.modes.pop();
            }
            crate::bookmark_routes::refresh_bookmarks_with_runner(state, bookmarks_source, runner)
        }
        Ok(output) => {
            let message =
                command_failure_message(pending.failure_label, &output.stderr, &output.stdout);
            if let Some(InputMode::BookmarkMutation { error, .. }) = state.modes.active_mut() {
                *error = Some(message.clone());
            }
            if let AppView::Bookmarks { view } = state.views.active_mut() {
                view.show_error(message);
            }
        }
        Err(error) => {
            if let Some(InputMode::BookmarkMutation {
                error: prompt_error,
                ..
            }) = state.modes.active_mut()
            {
                *prompt_error = Some(format!("failed to run {}: {error}", pending.failure_label));
            }
            if let AppView::Bookmarks { view } = state.views.active_mut() {
                view.show_error(format!("failed to run {}: {error}", pending.failure_label));
            }
        }
    }
}

/// Executes a confirmed fetch or push dry-run and displays captured output.
pub fn confirm_remote_command_preview(
    state: &mut AppState,
    bookmarks_source: &JjBookmarks,
    pending: PendingCommandPreview,
) {
    confirm_remote_command_preview_with_runner(
        state,
        bookmarks_source,
        pending,
        crate::runner::system_runner(),
    );
}

fn confirm_remote_command_preview_with_runner(
    state: &mut AppState,
    bookmarks_source: &JjBookmarks,
    pending: PendingCommandPreview,
    runner: impl JjCommandRunner,
) {
    state.cancel_log_refresh();
    let command_source = CommandSource::new(SourceView::Bookmarks, pending.source_action.clone())
        .with_key(pending.source_key);
    let mut runner = RecordingJjCommandRunner::new(runner, &mut state.history, command_source);
    let command_line = pending.preview.command_line.clone();
    let result = runner.run_confirmed_mutation(&pending.preview.spec);
    let snapshot = match &result {
        Ok(output) => crate::command_mode::command_mode_snapshot(&command_line, Ok(output)),
        Err(error) => crate::command_mode::command_mode_snapshot(&command_line, Err(error)),
    };
    let runner = runner.into_inner();
    if pending.source_action == SourceAction::GitFetch {
        crate::bookmark_routes::refresh_bookmarks_with_runner(state, bookmarks_source, runner);
    }
    state.views.push(AppView::CommandOutput {
        view: jk_tui::rendered_view::RenderedView::new(snapshot),
        input: command_line,
        kind: crate::state::CommandInputKind::Jj,
    });
}

/// Executes an empty revision's abandon immediately, or opens a destructive preview otherwise.
///
/// The emptiness probe snapshots working-copy edits before checking. If it fails, this takes the
/// conservative path and opens the same destructive preview used for a non-empty revision. The user
/// can inspect or cancel instead of having an uncertain probe turn into an immediate mutation.
pub fn abandon_or_preview(state: &mut AppState, source: &mut JjLog, abandon_source: &JjAbandon) {
    abandon_or_preview_with_runner(
        state,
        source,
        abandon_source,
        crate::runner::system_runner(),
    );
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
    let query = AbandonQuery::new(&rev);
    let probe = abandon_source.is_empty_with_runner(&query, &mut runner);
    if matches!(probe, Ok(true)) {
        execute_pending_command_with_runner(
            state,
            source,
            PendingCommandPreview::abandon(abandon_source.spec_for(&query).command_preview()),
            runner,
        );
        return;
    }
    let details = abandon_source
        .details_with_runner(&query, &mut runner)
        .map_err(|error| error.to_string());
    state.modes.push(InputMode::AbandonConfirmation {
        pending: PendingCommandPreview::abandon(abandon_source.spec_for(&query).command_preview()),
        dialog: Box::new(AbandonConfirmation::new(rev, details, probe.is_err())),
    });
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
        crate::runner::system_runner(),
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
    source: &JjLog,
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

fn select_change(state: &mut AppState, change_id: &str) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    let _ = log.select_change_id(change_id);
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

#[cfg(test)]
mod working_copy_tests {
    use jk_core::{CommandHistory, GlobalOptions, JjCommandSpec, SafetyClass};

    use super::*;
    use crate::test_support::{SequencedRunner, log_app_view, output};

    #[test]
    fn post_command_refresh_respects_ignore_without_changing_later_manual_refresh() {
        let policies = [
            (GlobalOptions::default(), false),
            (
                GlobalOptions::default().with_working_copy(WorkingCopyPolicy::Ignore),
                true,
            ),
            (
                GlobalOptions::default()
                    .with_operation(OperationLoadPolicy::AtOperation("012345abcdef".to_owned())),
                true,
            ),
        ];
        for (options, ignore_refresh) in policies {
            let original = JjLog::default();
            let mut source = original.clone();
            let mut state = AppState::new(log_app_view("selected"));
            let spec = JjCommandSpec::confirm_mutation(
                ["rebase", "-r", "source", "-o", "destination"],
                SafetyClass::LocalRewrite,
            )
            .with_global_options(options);
            let runner = SequencedRunner::successes(vec![
                output(0, "111111111111\n", ""),
                output(0, "", ""),
                output(0, "222222222222\n", ""),
                output(0, "", ""),
                output(0, "", ""),
            ]);

            execute_pending_command_with_runner(
                &mut state,
                &mut source,
                PendingCommandPreview::rebase(spec.command_preview(), Vec::new()),
                runner,
            );

            let refreshes = state
                .history
                .records()
                .filter(|record| record.source.action == SourceAction::Refresh)
                .collect::<Vec<_>>();
            assert_eq!(refreshes.len(), 2);
            for record in refreshes {
                assert_eq!(
                    record
                        .command
                        .argv
                        .iter()
                        .any(|arg| arg == "--ignore-working-copy"),
                    ignore_refresh
                );
                assert!(
                    !record
                        .command
                        .argv
                        .iter()
                        .any(|arg| arg == "--at-operation")
                );
            }
            assert_eq!(source, original);

            let mut history = CommandHistory::default();
            let runner = SequencedRunner::successes(vec![output(0, "", ""), output(0, "", "")]);
            let mut recorder = RecordingJjCommandRunner::new(
                runner,
                &mut history,
                CommandSource::new(SourceView::Log, SourceAction::Refresh),
            );
            source
                .load_with_runner(&mut recorder)
                .expect("manual refresh succeeds");
            for record in history.records() {
                assert!(
                    !record
                        .command
                        .argv
                        .iter()
                        .any(|arg| arg == "--ignore-working-copy")
                );
                assert!(!record.command.argv.iter().any(|arg| arg == "log"));
            }
        }
    }
}

#[cfg(test)]
mod refs_tests {
    use jk_cli::JjGitRemote;
    use jk_tui::bookmark_view::{BookmarkRow, BookmarkView, BookmarkViewSnapshot};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::test_support::{SequencedRunner, output};

    #[test]
    fn failed_bookmark_create_keeps_input_and_jj_diagnostic() {
        let mut state = AppState::new(AppView::Bookmarks {
            view: BookmarkView::new(BookmarkViewSnapshot::new(Vec::new())),
        });
        crate::bookmark_routes::open_bookmark_create_prompt(&mut state);
        if let Some(InputMode::BookmarkMutation { name, .. }) = state.modes.active_mut() {
            *name = "existing".to_owned();
        }
        let source = JjBookmarks::default();
        let spec = source.mutation_spec(&jk_cli::BookmarkMutation::Create {
            name: "existing".to_owned(),
            revision: "@".to_owned(),
        });
        confirm_bookmark_command_preview_with_runner(
            &mut state,
            &source,
            PendingCommandPreview::bookmark_create(spec.command_preview()),
            SequencedRunner::successes(vec![
                output(0, "111111111111\n", ""),
                output(1, "", "Bookmark already exists: existing"),
            ]),
        );
        assert!(
            matches!(state.modes.active(), Some(InputMode::BookmarkMutation { name, revision, error: Some(error), .. }) if name == "existing" && revision == "@" && error.contains("Bookmark already exists: existing"))
        );
        assert_eq!(state.history.records().count(), 1);
    }

    #[test]
    fn failed_fetch_retains_output_history_and_last_bookmark_snapshot() {
        let mut state = AppState::new(AppView::Bookmarks {
            view: BookmarkView::new(BookmarkViewSnapshot::new(vec![BookmarkRow::new(
                "topic",
                vec!["before".into()],
            )])),
        });
        let pending = PendingCommandPreview::git_fetch(
            JjGitRemote::default()
                .fetch_spec("fixture")
                .command_preview(),
        );
        confirm_remote_command_preview_with_runner(
            &mut state,
            &JjBookmarks::default(),
            pending,
            SequencedRunner::successes(vec![
                output(0, "111111111111\n", ""),
                output(1, "partial output", "fixture unavailable"),
                output(1, "", "refresh unavailable"),
            ]),
        );
        assert_eq!(state.history.records().count(), 2);
        assert_eq!(
            state
                .history
                .records()
                .next()
                .expect("fetch record")
                .source
                .action,
            SourceAction::GitFetch
        );
        let AppView::CommandOutput { view, .. } = state.views.active_mut() else {
            panic!("output");
        };
        let mut terminal = Terminal::new(TestBackend::new(100, 20)).unwrap();
        terminal.draw(|frame| view.render(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = (0..20)
            .map(|y| crate::test_support::buffer_line(buffer, y))
            .collect::<String>();
        assert!(rendered.contains("fixture unavailable"));
        assert!(rendered.contains("partial output"));
        state.views.pop();
        let AppView::Bookmarks { view } = state.views.active() else {
            panic!("bookmarks");
        };
        assert_eq!(
            view.selected_row().expect("preserved bookmark").targets,
            ["before"]
        );
    }

    #[test]
    fn successful_fetch_refreshes_bookmarks_before_showing_output() {
        let mut state = AppState::new(AppView::Bookmarks {
            view: BookmarkView::new(BookmarkViewSnapshot::new(Vec::new())),
        });
        let pending = PendingCommandPreview::git_fetch(
            JjGitRemote::default()
                .fetch_spec("fixture")
                .command_preview(),
        );
        confirm_remote_command_preview_with_runner(
            &mut state,
            &JjBookmarks::default(),
            pending,
            SequencedRunner::successes(vec![
                output(0, "111111111111\n", ""),
                output(0, "", "fetched"),
                output(0, "222222222222\n", ""),
                output(0, "{\"name\":\"topic\",\"target\":[\"after\"]}\n", ""),
            ]),
        );
        assert!(matches!(
            state.views.active(),
            AppView::CommandOutput { .. }
        ));
        assert_eq!(
            state
                .history
                .records()
                .next()
                .expect("fetch history")
                .operation_id
                .as_deref(),
            Some("222222222222")
        );
        state.views.pop();
        let AppView::Bookmarks { view } = state.views.active() else {
            panic!("bookmarks");
        };
        assert_eq!(
            view.selected_row().expect("fetched bookmark").targets,
            ["after"]
        );
    }
}
