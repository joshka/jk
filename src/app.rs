//! Terminal event loop and app-level orchestration.
//!
//! Feature slices own their view behavior. The app owns cross-cutting concerns:
//! key dispatch, pending key-prefix state, mode handoff, refresh, search state,
//! and routing view effects to the app submodule that owns the detailed policy.

use std::env;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::app_screen::InteractionMode;
use crate::app_status::{StatusKind, StatusLine};
use crate::command::{
    Binding, BindingMatch, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect,
    match_binding_sequence,
};
use crate::jj::{DiffFormat, JjBookmarkMutationKind, JjWorkingCopyNavigationKind};
use crate::search::SearchQuery;
use crate::tui;
use crate::view_state::ViewState;

mod action_flow;
mod action_lifecycle;
mod mode_input;
mod navigation;
mod services;

use self::services::AppServices;

pub fn run() -> Result<()> {
    let mut app = App::load(env::args_os().skip(1).collect())?;

    ratatui::run(|terminal| app.run(terminal))
}

struct App {
    view: ViewState,
    stack: Vec<ViewState>,
    startup_log_args: Option<Vec<String>>,
    diff_format: DiffFormat,
    status: StatusLine,
    mode: InteractionMode,
    pending_command: Option<PendingCommand>,
    search: Option<SearchQuery>,
    should_quit: bool,
    services: AppServices,
}

const APP_BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('q'), Command::Quit),
    Binding::new(KeyPattern::code(KeyCode::Esc), Command::Quit),
    Binding::new(KeyPattern::char('?'), Command::Help),
    Binding::new(KeyPattern::char('/'), Command::SearchPrompt),
    Binding::new(KeyPattern::char('W'), Command::PromptLogRevset),
    Binding::new(KeyPattern::char('S'), Command::OpenStatus),
    Binding::new(KeyPattern::char('R'), Command::OpenResolve),
    Binding::new(KeyPattern::char('B'), Command::OpenBookmarks),
    Binding::new(KeyPattern::char('O'), Command::OpenOperationLog),
    Binding::new(KeyPattern::char('D'), Command::Describe),
    Binding::new(KeyPattern::char('C'), Command::Commit),
    Binding::new(KeyPattern::char('b'), Command::BookmarkCreate),
    Binding::sequence(BOOKMARK_CREATE_KEYS, Command::BookmarkCreate),
    Binding::sequence(BOOKMARK_RENAME_KEYS, Command::BookmarkRename),
    Binding::new(KeyPattern::char('='), Command::BookmarkSet),
    Binding::new(KeyPattern::char('m'), Command::BookmarkMove),
    Binding::new(KeyPattern::char('f'), Command::Fetch),
    Binding::new(KeyPattern::char('F'), Command::FetchRemote),
    Binding::new(KeyPattern::char('y'), Command::Copy),
    Binding::new(KeyPattern::char('p'), Command::Push),
    Binding::new(KeyPattern::char('v'), Command::ViewFormat),
    Binding::new(KeyPattern::char('r'), Command::Refresh),
    Binding::new(KeyPattern::char('h'), Command::Back),
    Binding::new(KeyPattern::code(KeyCode::Left), Command::Back),
    Binding::new(KeyPattern::char('L'), Command::SwitchLog),
    Binding::new(KeyPattern::char('J'), Command::SwitchDefault),
];

const BOOKMARK_CREATE_KEYS: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('c')];
const BOOKMARK_RENAME_KEYS: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('r')];
const COMMAND_PREFIX_TIMEOUT: Duration = Duration::from_millis(700);

fn current_viewport_width() -> u16 {
    crossterm::terminal::size()
        .map(|(width, _)| width)
        .unwrap_or(u16::MAX)
}

