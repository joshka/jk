//! Terminal event loop and app-level modes.
//!
//! Feature slices own their view behavior. The app owns cross-cutting concerns:
//! key dispatch, modal state, the view stack, search state, and the selected
//! diff format used when opening detail views.

use std::env;
use std::ffi::OsString;
use std::time::Duration;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::action_menu::{ActionMenu, FollowUp, RolePrompt};
use crate::clipboard;
use crate::command::{
    Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect, find_binding,
    project_help,
};
use crate::copy::CopyOption;
use crate::jj::{
    DiffFormat, JjCommand, JjGitPush, JjGitPushTarget, LogViewMode, ViewSpec, git_fetch,
    git_remotes, new_trunk, resolve_exact_change_id,
};
use crate::search::SearchQuery;
use crate::tui::{self, Overlay, StatusHints};
use crate::view_state::ViewState;

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
    search: Option<SearchQuery>,
    should_quit: bool,
}

enum InteractionMode {
    Normal,
    Help,
    SearchPrompt(String),
    LogRevsetPrompt(String),
    CopyMenu {
        options: Vec<CopyOption>,
        selected: usize,
    },
    ViewMenu {
        selected: usize,
    },
    ActionMenu {
        menu: ActionMenu,
        selected: usize,
    },
    RolePrompt {
        prompt: RolePrompt,
        selected: usize,
    },
    PushRemotePrompt {
        target: JjGitPushTarget,
        remotes: Vec<String>,
        selected: usize,
    },
    PushPreview {
        push: JjGitPush,
        command_label: String,
        preview_output: String,
        status_context: Option<String>,
        completed: bool,
    },
}

const APP_BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('q'), Command::Quit),
    Binding::new(KeyPattern::code(KeyCode::Esc), Command::Quit),
    Binding::new(KeyPattern::char('?'), Command::Help),
    Binding::new(KeyPattern::char('/'), Command::SearchPrompt),
    Binding::new(KeyPattern::char('W'), Command::PromptLogRevset),
    Binding::new(KeyPattern::char('S'), Command::OpenStatus),
    Binding::new(KeyPattern::char('B'), Command::OpenBookmarks),
    Binding::new(KeyPattern::char('O'), Command::OpenOperationLog),
    Binding::new(KeyPattern::char('f'), Command::Fetch),
    Binding::new(KeyPattern::char('y'), Command::Copy),
    Binding::new(KeyPattern::char('p'), Command::Push),
    Binding::new(KeyPattern::char('v'), Command::ViewFormat),
    Binding::new(KeyPattern::char('r'), Command::Refresh),
    Binding::new(KeyPattern::char('h'), Command::Back),
    Binding::new(KeyPattern::code(KeyCode::Left), Command::Back),
    Binding::new(KeyPattern::char('L'), Command::SwitchLog),
    Binding::new(KeyPattern::char('J'), Command::SwitchDefault),
];

impl App {
    fn load(args: Vec<OsString>) -> Result<Self> {
        let initial_spec = initial_view(args)?;
        let startup_log_args =
            (initial_spec.command() == JjCommand::Log).then(|| initial_spec.args().to_vec());
        let diff_format = initial_spec.diff_format();
        let view = ViewState::load(initial_spec)?;
        let status = StatusLine::ready(&view);

        Ok(Self {
            view,
            stack: Vec::new(),
            startup_log_args,
            diff_format,
            status,
            mode: InteractionMode::Normal,
            search: None,
            should_quit: false,
        })
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| {
                let status = self.render_status();
                let areas: tui::Areas = tui::areas(frame.area());
                tui::render_chrome(frame, areas, &status);
                self.view.render(frame, areas.main, self.search.as_ref());
                tui::render_overlay(frame, &status, self.overlay());
            })?;

