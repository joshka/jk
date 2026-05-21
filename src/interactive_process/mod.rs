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
mod tests;
