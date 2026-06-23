use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, SystemTime};

use super::*;
use crate::{
    ConfigOverlay, ExecutionMode, GlobalOptions, JjCommandSpec, OutputPolicy, PagerPolicy,
    SafetyClass,
};

fn strings(argv: &[OsString]) -> Vec<String> {
    argv.iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

fn source(view: SourceView, action: SourceAction) -> CommandSource {
    CommandSource::new(view, action)
}

fn started_at() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_000)
}

fn finish_at() -> SystemTime {
    started_at() + Duration::from_millis(42)
}

fn start_from_spec(spec: &JjCommandSpec, source: CommandSource) -> CommandRecordStart {
    CommandRecordStart::from_spec(spec, source).with_started_at(started_at())
}

#[test]
fn starting_and_finishing_record_stores_completed_result() {
    let spec = JjCommandSpec::render_read_only(["status"]);
    let mut history = CommandHistory::new(4);
    let pending = history.start(start_from_spec(
        &spec,
        source(SourceView::Status, SourceAction::InitialLoad),
    ));
    let pending_id = pending.id();

    assert!(history.finish(
        &pending,
        CommandRecordFinish::from_exit_code(0, "clean\n", "", finish_at()),
    ));

    let record = history.records().next().expect("record");
    assert_eq!(record.id, pending_id);
    assert_eq!(record.result.exit_status, Some(ExitStatusSummary::code(0)));
    assert_eq!(record.result.stdout.snippet, "clean\n");
    assert_eq!(record.timing.duration, Some(Duration::from_millis(42)));
}

