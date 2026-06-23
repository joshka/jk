//! Shared execution adapter for typed `jj` command specs.

use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::time::SystemTime;

use jk_core::{
    ColorPolicy, CommandHistory, CommandRecordFinish, CommandRecordStart, CommandResultSummary,
    CommandSource, ExecutionMode, ExitStatusSummary, ImmutabilityPolicy, JjCommandSpec,
    OperationIntegrationPolicy, OperationLoadPolicy, OutputPolicy, SafetyClass, StreamSummary,
    WorkingCopyPolicy,
};

const HISTORY_STREAM_LIMIT: usize = 8 * 1024;

/// Runs typed `jj` command specs.
///
/// Loaders may call [`JjCommandRunner::run`] more than once for a single user action when they
/// need both rendered output and secondary metadata. Implementations should therefore avoid
/// assuming one-shot use unless the caller documents that restriction explicitly.
pub trait JjCommandRunner {
    /// Runs a typed `jj` command spec.
    ///
    /// Returns the child-process output when the command starts, writes any stdin, and exits
    /// successfully enough for the caller to inspect the [`Output`].
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when spawning, writing, or waiting fails.
    fn run(&mut self, spec: &JjCommandSpec) -> std::io::Result<Output>;
}

/// Executes `jj` commands with the system `jj` binary.
///
/// Each call spawns a fresh `jj` process. Callers that use loaders with multiple passes should
/// expect multiple invocations and the corresponding I/O errors if the binary cannot be started or
/// read.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemJjCommandRunner;

impl JjCommandRunner for SystemJjCommandRunner {
    fn run(&mut self, spec: &JjCommandSpec) -> std::io::Result<Output> {
        run_system_jj_spec(spec)
    }
}

/// Records command-history entries around another `jj` runner.
///
/// Each call to [`JjCommandRunner::run`] records one command-history entry, so loaders that call
/// the runner multiple times will create multiple retained records for the same user action.
#[derive(Debug)]
pub struct RecordingJjCommandRunner<'a, R> {
    inner: R,
    history: &'a mut CommandHistory,
    source: CommandSource,
}

impl<'a, R> RecordingJjCommandRunner<'a, R> {
    /// Creates a recording runner for commands from one source action.
    ///
    /// Every invocation records the same source metadata alongside the command spec.
    pub fn new(inner: R, history: &'a mut CommandHistory, source: CommandSource) -> Self {
        Self {
            inner,
            history,
            source,
        }
    }
}

impl<R> JjCommandRunner for RecordingJjCommandRunner<'_, R>
where
    R: JjCommandRunner,
{
    fn run(&mut self, spec: &JjCommandSpec) -> std::io::Result<Output> {
        let pending = self
            .history
            .start(CommandRecordStart::from_spec(spec, self.source.clone()));
        let result = self.inner.run(spec);
        let finish = match &result {
            Ok(output) => finish_from_output(output, SystemTime::now()),
            Err(error) => {
                CommandRecordFinish::from_spawn_error(error.to_string(), "", "", SystemTime::now())
            }
        };
        self.history.finish(&pending, finish);
        result
    }
}

impl<R> RecordingJjCommandRunner<'_, R>
where
    R: JjCommandRunner,
{
    /// Runs a confirmed mutation and records the resulting operation id when the current operation
    /// context advances in a bounded before/after probe.
    ///
    /// The operation probes are intentionally not recorded in command history. Callers should use
    /// this only after user confirmation, and ordinary read-only specs fall back to [`run`].
    pub fn run_confirmed_mutation(&mut self, spec: &JjCommandSpec) -> std::io::Result<Output> {
        if !should_probe_resulting_operation(spec) {
            return self.run(spec);
        }

        let before_operation_id = current_operation_id(&mut self.inner, spec).ok();
        let pending = self
            .history
            .start(CommandRecordStart::from_spec(spec, self.source.clone()));
        let result = self.inner.run(spec);
        let mut finish = match &result {
            Ok(output) => finish_from_output(output, SystemTime::now()),
            Err(error) => {
                CommandRecordFinish::from_spawn_error(error.to_string(), "", "", SystemTime::now())
            }
        };

        if let (Ok(output), Some(before_operation_id)) = (&result, before_operation_id) {
            if output.status.success() {
                if let Ok(after_operation_id) = current_operation_id(&mut self.inner, spec) {
                    if after_operation_id != before_operation_id {
                        finish.operation_id = Some(after_operation_id);
                    }
                }
            }
        }

        self.history.finish(&pending, finish);
        result
    }
}