#[derive(Clone)]
struct PendingCommand {
    keys: Vec<crossterm::event::KeyEvent>,
    fallback: Option<Binding>,
    deadline: Instant,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| {
                let status = self.mode.status_line(&self.view, &self.status);
                let areas: tui::Areas = tui::areas(frame.area());
                tui::render_chrome(frame, areas, &status);
                self.view.render(frame, areas.main, self.search.as_ref());
                tui::render_overlay(frame, &status, self.mode.overlay(&self.view, APP_BINDINGS));
            })?;

            let viewport_height = terminal.size()?.height.saturating_sub(2);
            if event::poll(Duration::from_millis(200))? {
                self.handle_event(terminal, event::read()?, viewport_height)?;
            } else {
                self.flush_expired_pending_command(viewport_height)?;
            }
        }

        Ok(())
    }

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

    fn handle_normal_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
    ) -> Result<bool> {
        self.handle_normal_key_at(key, viewport_height, Instant::now())
    }

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
                    format!("prefix: {}", binding_key_label(&keys)),
                );
                Ok(false)
            }
        }
    }

    fn handle_pending_command_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if self
            .pending_command
            .as_ref()
            .is_some_and(|pending| now >= pending.deadline)
        {
            self.run_pending_fallback(viewport_height)?;
            return self.handle_key_after_prefix_fallback(key, viewport_height, now);
        }

        if key.code == KeyCode::Esc {
            self.pending_command = None;
            self.status = StatusLine::with_message(&self.view, "prefix cancelled");
            return Ok(false);
        }

        let Some(mut pending) = self.pending_command.take() else {
            return Ok(true);
        };
        pending.keys.push(key);

        match match_binding_sequence(&[APP_BINDINGS, self.view.bindings()], &pending.keys) {
            Some(BindingMatch::Exact(binding)) => self.execute_binding(binding, viewport_height),
            Some(BindingMatch::Prefix { fallback }) => {
                self.status = StatusLine::with_message(
                    &self.view,
                    format!("prefix: {}", binding_key_label(&pending.keys)),
                );
                self.pending_command = Some(PendingCommand {
                    keys: pending.keys,
                    fallback,
                    deadline: now + COMMAND_PREFIX_TIMEOUT,
                });
                Ok(false)
            }
            None => {
                if let Some(fallback) = pending.fallback {
                    self.run_binding_with_status_refresh(fallback, viewport_height)?;
                    self.handle_key_after_prefix_fallback(key, viewport_height, now)
                } else {
                    self.status = StatusLine::with_message(&self.view, "unknown command prefix");
                    Ok(false)
                }
            }
        }
    }

    fn flush_expired_pending_command(&mut self, viewport_height: u16) -> Result<()> {
        let Some(pending) = self.pending_command.as_ref() else {
            return Ok(());
        };
        if Instant::now() < pending.deadline {
            return Ok(());
        }

        self.run_pending_fallback(viewport_height)?;
        Ok(())
    }

    fn run_pending_fallback(&mut self, viewport_height: u16) -> Result<()> {
        let fallback = self
            .pending_command
            .take()
            .and_then(|pending| pending.fallback);
        if matches!(self.mode, InteractionMode::Help) {
            if let Some(binding) = fallback {
                self.execute_help_binding(binding, viewport_height)?;
            } else {
                self.status = StatusLine::with_message(&self.view, "unknown help command prefix");
            }
        } else if let Some(binding) = fallback {
            self.run_binding_with_status_refresh(binding, viewport_height)?;
        } else {
            self.status = StatusLine::ready(&self.view);
        }
        Ok(())
    }

    fn run_binding_with_status_refresh(
        &mut self,
        binding: Binding,
        viewport_height: u16,
    ) -> Result<()> {
        let refresh_status = self.execute_binding(binding, viewport_height)?;
        if refresh_status && matches!(self.status.kind(), StatusKind::Ready) {
            self.status = StatusLine::ready(&self.view);
        }
        Ok(())
    }

    fn handle_key_after_prefix_fallback(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if matches!(self.mode, InteractionMode::Normal) {
            self.handle_normal_key_at(key, viewport_height, now)
        } else {
            self.handle_mode_key_event(key, viewport_height)
        }
    }

    fn execute_binding(&mut self, binding: Binding, viewport_height: u16) -> Result<bool> {
        match binding.command() {
            Command::Quit => {
                self.should_quit = true;
                Ok(false)
            }
            Command::Help => {
                self.mode = InteractionMode::Help;
                Ok(true)
            }
            Command::SearchPrompt => {
                self.mode = InteractionMode::SearchPrompt(String::new());
                Ok(true)
            }
            Command::PromptLogRevset => {
                self.open_log_revset_prompt();
                Ok(true)
            }
            Command::OpenStatus => {
                self.open_status()?;
                Ok(true)
            }
            Command::OpenResolve => {
                self.open_resolve()?;
                Ok(true)
            }
            Command::OpenBookmarks => {
                self.open_bookmarks()?;
                Ok(true)
            }
            Command::OpenOperationLog => {
                self.open_operation_log()?;
                Ok(true)
            }
            Command::OperationUndo | Command::OperationRedo => {
                if let Some(kind) = binding.command().operation_recovery() {
                    self.open_operation_recovery_preview(kind);
                }
                Ok(false)
            }
            Command::Edit => {
                self.open_graph_working_copy_navigation_preview(JjWorkingCopyNavigationKind::Edit);
                Ok(false)
            }
            Command::NextEdit => {
                self.open_graph_working_copy_navigation_preview(JjWorkingCopyNavigationKind::Next);
                Ok(false)
            }
            Command::PrevEdit => {
                self.open_graph_working_copy_navigation_preview(JjWorkingCopyNavigationKind::Prev);
                Ok(false)
            }
            Command::Describe => {
                self.open_describe_prompt();
                Ok(false)
            }
            Command::Commit => {
                self.open_commit_prompt();
                Ok(false)
            }
            Command::BookmarkCreate => {
                self.open_bookmark_name_prompt(JjBookmarkMutationKind::Create);
                Ok(false)
            }
            Command::BookmarkSet => {
                self.open_bookmark_name_prompt(JjBookmarkMutationKind::Set);
                Ok(false)
            }
            Command::BookmarkMove => {
                self.open_bookmark_name_prompt(JjBookmarkMutationKind::Move);
                Ok(false)
            }
            Command::BookmarkRename => {
                self.open_bookmark_rename_prompt();
                Ok(false)
            }
            Command::BookmarkDelete => {
                self.open_bookmark_delete_preview();
                Ok(false)
            }
            Command::Fetch => {
                self.fetch(viewport_height);
                Ok(false)
            }
            Command::FetchRemote => {
                self.open_fetch_remote_prompt();
                Ok(false)
            }
            Command::Push => self.open_push_prompt(),
            Command::Copy => {
                self.open_copy_menu(viewport_height);
                Ok(true)
            }
            Command::ViewFormat => {
                self.open_view_menu();
                Ok(true)
            }
            Command::Refresh => {
                self.refresh(viewport_height);
                Ok(false)
            }
            Command::Back => {
                self.pop_view();
                Ok(true)
            }
            Command::SwitchLog => {
                self.switch_to_log()?;
                Ok(true)
            }
            Command::SwitchDefault => {
                self.switch_to_default()?;
                Ok(true)
            }
            Command::View(ViewCommand::OpenActionMenu) => self.open_action_menu(viewport_height),
            Command::View(command) => {
                let effect = self.execute_view(command, viewport_height);
                self.apply_view_effect(effect, viewport_height)
            }
        }
    }

    fn refresh(&mut self, viewport_height: u16) {
        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                self.status = StatusLine::ready(&self.view);
            }
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

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

    fn apply_view_effect(&mut self, effect: ViewEffect, viewport_height: u16) -> Result<bool> {
        match effect {
            ViewEffect::Ignored | ViewEffect::Handled => Ok(true),
            ViewEffect::StatusMessage(message) => {
                self.status = StatusLine::with_message(&self.view, message);
                Ok(false)
            }
            ViewEffect::StatusError(message) => {
                self.status = StatusLine::error(&self.view, message);
                Ok(false)
            }
            ViewEffect::RunNewTrunk => {
                self.run_new_trunk(viewport_height);
                Ok(false)
            }
            ViewEffect::OpenDetail(command, revset) => {
                self.push_detail(command, revset)?;
                Ok(true)
            }
            ViewEffect::OpenView(spec) => {
                self.push_view(spec)?;
                Ok(true)
            }
            ViewEffect::SearchMoved => {
                if let Some(query) = &self.search {
                    self.status =
                        StatusLine::with_message(&self.view, format!("search: {}", query.text()));
                }
                Ok(false)
            }
            ViewEffect::SearchStarted { matches } => {
                self.status = StatusLine::with_message(&self.view, format!("{matches} matches"));
                Ok(false)
            }
            ViewEffect::OpenActionMenu(menu) => {
                self.mode = InteractionMode::ActionMenu { menu, selected: 0 };
                Ok(false)
            }
            ViewEffect::CopyOptions(options) => {
                if options.is_empty() {
                    self.status = StatusLine::with_message(&self.view, "nothing to copy");
                } else {
                    self.mode = InteractionMode::CopyMenu {
                        options,
                        selected: 0,
                    };
                }
                Ok(false)
            }
        }
    }
}

fn binding_key_label(keys: &[crossterm::event::KeyEvent]) -> String {
    keys.iter()
        .map(|key| match key.code {
            KeyCode::Char(character) if key.modifiers.is_empty() => character.to_string(),
            KeyCode::Char(character) => format!("{:?}-{character}", key.modifiers),
            _ => format!("{:?}", key.code),
        })
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests;
