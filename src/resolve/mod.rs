//! `jk resolve` conflict list view state and path-first navigation.
//!
//! The first pass stays read-only. It lists conflicted paths from a machine
//! template contract, preserves exact paths for refresh and copy behavior, and
//! opens `jj file show` for inspection without launching external resolvers or
//! mutating files.

use crate::command::{Binding, Command, KeyPattern, ViewCommand};
use crate::jj::ViewSpec;
use crate::selection::Selection;

mod render;
mod rows;
mod state;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub use rows::RESOLVE_CONFLICT_TEMPLATE;
pub use rows::{ResolveEntry, load_resolve_entries};

pub const BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Down),
        Command::View(ViewCommand::MoveDown),
    ),
    Binding::new(KeyPattern::char('k'), Command::View(ViewCommand::MoveUp)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Up),
        Command::View(ViewCommand::MoveUp),
    ),
    Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Home),
        Command::View(ViewCommand::MoveFirst),
    ),
    Binding::new(KeyPattern::char('G'), Command::View(ViewCommand::MoveLast)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::End),
        Command::View(ViewCommand::MoveLast),
    ),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenItem)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenItem),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Enter),
        Command::View(ViewCommand::OpenItem),
    ),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Selectable conflict list output from the resolve template contract.
pub struct ResolveView {
    /// View identity used to reload the resolve list.
    spec: ViewSpec,
    /// Conflicted-path rows loaded from the resolve template contract.
    entries: Vec<ResolveEntry>,
    /// Current selected row within the resolve list.
    selection: Selection,
}

impl ResolveView {
    #[cfg(test)]
    pub fn test_new(entries: Vec<ResolveEntry>) -> Self {
        Self {
            spec: ViewSpec::resolve_current(),
            entries,
            selection: Selection::default(),
        }
    }

    #[cfg(test)]
    pub fn test_with_spec(spec: ViewSpec, entries: Vec<ResolveEntry>) -> Self {
        Self {
            spec,
            entries,
            selection: Selection::default(),
        }
    }

    /// Returns the key bindings owned by the resolve view.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Returns the view spec that identifies this resolve surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the number of selectable resolve rows.
    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the exact conflicted path for the selected row, if available.
    pub fn selected_path(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(ResolveEntry::path)
    }
}