fn should_probe_resulting_operation(spec: &JjCommandSpec) -> bool {
    matches!(spec.mode(), ExecutionMode::ConfirmMutation)
        && matches!(
            spec.safety(),
            SafetyClass::LocalMetadata | SafetyClass::LocalRewrite | SafetyClass::DestructiveLocal
        )
}

fn current_operation_id(
    runner: &mut impl JjCommandRunner,
    spec: &JjCommandSpec,
) -> std::io::Result<String> {
    let output = runner.run(&current_operation_id_spec(spec))?;
    if !output.status.success() {
        return Err(std::io::Error::other("jj op log failed"));
    }

    parse_single_operation_id(&output.stdout)
        .ok_or_else(|| std::io::Error::other("jj op log did not return one operation id"))
}

fn current_operation_id_spec(spec: &JjCommandSpec) -> JjCommandSpec {
    let global_options = spec
        .global_options()
        .clone()
        .with_working_copy(WorkingCopyPolicy::Ignore)
        .with_operation(OperationLoadPolicy::AtOperation("@".to_owned()))
        .with_operation_integration(OperationIntegrationPolicy::Integrate)
        .with_immutability(ImmutabilityPolicy::Enforce)
        .with_output(OutputPolicy {
            color: ColorPolicy::Never,
            ..OutputPolicy::default()
        });
    JjCommandSpec::render_read_only(["op", "log", "--no-graph", "-T", "id ++ \"\\n\"", "-n", "1"])
        .with_global_options(global_options)
}

fn parse_single_operation_id(stdout: &[u8]) -> Option<String> {
    let rendered = String::from_utf8_lossy(stdout);
    let mut lines = rendered
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty());
    let operation_id = lines.next()?;
    if lines.next().is_some() || !looks_like_operation_id(operation_id) {
        return None;
    }
    Some(operation_id.to_owned())
}

