//! Log view state, rendering, and item-based navigation.
//!
//! Rows are grouped from `jj`'s rendered graph output. Detail navigation uses
//! the change id for the selected row because jj workflows and revsets are
//! change-centric; commit ids remain available through copy actions.

use ratatui::style::Style;

use super::LogItem;
use crate::command::{Binding, Command, KeyPattern, ViewCommand};
use crate::jj::{JjCommand, LogViewMode, ViewSpec};
use crate::selection::Selection;
use crate::tui::theme;

mod commands;
mod render;
mod state;

pub const BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('w'), Command::View(ViewCommand::CycleMode)),
    Binding::new(KeyPattern::char('c'), Command::View(ViewCommand::NewTrunk)),
    Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Down),
        Command::View(ViewCommand::MoveDown),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageDown),
        Command::View(ViewCommand::PageDown),
    ),
    Binding::new(KeyPattern::char('k'), Command::View(ViewCommand::MoveUp)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Up),
        Command::View(ViewCommand::MoveUp),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageUp),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
    Binding::sequence(GIT_FETCH_KEYS, Command::Fetch),
    Binding::sequence(GIT_PUSH_KEYS, Command::Push),
    Binding::sequence(GIT_FETCH_REMOTE_KEYS, Command::FetchRemote),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Home),
        Command::View(ViewCommand::MoveFirst),
    ),
    Binding::new(KeyPattern::char('G'), Command::View(ViewCommand::MoveLast)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::End),
        Command::View(ViewCommand::MoveLast),
    ),
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenShow)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(KeyPattern::char('d'), Command::View(ViewCommand::OpenDiff)),
    Binding::new(KeyPattern::char('e'), Command::Edit),
    Binding::new(KeyPattern::char(']'), Command::NextEdit),
    Binding::new(KeyPattern::char('['), Command::PrevEdit),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
    Binding::new(
        KeyPattern::char(' '),
        Command::View(ViewCommand::ToggleSelect),
    ),
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
];

const GIT_FETCH_KEYS: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('f')];
const GIT_PUSH_KEYS: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('p')];
const GIT_FETCH_REMOTE_KEYS: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('r')];

fn explicit_selection_style() -> Style {
    theme::marked_row_style()
}

#[cfg(test)]
pub fn test_explicit_selection_style() -> Style {
    explicit_selection_style()
}

/// Selectable graph output from `jj` or `jj log`.
pub struct LogView {
    /// Top-level command that established this log surface, used when cycling modes back home.
    home_command: JjCommand,
    /// Current log presentation mode derived from the active `ViewSpec`.
    mode: LogViewMode,
    /// Original spec used to load and later refresh this view.
    spec: ViewSpec,
    /// Selectable rendered log items loaded from jj output.
    entries: Vec<LogItem>,
    /// Current cursor position within `entries`.
    selection: Selection,
    /// Explicitly marked change ids that stay highlighted independently of the cursor.
    selected_change_ids: Vec<String>,
}

impl LogView {
    #[cfg(test)]
    pub fn test_new(entries: Vec<LogItem>) -> Self {
        Self {
            home_command: JjCommand::Default,
            mode: LogViewMode::Default,
            spec: ViewSpec::new(JjCommand::Default, Vec::new()),
            entries,
            selection: Selection::default(),
            selected_change_ids: Vec::new(),
        }
    }

    #[cfg(test)]
    pub fn test_with_spec(spec: ViewSpec, entries: Vec<LogItem>) -> Self {
        Self {
            home_command: spec.command(),
            mode: LogViewMode::from_spec(&spec),
            spec,
            entries,
            selection: Selection::default(),
            selected_change_ids: Vec::new(),
        }
    }

    pub fn bindings(&self) -> &'static [Binding] {
        super::BINDINGS
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn mode_label(&self) -> &str {
        self.mode.label()
    }

    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    pub fn line_count(&self) -> usize {
        self.entries.iter().map(LogItem::line_count).sum()
    }
}

#[cfg(test)]
pub fn test_entry_lines(
    entry: &LogItem,
    search: Option<&crate::search::SearchQuery>,
    is_selected: bool,
) -> Vec<ratatui::text::Line<'static>> {
    self::render::test_entry_lines(entry, search, is_selected)
}

#[cfg(test)]
pub fn test_restore_selection(
    selection: &mut crate::selection::Selection,
    entries: &[LogItem],
    previous_index: usize,
    previous_change_id: Option<String>,
) {
    self::state::test_restore_selection(selection, entries, previous_index, previous_change_id);
}
