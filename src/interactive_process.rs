//! Terminal suspension runner for inherited-stdio commands.
//!
//! `jj_actions` and the app service boundary use this to suspend the UI, run
//! a child command with inherited stdio, and restore the terminal afterward.

#[cfg(test)]
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::stdout;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use color_eyre::Result;
use color_eyre::eyre::{WrapErr, eyre};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};
use ratatui::DefaultTerminal;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InteractiveCommand {
    program: OsString,
    args: Vec<OsString>,
    current_dir: Option<PathBuf>,
    label: String,
}

impl InteractiveCommand {
    pub(crate) fn new(program: impl Into<OsString>, label: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            label: label.into(),
        }
    }

    pub(crate) fn arg(&mut self, arg: impl Into<OsString>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    pub(crate) fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    #[allow(dead_code)]
    pub(crate) fn current_dir(&mut self, current_dir: impl Into<PathBuf>) -> &mut Self {
        self.current_dir = Some(current_dir.into());
        self
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    #[cfg(test)]
    pub(crate) fn program(&self) -> &OsStr {
        &self.program
    }

    #[cfg(test)]
    pub(crate) fn argv(&self) -> Vec<&OsStr> {
        self.args.iter().map(OsString::as_os_str).collect()
    }

    #[cfg(test)]
    pub(crate) fn current_dir_path(&self) -> Option<&Path> {
        self.current_dir.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn stdio_intent(&self) -> StdioIntent {
        StdioIntent::Inherit
    }

    fn process_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command
            .args(&self.args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        if let Some(current_dir) = &self.current_dir {
            command.current_dir(current_dir);
        }
        command
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(test)]
pub(crate) enum StdioIntent {
    Inherit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InteractiveExitStatus {
    success: bool,
    description: String,
}

impl InteractiveExitStatus {
    #[cfg(test)]
    pub(crate) fn success(description: impl Into<String>) -> Self {
        Self {
            success: true,
            description: description.into(),
        }
    }

    #[cfg(test)]
    pub(crate) fn failure(description: impl Into<String>) -> Self {
        Self {
            success: false,
            description: description.into(),
        }
    }

    fn from_process_status(status: std::process::ExitStatus) -> Self {
        Self {
            success: status.success(),
            description: status.to_string(),
        }
    }

    pub(crate) fn is_success(&self) -> bool {
        self.success
    }

    pub(crate) fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InteractiveCommandResult {
    label: String,
    status: InteractiveExitStatus,
}

impl InteractiveCommandResult {
    #[allow(dead_code)]
    pub(crate) fn message(&self) -> String {
        format!("{} completed", self.label)
    }

    #[allow(dead_code)]
    pub(crate) fn status(&self) -> &InteractiveExitStatus {
        &self.status
    }
}

#[allow(dead_code)]
pub(crate) fn run_with_ratatui_terminal(
    terminal: &mut DefaultTerminal,
    command: &InteractiveCommand,
) -> Result<InteractiveCommandResult> {
    let mut lifecycle = RatatuiTerminalLifecycle { terminal };
    let mut spawner = ProcessSpawner;
    run_interactive_command(&mut lifecycle, &mut spawner, command)
}

pub(crate) fn run_interactive_command<L, S>(
    lifecycle: &mut L,
    spawner: &mut S,
    command: &InteractiveCommand,
) -> Result<InteractiveCommandResult>
where
    L: TerminalLifecycle,
    S: InteractiveCommandSpawner,
{
    if let Err(suspend_error) = lifecycle.suspend() {
        return match lifecycle.restore() {
            Ok(()) => Err(eyre!(
                "{} was not started because terminal suspension failed: {suspend_error}",
                command.label()
            )),
            Err(restore_error) => Err(eyre!(
                "{} was not started because terminal suspension failed: {suspend_error}; \
                 additionally failed to restore terminal: {restore_error}",
                command.label()
            )),
        };
    }

    let mut restore_guard = TerminalRestoreGuard::new(lifecycle);
    let command_result = spawner.spawn_and_wait(command);
    let restore_result = restore_guard.restore();

    match (command_result, restore_result) {
        (Ok(status), Ok(())) if status.is_success() => Ok(InteractiveCommandResult {
            label: command.label().to_owned(),
            status,
        }),
        (Ok(status), Ok(())) => Err(eyre!(
            "{} failed with status {}",
            command.label(),
            status.description()
        )),
        (Err(command_error), Ok(())) => Err(eyre!(
            "{} failed while running inherited-stdio command: {command_error}",
            command.label()
        )),
        (Ok(status), Err(restore_error)) if status.is_success() => Err(eyre!(
            "{} completed with status {} but failed to restore terminal: {restore_error}",
            command.label(),
            status.description()
        )),
        (Ok(status), Err(restore_error)) => Err(eyre!(
            "{} failed with status {}; additionally failed to restore terminal: {restore_error}",
            command.label(),
            status.description()
        )),
        (Err(command_error), Err(restore_error)) => Err(eyre!(
            "{} failed while running inherited-stdio command: {command_error}; additionally \
             failed to restore terminal: {restore_error}",
            command.label()
        )),
    }
}

pub(crate) trait TerminalLifecycle {
    fn suspend(&mut self) -> Result<()>;

    fn restore(&mut self) -> Result<()>;
}

pub(crate) trait InteractiveCommandSpawner {
    fn spawn_and_wait(&mut self, command: &InteractiveCommand) -> Result<InteractiveExitStatus>;
}

struct RatatuiTerminalLifecycle<'a> {
    terminal: &'a mut DefaultTerminal,
}

impl TerminalLifecycle for RatatuiTerminalLifecycle<'_> {
    fn suspend(&mut self) -> Result<()> {
        self.terminal
            .show_cursor()
            .wrap_err("failed to show terminal cursor before inherited-stdio command")?;
        ratatui::try_restore().wrap_err("failed to leave Ratatui terminal mode")
    }

    fn restore(&mut self) -> Result<()> {
        enable_raw_mode().wrap_err("failed to re-enable terminal raw mode")?;
        execute!(stdout(), EnterAlternateScreen).wrap_err("failed to re-enter alternate screen")?;
        self.terminal
            .clear()
            .wrap_err("failed to clear Ratatui terminal after inherited-stdio command")
    }
}

struct ProcessSpawner;

impl InteractiveCommandSpawner for ProcessSpawner {
    fn spawn_and_wait(&mut self, command: &InteractiveCommand) -> Result<InteractiveExitStatus> {
        let status = command
            .process_command()
            .spawn()
            .wrap_err_with(|| format!("failed to spawn {}", command.label()))?
            .wait()
            .wrap_err_with(|| format!("failed to wait for {}", command.label()))?;
        Ok(InteractiveExitStatus::from_process_status(status))
    }
}

struct TerminalRestoreGuard<'a, L: TerminalLifecycle> {
    lifecycle: Option<&'a mut L>,
}

impl<'a, L: TerminalLifecycle> TerminalRestoreGuard<'a, L> {
    fn new(lifecycle: &'a mut L) -> Self {
        Self {
            lifecycle: Some(lifecycle),
        }
    }

    fn restore(&mut self) -> Result<()> {
        self.lifecycle
            .take()
            .expect("terminal restore guard should restore at most once")
            .restore()
    }
}

impl<L: TerminalLifecycle> Drop for TerminalRestoreGuard<'_, L> {
    fn drop(&mut self) {
        if let Some(lifecycle) = self.lifecycle.take() {
            let _ = lifecycle.restore();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    #[test]
    fn command_records_inherited_stdio_intent() {
        let mut command = InteractiveCommand::new("jj", "jj split");
        command.args(["--no-pager", "split"]);

        assert_eq!(command.program(), OsStr::new("jj"));
        assert_eq!(
            command.argv(),
            vec![OsStr::new("--no-pager"), OsStr::new("split")]
        );
        assert_eq!(command.stdio_intent(), StdioIntent::Inherit);
    }

    #[test]
    fn command_records_current_dir_intent() {
        let mut command = InteractiveCommand::new("jj", "jj split");
        command.current_dir("/tmp/jk-runner-proof");

        assert_eq!(
            command.current_dir_path(),
            Some(Path::new("/tmp/jk-runner-proof"))
        );
    }

    #[test]
    fn runner_restores_terminal_after_spawn_error() {
        let mut lifecycle = FakeLifecycle::default();
        let mut spawner = FakeSpawner::error("spawn error");
        let command = InteractiveCommand::new("jj", "jj split");

        let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
            .expect_err("spawn errors should be reported");

        assert_eq!(lifecycle.events, ["suspend", "restore"]);
        assert_eq!(spawner.labels, ["jj split"]);
        assert!(
            error
                .to_string()
                .contains("jj split failed while running inherited-stdio command: spawn error")
        );
    }

    #[test]
    fn runner_reports_nonzero_status_after_restoring_terminal() {
        let mut lifecycle = FakeLifecycle::default();
        let mut spawner = FakeSpawner::status(InteractiveExitStatus::failure("exit status: 1"));
        let command = InteractiveCommand::new("jj", "jj split");

        let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
            .expect_err("nonzero exits should be reported");

        assert_eq!(lifecycle.events, ["suspend", "restore"]);
        assert_eq!(
            error.to_string(),
            "jj split failed with status exit status: 1"
        );
    }

    #[test]
    fn runner_reports_restore_failure_after_success() {
        let mut lifecycle = FakeLifecycle {
            restore_error: Some("restore error"),
            ..FakeLifecycle::default()
        };
        let mut spawner = FakeSpawner::status(InteractiveExitStatus::success("exit status: 0"));
        let command = InteractiveCommand::new("jj", "jj split");

        let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
            .expect_err("restore failures should be reported");

        assert_eq!(lifecycle.events, ["suspend", "restore"]);
        assert_eq!(
            error.to_string(),
            "jj split completed with status exit status: 0 but failed to restore terminal: \
             restore error"
        );
    }

    #[test]
    fn runner_attempts_restore_after_suspend_failure() {
        let mut lifecycle = FakeLifecycle {
            suspend_error: Some("suspend error"),
            ..FakeLifecycle::default()
        };
        let mut spawner = FakeSpawner::status(InteractiveExitStatus::success("exit status: 0"));
        let command = InteractiveCommand::new("jj", "jj split");

        let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
            .expect_err("suspend failures should be reported");

        assert_eq!(lifecycle.events, ["suspend", "restore"]);
        assert!(spawner.labels.is_empty());
        assert_eq!(
            error.to_string(),
            "jj split was not started because terminal suspension failed: suspend error"
        );
    }

    #[test]
    fn restore_guard_restores_if_spawner_panics() {
        let mut lifecycle = FakeLifecycle::default();
        let mut spawner = FakeSpawner::panic();
        let command = InteractiveCommand::new("jj", "jj split");

        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = run_interactive_command(&mut lifecycle, &mut spawner, &command);
        }));

        assert!(panic.is_err());
        assert_eq!(lifecycle.events, ["suspend", "restore"]);
    }

    #[test]
    #[ignore = "manual proof uses JK_INTERACTIVE_PROOF_REPO to force cwd into a /tmp jj repo"]
    fn real_runner_reports_jj_failure_from_tmp_repo() {
        let repo = canonical_tmp_proof_repo();

        let mut lifecycle = FakeLifecycle::default();
        let mut spawner = ProcessSpawner;
        let mut command = InteractiveCommand::new("jj", "jj split --tool false");
        command
            .current_dir(repo)
            .args(["--no-pager", "split", "--tool", "false"]);

        let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
            .expect_err("jj split --tool false should report a clean runner failure");

        assert_eq!(lifecycle.events, ["suspend", "restore"]);
        assert!(error.to_string().contains("jj split --tool false failed"));
    }

    #[test]
    #[ignore = "manual proof requires a live terminal and JK_INTERACTIVE_PROOF_REPO under /tmp"]
    fn real_ratatui_runner_reports_jj_failure_from_tmp_repo() {
        let repo = canonical_tmp_proof_repo();

        let mut terminal = ratatui::try_init().expect("manual proof requires a live terminal");
        let mut command = InteractiveCommand::new("jj", "jj split --tool false");
        command
            .current_dir(repo)
            .args(["--no-pager", "split", "--tool", "false"]);

        let result = run_with_ratatui_terminal(&mut terminal, &command);
        let cleanup_result = ratatui::try_restore();
        let error = result.expect_err("jj split --tool false should fail cleanly");
        let error = error.to_string();

        assert!(
            error.contains("jj split --tool false failed with status"),
            "expected a clean child nonzero status, got {error}"
        );
        assert!(
            !error.contains("failed to restore terminal"),
            "expected child failure without terminal restore failure, got {error}"
        );
        assert!(
            !error.contains("additionally failed to restore terminal"),
            "expected child failure without additional restore failure, got {error}"
        );

        cleanup_result.expect("manual proof should restore the terminal for the shell");
    }

    fn canonical_tmp_proof_repo() -> PathBuf {
        let repo = std::env::var_os("JK_INTERACTIVE_PROOF_REPO")
            .expect("set JK_INTERACTIVE_PROOF_REPO to a disposable /tmp jj repo");
        let canonical_repo = PathBuf::from(repo)
            .canonicalize()
            .expect("proof repo must exist before running the manual proof");
        let canonical_tmp = Path::new("/tmp")
            .canonicalize()
            .expect("the system /tmp directory should exist");

        assert!(
            canonical_repo.starts_with(&canonical_tmp),
            "proof repo must resolve inside {}, got {}",
            canonical_tmp.display(),
            canonical_repo.display()
        );

        canonical_repo
    }

    #[derive(Default)]
    struct FakeLifecycle {
        events: Vec<&'static str>,
        suspend_error: Option<&'static str>,
        restore_error: Option<&'static str>,
    }

    impl TerminalLifecycle for FakeLifecycle {
        fn suspend(&mut self) -> Result<()> {
            self.events.push("suspend");
            match self.suspend_error {
                Some(error) => Err(eyre!(error)),
                None => Ok(()),
            }
        }

        fn restore(&mut self) -> Result<()> {
            self.events.push("restore");
            match self.restore_error {
                Some(error) => Err(eyre!(error)),
                None => Ok(()),
            }
        }
    }

    enum FakeSpawnResult {
        Status(InteractiveExitStatus),
        Error(&'static str),
        Panic,
    }

    struct FakeSpawner {
        result: FakeSpawnResult,
        labels: Vec<String>,
    }

    impl FakeSpawner {
        fn status(status: InteractiveExitStatus) -> Self {
            Self {
                result: FakeSpawnResult::Status(status),
                labels: Vec::new(),
            }
        }

        fn error(error: &'static str) -> Self {
            Self {
                result: FakeSpawnResult::Error(error),
                labels: Vec::new(),
            }
        }

        fn panic() -> Self {
            Self {
                result: FakeSpawnResult::Panic,
                labels: Vec::new(),
            }
        }
    }

    impl InteractiveCommandSpawner for FakeSpawner {
        fn spawn_and_wait(
            &mut self,
            command: &InteractiveCommand,
        ) -> Result<InteractiveExitStatus> {
            self.labels.push(command.label().to_owned());
            match &self.result {
                FakeSpawnResult::Status(status) => Ok(status.clone()),
                FakeSpawnResult::Error(error) => Err(eyre!(*error)),
                FakeSpawnResult::Panic => panic!("spawner panic"),
            }
        }
    }
}
