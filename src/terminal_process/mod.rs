//! Terminal suspension runner for inherited-stdio commands.
//!
//! `actions` and the app service boundary use this to suspend the UI, run
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
    /// Executable to run with inherited stdio.
    program: OsString,
    /// Positional and option arguments passed to the child process.
    args: Vec<OsString>,
    /// Optional working directory for the child process.
    current_dir: Option<PathBuf>,
    /// User-facing label used in errors and result reporting.
    label: String,
}

impl InteractiveCommand {
    /// Build a new inherited-stdio command with a user-facing label.
    pub(crate) fn new(program: impl Into<OsString>, label: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            current_dir: None,
            label: label.into(),
        }
    }

    /// Append one argument to the child process argv.
    pub(crate) fn arg(&mut self, arg: impl Into<OsString>) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Extend the child process argv from an iterator of arguments.
    pub(crate) fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set the working directory the child process should inherit.
    #[allow(dead_code)]
    pub(crate) fn current_dir(&mut self, current_dir: impl Into<PathBuf>) -> &mut Self {
        self.current_dir = Some(current_dir.into());
        self
    }

    /// Return the user-facing label for this interactive command.
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

    /// Build the `std::process::Command` configured for inherited stdio execution.
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
    /// Whether the child process exited successfully.
    success: bool,
    /// Human-readable exit status description.
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

    /// Return whether the child process exited successfully.
    pub(crate) fn is_success(&self) -> bool {
        self.success
    }

    /// Return the human-readable exit status description.
    pub(crate) fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InteractiveCommandResult {
    /// User-facing label for the completed command.
    label: String,
    /// Exit status returned by the child process.
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

/// Run one interactive command using the live Ratatui terminal lifecycle.
#[allow(dead_code)]
pub(crate) fn run_with_ratatui_terminal(
    terminal: &mut DefaultTerminal,
    command: &InteractiveCommand,
) -> Result<InteractiveCommandResult> {
    let mut lifecycle = RatatuiTerminalLifecycle { terminal };
    let mut spawner = ProcessSpawner;
    run_interactive_command(&mut lifecycle, &mut spawner, command)
}

/// Suspend the terminal, run one inherited-stdio command, and restore the terminal afterward.
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

/// Terminal lifecycle boundary needed around inherited-stdio commands.
pub(crate) trait TerminalLifecycle {
    fn suspend(&mut self) -> Result<()>;

    fn restore(&mut self) -> Result<()>;
}

/// Child-process boundary used by the interactive runner.
pub(crate) trait InteractiveCommandSpawner {
    fn spawn_and_wait(&mut self, command: &InteractiveCommand) -> Result<InteractiveExitStatus>;
}

/// Real Ratatui terminal lifecycle that leaves and re-enters the alternate-screen UI.
struct RatatuiTerminalLifecycle<'a> {
    /// Live Ratatui terminal that must be suspended and restored.
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
    /// Lifecycle to restore exactly once, either explicitly or on drop.
    lifecycle: Option<&'a mut L>,
}

impl<'a, L: TerminalLifecycle> TerminalRestoreGuard<'a, L> {
    /// Create a restore guard immediately after terminal suspension succeeds.
    fn new(lifecycle: &'a mut L) -> Self {
        Self {
            lifecycle: Some(lifecycle),
        }
    }

    /// Restore the terminal once and consume the stored lifecycle reference.
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
mod tests;
