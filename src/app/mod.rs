//! Terminal event loop and app-level orchestration.
//!
//! Feature slices own their view behavior. The app owns cross-cutting concerns: key dispatch,
//! pending key-prefix state, mode handoff, refresh, search state, and routing view effects to the
//! app submodule that owns the detailed policy. `services` provides the single injected side-effect
//! seam; child app modules call it directly unless the operation must couple that effect to current
//! app-owned state.

use std::env;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::Frame;

use crate::command::{
    Binding, BindingMatch, CommandContext, ViewCommand, ViewEffect, binding_prefix_next_labels,
    match_binding_sequence,
};
use crate::jj::DiffFormat;
use crate::modes::InteractionMode;
use crate::search::SearchQuery;
use crate::tui;
use crate::view_state::ViewState;

pub mod actions;
mod bindings;
mod dispatch;
mod effects;
mod input;
mod navigation;
mod reducers;
mod services;
pub mod status_line;

use self::dispatch::prefix_status_message;
use self::services::AppServices;
use self::status_line::{StatusKind, StatusLine};

pub use self::bindings::APP_BINDINGS;

/// Start the terminal application from process arguments.
///
/// This is the binary boundary: it loads the initial rendered jj view, enters the Ratatui event
/// loop, and lets app-owned dispatch surface terminal, clipboard, and jj errors through
/// `color_eyre`.
pub fn run() -> Result<()> {
    let mut app = App::load(env::args_os().skip(1).collect())?;

    ratatui::run(|terminal| app.run(terminal))
}

/// App-owned runtime state for dispatch, view history, and prompt handoff.
///
/// View modules own the rendered content and local navigation policy. This struct keeps the
/// cross-view stack, pending prefix state, search scope, and the single injected service seam
/// together so dispatch does not have to reconstruct cross-view state or effects from rendered
/// output.
pub struct App {
    /// Currently active feature view.
    view: ViewState,
    /// Back-stack of previously active views for app-level history navigation.
    stack: Vec<ViewState>,
    /// Startup `jj log` argv restored by direct log switching.
    startup_log_args: Option<Vec<String>>,
    /// Active show/diff presentation format chosen at the app level.
    diff_format: DiffFormat,
    /// Current status-line state shown in shared chrome.
    status: StatusLine,
    /// Active modal or prompt state layered over the current view.
    mode: InteractionMode,
    /// In-progress multi-key command prefix waiting for resolution or timeout.
    pending_command: Option<PendingCommand>,
    /// Active search query shared with the current view.
    search: Option<SearchQuery>,
    /// Exit flag set by app-level quit handling.
    should_quit: bool,
    /// Injected seam for jj, refresh, and alternate-view side effects.
    services: AppServices,
}

/// Keep multi-key prefixes responsive without making fallback bindings feel hair-trigger.
const COMMAND_PREFIX_TIMEOUT: Duration = Duration::from_millis(700);

/// Pending multi-key input tracked until the prefix resolves or expires.
///
/// `App` keeps the original keys, any exact fallback binding, and the timeout together so prefix
/// dispatch can finish, cancel, or replay without rebuilding state from the current view.
#[derive(Clone)]
struct PendingCommand {
    /// Keys already typed for the prefix currently being resolved.
    keys: Vec<crossterm::event::KeyEvent>,
    /// Exact binding to run if the prefix expires or no longer matches a longer sequence.
    fallback: Option<Binding>,
    /// Instant after which the prefix should fall back automatically.
    deadline: Instant,
}

impl App {
    /// Drive the terminal redraw and input loop until the app requests exit.
    ///
    /// The runtime path is deliberate: draw the current view, derive the live viewport height, then
    /// either dispatch the next terminal event or expire any pending multi-key prefix.
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            let completed_frame = terminal.draw(|frame| self.render(frame))?;
            let viewport_height = tui::areas(completed_frame.area).main.height;