fn looks_like_operation_id(value: &str) -> bool {
    value.len() >= 12 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn run_system_jj_spec(spec: &JjCommandSpec) -> std::io::Result<Output> {
    let mut command = build_jj_command(spec);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if spec.stdin().is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command.spawn()?;
    if let Some(stdin) = spec.stdin() {
        let child_stdin = child.stdin.as_mut().expect("stdin was configured as piped");
        child_stdin.write_all(stdin.as_bytes())?;
    }

    child.wait_with_output()
}

fn finish_from_output(output: &Output, ended_at: SystemTime) -> CommandRecordFinish {
    CommandRecordFinish::from_result(
        CommandResultSummary {
            exit_status: Some(exit_status_summary(output.status)),
            spawn_error: None,
            stdout: StreamSummary::from_bytes(&output.stdout, HISTORY_STREAM_LIMIT),
            stderr: StreamSummary::from_bytes(&output.stderr, HISTORY_STREAM_LIMIT),
        },
        None,
        ended_at,
    )
}

#[cfg(unix)]
fn exit_status_summary(status: std::process::ExitStatus) -> ExitStatusSummary {
    use std::os::unix::process::ExitStatusExt;

    if let Some(code) = status.code() {
        ExitStatusSummary::code(code)
    } else if let Some(signal) = status.signal() {
        ExitStatusSummary::signal(signal)
    } else {
        ExitStatusSummary {
            code: None,
            signal: None,
            success: status.success(),
        }
    }
}

#[cfg(not(unix))]
fn exit_status_summary(status: std::process::ExitStatus) -> ExitStatusSummary {
    ExitStatusSummary {
        code: status.code(),
        signal: None,
        success: status.success(),
    }
}

/// Builds the process command for a typed `jj` command spec.
pub(crate) fn build_jj_command(spec: &JjCommandSpec) -> Command {
    let mut command = Command::new("jj");
    command.args(spec.global_argv());
    command.env_remove("NO_COLOR");
    command.env_remove("CLICOLOR");
    command.env_remove("CLICOLOR_FORCE");

    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }

    command.args(spec.argv());
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    use jk_core::{SafetyClass, SourceAction, SourceView};

    fn strings(argv: impl IntoIterator<Item = std::ffi::OsString>) -> Vec<String> {
        argv.into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn command_adapter_forces_color_and_cleans_color_env() {
        let command = build_jj_command(&JjCommandSpec::render_read_only(["log"]));
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let envs = command
            .get_envs()
            .map(|(key, value)| (key.to_string_lossy().into_owned(), value.is_none()))
            .collect::<Vec<_>>();

        assert!(args.windows(2).any(|args| args == ["--color", "always"]));
        assert!(args.iter().any(|arg| arg == "log"));
        assert!(envs.contains(&("NO_COLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR_FORCE".to_owned(), true)));
    }

    #[test]
    fn command_adapter_includes_repository_before_spec_argv() {
        let spec =
            JjCommandSpec::render_read_only(["diff", "-r", "@"]).with_repository("/tmp/repository");
        let command = build_jj_command(&spec);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            args,
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repository",
                "diff",
                "-r",
                "@"
            ]
        );
        assert_eq!(
            args.iter()
                .filter(|arg| arg.as_str() == "--repository")
                .count(),
            1
        );
    }

    #[test]
    fn command_adapter_uses_spec_rendered_process_argv() {
        let spec = JjCommandSpec::render_read_only(["status"]).with_repository("/tmp/repository");
        let command = build_jj_command(&spec);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let spec_args = spec
            .process_argv()
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(args, spec_args);
    }

    #[test]
    fn command_adapter_captures_stdout() {
        let spec = JjCommandSpec::render_read_only(["--version"]);
        let output = SystemJjCommandRunner
            .run(&spec)
            .expect("jj --version should run");

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("jj "));
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn recording_runner_records_successful_output() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::render_read_only(["status"]);
        let mut runner = RecordingJjCommandRunner::new(
            FakeRunner::success(0, "clean\n", ""),
            &mut history,
            CommandSource::new(SourceView::Status, SourceAction::InitialLoad),
        );

        let output = runner.run(&spec).expect("fake runner succeeds");

        assert!(output.status.success());
        let record = history.records().next().expect("recorded command");
        assert_eq!(record.command.spec_preview, "jj status");
        assert_eq!(record.result.exit_status, Some(ExitStatusSummary::code(0)));
        assert_eq!(record.result.stdout.snippet, "clean\n");
        assert!(record.result.spawn_error.is_none());
    }

    #[test]
    fn recording_runner_records_spawn_failure() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::render_read_only(["log"]);
        let mut runner = RecordingJjCommandRunner::new(
            FakeRunner::spawn_error("jj missing"),
            &mut history,
            CommandSource::new(SourceView::Log, SourceAction::Refresh),
        );

        let error = runner.run(&spec).expect_err("fake runner fails to spawn");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        let record = history.records().next().expect("recorded command");
        assert_eq!(record.result.exit_status, None);
        assert_eq!(record.result.spawn_error.as_deref(), Some("jj missing"));
    }

    #[test]
    fn recording_runner_carries_source_view_and_action() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::render_read_only(["diff"]);
        let source = CommandSource::new(SourceView::Log, SourceAction::OpenDiff).with_key("enter");
        let mut runner =
            RecordingJjCommandRunner::new(FakeRunner::success(0, "diff", ""), &mut history, source);

        runner.run(&spec).expect("fake runner succeeds");

        let record = history.records().next().expect("recorded command");
        assert_eq!(record.source.view, SourceView::Log);
        assert_eq!(record.source.action, SourceAction::OpenDiff);
        assert_eq!(record.source.key.as_deref(), Some("enter"));
    }

    #[test]
    fn recording_runner_records_update_stale_as_workspace_action() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::render_read_only(["workspace", "update-stale"])
            .with_repository("/tmp/workspace")
            .with_safety(SafetyClass::LocalMetadata);
        let mut runner = RecordingJjCommandRunner::new(
            FakeRunner::success(0, "", "updated\n"),
            &mut history,
            CommandSource::new(SourceView::Workspaces, SourceAction::WorkspaceUpdateStale),
        );

        runner.run(&spec).expect("fake runner succeeds");

        let record = history.records().next().expect("recorded command");
        assert_eq!(record.source.view, SourceView::Workspaces);
        assert_eq!(record.source.action, SourceAction::WorkspaceUpdateStale);
        assert_eq!(record.safety, SafetyClass::LocalMetadata);
        assert_eq!(record.result.stderr.snippet, "updated\n");
    }

    #[test]
    fn confirmed_mutation_records_changed_operation_id() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::confirm_mutation(
            ["describe", "-m", "message", "abc123"],
            SafetyClass::LocalRewrite,
        );
        let mut runner = RecordingJjCommandRunner::new(
            SequencedRunner::successes(vec![
                output(0, "111111111111\n", ""),
                output(0, "described\n", ""),
                output(0, "222222222222\n", ""),
            ]),
            &mut history,
            CommandSource::new(SourceView::Log, SourceAction::DescribeRevision),
        );

        runner
            .run_confirmed_mutation(&spec)
            .expect("fake mutation succeeds");

        let record = history.records().next().expect("recorded mutation");
        assert_eq!(record.command.spec_preview, "jj describe -m message abc123");
        assert_eq!(record.operation_id.as_deref(), Some("222222222222"));
        assert_eq!(history.records().count(), 1);
    }

    #[test]
    fn confirmed_mutation_operation_probe_disables_color() {
        let spec = JjCommandSpec::confirm_mutation(
            ["describe", "-m", "message", "abc123"],
            SafetyClass::LocalRewrite,
        );

        let argv = strings(current_operation_id_spec(&spec).process_argv());

        assert!(argv.windows(2).any(|args| args == ["--color", "never"]));
        assert!(!argv.windows(2).any(|args| args == ["--color", "always"]));
    }

    #[test]
    fn confirmed_mutation_leaves_operation_id_empty_when_context_does_not_change() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::confirm_mutation(
            ["describe", "-m", "message", "abc123"],
            SafetyClass::LocalRewrite,
        );
        let mut runner = RecordingJjCommandRunner::new(
            SequencedRunner::successes(vec![
                output(0, "111111111111\n", ""),
                output(0, "described\n", ""),
                output(0, "111111111111\n", ""),
            ]),
            &mut history,
            CommandSource::new(SourceView::Log, SourceAction::DescribeRevision),
        );

        runner
            .run_confirmed_mutation(&spec)
            .expect("fake mutation succeeds");

        let record = history.records().next().expect("recorded mutation");
        assert_eq!(record.operation_id, None);
        assert_eq!(history.records().count(), 1);
    }

    #[test]
    fn confirmed_mutation_leaves_operation_id_empty_when_mutation_fails() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::confirm_mutation(
            ["describe", "-m", "message", "abc123"],
            SafetyClass::LocalRewrite,
        );
        let mut runner = RecordingJjCommandRunner::new(
            SequencedRunner::successes(vec![
                output(0, "111111111111\n", ""),
                output(1, "", "failed\n"),
            ]),
            &mut history,
            CommandSource::new(SourceView::Log, SourceAction::DescribeRevision),
        );

        let output = runner
            .run_confirmed_mutation(&spec)
            .expect("fake mutation runs");

        assert!(!output.status.success());
        let record = history.records().next().expect("recorded mutation");
        assert_eq!(record.operation_id, None);
        assert_eq!(history.records().count(), 1);
    }

    #[test]
    fn confirmed_mutation_runner_does_not_probe_read_only_specs() {
        let mut history = CommandHistory::new(4);
        let spec = JjCommandSpec::render_read_only(["status"]);
        let mut runner = RecordingJjCommandRunner::new(
            SequencedRunner::successes(vec![output(0, "clean\n", "")]),
            &mut history,
            CommandSource::new(SourceView::Status, SourceAction::Refresh),
        );

        runner
            .run_confirmed_mutation(&spec)
            .expect("fake read-only command succeeds");

        let record = history.records().next().expect("recorded command");
        assert_eq!(record.command.spec_preview, "jj status");
        assert_eq!(record.operation_id, None);
        assert_eq!(history.records().count(), 1);
    }

    struct FakeRunner {
        result: io::Result<Output>,
    }

    impl FakeRunner {
        fn success(code: i32, stdout: &str, stderr: &str) -> Self {
            Self {
                result: Ok(Output {
                    status: exit_status(code),
                    stdout: stdout.as_bytes().to_vec(),
                    stderr: stderr.as_bytes().to_vec(),
                }),
            }
        }

        fn spawn_error(message: &str) -> Self {
            Self {
                result: Err(io::Error::new(io::ErrorKind::NotFound, message)),
            }
        }
    }

    impl JjCommandRunner for FakeRunner {
        fn run(&mut self, _spec: &JjCommandSpec) -> io::Result<Output> {
            std::mem::replace(
                &mut self.result,
                Err(io::Error::other("fake runner result already consumed")),
            )
        }
    }

    struct SequencedRunner {
        outputs: Vec<Output>,
    }

    impl SequencedRunner {
        fn successes(outputs: Vec<Output>) -> Self {
            let mut outputs = outputs;
            outputs.reverse();
            Self { outputs }
        }
    }

    impl JjCommandRunner for SequencedRunner {
        fn run(&mut self, _spec: &JjCommandSpec) -> io::Result<Output> {
            self.outputs
                .pop()
                .ok_or_else(|| io::Error::other("fake runner output already consumed"))
        }
    }

    fn output(code: i32, stdout: &str, stderr: &str) -> Output {
        Output {
            status: exit_status(code),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[cfg(unix)]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(not(unix))]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        Command::new(if cfg!(windows) { "cmd" } else { "sh" })
            .args(if cfg!(windows) {
                vec!["/C".into(), format!("exit {code}").into()]
            } else {
                vec!["-c".into(), format!("exit {code}").into()]
            })
            .status()
            .unwrap()
    }
}