            if event::poll(Duration::from_millis(200))? {
                let viewport_height = terminal.size()?.height.saturating_sub(2);
                self.handle_event(event::read()?, viewport_height)?;
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event, viewport_height: u16) -> Result<()> {
        let Event::Key(key) = event else {
            return Ok(());
        };

        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        if self.handle_mode_key(key.code, viewport_height)? {
            return Ok(());
        }

        let refresh_status = self.handle_normal_key(key, viewport_height)?;
        if refresh_status && matches!(self.status.kind, StatusKind::Ready) {
            self.status = StatusLine::ready(&self.view);
        }

        Ok(())
    }

    fn handle_normal_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
    ) -> Result<bool> {
        let Some(binding) =
            find_binding(APP_BINDINGS, key).or_else(|| find_binding(self.view.bindings(), key))
        else {
            return Ok(true);
        };

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
            Command::OpenBookmarks => {
                self.open_bookmarks()?;
                Ok(true)
            }
            Command::OpenOperationLog => {
                self.open_operation_log()?;
                Ok(true)
            }
            Command::Fetch => {
                self.fetch(viewport_height);
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
            Command::View(command) => {
                let effect = self.execute_view(command, viewport_height);
                self.apply_view_effect(effect, viewport_height)
            }
        }
    }

    fn refresh(&mut self, viewport_height: u16) {
        match self.view.refresh() {
            Ok(()) => {
                self.view.clamp(viewport_height);
                self.status = StatusLine::ready(&self.view);
            }
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    fn fetch(&mut self, viewport_height: u16) {
        match git_fetch() {
            Ok(output) => match self.view.refresh() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
                    self.status = StatusLine::with_message(
                        &self.view,
                        format!("fetch: {}", output.message()),
                    );
                }
                Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
            },
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    fn handle_mode_key(&mut self, code: KeyCode, viewport_height: u16) -> Result<bool> {
        match &mut self.mode {
            InteractionMode::Normal => Ok(false),
            InteractionMode::Help => {
                match code {
                    KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
                        self.mode = InteractionMode::Normal;
                    }
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::SearchPrompt(input) => {
                match code {
                    KeyCode::Esc => self.mode = InteractionMode::Normal,
                    KeyCode::Enter => {
                        self.search = SearchQuery::new(input.clone());
                        self.mode = InteractionMode::Normal;
                        self.status = if self.search.is_some() {
                            match self.execute_view(ViewCommand::StartSearch, viewport_height) {
                                ViewEffect::SearchStarted { matches } => StatusLine::with_message(
                                    &self.view,
                                    format!("{matches} matches"),
                                ),
                                _ => StatusLine::ready(&self.view),
                            }
                        } else {
                            StatusLine::ready(&self.view)
                        };
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(character) => input.push(character),
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::LogRevsetPrompt(input) => {
                match code {
                    KeyCode::Esc => self.mode = InteractionMode::Normal,
                    KeyCode::Enter => {
                        let revset = std::mem::take(input);
                        self.mode = InteractionMode::Normal;
                        self.apply_custom_log_revset(revset);
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(character) => input.push(character),
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::CopyMenu { options, selected } => {
                match code {
                    KeyCode::Esc | KeyCode::Char('q') => self.mode = InteractionMode::Normal,
                    KeyCode::Char('j') | KeyCode::Down if *selected + 1 < options.len() => {
                        *selected += 1;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        *selected = selected.saturating_sub(1);
                    }
                    KeyCode::Enter => {
                        if let Some(option) = options.get(*selected) {
                            match clipboard::copy(option.value()) {
                                Ok(()) => {
                                    self.status = StatusLine::with_message(
                                        &self.view,
                                        format!("copied {}", option.label()),
                                    );
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                }
                            }
                        }
                        self.mode = InteractionMode::Normal;
                    }
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::ViewMenu { selected } => {
                match code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                        self.mode = InteractionMode::Normal;
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        *selected = (*selected + 1).min(view_formats().len().saturating_sub(1));
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        *selected = selected.saturating_sub(1);
                    }
                    KeyCode::Enter => {
                        let diff_format = view_formats()[*selected].format();
                        self.mode = InteractionMode::Normal;
                        self.apply_diff_format(diff_format, viewport_height)?;
                    }
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::ActionMenu { menu, selected } => {
                match code {
                    KeyCode::Esc | KeyCode::Char('q') => self.mode = InteractionMode::Normal,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if *selected + 1 < menu.items().len() {
                            *selected += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        *selected = selected.saturating_sub(1);
                    }
                    KeyCode::Enter => {
                        if let Some(action) = menu.items().get(*selected) {
                            match action.follow_up() {
                                FollowUp::StatusMessage(message) => {
                                    self.status =
                                        StatusLine::with_message(&self.view, message.as_str());
                                    self.mode = InteractionMode::Normal;
                                }
                                FollowUp::RolePrompt(prompt) => {
                                    self.mode = InteractionMode::RolePrompt {
                                        prompt: prompt.clone(),
                                        selected: 0,
                                    };
                                }
                            }
                        } else {
                            self.mode = InteractionMode::Normal;
                        }
                    }
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::RolePrompt { prompt, selected } => {
                match code {
                    KeyCode::Esc | KeyCode::Char('q') => self.mode = InteractionMode::Normal,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if *selected + 1 < prompt.options().len() {
                            *selected += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        *selected = selected.saturating_sub(1);
                    }
                    KeyCode::Enter => {
                        let next_status = prompt.status_message();
                        self.mode = InteractionMode::Normal;
                        self.status = StatusLine::with_message(&self.view, next_status);
                    }
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::PushRemotePrompt {
                target,
                remotes,
                selected,
            } => {
                match code {
                    KeyCode::Esc | KeyCode::Char('q') => self.mode = InteractionMode::Normal,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if *selected + 1 < remotes.len() {
                            *selected += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        *selected = selected.saturating_sub(1);
                    }
                    KeyCode::Enter => {
                        let target = target.clone();
                        let selected_remote = remotes.get(*selected).cloned();
                        self.mode = InteractionMode::Normal;
                        match selected_remote {
                            Some(remote) => self.open_push_preview(target, remote),
                            None => {
                                self.status = StatusLine::error(
                                    &self.view,
                                    "no remote selected for push".to_owned(),
                                );
                            }
                        }
                    }
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::PushPreview {
                push,
                command_label: _,
                preview_output: _,
                status_context,
                completed,
            } => {
                let (push, status_context, completed) =
                    { (push.clone(), status_context.clone(), *completed) };
                match code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.mode = InteractionMode::Normal;
                        if !completed {
                            self.status =
                                StatusLine::with_message(&self.view, "push cancelled".to_owned());
                        }
                    }
                    KeyCode::Enter => {
                        if completed {
                            self.mode = InteractionMode::Normal;
                            return Ok(true);
                        }

                        self.confirm_push(push, status_context, viewport_height);
                    }
                    _ => {}
                }
                Ok(true)
            }
        }
    }

    fn open_copy_menu(&mut self, viewport_height: u16) {
        let options = match self.execute_view(ViewCommand::Copy, viewport_height) {
            ViewEffect::CopyOptions(options) => options,
            _ => Vec::new(),
        };
        if options.is_empty() {
            self.status = StatusLine::with_message(&self.view, "nothing to copy");
        } else {
            self.mode = InteractionMode::CopyMenu {
                options,
                selected: 0,
            };
        }
    }

    fn open_log_revset_prompt(&mut self) {
        if matches!(self.view.command(), JjCommand::Default | JjCommand::Log) {
            self.mode = InteractionMode::LogRevsetPrompt(String::new());
        }
    }

    fn open_push_prompt(&mut self) -> Result<bool> {
        let target = match self.view.push_target() {
            Ok(Some(target)) => target,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "push is only available from graph, status, or bookmarks views".to_owned(),
                );
                return Ok(false);
            }
            Err(message) => {
                self.status = StatusLine::error(&self.view, message.to_string());
                return Ok(false);
            }
        };

        match git_remotes() {
            Ok(remotes) => {
                if remotes.is_empty() {
                    self.status = StatusLine::error(
                        &self.view,
                        "no git remotes found; add a remote before pushing".to_owned(),
                    );
                    return Ok(false);
                }

                self.mode = InteractionMode::PushRemotePrompt {
                    target,
                    remotes,
                    selected: 0,
                };
                Ok(false)
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                Ok(false)
            }
        }
    }

    fn open_push_preview(&mut self, target: JjGitPushTarget, remote: String) {
        let status_context = match &target {
            JjGitPushTarget::Status => Some(format!(
                "status push uses jj default target for remote {remote}"
            )),
            _ => None,
        };
        let push = match target {
            JjGitPushTarget::Bookmark(name) => JjGitPush::for_bookmark(name).with_remote(remote),
            JjGitPushTarget::Revision(name) => JjGitPush::for_revision(name).with_remote(remote),
            JjGitPushTarget::Status => JjGitPush::for_status().with_remote(remote),
        };

        match push.run_preview() {
            Ok(output) => {
                let command_label = push.command_label(true);
                self.mode = InteractionMode::PushPreview {
                    push,
                    command_label,
                    preview_output: output.message().to_owned(),
                    status_context,
                    completed: false,
                };
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                self.mode = InteractionMode::Normal;
            }
        }
    }

    fn confirm_push(
        &mut self,
        push: JjGitPush,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = push.command_label(false);
        let result_message = match push.run() {
            Ok(output) => match self.view.refresh() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
                    let message = output.message().to_owned();
                    self.status = StatusLine::with_message(&self.view, message.as_str());
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!("refresh failed: {error}")
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::PushPreview {
            push,
            command_label,
            preview_output: result_message,
            status_context,
            completed: true,
        }
    }

    fn execute_view(&mut self, command: ViewCommand, viewport_height: u16) -> ViewEffect {
        self.view.execute(
            command,
            CommandContext {
                viewport_height,
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

    fn open_view_menu(&mut self) {
        let selected = view_formats()
            .iter()
            .position(|option| option.format == self.diff_format)
            .unwrap_or(0);
        self.mode = InteractionMode::ViewMenu { selected };
    }

    fn apply_custom_log_revset(&mut self, revset: String) {
        if revset.trim().is_empty() {
            self.status = StatusLine::ready(&self.view);
            return;
        }

        match self.view.set_graph_mode(LogViewMode::CustomRevset(revset)) {
            Ok(()) => self.status = StatusLine::with_message(&self.view, "mode: custom revset"),
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    fn run_new_trunk(&mut self, viewport_height: u16) {
        if let Err(error) = resolve_exact_change_id("trunk()") {
            self.status = StatusLine::error(&self.view, error.to_string());
            return;
        }

        match new_trunk() {
            Ok(_) => {
                let new_change_id = match resolve_exact_change_id("@") {
                    Ok(change_id) => change_id,
                    Err(error) => {
                        self.status = StatusLine::error(&self.view, error.to_string());
                        return;
                    }
                };
                match self.view.refresh() {
                    Ok(()) => {
                        self.view.clamp(viewport_height);
                        let revealed_in_recent = match self
                            .view
                            .reveal_graph_change(&new_change_id, LogViewMode::Recent)
                        {
                            Ok(switched_modes) => {
                                self.view.clamp(viewport_height);
                                switched_modes
                            }
                            Err(error) => {
                                self.status = StatusLine::error(&self.view, error.to_string());
                                return;
                            }
                        };
                        let message = if revealed_in_recent {
                            "created new change from trunk | showing recent work | jj undo"
                        } else {
                            "created new change from trunk | jj undo"
                        };
                        self.status = StatusLine::with_message(&self.view, message);
                    }
                    Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
                }
            }
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    fn apply_diff_format(&mut self, diff_format: DiffFormat, viewport_height: u16) -> Result<()> {
        self.diff_format = diff_format;
        if !matches!(self.view.command(), JjCommand::Show | JjCommand::Diff) {
            self.status =
                StatusLine::with_message(&self.view, format!("view: {}", diff_format.label()));
            return Ok(());
        }

        let scroll_offset = self.view.scroll_offset();
        let spec = self.view.spec().with_diff_format(diff_format);
        self.view = ViewState::load(spec)?;
        self.view.set_scroll_offset(viewport_height, scroll_offset);
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    fn render_status(&self) -> StatusLine {
        match &self.mode {
            InteractionMode::SearchPrompt(input) => {
                StatusLine::with_message(&self.view, format!("/{input}"))
            }
            InteractionMode::LogRevsetPrompt(input) => {
                StatusLine::with_message(&self.view, format!("revset: {input}"))
            }
            _ => self.status.clone(),
        }
    }

    fn overlay(&self) -> Overlay<'_> {
        match &self.mode {
            InteractionMode::Help => Overlay::Help {
                sections: project_help(
                    APP_BINDINGS,
                    self.view.bindings(),
                    self.view.help_context(),
                ),
            },
            InteractionMode::CopyMenu { options, selected } => Overlay::CopyMenu {
                options,
                selected: *selected,
            },
            InteractionMode::ViewMenu { selected } => Overlay::ViewMenu {
                options: view_formats(),
                selected: *selected,
            },
            InteractionMode::ActionMenu { menu, selected } => Overlay::ActionMenu {
                menu,
                selected: *selected,
            },
            InteractionMode::RolePrompt { prompt, selected } => Overlay::RolePrompt {
                prompt,
                selected: *selected,
            },
            InteractionMode::PushRemotePrompt {
                remotes, selected, ..
            } => Overlay::PushRemotePrompt {
                remotes,
                selected: *selected,
            },
            InteractionMode::PushPreview {
                command_label,
                preview_output,
                status_context,
                completed,
                ..
            } => Overlay::PushPreview {
                command_label: command_label.as_str(),
                preview_output: preview_output.as_str(),
                status_context: status_context.as_ref(),
                completed: *completed,
            },
            InteractionMode::Normal
            | InteractionMode::SearchPrompt(_)
            | InteractionMode::LogRevsetPrompt(_) => Overlay::None,
        }
    }

    fn push_detail(&mut self, command: JjCommand, revset: String) -> Result<()> {
        let spec = match command {
            JjCommand::Show => ViewSpec::show(revset, self.diff_format),
            JjCommand::Diff => ViewSpec::diff(revset, self.diff_format),
            JjCommand::FileShow => {
                ViewSpec::file_show(self.view.spec().navigation_revset(), revset)
            }
            JjCommand::Default
            | JjCommand::Log
            | JjCommand::Status
            | JjCommand::FileList
            | JjCommand::Bookmarks
            | JjCommand::OperationLog => return Ok(()),
        };
        self.push_view(spec)
    }

    fn open_status(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Status) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Status, Vec::new()))
    }

    fn open_operation_log(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::OperationLog) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::OperationLog, Vec::new()))
    }

    fn open_bookmarks(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Bookmarks) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Bookmarks, Vec::new()))
    }

    fn push_view(&mut self, spec: ViewSpec) -> Result<()> {
        let next = ViewState::load(spec)?;
        let previous = std::mem::replace(&mut self.view, next);
        self.stack.push(previous);
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    fn pop_view(&mut self) {
        if let Some(previous) = self.stack.pop() {
            self.view = previous;
            self.status = StatusLine::ready(&self.view);
        }
    }

    fn switch_to_log(&mut self) -> Result<()> {
        let args = self.startup_log_args.clone().unwrap_or_default();
        self.stack.clear();
        self.view = ViewState::load(ViewSpec::new(JjCommand::Log, args))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    fn switch_to_default(&mut self) -> Result<()> {
        self.stack.clear();
        self.view = ViewState::load(ViewSpec::new(JjCommand::Default, Vec::new()))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ViewFormatOption {
    label: &'static str,
    format: DiffFormat,
}

impl ViewFormatOption {
    pub fn label(self) -> &'static str {
        self.label
    }

    pub fn format(self) -> DiffFormat {
        self.format
    }
}

pub fn view_formats() -> &'static [ViewFormatOption] {
    &[
        ViewFormatOption {
            label: "default jj diff",
            format: DiffFormat::Default,
        },
        ViewFormatOption {
            label: "git diff (--git)",
            format: DiffFormat::Git,
        },
    ]
}

fn initial_view(args: Vec<OsString>) -> Result<ViewSpec> {
    let args = args
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|arg| eyre!("argument is not valid UTF-8: {arg:?}"))
        })
        .collect::<Result<Vec<_>>>()?;

    let Some((command, rest)) = args.split_first() else {
        return Ok(ViewSpec::new(JjCommand::Default, Vec::new()));
    };

    match command.as_str() {
        "log" => Ok(ViewSpec::new(JjCommand::Log, rest.to_vec())),
        "show" => Ok(ViewSpec::new(JjCommand::Show, rest.to_vec())),
        "diff" => Ok(ViewSpec::new(JjCommand::Diff, rest.to_vec())),
        "status" => Ok(ViewSpec::new(JjCommand::Status, rest.to_vec())),
        "bookmarks" => Ok(ViewSpec::new(JjCommand::Bookmarks, rest.to_vec())),
        "operation-log" => Ok(ViewSpec::new(JjCommand::OperationLog, rest.to_vec())),
        unknown => Err(eyre!(
            "unsupported jk command '{unknown}'. Expected one of: log, show, diff, status, bookmarks, operation-log"
        )),
    }
}

#[derive(Clone, Debug)]
pub struct StatusLine {
    title: String,
    message: String,
    kind: StatusKind,
    hints: StatusHints,
}

impl StatusLine {
    fn ready(view: &ViewState) -> Self {
        let message = if let Some(item_count) = view.item_count() {
            item_count_message(view, item_count)
        } else {
            format!(
                "{}/{} lines",
                view.scroll_offset()
                    .saturating_add(1)
                    .min(view.document_line_count()),
                view.document_line_count()
            )
        };
        Self {
            title: view.spec().app_label(),
            message,
            kind: StatusKind::Ready,
            hints: view.status_hints(),
        }
    }

    fn error(view: &ViewState, message: String) -> Self {
        Self {
            title: view.spec().app_label(),
            message,
            kind: StatusKind::Error,
            hints: view.status_hints(),
        }
    }

    fn with_message(view: &ViewState, message: impl Into<String>) -> Self {
        Self {
            title: view.spec().app_label(),
            message: message.into(),
            kind: StatusKind::Ready,
            hints: view.status_hints(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn kind(&self) -> &StatusKind {
        &self.kind
    }

    pub fn hints(&self) -> StatusHints {
        self.hints
    }
}

fn graph_status_message(item_count: usize, mode_label: Option<&str>) -> String {
    let base = format!("{item_count} items");
    match mode_label {
        Some(mode_label) => format!("{base} | {mode_label}"),
        None => base,
    }
}

fn item_count_message(view: &ViewState, item_count: usize) -> String {
    match view.command() {
        JjCommand::FileList => format!("{item_count} files"),
        JjCommand::Bookmarks => format!("{item_count} bookmarks"),
        JjCommand::OperationLog => format!("{item_count} operations"),
        JjCommand::Default | JjCommand::Log => {
            graph_status_message(item_count, view.graph_mode_label())
        }
        JjCommand::Show | JjCommand::Diff | JjCommand::Status | JjCommand::FileShow => {
            format!("{item_count} items")
        }
    }
}

#[derive(Clone, Debug)]
pub enum StatusKind {
    Ready,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(view: ViewState) -> App {
        App {
            status: StatusLine::ready(&view),
            view,
            stack: Vec::new(),
            startup_log_args: None,
            diff_format: DiffFormat::Default,
            mode: InteractionMode::Normal,
            search: None,
            should_quit: false,
        }
    }

    #[test]
    fn parses_default_startup_view() {
        let spec = initial_view(Vec::new()).unwrap();

        assert_eq!(spec.command(), JjCommand::Default);
        assert!(spec.args().is_empty());
    }

    #[test]
    fn parses_passthrough_startup_view() {
        let spec = initial_view(vec!["log".into(), "-r".into(), "::".into()]).unwrap();

        assert_eq!(spec.command(), JjCommand::Log);
        assert_eq!(spec.args(), ["-r", "::"]);
    }

    #[test]
    fn parses_show_startup_view() {
        let spec = initial_view(vec!["show".into(), "--git".into(), "main".into()]).unwrap();

        assert_eq!(spec.command(), JjCommand::Show);
        assert_eq!(spec.args(), ["--git", "main"]);
        assert_eq!(spec.diff_format(), DiffFormat::Git);
    }

    #[test]
    fn parses_diff_startup_view() {
        let spec = initial_view(vec!["diff".into(), "-r".into(), "main".into()]).unwrap();

        assert_eq!(spec.command(), JjCommand::Diff);
        assert_eq!(spec.args(), ["-r", "main"]);
    }

    #[test]
    fn parses_status_startup_view() {
        let spec = initial_view(vec!["status".into()]).unwrap();

        assert_eq!(spec.command(), JjCommand::Status);
        assert!(spec.args().is_empty());
    }

    #[test]
    fn parses_operation_log_startup_view() {
        let spec = initial_view(vec!["operation-log".into()]).unwrap();

        assert_eq!(spec.command(), JjCommand::OperationLog);
        assert!(spec.args().is_empty());
    }

    #[test]
    fn parses_bookmarks_startup_view() {
        let spec = initial_view(vec!["bookmarks".into()]).unwrap();

        assert_eq!(spec.command(), JjCommand::Bookmarks);
        assert!(spec.args().is_empty());
    }

    #[test]
    fn rejects_unknown_startup_command() {
        assert!(initial_view(vec!["bookmark".into()]).is_err());
    }

    #[test]
    fn graph_status_message_includes_mode_label() {
        assert_eq!(
            graph_status_message(4, Some("trunk work")),
            "4 items | trunk work"
        );
        assert_eq!(graph_status_message(4, None), "4 items");
    }

    #[test]
    fn open_push_prompt_requires_exact_graph_revision() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), None, None),
        ])));

        assert!(!app.open_push_prompt().unwrap());
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(
            app.status.message(),
            "push from graph requires a selected row with an exact revision"
        );
    }

    #[test]
    fn push_preview_entering_cancel_restores_normal_mode() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.mode = InteractionMode::PushPreview {
            push: JjGitPush::for_status().with_remote("origin"),
            command_label: "jj git push --remote origin --revision abcdef".to_owned(),
            preview_output: "preview only".to_owned(),
            status_context: None,
            completed: false,
        };

        assert!(
            app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
                .is_ok()
        );
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "push cancelled");
    }

    #[test]
    fn push_preview_completion_stays_until_closed() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.mode = InteractionMode::PushPreview {
            push: JjGitPush::for_status().with_remote("origin"),
            command_label: "jj git push --remote origin".to_owned(),
            preview_output: "pushed".to_owned(),
            status_context: Some("status push uses jj default target for remote origin".to_owned()),
            completed: true,
        };
        app.status = StatusLine::with_message(&app.view, "pushed");

        assert!(
            app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
                .is_ok()
        );
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "pushed");
    }

    #[test]
    fn push_remote_prompt_without_selection_stays_ready() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.mode = InteractionMode::PushRemotePrompt {
            target: JjGitPushTarget::Revision("abcdef".to_owned()),
            remotes: Vec::new(),
            selected: 0,
        };

        assert!(
            app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
                .is_ok()
        );
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "no remote selected for push");
    }
}