            if event::poll(Duration::from_millis(200))? {
                self.handle_event(terminal, event::read()?, viewport_height)?;
            } else {
                self.flush_expired_pending_command(viewport_height)?;
            }
        }

        Ok(())
    }

    /// Render the current app frame, including shared chrome and any active overlay.
    fn render(&self, frame: &mut Frame) {
        let status = self.mode.status_line(&self.view, &self.status);
        let areas = tui::areas(frame.area());
        tui::render_chrome(frame, areas, &status);
        self.view.render(frame, areas.main, self.search.as_ref());
        tui::render_overlay(frame, &status, self.mode.overlay(&self.view, APP_BINDINGS));
    }

    /// Route one terminal event into resize handling, modal dispatch, or normal dispatch.
    ///
    /// Resize is handled here because it updates app-owned presentation state immediately. Key
    /// events first go through the active mode, then fall through to normal bindings only when the
    /// mode does not consume them.
    fn handle_event(
        &mut self,
        terminal: &mut DefaultTerminal,
        event: Event,
        viewport_height: u16,
    ) -> Result<()> {
        let key = match event {
            Event::Key(key) => key,
            Event::Resize(width, height) => {
                self.view.clamp(height.saturating_sub(2), width);
                if matches!(self.status.kind(), StatusKind::Ready) {
                    self.status = StatusLine::ready(&self.view);
                }
                return Ok(());
            }
            _ => return Ok(()),
        };

        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        if self.handle_mode_key_event_with_terminal(key, viewport_height, Some(terminal))? {
            return Ok(());
        }

        let refresh_status = self.handle_normal_key(key, viewport_height)?;
        if refresh_status && matches!(self.status.kind(), StatusKind::Ready) {
            self.status = StatusLine::ready(&self.view);
        }

        Ok(())
    }

    /// Dispatch a normal-mode key using the current timestamp as prefix resolution time.
    fn handle_normal_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
    ) -> Result<bool> {
        self.handle_normal_key_at(key, viewport_height, Instant::now())
    }

    /// Dispatch a normal-mode key while making prefix timeout evaluation explicit.
    ///
    /// Tests call this variant with a controlled timestamp so prefix fallback behavior stays
    /// deterministic.
    fn handle_normal_key_at(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if self.pending_command.is_some() {
            return self.handle_pending_command_key(key, viewport_height, now);
        }

        let keys = [key];
        let Some(binding_match) =
            match_binding_sequence(&[APP_BINDINGS, self.view.bindings()], &keys)
        else {
            return Ok(true);
        };

        match binding_match {
            BindingMatch::Exact(binding) => self.execute_binding(binding, viewport_height),
            BindingMatch::Prefix { fallback } => {
                self.pending_command = Some(PendingCommand {
                    keys: keys.to_vec(),
                    fallback,
                    deadline: now + COMMAND_PREFIX_TIMEOUT,
                });
                self.status = StatusLine::with_message(
                    &self.view,
                    prefix_status_message(
                        "prefix",
                        &keys,
                        &binding_prefix_next_labels(&[APP_BINDINGS, self.view.bindings()], &keys),
                    ),
                );
                Ok(false)
            }
        }
    }

    /// Rebuild the active view and convert refresh failure into status.
    ///
    /// ViewState owns the actual reload and clamp policy; this method only keeps the app status in
    /// sync with the result.
    fn refresh(&mut self, viewport_height: u16) {
        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                self.status = StatusLine::ready(&self.view);
            }
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    /// Ask the active view slice to interpret one view-local command.
    fn execute_view(&mut self, command: ViewCommand, viewport_height: u16) -> ViewEffect {
        self.view.execute(
            command,
            CommandContext {
                viewport_height,
                viewport_width: current_viewport_width(),
                search: self.search.as_ref(),
            },
        )
    }
}

/// Read the current terminal width at dispatch time.
///
/// View clamping uses live terminal size instead of cached geometry so resize handling stays local
/// to the next refresh or view effect.
fn current_viewport_width() -> u16 {
    crossterm::terminal::size()
        .map(|(width, _)| width)
        .unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests;
