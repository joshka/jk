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

use crate::action_menu::{ActionKind, ActionMenu, FollowUp, RolePrompt};
use crate::action_output::ActionOutput;
use crate::clipboard;
use crate::command::{
    Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect, find_binding,
    project_help,
};
use crate::copy::CopyOption;
use crate::jj::{
    DiffFormat, JjAbandonPlan, JjAbandonPreview, JjCommand, JjGitPush, JjGitPushTarget, JjNewPlan,
    JjOperationRecovery, JjOperationRecoveryKind, JjRebasePlan, LogViewMode, ViewSpec, git_fetch,
    git_remotes, new_trunk, resolve_exact_change_id,
};
use crate::search::SearchQuery;
use crate::tui::{self, Overlay, StatusHints};
use crate::view_state::ViewState;

#[cfg(test)]
type NewRun = fn(&JjNewPlan) -> Result<String>;
#[cfg(test)]
type RebaseRun = fn(&JjRebasePlan) -> Result<String>;
#[cfg(test)]
type AbandonPreviewLoad = fn(&JjAbandonPlan) -> Result<JjAbandonPreview>;
#[cfg(test)]
type AbandonRun = fn(&JjAbandonPlan) -> Result<String>;
#[cfg(test)]
type OperationRecoveryRun = fn(&JjOperationRecovery) -> Result<String>;
#[cfg(test)]
type ResolveRevision = fn(&str) -> Result<String>;
#[cfg(test)]
type RefreshView = fn(&mut ViewState) -> Result<()>;
#[cfg(test)]
type RevealGraphChange = fn(&mut ViewState, &str, LogViewMode) -> Result<bool>;

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
    #[cfg(test)]
    new_run: NewRun,
    #[cfg(test)]
    rebase_run: RebaseRun,
    #[cfg(test)]
    abandon_preview_load: AbandonPreviewLoad,
    #[cfg(test)]
    abandon_run: AbandonRun,
    #[cfg(test)]
    operation_recovery_run: OperationRecoveryRun,
    #[cfg(test)]
    resolve_revision: ResolveRevision,
    #[cfg(test)]
    refresh_view: RefreshView,
    #[cfg(test)]
    reveal_graph_change: RevealGraphChange,
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
        action: ActionKind,
        prompt: RolePrompt,
        selected: usize,
    },
    NewPreview {
        new_change: JjNewPlan,
        output: ActionOutput,
    },
    RebasePreview {
        rebase: JjRebasePlan,
        output: ActionOutput,
    },
    AbandonPreview {
        abandon: JjAbandonPlan,
        preview: JjAbandonPreview,
        output: ActionOutput,
    },
    AbandonConfirm {
        abandon: JjAbandonPlan,
        input: String,
        output: ActionOutput,
    },
    PushRemotePrompt {
        target: JjGitPushTarget,
        remotes: Vec<String>,
        selected: usize,
    },
    PushPreview {
        push: JjGitPush,
        output: ActionOutput,
    },
    OperationRecoveryPreview {
        recovery: JjOperationRecovery,
        output: ActionOutput,
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

#[cfg(test)]
fn default_new_run(new_change: &JjNewPlan) -> Result<String> {
    new_change.run().map(|output| output.message().to_owned())
}

#[cfg(test)]
fn default_rebase_run(rebase: &JjRebasePlan) -> Result<String> {
    rebase.run().map(|output| output.message().to_owned())
}

#[cfg(test)]
fn default_abandon_preview_load(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    abandon.run_preview()
}

#[cfg(test)]
fn default_abandon_run(abandon: &JjAbandonPlan) -> Result<String> {
    abandon.run().map(|output| output.message().to_owned())
}

#[cfg(test)]
fn default_operation_recovery_run(recovery: &JjOperationRecovery) -> Result<String> {
    recovery.run().map(|output| output.message().to_owned())
}

#[cfg(test)]
fn default_resolve_revision(revset: &str) -> Result<String> {
    resolve_exact_change_id(revset)
}

#[cfg(test)]
fn default_refresh_view(view: &mut ViewState) -> Result<()> {
    view.refresh()
}

#[cfg(test)]
fn default_reveal_graph_change(
    view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    view.reveal_graph_change(change_id, fallback_mode)
}

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
            #[cfg(test)]
            new_run: default_new_run,
            #[cfg(test)]
            rebase_run: default_rebase_run,
            #[cfg(test)]
            abandon_preview_load: default_abandon_preview_load,
            #[cfg(test)]
            abandon_run: default_abandon_run,
            #[cfg(test)]
            operation_recovery_run: default_operation_recovery_run,
            #[cfg(test)]
            resolve_revision: default_resolve_revision,
            #[cfg(test)]
            refresh_view: default_refresh_view,
            #[cfg(test)]
            reveal_graph_change: default_reveal_graph_change,
        })
    }

    #[cfg(test)]
    fn run_new_change(&self, new_change: &JjNewPlan) -> Result<String> {
        (self.new_run)(new_change)
    }

    #[cfg(not(test))]
    fn run_new_change(&self, new_change: &JjNewPlan) -> Result<String> {
        new_change.run().map(|output| output.message().to_owned())
    }

    #[cfg(test)]
    fn run_rebase(&self, rebase: &JjRebasePlan) -> Result<String> {
        (self.rebase_run)(rebase)
    }

    #[cfg(not(test))]
    fn run_rebase(&self, rebase: &JjRebasePlan) -> Result<String> {
        rebase.run().map(|output| output.message().to_owned())
    }

    #[cfg(test)]
    fn load_abandon_preview(&self, abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
        (self.abandon_preview_load)(abandon)
    }

    #[cfg(not(test))]
    fn load_abandon_preview(&self, abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
        abandon.run_preview()
    }

    #[cfg(test)]
    fn run_abandon(&self, abandon: &JjAbandonPlan) -> Result<String> {
        (self.abandon_run)(abandon)
    }

    #[cfg(not(test))]
    fn run_abandon(&self, abandon: &JjAbandonPlan) -> Result<String> {
        abandon.run().map(|output| output.message().to_owned())
    }

    #[cfg(test)]
    fn run_operation_recovery(&self, recovery: &JjOperationRecovery) -> Result<String> {
        (self.operation_recovery_run)(recovery)
    }

    #[cfg(not(test))]
    fn run_operation_recovery(&self, recovery: &JjOperationRecovery) -> Result<String> {
        recovery.run().map(|output| output.message().to_owned())
    }

    #[cfg(test)]
    fn resolve_revision(&self, revset: &str) -> Result<String> {
        (self.resolve_revision)(revset)
    }

    #[cfg(not(test))]
    fn resolve_revision(&self, revset: &str) -> Result<String> {
        resolve_exact_change_id(revset)
    }

    #[cfg(test)]
    fn refresh_view_state(&mut self) -> Result<()> {
        (self.refresh_view)(&mut self.view)
    }

    #[cfg(not(test))]
    fn refresh_view_state(&mut self) -> Result<()> {
        self.view.refresh()
    }

    #[cfg(test)]
    fn reveal_graph_change(&mut self, change_id: &str, fallback_mode: LogViewMode) -> Result<bool> {
        (self.reveal_graph_change)(&mut self.view, change_id, fallback_mode)
    }

    #[cfg(not(test))]
    fn reveal_graph_change(&mut self, change_id: &str, fallback_mode: LogViewMode) -> Result<bool> {
        self.view.reveal_graph_change(change_id, fallback_mode)
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
            Command::OperationUndo | Command::OperationRedo => {
                if let Some(kind) = binding.command().operation_recovery() {
                    self.open_operation_recovery_preview(kind);
                }
                Ok(false)
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
                                FollowUp::ExactRevision { revision } => {
                                    let action = action.action();
                                    let revision = revision.clone();
                                    self.mode = InteractionMode::Normal;
                                    match action {
                                        ActionKind::Abandon => {
                                            self.open_abandon_preview(JjAbandonPlan::new(revision));
                                        }
                                        ActionKind::New
                                        | ActionKind::Split
                                        | ActionKind::Rebase
                                        | ActionKind::Squash => {
                                            self.status = StatusLine::with_message(
                                                &self.view,
                                                "preview not yet implemented",
                                            );
                                        }
                                    }
                                }
                                FollowUp::NewParents { parents } => {
                                    let parents = parents.clone();
                                    self.mode = InteractionMode::Normal;
                                    self.open_new_preview(JjNewPlan::new(parents));
                                }
                                FollowUp::RolePrompt(prompt) => {
                                    self.mode = InteractionMode::RolePrompt {
                                        action: action.action(),
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
            InteractionMode::RolePrompt {
                action,
                prompt,
                selected,
            } => {
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
                        let action = *action;
                        let rebase_plan = match action {
                            ActionKind::Rebase => rebase_plan_from_prompt(prompt),
                            _ => None,
                        };

                        self.mode = InteractionMode::Normal;

                        match action {
                            ActionKind::Rebase => match rebase_plan {
                                Some(rebase) => self.open_rebase_preview(rebase),
                                None => {
                                    self.status =
                                        StatusLine::error(&self.view, next_status.to_owned());
                                }
                            },
                            ActionKind::Squash => {
                                self.status = StatusLine::with_message(
                                    &self.view,
                                    "squash preview not yet implemented",
                                );
                            }
                            ActionKind::New | ActionKind::Split | ActionKind::Abandon => {
                                self.status =
                                    StatusLine::with_message(&self.view, next_status.to_owned());
                            }
                        }
                    }
                    _ => {}
                }
                Ok(true)
            }
            InteractionMode::NewPreview { new_change, output } => {
                let (new_change, status_context, completed) = {
                    (
                        new_change.clone(),
                        output.status_context().cloned(),
                        output.completed(),
                    )
                };
                let visible_lines = action_output_visible_lines(viewport_height);
                match handle_action_output_key(code, output, visible_lines) {
                    ActionOutputKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        if !completed {
                            self.status = StatusLine::with_message(
                                &self.view,
                                "new change cancelled".to_owned(),
                            );
                        }
                    }
                    ActionOutputKey::Primary => {
                        if completed {
                            self.mode = InteractionMode::Normal;
                            return Ok(true);
                        }

                        self.confirm_new_change(new_change, status_context, viewport_height);
                    }
                    ActionOutputKey::Handled | ActionOutputKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::RebasePreview { rebase, output } => {
                let (rebase, status_context, completed) = {
                    (
                        rebase.clone(),
                        output.status_context().cloned(),
                        output.completed(),
                    )
                };
                let visible_lines = action_output_visible_lines(viewport_height);
                match handle_action_output_key(code, output, visible_lines) {
                    ActionOutputKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        if !completed {
                            self.status =
                                StatusLine::with_message(&self.view, "rebase cancelled".to_owned());
                        }
                    }
                    ActionOutputKey::Primary => {
                        if completed {
                            self.mode = InteractionMode::Normal;
                            return Ok(true);
                        }

                        self.confirm_rebase(rebase, status_context, viewport_height);
                    }
                    ActionOutputKey::Handled | ActionOutputKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::AbandonPreview {
                abandon,
                preview,
                output,
            } => {
                let (abandon, preview, status_context, completed) = {
                    (
                        abandon.clone(),
                        preview.clone(),
                        output.status_context().cloned(),
                        output.completed(),
                    )
                };
                let visible_lines = action_output_visible_lines(viewport_height);
                match handle_action_output_key(code, output, visible_lines) {
                    ActionOutputKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        if !completed {
                            self.status = StatusLine::with_message(
                                &self.view,
                                "abandon cancelled".to_owned(),
                            );
                        }
                    }
                    ActionOutputKey::Primary => {
                        if completed {
                            self.mode = InteractionMode::Normal;
                            return Ok(true);
                        }

                        if preview.is_empty_change() {
                            self.confirm_empty_abandon_after_recheck(
                                abandon,
                                status_context,
                                viewport_height,
                            );
                        } else {
                            self.mode = InteractionMode::AbandonConfirm {
                                abandon,
                                input: String::new(),
                                output: output.clone(),
                            };
                        }
                    }
                    ActionOutputKey::Handled | ActionOutputKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::AbandonConfirm {
                abandon,
                input,
                output,
            } => {
                let (abandon_plan, status_context) =
                    (abandon.clone(), output.status_context().cloned());
                let visible_lines = action_output_visible_lines(viewport_height);
                match code {
                    KeyCode::Esc => {
                        self.mode = InteractionMode::Normal;
                        self.status =
                            StatusLine::with_message(&self.view, "abandon cancelled".to_owned());
                    }
                    KeyCode::Enter => {
                        if input == abandon.revision() {
                            self.confirm_abandon(abandon_plan, status_context, viewport_height);
                        } else {
                            self.status = StatusLine::error(
                                &self.view,
                                "confirmation did not match; abandon not run".to_owned(),
                            );
                        }
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(character) => input.push(character),
                    KeyCode::Down => output.scroll_down(visible_lines),
                    KeyCode::Up => output.scroll_up(),
                    KeyCode::PageDown => output.page_down(visible_lines),
                    KeyCode::PageUp => output.page_up(visible_lines),
                    KeyCode::Home => output.scroll_to_top(),
                    KeyCode::End => output.scroll_to_bottom(visible_lines),
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
            InteractionMode::PushPreview { push, output } => {
                let (push, status_context, completed) = {
                    (
                        push.clone(),
                        output.status_context().cloned(),
                        output.completed(),
                    )
                };
                let visible_lines = action_output_visible_lines(viewport_height);
                match handle_action_output_key(code, output, visible_lines) {
                    ActionOutputKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        if !completed {
                            self.status =
                                StatusLine::with_message(&self.view, "push cancelled".to_owned());
                        }
                    }
                    ActionOutputKey::Primary => {
                        if completed {
                            self.mode = InteractionMode::Normal;
                            return Ok(true);
                        }

                        self.confirm_push(push, status_context, viewport_height);
                    }
                    ActionOutputKey::Handled | ActionOutputKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::OperationRecoveryPreview { recovery, output } => {
                let (recovery, status_context, completed) = {
                    (
                        recovery.clone(),
                        output.status_context().cloned(),
                        output.completed(),
                    )
                };
                let visible_lines = action_output_visible_lines(viewport_height);
                match handle_action_output_key(code, output, visible_lines) {
                    ActionOutputKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        if !completed {
                            self.status = StatusLine::with_message(
                                &self.view,
                                format!("{} cancelled", recovery.status_action()),
                            );
                        }
                    }
                    ActionOutputKey::Primary => {
                        if completed {
                            self.mode = InteractionMode::Normal;
                            return Ok(true);
                        }

                        self.confirm_operation_recovery(recovery, status_context, viewport_height);
                    }
                    ActionOutputKey::Handled | ActionOutputKey::Ignored => {}
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
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = push.command_label(true);
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::PushPreview {
                    push,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    fn open_operation_recovery_preview(&mut self, kind: JjOperationRecoveryKind) {
        let recovery = JjOperationRecovery::new(kind);
        let status_context = Some(format!(
            "global current-repo {} from {}",
            recovery.status_action(),
            self.view.spec().app_label()
        ));
        self.mode = InteractionMode::OperationRecoveryPreview {
            output: ActionOutput::pending(
                recovery.command_label().to_owned(),
                recovery.preview_text().to_owned(),
                status_context,
            ),
            recovery,
        };
    }

    fn open_new_preview(&mut self, new_change: JjNewPlan) {
        let parent_labels = new_change
            .parents()
            .iter()
            .map(|parent| short_id(parent))
            .collect::<Vec<_>>()
            .join(", ");
        let status_context = Some(format!(
            "new from {} parent(s) from {} | parent(s): {}",
            new_change.parents().len(),
            self.view.spec().app_label(),
            parent_labels
        ));

        match new_change.run_preview() {
            Ok(output) => {
                let command_label = new_change.command_label();
                self.mode = InteractionMode::NewPreview {
                    new_change,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = new_change.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::NewPreview {
                    new_change,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    fn open_rebase_preview(&mut self, rebase: JjRebasePlan) {
        let status_context = Some(format!(
            "rebase from {} source(s) into {} from {}",
            rebase.sources().len(),
            rebase.destination(),
            self.view.spec().app_label()
        ));
        let source_labels = rebase
            .sources()
            .iter()
            .map(|source| short_id(source))
            .collect::<Vec<_>>()
            .join(", ");
        let status_context = if source_labels.is_empty() {
            status_context
        } else {
            status_context
                .map(|status_context| format!("{status_context} | source(s): {source_labels}"))
        };

        match rebase.run_preview() {
            Ok(output) => {
                let command_label = rebase.command_label(true);
                self.mode = InteractionMode::RebasePreview {
                    rebase,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = rebase.command_label(true);
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::RebasePreview {
                    rebase,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    fn open_abandon_preview(&mut self, abandon: JjAbandonPlan) {
        let status_context = Some(format!(
            "abandon exact revision {} from {}",
            abandon.revision(),
            self.view.spec().app_label()
        ));

        match self.load_abandon_preview(&abandon) {
            Ok(preview) => {
                let command_label = abandon.command_label();
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    output: ActionOutput::pending(
                        command_label,
                        preview.preview_text(),
                        status_context,
                    ),
                    preview,
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = abandon.diff_summary_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    preview: JjAbandonPreview::new(String::new(), None, String::new()),
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    fn confirm_new_change(
        &mut self,
        new_change: JjNewPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = new_change.command_label();
        let result_message = match self.run_new_change(&new_change) {
            Ok(output) => {
                let new_change_id = match self.resolve_revision("@") {
                    Ok(change_id) => change_id,
                    Err(error) => {
                        let message =
                            format!("{} | resolve @ failed: {error} | jj undo", output.trim());
                        self.status = StatusLine::error(&self.view, error.to_string());
                        self.mode = InteractionMode::NewPreview {
                            new_change,
                            output: ActionOutput::finished(command_label, message, status_context),
                        };
                        return;
                    }
                };

                match self.refresh_view_state() {
                    Ok(()) => {
                        self.view.clamp(viewport_height);
                        let mut reveal_error = None;
                        let revealed_in_recent =
                            match self.reveal_graph_change(&new_change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height);
                                    Some(switched_modes)
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                    reveal_error = Some(format!(
                                        "{} | reveal failed: {} | jj undo",
                                        output.trim(),
                                        error
                                    ));
                                    None
                                }
                            };

                        let message = match revealed_in_recent {
                            Some(switched_modes) => {
                                if switched_modes {
                                    format!("{} | showing recent work | jj undo", output.trim())
                                } else {
                                    format!("{} | jj undo", output.trim())
                                }
                            }
                            None => match reveal_error.as_deref() {
                                Some(message) => message.to_owned(),
                                None => format!("{} | jj undo", output.trim()),
                            },
                        };
                        if reveal_error.is_none() {
                            self.status = StatusLine::with_message(&self.view, message.as_str());
                        }
                        message
                    }
                    Err(error) => {
                        self.status = StatusLine::error(&self.view, error.to_string());
                        format!("{} | refresh failed: {error} | jj undo", output.trim())
                    }
                }
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::NewPreview {
            new_change,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
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
            output: ActionOutput::finished(command_label, result_message, status_context),
        }
    }

    fn confirm_operation_recovery(
        &mut self,
        recovery: JjOperationRecovery,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = recovery.command_label().to_owned();
        let result_message = match self.run_operation_recovery(&recovery) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
                    let message = format!("{} | {}", output.trim(), recovery.success_hint());
                    self.status = StatusLine::with_message(&self.view, message.as_str());
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!(
                        "{} | refresh failed: {error} | {}",
                        output.trim(),
                        recovery.success_hint()
                    )
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::OperationRecoveryPreview {
            recovery,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    fn confirm_abandon(
        &mut self,
        abandon: JjAbandonPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = abandon.command_label();
        let result_message = match self.run_abandon(&abandon) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
                    let message = format!("{} | jj undo", output.trim());
                    self.status = StatusLine::with_message(&self.view, message.as_str());
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!("{} | refresh failed: {error} | jj undo", output.trim())
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::AbandonPreview {
            abandon,
            preview: JjAbandonPreview::new(String::new(), None, String::new()),
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    fn confirm_empty_abandon_after_recheck(
        &mut self,
        abandon: JjAbandonPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        match self.load_abandon_preview(&abandon) {
            Ok(preview) if preview.is_empty_change() => {
                self.confirm_abandon(abandon, status_context, viewport_height);
            }
            Ok(preview) => {
                let message = "change is no longer empty; type exact revision to confirm abandon";
                self.status = StatusLine::error(&self.view, message.to_owned());
                let command_label = abandon.command_label();
                let output = format!("{message}\n\n{}", preview.preview_text());
                self.mode = InteractionMode::AbandonConfirm {
                    abandon,
                    input: String::new(),
                    output: ActionOutput::pending(command_label, output, status_context),
                };
            }
            Err(error) => {
                let message = error.to_string();
                self.status = StatusLine::error(&self.view, message.clone());
                let command_label = abandon.diff_summary_label();
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    preview: JjAbandonPreview::new(String::new(), None, String::new()),
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
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

    fn confirm_rebase(
        &mut self,
        rebase: JjRebasePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = rebase.command_label(false);
        let primary_source = rebase.sources().first().cloned();
        let result_message = match self.run_rebase(&rebase) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
                    let mut reveal_error = None;
                    let revealed_in_recent = match primary_source.as_deref() {
                        Some(change_id) => {
                            match self.reveal_graph_change(change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height);
                                    Some(switched_modes)
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                    reveal_error = Some(format!(
                                        "{} | reveal failed: {} | jj undo",
                                        output.trim(),
                                        error
                                    ));
                                    None
                                }
                            }
                        }
                        None => None,
                    };

                    let message = match revealed_in_recent {
                        Some(switched_modes) => {
                            if switched_modes {
                                format!("{} | showing recent work | jj undo", output.trim())
                            } else {
                                format!("{} | jj undo", output.trim())
                            }
                        }
                        None => match reveal_error.as_deref() {
                            Some(message) => message.to_owned(),
                            None => format!("{} | jj undo", output.trim()),
                        },
                    };
                    if reveal_error.is_none() {
                        self.status = StatusLine::with_message(&self.view, message.as_str());
                    }
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

        self.mode = InteractionMode::RebasePreview {
            rebase,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
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
        if let Err(error) = self.resolve_revision("trunk()") {
            self.status = StatusLine::error(&self.view, error.to_string());
            return;
        }

        match new_trunk() {
            Ok(_) => {
                let new_change_id = match self.resolve_revision("@") {
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
            InteractionMode::AbandonConfirm { input, .. } => StatusLine::with_message(
                &self.view,
                format!("type exact revision to confirm abandon: {input}"),
            ),
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
            InteractionMode::RolePrompt {
                prompt, selected, ..
            } => Overlay::RolePrompt {
                prompt,
                selected: *selected,
            },
            InteractionMode::NewPreview { output, .. } => Overlay::NewPreview { output },
            InteractionMode::RebasePreview { output, .. } => Overlay::RebasePreview { output },
            InteractionMode::AbandonPreview { output, .. } => Overlay::AbandonPreview { output },
            InteractionMode::AbandonConfirm { input, output, .. } => {
                Overlay::AbandonConfirm { input, output }
            }
            InteractionMode::PushRemotePrompt {
                remotes, selected, ..
            } => Overlay::PushRemotePrompt {
                remotes,
                selected: *selected,
            },
            InteractionMode::PushPreview { output, .. } => Overlay::PushPreview { output },
            InteractionMode::OperationRecoveryPreview { output, .. } => {
                Overlay::OperationRecoveryPreview { output }
            }
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
            | JjCommand::OperationLog
            | JjCommand::OperationShow
            | JjCommand::OperationDiff => return Ok(()),
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

    #[cfg(test)]
    pub(crate) fn test(
        title: impl Into<String>,
        message: impl Into<String>,
        kind: StatusKind,
        hints: StatusHints,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            kind,
            hints,
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
        JjCommand::Show
        | JjCommand::Diff
        | JjCommand::Status
        | JjCommand::FileShow
        | JjCommand::OperationShow
        | JjCommand::OperationDiff => format!("{item_count} items"),
    }
}

fn rebase_plan_from_prompt(prompt: &RolePrompt) -> Option<JjRebasePlan> {
    let destination = prompt.destination_revision()?;
    let sources = prompt
        .source_revisions()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    (!sources.is_empty()).then(|| JjRebasePlan::new(sources, destination.to_owned()))
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

fn action_output_visible_lines(viewport_height: u16) -> u16 {
    viewport_height.saturating_sub(1).max(1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActionOutputKey {
    Primary,
    Cancel,
    Handled,
    Ignored,
}

fn handle_action_output_key(
    code: KeyCode,
    output: &mut ActionOutput,
    visible_lines: u16,
) -> ActionOutputKey {
    match code {
        KeyCode::Enter => ActionOutputKey::Primary,
        KeyCode::Esc | KeyCode::Char('q') => ActionOutputKey::Cancel,
        KeyCode::Char('j') | KeyCode::Down => {
            output.scroll_down(visible_lines);
            ActionOutputKey::Handled
        }
        KeyCode::Char('k') | KeyCode::Up => {
            output.scroll_up();
            ActionOutputKey::Handled
        }
        KeyCode::Char(' ') | KeyCode::PageDown => {
            output.page_down(visible_lines);
            ActionOutputKey::Handled
        }
        KeyCode::Char('b') | KeyCode::PageUp => {
            output.page_up(visible_lines);
            ActionOutputKey::Handled
        }
        KeyCode::Char('g') | KeyCode::Home => {
            output.scroll_to_top();
            ActionOutputKey::Handled
        }
        KeyCode::Char('G') | KeyCode::End => {
            output.scroll_to_bottom(visible_lines);
            ActionOutputKey::Handled
        }
        _ => ActionOutputKey::Ignored,
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
    use crate::action_menu::RolePromptOption;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ABANDON_DRIFT_RECHECK_CALLS: AtomicUsize = AtomicUsize::new(0);
    static ABANDON_FAILED_RECHECK_CALLS: AtomicUsize = AtomicUsize::new(0);

    fn mock_new_success(new_change: &JjNewPlan) -> Result<String> {
        Ok(format!("new parents: {}", new_change.parents().join(",")))
    }

    fn mock_new_failure(_: &JjNewPlan) -> Result<String> {
        Err(eyre!("jj new failed: first line\nsecond line"))
    }

    fn mock_rebase_success(_: &JjRebasePlan) -> Result<String> {
        Ok("rebased".to_owned())
    }

    fn mock_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
        Ok(JjAbandonPreview::new(
            abandon.revision().to_owned(),
            Some("Empty change".to_owned()),
            String::new(),
        ))
    }

    fn mock_non_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
        Ok(JjAbandonPreview::new(
            abandon.revision().to_owned(),
            Some("Edit change".to_owned()),
            "M src/main.rs\n".to_owned(),
        ))
    }

    fn mock_abandon_preview_drifts_to_non_empty(
        abandon: &JjAbandonPlan,
    ) -> Result<JjAbandonPreview> {
        if ABANDON_DRIFT_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
            mock_empty_abandon_preview(abandon)
        } else {
            mock_non_empty_abandon_preview(abandon)
        }
    }

    fn mock_abandon_preview_recheck_failure(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
        if ABANDON_FAILED_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
            mock_empty_abandon_preview(abandon)
        } else {
            Err(eyre!("jj diff -r change-a --summary failed: disappeared"))
        }
    }

    fn mock_abandon_success(_: &JjAbandonPlan) -> Result<String> {
        Ok("abandoned".to_owned())
    }

    fn mock_abandon_failure(_: &JjAbandonPlan) -> Result<String> {
        Err(eyre!("jj abandon change-a failed: first line\nsecond line"))
    }

    fn mock_operation_recovery_success(recovery: &JjOperationRecovery) -> Result<String> {
        Ok(match recovery.kind() {
            JjOperationRecoveryKind::Undo => "undone operation".to_owned(),
            JjOperationRecoveryKind::Redo => "redone operation".to_owned(),
        })
    }

    fn mock_operation_recovery_failure(recovery: &JjOperationRecovery) -> Result<String> {
        Err(eyre!(
            "{} failed: no operation to {} available\nhint: run the opposite recovery command first",
            recovery.command_label(),
            recovery.status_action()
        ))
    }

    fn mock_resolve_current_change_id(revset: &str) -> Result<String> {
        assert_eq!(revset, "@");
        Ok("new-working-copy".to_owned())
    }

    fn panic_abandon_run(_: &JjAbandonPlan) -> Result<String> {
        panic!("abandon should not run without exact confirmation")
    }

    fn mock_refresh_ok(_view: &mut ViewState) -> Result<()> {
        Ok(())
    }

    fn mock_reveal_graph_change_error(
        _view: &mut ViewState,
        _change_id: &str,
        _fallback_mode: LogViewMode,
    ) -> Result<bool> {
        Err(eyre!(
            "refreshed graph did not include the new working-copy change"
        ))
    }

    fn mock_reveal_new_change_in_recent(
        _view: &mut ViewState,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        assert_eq!(change_id, "new-working-copy");
        assert_eq!(fallback_mode, LogViewMode::Recent);
        Ok(true)
    }

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
            #[cfg(test)]
            new_run: mock_new_success,
            #[cfg(test)]
            rebase_run: mock_rebase_success,
            #[cfg(test)]
            abandon_preview_load: mock_empty_abandon_preview,
            #[cfg(test)]
            abandon_run: mock_abandon_success,
            #[cfg(test)]
            operation_recovery_run: mock_operation_recovery_success,
            #[cfg(test)]
            resolve_revision: mock_resolve_current_change_id,
            #[cfg(test)]
            refresh_view: mock_refresh_ok,
            #[cfg(test)]
            reveal_graph_change: default_reveal_graph_change,
        }
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
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
            output: ActionOutput::pending(
                "jj git push --remote origin --revision abcdef".to_owned(),
                "preview only".to_owned(),
                None,
            ),
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
            output: ActionOutput::finished(
                "jj git push --remote origin".to_owned(),
                "pushed".to_owned(),
                Some("status push uses jj default target for remote origin".to_owned()),
            ),
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
    fn action_output_scroll_keys_clamp_to_visible_body() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.mode = InteractionMode::PushPreview {
            push: JjGitPush::for_status().with_remote("origin"),
            output: ActionOutput::pending(
                "jj git push --preview --remote origin".to_owned(),
                (0..8)
                    .map(|line| format!("line {line}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                None,
            ),
        };

        app.handle_mode_key(crossterm::event::KeyCode::Char('j'), 4)
            .unwrap();
        app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
            .unwrap();
        app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
            .unwrap();
        app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::PushPreview { output, .. } => output,
            _ => panic!("expected push preview mode"),
        };
        assert_eq!(
            output.scroll(),
            output.max_scroll(action_output_visible_lines(4))
        );

        app.handle_mode_key(crossterm::event::KeyCode::PageUp, 4)
            .unwrap();
        app.handle_mode_key(crossterm::event::KeyCode::Char('k'), 4)
            .unwrap();
        app.handle_mode_key(crossterm::event::KeyCode::Char('g'), 4)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::PushPreview { output, .. } => output,
            _ => panic!("expected push preview mode"),
        };
        assert_eq!(output.scroll(), 0);
    }

    #[test]
    fn closing_action_output_preserves_graph_selection() {
        let mut graph = crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("first".to_owned()), None),
            crate::jj::LogItem::new(Vec::new(), Some("second".to_owned()), None),
        ]);
        graph.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 12,
                search: None,
            },
        );
        let mut app = test_app(ViewState::Graph(graph));
        app.mode = InteractionMode::PushPreview {
            push: JjGitPush::for_status().with_remote("origin"),
            output: ActionOutput::pending(
                "jj git push --preview --remote origin".to_owned(),
                "preview only".to_owned(),
                None,
            ),
        };

        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .unwrap();

        let ViewState::Graph(graph) = &app.view else {
            panic!("expected graph view");
        };
        assert_eq!(graph.selected_revision(), Some("second"));
        assert!(matches!(app.mode, InteractionMode::Normal));
    }

    #[test]
    fn rebase_plan_from_prompt_respects_explicit_roles() {
        let prompt = RolePrompt::new(
            "confirm role assignment",
            vec![
                RolePromptOption::new("source", "bbbbbbbb1111111111111111111111111111111111"),
                RolePromptOption::new("destination", "cccccccc2222222222222222222222222222222222"),
                RolePromptOption::new("source", "aaaaaaaa3333333333333333333333333333333333"),
            ],
            "Preview required before execution.",
        );

        let rebase =
            rebase_plan_from_prompt(&prompt).expect("role prompt should include a destination");

        assert_eq!(
            rebase.sources(),
            &[
                "bbbbbbbb1111111111111111111111111111111111",
                "aaaaaaaa3333333333333333333333333333333333"
            ]
        );
        assert_eq!(
            rebase.destination(),
            "cccccccc2222222222222222222222222222222222"
        );
    }

    #[test]
    fn new_action_menu_enter_opens_preview_with_exact_parents() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
        ])));
        app.mode = InteractionMode::ActionMenu {
            menu: crate::action_menu::build_action_menu(
                &crate::action_menu::ExactActionContext::with_current("current")
                    .with_sources(["parent-a", "parent-b"]),
            ),
            selected: 0,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let (parents, command_label, body) = match &app.mode {
            InteractionMode::NewPreview { new_change, output } => (
                new_change.parents().to_vec(),
                output.command_label().to_owned(),
                output.body_lines().join("\n"),
            ),
            _ => panic!("expected new preview mode"),
        };
        assert_eq!(parents, ["parent-a", "parent-b"]);
        assert_eq!(command_label, "jj new parent-a parent-b");
        assert!(body.contains("parent: parent-a"));
        assert!(body.contains("parent: parent-b"));
        assert!(body.contains("undo path: jj undo"));
    }

    #[test]
    fn new_preview_cancel_restores_normal_mode() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
        ])));
        app.mode = InteractionMode::NewPreview {
            new_change: JjNewPlan::new(vec!["parent-a".to_owned()]),
            output: ActionOutput::pending(
                "jj new parent-a".to_owned(),
                "preview only".to_owned(),
                Some("new preview context".to_owned()),
            ),
        };

        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .unwrap();

        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "new change cancelled");
    }

    #[test]
    fn new_confirm_success_refreshes_and_reveals_working_copy() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
        ])));
        app.reveal_graph_change = mock_reveal_new_change_in_recent;
        app.mode = InteractionMode::NewPreview {
            new_change: JjNewPlan::new(vec!["parent-a".to_owned(), "parent-b".to_owned()]),
            output: ActionOutput::pending(
                "jj new parent-a parent-b".to_owned(),
                "preview only".to_owned(),
                Some("new preview context".to_owned()),
            ),
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::NewPreview { output, .. } => output,
            _ => panic!("expected new result mode"),
        };
        let body = output.body_lines().join("\n");
        assert_eq!(output.command_label(), "jj new parent-a parent-b");
        assert!(output.completed());
        assert!(body.contains("new parents: parent-a,parent-b | showing recent work | jj undo"));
        assert_eq!(
            app.status.message(),
            "new parents: parent-a,parent-b | showing recent work | jj undo"
        );
    }

    #[test]
    fn new_failure_keeps_full_error_output_readable() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
        ])));
        app.new_run = mock_new_failure;
        app.mode = InteractionMode::NewPreview {
            new_change: JjNewPlan::new(vec!["parent-a".to_owned()]),
            output: ActionOutput::pending(
                "jj new parent-a".to_owned(),
                "preview only".to_owned(),
                None,
            ),
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::NewPreview { output, .. } => output,
            _ => panic!("expected new result mode"),
        };
        let body = output.body_lines().join("\n");
        assert_eq!(output.command_label(), "jj new parent-a");
        assert!(output.completed());
        assert!(body.contains("jj new failed: first line"));
        assert!(body.contains("second line"));
        assert_eq!(
            app.status.message(),
            "jj new failed: first line\nsecond line"
        );
    }

    #[test]
    fn rebase_preview_entering_cancel_restores_normal_mode() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.mode = InteractionMode::RebasePreview {
            rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
            output: ActionOutput::pending(
                "jj rebase -r source-a -o dest".to_owned(),
                "preview only".to_owned(),
                Some("rebase preview context".to_owned()),
            ),
        };

        assert!(
            app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
                .is_ok()
        );
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "rebase cancelled");
    }

    #[test]
    fn rebase_preview_completion_stays_until_closed() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.mode = InteractionMode::RebasePreview {
            rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
            output: ActionOutput::finished(
                "jj rebase -r source-a -o dest".to_owned(),
                "rebased".to_owned(),
                None,
            ),
        };
        app.status = StatusLine::with_message(&app.view, "rebased");

        assert!(
            app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
                .is_ok()
        );
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "rebased");
    }

    #[test]
    fn rebase_confirm_success_with_reveal_failure_stays_completed() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.rebase_run = mock_rebase_success;
        app.refresh_view = mock_refresh_ok;
        app.reveal_graph_change = mock_reveal_graph_change_error;
        app.mode = InteractionMode::RebasePreview {
            rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
            output: ActionOutput::pending(
                "jj rebase -r source-a -o dest".to_owned(),
                "preview only".to_owned(),
                Some("rebase preview context".to_owned()),
            ),
        };

        assert!(
            app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
                .is_ok()
        );

        let output = match app.mode {
            InteractionMode::RebasePreview { ref output, .. } => output,
            _ => panic!("expected rebase preview mode"),
        };
        assert_eq!(output.command_label(), "jj rebase -r source-a -o dest");
        assert_eq!(
            output.status_context().map(String::as_str),
            Some("rebase preview context")
        );
        assert!(output.completed());
        assert!(output.body_lines().join("\n").contains(
            "reveal failed: refreshed graph did not include the new working-copy change"
        ));
        assert!(matches!(app.status.kind(), StatusKind::Error));
        assert_eq!(
            app.status.message(),
            "refreshed graph did not include the new working-copy change"
        );

        assert!(
            app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
                .is_ok()
        );
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(
            app.status.message(),
            "refreshed graph did not include the new working-copy change"
        );
    }

    #[test]
    fn rebase_role_prompt_enters_preview_with_explicit_plan() {
        let prompt = RolePrompt::new(
            "confirm role assignment",
            vec![
                RolePromptOption::new("source", "source-a".to_owned()),
                RolePromptOption::new("destination", "dest".to_owned()),
            ],
            "Preview required before execution.",
        );
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
        ])));
        app.mode = InteractionMode::RolePrompt {
            action: ActionKind::Rebase,
            prompt,
            selected: 0,
        };

        let result = app.handle_mode_key(crossterm::event::KeyCode::Enter, 12);
        assert!(result.is_ok());
        let (command_label, status_context, preview_output) = match app.mode {
            InteractionMode::RebasePreview { ref output, .. } => (
                output.command_label().to_owned(),
                output.status_context().cloned(),
                output.body_lines().join("\n"),
            ),
            _ => panic!("expected rebase preview mode"),
        };
        assert_eq!(command_label, "jj rebase -r source-a -o dest");
        assert_eq!(
            status_context.as_deref(),
            Some("rebase from 1 source(s) into dest from jk | source(s): source-a")
        );
        assert_eq!(
            preview_output,
            "command: jj rebase -r source-a -o dest\ncontext: rebase from 1 source(s) into dest from jk | source(s): source-a\noutput:\n  command: jj rebase -r source-a -o dest\n  \n  source: source-a\n  \n  destination: dest\n  \n  graph effect: rebases the selected revisions onto the destination and preserves dependencies within the selected set\n  \n  undo path: jj undo"
        );
    }

    #[test]
    fn abandon_action_menu_enter_opens_preview_with_exact_target() {
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
        ])));
        app.abandon_preview_load = mock_non_empty_abandon_preview;
        app.mode = InteractionMode::ActionMenu {
            menu: crate::action_menu::build_action_menu(
                &crate::action_menu::ExactActionContext::with_current("change-a"),
            ),
            selected: 2,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let (revision, command_label, body) = match &app.mode {
            InteractionMode::AbandonPreview {
                abandon, output, ..
            } => (
                abandon.revision().to_owned(),
                output.command_label().to_owned(),
                output.body_lines().join("\n"),
            ),
            _ => panic!("expected abandon preview mode"),
        };
        assert_eq!(revision, "change-a");
        assert_eq!(command_label, "jj abandon change-a");
        assert!(body.contains("change: change-a"));
        assert!(body.contains("title: Edit change"));
    }

    #[test]
    fn empty_abandon_preview_enter_runs_and_keeps_undo_visible() {
        let preview = JjAbandonPreview::new(
            "change-a".to_owned(),
            Some("Empty change".to_owned()),
            String::new(),
        );
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
        ])));
        app.mode = InteractionMode::AbandonPreview {
            abandon: JjAbandonPlan::new("change-a"),
            output: ActionOutput::pending(
                "jj abandon change-a".to_owned(),
                preview.preview_text(),
                Some("abandon exact revision change-a from jk".to_owned()),
            ),
            preview,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::AbandonPreview { output, .. } => output,
            _ => panic!("expected abandon result mode"),
        };
        assert!(output.completed());
        assert!(
            output
                .body_lines()
                .join("\n")
                .contains("abandoned | jj undo")
        );
        assert_eq!(app.status.message(), "abandoned | jj undo");
    }

    #[test]
    fn empty_abandon_rechecks_before_running_and_requires_confirmation_after_drift() {
        ABANDON_DRIFT_RECHECK_CALLS.store(1, Ordering::SeqCst);
        let preview = JjAbandonPreview::new(
            "change-a".to_owned(),
            Some("Empty change".to_owned()),
            String::new(),
        );
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
        ])));
        app.abandon_preview_load = mock_abandon_preview_drifts_to_non_empty;
        app.abandon_run = panic_abandon_run;
        app.mode = InteractionMode::AbandonPreview {
            abandon: JjAbandonPlan::new("change-a"),
            output: ActionOutput::pending(
                "jj abandon change-a".to_owned(),
                preview.preview_text(),
                Some("abandon exact revision change-a from jk".to_owned()),
            ),
            preview,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let (input, body) = match &app.mode {
            InteractionMode::AbandonConfirm { input, output, .. } => {
                (input.as_str(), output.body_lines().join("\n"))
            }
            _ => panic!("expected abandon confirmation after recheck drift"),
        };
        assert_eq!(input, "");
        assert!(body.contains("change is no longer empty"));
        assert!(body.contains("M src/main.rs"));
        assert_eq!(
            app.status.message(),
            "change is no longer empty; type exact revision to confirm abandon"
        );

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();
        assert_eq!(
            app.status.message(),
            "confirmation did not match; abandon not run"
        );

        app.abandon_run = mock_abandon_success;
        for character in "change-a".chars() {
            app.handle_mode_key(crossterm::event::KeyCode::Char(character), 12)
                .unwrap();
        }
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::AbandonPreview { output, .. } => output,
            _ => panic!("expected abandon result mode"),
        };
        assert!(output.completed());
        assert!(
            output
                .body_lines()
                .join("\n")
                .contains("abandoned | jj undo")
        );
    }

    #[test]
    fn empty_abandon_recheck_failure_stays_readable_without_running() {
        ABANDON_FAILED_RECHECK_CALLS.store(1, Ordering::SeqCst);
        let preview = JjAbandonPreview::new(
            "change-a".to_owned(),
            Some("Empty change".to_owned()),
            String::new(),
        );
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
        ])));
        app.abandon_preview_load = mock_abandon_preview_recheck_failure;
        app.abandon_run = panic_abandon_run;
        app.mode = InteractionMode::AbandonPreview {
            abandon: JjAbandonPlan::new("change-a"),
            output: ActionOutput::pending(
                "jj abandon change-a".to_owned(),
                preview.preview_text(),
                None,
            ),
            preview,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::AbandonPreview { output, .. } => output,
            _ => panic!("expected readable abandon recheck failure"),
        };
        let body = output.body_lines().join("\n");
        assert!(output.completed());
        assert!(body.contains("jj diff -r change-a --summary failed: disappeared"));
        assert_eq!(
            app.status.message(),
            "jj diff -r change-a --summary failed: disappeared"
        );
    }

    #[test]
    fn non_empty_abandon_requires_exact_typed_revision() {
        let preview = JjAbandonPreview::new(
            "change-a".to_owned(),
            Some("Edit change".to_owned()),
            "M src/main.rs\n".to_owned(),
        );
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
        ])));
        app.abandon_run = panic_abandon_run;
        app.mode = InteractionMode::AbandonPreview {
            abandon: JjAbandonPlan::new("change-a"),
            output: ActionOutput::pending(
                "jj abandon change-a".to_owned(),
                preview.preview_text(),
                Some("abandon exact revision change-a from jk".to_owned()),
            ),
            preview,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();
        assert!(matches!(app.mode, InteractionMode::AbandonConfirm { .. }));

        app.handle_mode_key(crossterm::event::KeyCode::Char('x'), 12)
            .unwrap();
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();
        assert_eq!(
            app.status.message(),
            "confirmation did not match; abandon not run"
        );

        app.abandon_run = mock_abandon_success;
        app.handle_mode_key(crossterm::event::KeyCode::Backspace, 12)
            .unwrap();
        for character in "change-a".chars() {
            app.handle_mode_key(crossterm::event::KeyCode::Char(character), 12)
                .unwrap();
        }
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::AbandonPreview { output, .. } => output,
            _ => panic!("expected abandon result mode"),
        };
        assert!(output.completed());
        assert!(
            output
                .body_lines()
                .join("\n")
                .contains("abandoned | jj undo")
        );
    }

    #[test]
    fn abandon_cancel_restores_normal_mode_and_selection() {
        let mut graph = crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("first".to_owned()), None),
            crate::jj::LogItem::new(Vec::new(), Some("second".to_owned()), None),
        ]);
        graph.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 12,
                search: None,
            },
        );
        let preview = JjAbandonPreview::new("second".to_owned(), None, String::new());
        let mut app = test_app(ViewState::Graph(graph));
        app.mode = InteractionMode::AbandonPreview {
            abandon: JjAbandonPlan::new("second"),
            output: ActionOutput::pending(
                "jj abandon second".to_owned(),
                preview.preview_text(),
                None,
            ),
            preview,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .unwrap();

        let ViewState::Graph(graph) = &app.view else {
            panic!("expected graph view");
        };
        assert_eq!(graph.selected_revision(), Some("second"));
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "abandon cancelled");
    }

    #[test]
    fn abandon_failure_keeps_full_error_output_readable() {
        let preview = JjAbandonPreview::new("change-a".to_owned(), None, String::new());
        let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
            crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
        ])));
        app.abandon_run = mock_abandon_failure;
        app.mode = InteractionMode::AbandonPreview {
            abandon: JjAbandonPlan::new("change-a"),
            output: ActionOutput::pending(
                "jj abandon change-a".to_owned(),
                preview.preview_text(),
                None,
            ),
            preview,
        };

        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::AbandonPreview { output, .. } => output,
            _ => panic!("expected abandon result mode"),
        };
        let body = output.body_lines().join("\n");
        assert!(output.completed());
        assert!(body.contains("jj abandon change-a failed: first line"));
        assert!(body.contains("second line"));
        assert_eq!(
            app.status.message(),
            "jj abandon change-a failed: first line\nsecond line"
        );
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

    #[test]
    fn operation_log_undo_key_opens_global_preview_without_selected_operation_id() {
        let selected_operation_id = "b".repeat(128);
        let mut operation_log = crate::operation_log::OperationLogView::test_new(vec![
            crate::jj::OperationLogItem::new(
                vec![ratatui::text::Line::from("@  current")],
                Some("a".repeat(128)),
            ),
            crate::jj::OperationLogItem::new(
                vec![ratatui::text::Line::from("○  selected")],
                Some(selected_operation_id.clone()),
            ),
        ]);
        operation_log.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 12,
                search: None,
            },
        );
        let mut app = test_app(ViewState::OperationLog(operation_log));

        app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
            .unwrap();

        let output = match &app.mode {
            InteractionMode::OperationRecoveryPreview { recovery, output } => {
                assert_eq!(recovery.kind(), JjOperationRecoveryKind::Undo);
                output
            }
            _ => panic!("expected operation recovery preview"),
        };
        let body = output.body_lines().join("\n");
        assert_eq!(output.command_label(), "jj undo");
        assert!(body.contains("global current-repo undo from jk operation log"));
        assert!(body.contains("selected operation-log row is not an argument"));
        assert!(!body.contains(&selected_operation_id));
    }

    #[test]
    fn operation_recovery_preview_can_cancel_or_confirm_success() {
        let operation_log = crate::operation_log::OperationLogView::test_new(vec![
            crate::jj::OperationLogItem::new(
                vec![ratatui::text::Line::from("@  current")],
                Some("a".repeat(128)),
            ),
        ]);
        let mut app = test_app(ViewState::OperationLog(operation_log));

        app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
            .unwrap();
        app.handle_mode_key(KeyCode::Esc, 12).unwrap();
        assert!(matches!(app.mode, InteractionMode::Normal));
        assert_eq!(app.status.message(), "undo cancelled");

        app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
            .unwrap();
        app.handle_mode_key(KeyCode::Enter, 12).unwrap();

        let output = match &app.mode {
            InteractionMode::OperationRecoveryPreview { output, .. } => output,
            _ => panic!("expected operation recovery result"),
        };
        assert!(output.completed());
        assert!(
            output
                .body_lines()
                .join("\n")
                .contains("undone operation | jj redo")
        );
        assert_eq!(app.status.message(), "undone operation | jj redo");
    }

    #[test]
    fn operation_redo_failure_keeps_command_output_readable() {
        let operation_log = crate::operation_log::OperationLogView::test_new(vec![
            crate::jj::OperationLogItem::new(
                vec![ratatui::text::Line::from("@  current")],
                Some("a".repeat(128)),
            ),
        ]);
        let mut app = test_app(ViewState::OperationLog(operation_log));
        app.operation_recovery_run = mock_operation_recovery_failure;

        app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::CONTROL), 12)
            .unwrap();
        app.handle_mode_key(KeyCode::Enter, 12).unwrap();

        let output = match &app.mode {
            InteractionMode::OperationRecoveryPreview { recovery, output } => {
                assert_eq!(recovery.kind(), JjOperationRecoveryKind::Redo);
                output
            }
            _ => panic!("expected operation recovery result"),
        };
        let body = output.body_lines().join("\n");
        assert_eq!(output.command_label(), "jj redo");
        assert!(output.completed());
        assert!(body.contains("jj redo failed: no operation to redo available"));
        assert!(body.contains("hint: run the opposite recovery command first"));
        assert_eq!(
            app.status.message(),
            "jj redo failed: no operation to redo available\nhint: run the opposite recovery command first"
        );
    }

    #[test]
    fn back_from_operation_detail_returns_to_operation_log() {
        let operation_id = "abcdef".to_owned();
        let operation_log = crate::operation_log::OperationLogView::test_new(vec![
            crate::jj::OperationLogItem::new(
                vec![ratatui::text::Line::from("@  current")],
                Some(operation_id.clone()),
            ),
        ]);
        let detail = crate::operation_detail::OperationDetailView::test_new(
            ViewSpec::operation_show(operation_id),
            crate::rendered_jj::DocumentLines::new(vec![ratatui::text::Line::from(
                "operation details",
            )]),
        );
        let mut app = test_app(ViewState::OperationDetail(detail));
        app.stack.push(ViewState::OperationLog(operation_log));

        app.pop_view();

        assert!(matches!(app.view, ViewState::OperationLog(_)));
        assert_eq!(app.status.title(), "jk operation log");
        assert_eq!(app.status.message(), "1 operations");
    }
}
