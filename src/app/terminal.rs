//! Terminal mode lifecycle management.
//!
//! Entering creates an alternate-screen raw-mode session; dropping restores terminal state.

use std::io::{self, Stdout};

use crate::error::JkError;
use crossterm::cursor::{Hide, Show};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub(crate) type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

/// RAII guard for terminal session state.
pub(crate) struct TerminalSession {
    terminal: AppTerminal,
}

impl TerminalSession {
    /// Enter raw mode and alternate screen, hiding the cursor.
    pub(crate) fn enter() -> Result<Self, JkError> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(Self { terminal })
    }

    /// Return mutable ratatui terminal used by frame rendering.
    pub(crate) fn terminal_mut(&mut self) -> &mut AppTerminal {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    /// Best-effort terminal restoration on shutdown.
    fn drop(&mut self) {
        let _ = execute!(
            self.terminal.backend_mut(),
            Show,
            LeaveAlternateScreen,
            Clear(ClearType::All)
        );
        let _ = disable_raw_mode();
    }
}
