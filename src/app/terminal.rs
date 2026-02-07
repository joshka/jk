use std::io::{self, Stdout};

use crate::error::JkError;
use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};

pub(crate) struct TerminalSession {
    stdout: Stdout,
}

impl TerminalSession {
    pub(crate) fn enter() -> Result<Self, JkError> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self { stdout })
    }

    pub(crate) fn stdout_mut(&mut self) -> &mut Stdout {
        &mut self.stdout
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(
            self.stdout,
            Show,
            LeaveAlternateScreen,
            Clear(ClearType::All)
        );
        let _ = disable_raw_mode();
    }
}
