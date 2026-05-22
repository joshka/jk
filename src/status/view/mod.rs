//! `jj status` view state, rendering, and scroll navigation.
//!
//! Status stays close to rendered `jj status` output, but it also carries a
//! narrow row model for exact file-path actions. Rows that do not confidently
//! name one repo-relative tracked path remain visible and selectable, but file
//! mutation actions report why they are disabled instead of guessing.

use crate::command::{Binding, Command, KeyPattern, ViewCommand};
use crate::jj::ViewSpec;
use crate::selection::Selection;

use super::rows::StatusRow;

mod commands;
mod render;
mod state;

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
    Binding::new(KeyPattern::char(' '), Command::View(ViewCommand::PageDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageDown),
        Command::View(ViewCommand::PageDown),
    ),
    Binding::new(
        KeyPattern::modified_char('f', crossterm::event::KeyModifiers::CONTROL),
        Command::View(ViewCommand::PageDown),
    ),
    Binding::new(
        KeyPattern::modified_char(' ', crossterm::event::KeyModifiers::SHIFT),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageUp),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(
        KeyPattern::modified_char('b', crossterm::event::KeyModifiers::CONTROL),
        Command::View(ViewCommand::PageUp),
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
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenFiles)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenFiles),
    ),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
];

/// Rendered `jj status` output plus exact row action contracts.
pub struct StatusView {
    /// View specification used to reload the current status screen.
    spec: ViewSpec,
    /// Rendered status rows plus exact-path action contracts.
    rows: Vec<StatusRow>,
    /// Current selected row in the rendered status output.
    selection: Selection,
}

impl StatusView {
    /// Return the status-specific binding table.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Return the `ViewSpec` that owns this status screen.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Return the total rendered row count.
    pub fn line_count(&self) -> usize {
        self.rows.len()
    }

    /// Return the selected row index used as the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.selection.index()
    }

    /// Set the selected row index, clamped to the current row count.
    pub fn set_scroll_offset(&mut self, _viewport_height: u16, scroll_offset: usize) {
        self.selection.set(scroll_offset, self.rows.len());
    }

    pub fn clamp(&mut self, _viewport_height: u16) {
        self.selection.clamp(self.rows.len());
    }
}

#[cfg(test)]
mod tests;
