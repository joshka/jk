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

use crate::clipboard;
use crate::command::{
    Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect, find_binding,
};
use crate::copy::CopyOption;
use crate::jj::{DiffFormat, JjCommand, ViewSpec};
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
    CopyMenu {
        options: Vec<CopyOption>,
        selected: usize,
    },
    ViewMenu {
        selected: usize,
    },
}

const APP_BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('q'), Command::Quit),
    Binding::new(KeyPattern::code(KeyCode::Esc), Command::Quit),
    Binding::new(KeyPattern::char('?'), Command::Help),
    Binding::new(KeyPattern::char('/'), Command::SearchPrompt),
    Binding::new(KeyPattern::char('y'), Command::Copy),
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
                self.apply_view_effect(effect)
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

    fn execute_view(&mut self, command: ViewCommand, viewport_height: u16) -> ViewEffect {
        self.view.execute(
            command,
            CommandContext {
                viewport_height,
                search: self.search.as_ref(),
            },
        )
    }

    fn apply_view_effect(&mut self, effect: ViewEffect) -> Result<bool> {
        match effect {
            ViewEffect::Ignored | ViewEffect::Handled => Ok(true),
            ViewEffect::OpenDetail(command, revset) => {
                self.push_detail(command, revset)?;
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
            _ => self.status.clone(),
        }
    }

    fn overlay(&self) -> Overlay<'_> {
        match &self.mode {
            InteractionMode::Help => Overlay::Help,
            InteractionMode::CopyMenu { options, selected } => Overlay::CopyMenu {
                options,
                selected: *selected,
            },
            InteractionMode::ViewMenu { selected } => Overlay::ViewMenu {
                options: view_formats(),
                selected: *selected,
            },
            InteractionMode::Normal | InteractionMode::SearchPrompt(_) => Overlay::None,
        }
    }

    fn push_detail(&mut self, command: JjCommand, revset: String) -> Result<()> {
        let spec = match command {
            JjCommand::Show => ViewSpec::show(revset, self.diff_format),
            JjCommand::Diff => ViewSpec::diff(revset, self.diff_format),
            JjCommand::Default | JjCommand::Log => return Ok(()),
        };
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
        unknown => Err(eyre!(
            "unsupported jk command '{unknown}'. Expected one of: log, show, diff"
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
        let message = if let Some(item_count) = view.graph_item_count() {
            format!("{item_count} items")
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

#[derive(Clone, Debug)]
pub enum StatusKind {
    Ready,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn rejects_unknown_startup_command() {
        assert!(initial_view(vec!["status".into()]).is_err());
    }
}
