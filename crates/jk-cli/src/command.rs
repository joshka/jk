//! Shared execution adapter for typed `jj` command specs.

use std::io::{Read, Write};
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use jk_core::{
    ColorPolicy, CommandHistory, CommandRecordFinish, CommandRecordStart, CommandResultSummary,
    CommandSource, ExecutionMode, ExitStatusSummary, ExternalCommandSpec, ImmutabilityPolicy,
    JjCommandSpec, OperationIntegrationPolicy, OperationLoadPolicy, OutputPolicy, SafetyClass,
    StreamSummary, WorkingCopyPolicy,
};

const HISTORY_STREAM_LIMIT: usize = 8 * 1024;

/// Runs typed `jj` command specs.
///
/// Loaders may call [`JjCommandRunner::run`] more than once for a single user action when they need
/// both rendered output and secondary metadata. Implementations should therefore avoid assuming
/// one-shot use unless the caller documents that restriction explicitly.
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

/// Runs a shell-free external command spec and returns its captured output.
pub trait ExternalCommandRunner {
    /// Runs the executable and argv directly, without shell interpretation.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when the process cannot be spawned or waited on.
    fn run(&mut self, spec: &ExternalCommandSpec) -> std::io::Result<Output>;
}

/// Executes captured external commands with null stdin and piped stdout/stderr.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemExternalCommandRunner;

impl ExternalCommandRunner for SystemExternalCommandRunner {
    fn run(&mut self, spec: &ExternalCommandSpec) -> std::io::Result<Output> {
        let mut command = build_external_command(spec);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
    }
}

/// Records command-history entries around an external command runner.
#[derive(Debug)]
pub struct RecordingExternalCommandRunner<'a, R> {
    inner: R,
    history: &'a mut CommandHistory,
    source: CommandSource,
}

impl<'a, R> RecordingExternalCommandRunner<'a, R> {
    /// Creates a recording runner for one external command source action.
    pub const fn new(inner: R, history: &'a mut CommandHistory, source: CommandSource) -> Self {
        Self {
            inner,
            history,
            source,
        }
    }
}

