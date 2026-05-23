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
use crossterm::event;
use ratatui::layout::{Rect, Size};
use ratatui::{DefaultTerminal, Frame};

use crate::actions::JjSplitPlan;
use crate::command::{Binding, CommandContext, ViewCommand, ViewEffect};
use crate::jj::DiffFormat;
use crate::modes::InteractionMode;
use crate::search::SearchQuery;
use crate::tui;
use crate::view_state::ViewState;

mod abandon;
pub mod actions;
mod bindings;
mod dispatch;
mod effects;
mod help;
mod keyboard;
mod menus;
mod navigation;
mod prompts;
mod reducers;
mod services;
pub mod status_line;
#[cfg(test)]
mod tests;

pub use self::bindings::APP_BINDINGS;
#[cfg(test)]
pub use self::reducers::{rebase_plan_from_prompt, squash_plan_from_prompt};
use self::services::AppServices;
use self::status_line::StatusLine;

/// Start the terminal application from process arguments.
///
/// This is the binary boundary: it loads the initial rendered jj view, enters the Ratatui event
/// loop, and lets app-owned dispatch surface terminal, clipboard, and jj errors through
/// `color_eyre`.
pub fn run() -> Result<()> {
    let startup_args = env::args_os().skip(1).collect();
    let mut app = App::load(startup_args)?;

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
    /// Main viewport from the last completed frame, reused for immediate dispatch.
    viewport: Rect,
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
    /// Interactive command waiting for a top-level terminal handoff.
    pending_interactive_action: Option<PendingInteractiveAction>,
    /// Active search query shared with the current view.
    search: Option<SearchQuery>,
    /// Exit flag set by app-level quit handling.
    should_quit: bool,
    /// Injected seam for jj, refresh, and alternate-view side effects.
    services: AppServices,
}

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

/// Interactive app action that must run with the live terminal at the app boundary.
enum PendingInteractiveAction {
    Split {
        split: JjSplitPlan,
        status_context: Option<String>,
        viewport_height: u16,
    },
}

impl App {
    /// Drive the terminal redraw and input loop until the app requests exit.
    ///
    /// The runtime path is deliberate: draw the current view, snapshot the main viewport, then
    /// either dispatch the next terminal event or expire any pending multi-key prefix.
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            let completed_frame = terminal.draw(|frame| self.render(frame))?;
            let completed_viewport = viewport_from_completed_frame(completed_frame.area);
            self.viewport = completed_viewport;

            if event::poll(Duration::from_millis(200))? {
                self.handle_event(event::read()?)?;
            } else {
                self.flush_expired_pending_command(completed_viewport.height)?;
            }
            self.run_pending_interactive_action(Some(terminal))?;
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

    /// Route one terminal event into resize handling or pressed-key dispatch.
    ///
    /// Non-press key events and unrelated terminal events are ignored here.
    fn handle_event(&mut self, event: event::Event) -> Result<()> {
        match event {
            event::Event::Resize(width, height) => {
                self.handle_resize(width, height);
                Ok(())
            }
            event::Event::Key(key) if key.is_press() => {
                self.handle_key_press(key, self.viewport.height)
            }
            _ => Ok(()),
        }
    }

    /// Clamp the active view to the new terminal size and refresh ready status text.
    fn handle_resize(&mut self, width: u16, height: u16) {
        self.viewport = viewport_from_terminal_size(width, height);
        self.view.clamp(viewport_size(self.viewport));
        if self.status.is_ready() {
            self.status = StatusLine::ready(&self.view);
        }
    }

    /// Rebuild the active view and convert refresh failure into status.
    ///
    /// Refresh can run after external side effects, so the follow-up clamp uses a fresh terminal
    /// size snapshot instead of reusing geometry from the initiating frame.
    fn refresh(&mut self) {
        if let Err(error) = self.refresh_view_state() {
            self.status = StatusLine::error(&self.view, error.to_string());
            return;
        }

        clamp_view_to_current_viewport(&mut self.view);
        self.status = StatusLine::ready(&self.view);
    }

    /// Ask the active view slice to interpret one view-local command.
    ///
    /// Immediate dispatch reuses the main viewport from the last completed frame so height and
    /// width stay consistent for one event-loop turn.
    fn execute_view(&mut self, command: ViewCommand) -> ViewEffect {
        self.view.execute(
            command,
            CommandContext {
                size: viewport_size(self.viewport),
                search: self.search.as_ref(),
            },
        )
    }
}

/// Build the app's main viewport area from the last completed frame.
fn viewport_from_completed_frame(area: Rect) -> Rect {
    tui::areas(area).main
}

/// Clamp one view to the live main viewport using a single size snapshot.
///
/// Post-refresh and post-action clamp paths use this helper because the terminal may have changed
/// while an external command, reveal step, or terminal handoff was in flight. Sampling height and
/// width together keeps the clamp inputs consistent with each other.
fn clamp_view_to_current_viewport(view: &mut ViewState) {
    let viewport = current_viewport_rect();
    view.clamp(viewport_size(viewport));
}

/// Read the current main viewport area from one terminal-size snapshot.
fn current_viewport_rect() -> Rect {
    crossterm::terminal::size()
        .map(|(width, height)| viewport_from_terminal_size(width, height))
        .unwrap_or(Rect {
            x: 0,
            y: 0,
            height: u16::MAX,
            width: u16::MAX,
        })
}

/// Build the app's main viewport area from the raw terminal size.
fn viewport_from_terminal_size(width: u16, height: u16) -> Rect {
    Rect {
        x: 0,
        y: 0,
        height: height.saturating_sub(2),
        width,
    }
}

/// Convert one viewport area into the dimensions view dispatch and clamping need.
fn viewport_size(viewport: Rect) -> Size {
    Size {
        height: viewport.height,
        width: viewport.width,
    }
}