#[test]
fn record_ids_are_monotonic_across_eviction() {
    let spec = JjCommandSpec::render_read_only(["log"]);
    let mut history = CommandHistory::new(2);

    let first = history.start(start_from_spec(
        &spec,
        source(SourceView::Log, SourceAction::InitialLoad),
    ));
    let second = history.start(start_from_spec(
        &spec,
        source(SourceView::Log, SourceAction::Refresh),
    ));
    let third = history.start(start_from_spec(
        &spec,
        source(SourceView::Log, SourceAction::Refresh),
    ));

    assert_eq!(first.id().get(), 1);
    assert_eq!(second.id().get(), 2);
    assert_eq!(third.id().get(), 3);
    assert_eq!(
        history
            .records()
            .map(|record| record.id.get())
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
}

#[test]
fn history_limit_drops_oldest_records_first() {
    let spec = JjCommandSpec::render_read_only(["status"]);
    let mut history = CommandHistory::new(2);

    history.append(
        start_from_spec(&spec, source(SourceView::Status, SourceAction::InitialLoad)),
        CommandRecordFinish::from_exit_code(0, "one", "", finish_at()),
    );
    history.append(
        start_from_spec(&spec, source(SourceView::Status, SourceAction::Refresh)),
        CommandRecordFinish::from_exit_code(0, "two", "", finish_at()),
    );
    history.append(
        start_from_spec(&spec, source(SourceView::Status, SourceAction::Refresh)),
        CommandRecordFinish::from_exit_code(0, "three", "", finish_at()),
    );

    assert_eq!(
        history
            .records()
            .map(|record| record.result.stdout.snippet.as_str())
            .collect::<Vec<_>>(),
        vec!["two", "three"]
    );
}

#[test]
fn identity_uses_argv_and_global_options_from_spec() {
    let options = GlobalOptions::default()
        .with_output(OutputPolicy {
            pager: PagerPolicy::Disable,
            ..OutputPolicy::default()
        })
        .with_repository("/tmp/repo")
        .with_config_overlay(ConfigOverlay::Inline {
            name_value: "ui.color=always".to_owned(),
        });
    let spec = JjCommandSpec::render_read_only(["workspace", "update-stale"])
        .with_global_options(options)
        .with_title("jj -R /tmp/repo workspace update-stale");

    let start = CommandRecordStart::from_spec(
        &spec,
        source(SourceView::Workspaces, SourceAction::WorkspaceUpdateStale),
    );

    assert_eq!(
        strings(&start.command.argv),
        vec![
            "--no-pager",
            "--color",
            "always",
            "--repository",
            "/tmp/repo",
            "--config",
            "ui.color=always",
            "workspace",
            "update-stale"
        ]
    );
    assert_eq!(start.command.spec_preview, "jj workspace update-stale");
    assert_eq!(
        start.command.title,
        "jj -R /tmp/repo workspace update-stale"
    );
    assert_eq!(
        start.context.repository.as_deref(),
        Some(Path::new("/tmp/repo"))
    );
    assert_eq!(
        strings(&start.context.global_options.argv),
        vec![
            "--no-pager",
            "--color",
            "always",
            "--repository",
            "/tmp/repo",
            "--config",
            "ui.color=always"
        ]
    );
    assert_eq!(start.command.command_family, CommandFamily::JjWorkspace);
}

#[test]
fn command_mode_specs_use_user_command_family() {
    let spec = JjCommandSpec::render_read_only(["status"]).with_mode(ExecutionMode::CommandMode);
    let start = CommandRecordStart::from_spec(
        &spec,
        source(
            SourceView::Other("command mode".to_owned()),
            SourceAction::UserJjCommand,
        ),
    );

    assert_eq!(start.command.command_family, CommandFamily::UserJjCommand);
    assert_eq!(start.source.action, SourceAction::UserJjCommand);
}

#[test]
fn context_global_options_are_redacted() {
    let spec = JjCommandSpec::render_read_only(["log"]).with_global_options(
        GlobalOptions::default().with_config_overlay(ConfigOverlay::Inline {
            name_value: "auth.token=abc123".to_owned(),
        }),
    );

    let start =
        CommandRecordStart::from_spec(&spec, source(SourceView::Log, SourceAction::InitialLoad));

    assert_eq!(
        strings(&start.context.global_options.argv),
        vec![
            "--no-pager",
            "--color",
            "always",
            "--config",
            "auth.token=<redacted>"
        ]
    );
}

#[test]
fn source_action_is_independent_from_command_family() {
    let spec = JjCommandSpec::render_read_only(["status"]);
    let start = CommandRecordStart::from_spec(
        &spec,
        source(SourceView::Workspaces, SourceAction::WorkspaceStatus),
    );

    assert_eq!(start.command.command_family, CommandFamily::JjStatus);
    assert_eq!(start.source.view, SourceView::Workspaces);
    assert_eq!(start.source.action, SourceAction::WorkspaceStatus);
}

#[test]
fn new_specs_use_new_family_and_typed_source_action() {
    let spec = JjCommandSpec::confirm_mutation(["new", "abc123"], SafetyClass::LocalRewrite);
    let start =
        CommandRecordStart::from_spec(&spec, source(SourceView::Log, SourceAction::NewRevision));

    assert_eq!(start.command.command_family, CommandFamily::JjNew);
    assert_eq!(start.source.view, SourceView::Log);
    assert_eq!(start.source.action, SourceAction::NewRevision);
}

#[test]
fn operation_specs_use_operation_family_and_typed_source_actions() {
    let cases = [
        (
            JjCommandSpec::render_read_only(["op", "log"]),
            SourceView::OperationLog,
            SourceAction::OperationLog,
        ),
        (
            JjCommandSpec::render_read_only(["op", "show", "abc123"]),
            SourceView::OperationShow,
            SourceAction::OperationShow,
        ),
        (
            JjCommandSpec::render_read_only(["op", "diff", "--operation", "abc123"]),
            SourceView::OperationDiff,
            SourceAction::OperationDiff,
        ),
    ];

    for (spec, view, action) in cases {
        let start = CommandRecordStart::from_spec(&spec, source(view.clone(), action.clone()));

        assert_eq!(start.command.command_family, CommandFamily::JjOperation);
        assert_eq!(start.source.view, view);
        assert_eq!(start.source.action, action);
    }
}

#[test]
fn finish_records_duration() {
    let spec = JjCommandSpec::render_read_only(["diff"]);
    let mut history = CommandHistory::new(1);
    let pending = history.start(start_from_spec(
        &spec,
        source(SourceView::Diff, SourceAction::OpenDiff),
    ));

    assert!(history.finish(
        &pending,
        CommandRecordFinish::from_exit_code(0, "", "", finish_at()),
    ));

    assert_eq!(
        history
            .records()
            .next()
            .and_then(|record| record.timing.duration),
        Some(Duration::from_millis(42))
    );
}

#[test]
fn spawn_error_has_no_exit_code() {
    let spec = JjCommandSpec::render_read_only(["log"]);
    let mut history = CommandHistory::new(1);
    let pending = history.start(start_from_spec(
        &spec,
        source(SourceView::Log, SourceAction::InitialLoad),
    ));

    assert!(history.finish(
        &pending,
        CommandRecordFinish::from_spawn_error("failed to spawn jj", "", "", finish_at()),
    ));

    let record = history.records().next().expect("record");
    assert_eq!(record.result.exit_status, None);
    assert_eq!(
        record.result.spawn_error.as_deref(),
        Some("failed to spawn jj")
    );
}

#[test]
fn finish_keeps_first_result() {
    let spec = JjCommandSpec::render_read_only(["status"]);
    let mut history = CommandHistory::new(1);
    let pending = history.start(start_from_spec(
        &spec,
        source(SourceView::Status, SourceAction::InitialLoad),
    ));

    assert!(history.finish(
        &pending,
        CommandRecordFinish::from_exit_code(0, "first", "", finish_at()),
    ));
    assert!(!history.finish(
        &pending,
        CommandRecordFinish::from_exit_code(1, "second", "", finish_at()),
    ));

    let record = history.records().next().expect("record");
    assert_eq!(record.result.exit_status, Some(ExitStatusSummary::code(0)));
    assert_eq!(record.result.stdout.snippet, "first");
}

#[test]
fn finish_applies_record_retention_limits() {
    let spec = JjCommandSpec::render_read_only(["status"]);
    let start = start_from_spec(&spec, source(SourceView::Status, SourceAction::InitialLoad))
        .with_retention(OutputRetention::summary_only(3, 2));
    let mut history = CommandHistory::new(1);
    let pending = history.start(start);

    assert!(history.finish(
        &pending,
        CommandRecordFinish::from_exit_code(1, "abcdef", "wxyz", finish_at()),
    ));

    let record = history.records().next().expect("record");
    assert_eq!(record.result.stdout.snippet, "abc");
    assert!(record.result.stdout.truncated);
    assert_eq!(record.result.stderr.snippet, "wx");
    assert!(record.result.stderr.truncated);
}

#[test]
fn stream_summary_keeps_counts_and_truncates_on_byte_limit() {
    let summary = StreamSummary::from_text("alpha\nbeta\ngamma\n", 8);

    assert_eq!(summary.byte_len, 17);
    assert_eq!(summary.line_count, 3);
    assert_eq!(summary.snippet, "alpha\nbe");
    assert!(summary.truncated);
    assert!(!summary.redacted);
}

#[test]
fn stream_summary_redacts_secret_looking_values() {
    let summary = StreamSummary::from_text(
        "token=abc123\nnormal=value\nAuthorization: Bearer secret\n",
        1024,
    );

    assert_eq!(summary.byte_len, 55);
    assert_eq!(summary.line_count, 3);
    assert_eq!(
        summary.snippet,
        "token=<redacted>\nnormal=value\nAuthorization:<redacted>\n"
    );
    assert!(summary.redacted);
    assert!(!summary.truncated);
}

#[test]
fn stream_summary_does_not_redact_key_as_substring() {
    let summary = StreamSummary::from_text("monkey=value\napi-key=abc123\n", 1024);

    assert_eq!(summary.snippet, "monkey=value\napi-key=<redacted>\n");
    assert!(summary.redacted);
}

#[test]
fn stream_summary_redaction_handles_non_ascii_graph_prefixes() {
    let summary = StreamSummary::from_text("│  config.token=abc123\n", 1024);

    assert_eq!(summary.snippet, "│  config.token=<redacted>\n");
    assert!(summary.redacted);
}

#[test]
fn command_identity_redacts_secret_looking_args() {
    let spec = JjCommandSpec::render_read_only(["log"]).with_global_options(
        GlobalOptions::default().with_config_overlay(ConfigOverlay::Inline {
            name_value: "auth.token=abc123".to_owned(),
        }),
    );

    let identity = CommandIdentity::from_spec(&spec);

    assert_eq!(
        strings(&identity.argv),
        vec![
            "--no-pager",
            "--color",
            "always",
            "--config",
            "auth.token=<redacted>",
            "log"
        ]
    );
}