impl<R> ExternalCommandRunner for RecordingExternalCommandRunner<'_, R>
where
    R: ExternalCommandRunner,
{
    fn run(&mut self, spec: &ExternalCommandSpec) -> std::io::Result<Output> {
        let pending = self.history.start(CommandRecordStart::from_external_spec(
            spec,
            self.source.clone(),
        ));
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

/// Shared cancellation signal for a running read-only command.
///
/// Cancellation is cooperative at the runner boundary: the system runner observes the signal,
/// terminates the child process, drains its output, and reaps it before returning an interrupted
/// I/O error. Clones refer to the same signal.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Creates a signal in the active state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests cancellation of work observing this signal.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Returns whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

/// Executes system `jj` commands while observing a cancellation signal.
#[derive(Clone, Debug)]
pub struct CancellableSystemJjCommandRunner {
    cancellation: CancellationToken,
}

impl CancellableSystemJjCommandRunner {
    /// Creates a runner tied to `cancellation`.
    #[must_use]
    pub const fn new(cancellation: CancellationToken) -> Self {
        Self { cancellation }
    }
}

impl JjCommandRunner for CancellableSystemJjCommandRunner {
    fn run(&mut self, spec: &JjCommandSpec) -> std::io::Result<Output> {
        run_system_jj_spec_with_cancellation(spec, Some(&self.cancellation))
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
    pub const fn new(inner: R, history: &'a mut CommandHistory, source: CommandSource) -> Self {
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
    /// Returns the wrapped runner after recording is finished.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// Runs a confirmed mutation and records the resulting operation id when the current operation
    /// context advances in a bounded before/after probe.
    ///
    /// The operation probes are intentionally not recorded in command history. Callers should use
    /// this only after user confirmation, and ordinary read-only specs fall back to
    /// [`JjCommandRunner::run`].
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error when the mutation command or operation probe cannot be
    /// spawned, written to, or waited on.
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

        if let (Ok(output), Some(before_operation_id)) = (&result, before_operation_id)
            && output.status.success()
            && let Ok(after_operation_id) = current_operation_id(&mut self.inner, spec)
            && after_operation_id != before_operation_id
        {
            finish.operation_id = Some(after_operation_id);
        }

        self.history.finish(&pending, finish);
        result
    }
}

const fn should_probe_resulting_operation(spec: &JjCommandSpec) -> bool {
    matches!(
        spec.mode(),
        ExecutionMode::ConfirmMutation | ExecutionMode::ConfirmNetworkRead
    ) && matches!(
        spec.safety(),
        SafetyClass::LocalMetadata
            | SafetyClass::LocalRewrite
            | SafetyClass::DestructiveLocal
            | SafetyClass::NetworkRead
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
    run_system_jj_spec_with_cancellation(spec, None)
}

fn run_system_jj_spec_with_cancellation(
    spec: &JjCommandSpec,
    cancellation: Option<&CancellationToken>,
) -> std::io::Result<Output> {
    run_captured_command(
        build_jj_command(spec),
        spec.stdin().map(str::as_bytes),
        cancellation,
    )
}

/// Drains output while writing stdin so a child cannot deadlock on full pipes. Cancellable commands
/// own a Unix process group so terminating jj also closes pipes held by its helpers.
fn run_captured_command(
    mut command: Command,
    input: Option<&[u8]>,
    cancellation: Option<&CancellationToken>,
) -> std::io::Result<Output> {
    if cancellation.is_some_and(CancellationToken::is_cancelled) {
        return Err(cancelled_error());
    }
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    command.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    #[cfg(unix)]
    if cancellation.is_some() {
        use std::os::unix::process::CommandExt;

        command.process_group(0);
    }

    let mut child = command.spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdin = child.stdin.take();

    thread::scope(|scope| {
        let stdout_reader = scope.spawn(|| stdout.map_or_else(|| Ok(Vec::new()), read_all));
        let stderr_reader = scope.spawn(|| stderr.map_or_else(|| Ok(Vec::new()), read_all));
        let stdin_writer = scope.spawn(|| match (stdin, input) {
            (Some(mut stdin), Some(input)) => stdin.write_all(input),
            _ => Ok(()),
        });
        // Helpers can keep these pipes open after jj exits. Observe cancellation until they close,
        // and leave the child unreaped so its process-group id cannot be reused before termination.
        while !stdin_writer.is_finished()
            || !stdout_reader.is_finished()
            || !stderr_reader.is_finished()
        {
            if cancellation.is_some_and(CancellationToken::is_cancelled) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let status = wait_for_child(&mut child, cancellation);
        if status.is_err() {
            terminate_child(&mut child, cancellation.is_some());
            let _ = child.wait();
        }
        let (status, cancelled) = status?;
        let stdin_result = stdin_writer
            .join()
            .map_err(|_| std::io::Error::other("jj stdin writer panicked"))?;
        let stdout = join_reader(stdout_reader)?;
        let stderr = join_reader(stderr_reader)?;

        if cancelled {
            return Err(cancelled_error());
        }
        stdin_result?;

        Ok(Output {
            status,
            stdout,
            stderr,
        })
    })
}

fn cancelled_error() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Interrupted, "jj command cancelled")
}

fn read_all(mut reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_reader(
    reader: thread::ScopedJoinHandle<'_, std::io::Result<Vec<u8>>>,
) -> std::io::Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| std::io::Error::other("jj output reader panicked"))?
}

fn wait_for_child(
    child: &mut std::process::Child,
    cancellation: Option<&CancellationToken>,
) -> std::io::Result<(std::process::ExitStatus, bool)> {
    loop {
        if cancellation.is_some_and(CancellationToken::is_cancelled) {
            terminate_child(child, true);
            return child.wait().map(|status| (status, true));
        }
        if let Some(status) = child.try_wait()? {
            return Ok((status, false));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn terminate_child(child: &mut std::process::Child, process_group: bool) {
    #[cfg(unix)]
    if process_group {
        let pid = rustix::process::Pid::from_child(child);
        let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
    }
    #[cfg(not(unix))]
    let _ = process_group;
    let _ = child.kill();
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

    status.code().map_or_else(
        || {
            status.signal().map_or_else(
                || ExitStatusSummary {
                    code: None,
                    signal: None,
                    success: status.success(),
                },
                ExitStatusSummary::signal,
            )
        },
        ExitStatusSummary::code,
    )
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
pub fn build_jj_command(spec: &JjCommandSpec) -> Command {
    let mut command = Command::new("jj");
    command.args(spec.global_argv());
    command.env_remove("NO_COLOR");
    command.env_remove("CLICOLOR");
    command.env_remove("CLICOLOR_FORCE");
    if let Some(columns) = spec.display_columns() {
        command.env("COLUMNS", columns.to_string());
    }

    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }

    command.args(spec.argv());
    command
}

/// Builds a process command directly from an external executable and argv.
pub fn build_external_command(spec: &ExternalCommandSpec) -> Command {
    let Some((executable, argv)) = spec.argv().split_first() else {
        unreachable!("ExternalCommandSpec always contains an executable");
    };
    let mut command = Command::new(executable);
    command.args(argv);
    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }
    command
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::io;
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    use jk_core::{SafetyClass, SourceAction, SourceView};

    use super::*;

    fn strings(argv: impl IntoIterator<Item = std::ffi::OsString>) -> Vec<String> {
        argv.into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn external_command_keeps_program_args_and_cwd_distinct() {
        let spec = ExternalCommandSpec::new(["printf", "%s", "a; echo unsafe"])
            .expect("non-empty external argv")
            .with_cwd("/tmp");
        let command = build_external_command(&spec);

        assert_eq!(command.get_program(), "printf");
        assert_eq!(
            strings(command.get_args().map(std::ffi::OsStr::to_owned)),
            vec!["%s", "a; echo unsafe"]
        );
        assert_eq!(
            command.get_current_dir(),
            Some(std::path::Path::new("/tmp"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn captured_external_command_has_no_foreground_terminal_stdin() {
        let spec = ExternalCommandSpec::new(["test", "-t", "0"]).expect("non-empty external argv");
        let output = SystemExternalCommandRunner
            .run(&spec)
            .expect("test executable runs");

        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
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
    fn display_width_changes_only_the_child_environment() {
        let spec = JjCommandSpec::render_read_only(["log"]);
        let fitted = spec.clone().with_display_columns(37);
        let command = build_jj_command(&fitted);
        assert_eq!(fitted.process_argv(), spec.process_argv());
        assert!(
            command.get_envs().any(|(key, value)| {
                key == "COLUMNS" && value == Some(std::ffi::OsStr::new("37"))
            })
        );
        assert_eq!(spec.display_columns(), None);
        assert_eq!(spec.with_display_columns(0).display_columns(), Some(1));
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
        let output = match SystemJjCommandRunner.run(&spec) {
            Ok(output) => output,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) => panic!("jj --version should run: {error}"),
        };

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("jj "));
        assert!(output.stderr.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_kills_and_reaps_a_slow_child() {
        let mut child = Command::new("sh")
            .args(["-c", "while :; do :; done"])
            .spawn()
            .expect("spawn fake slow command");
        let cancellation = CancellationToken::new();
        cancellation.cancel();

        let (status, cancelled) =
            wait_for_child(&mut child, Some(&cancellation)).expect("reap cancelled child");

        assert!(cancelled);
        assert!(!status.success());
        assert!(child.try_wait().expect("child already reaped").is_some());
    }

    #[test]
    fn cancelled_work_does_not_spawn_another_process() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let command = Command::new("jk-test-executable-that-does-not-exist");

        let error = run_captured_command(command, None, Some(&cancellation))
            .expect_err("cancel before spawn");

        assert_eq!(error.kind(), io::ErrorKind::Interrupted);
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_terminates_helpers_holding_output_pipes() {
        let cancellation = CancellationToken::new();
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30 & wait"]);
        let started = std::time::Instant::now();
        thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(100));
                cancellation.cancel();
            });

            let error = run_captured_command(command, None, Some(&cancellation))
                .expect_err("cancel process and its helper");

            assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        });
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(unix)]
    #[test]
    fn cancellation_terminates_helpers_after_parent_exits() {
        let cancellation = CancellationToken::new();
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30 &"]);
        let started = std::time::Instant::now();
        thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(100));
                cancellation.cancel();
            });

            let error = run_captured_command(command, None, Some(&cancellation))
                .expect_err("cancel helper after its parent exits");

            assert_eq!(error.kind(), io::ErrorKind::Interrupted);
        });
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[cfg(unix)]
    #[test]
    fn captured_command_drains_output_while_writing_stdin() {
        let mut command = Command::new("sh");
        command.args(["-c", "head -c 131072 /dev/zero; cat"]);
        let input = vec![b'x'; 131_072];
        let cancellation = CancellationToken::new();
        let (finished_tx, finished_rx) = std::sync::mpsc::channel();
        thread::scope(|scope| {
            let timeout_cancellation = cancellation.clone();
            scope.spawn(move || {
                if finished_rx.recv_timeout(Duration::from_secs(5)).is_err() {
                    timeout_cancellation.cancel();
                }
            });
            let result = run_captured_command(command, Some(&input), Some(&cancellation));
            let _ = finished_tx.send(());
            let output = result.expect("drain both pipes without deadlocking");

            assert!(output.status.success());
            assert_eq!(output.stdout.len(), input.len() * 2);
            assert_eq!(&output.stdout[input.len()..], input);
        });
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
