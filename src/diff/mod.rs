//! `jj diff` view state, rendering, and navigation.
//!
//! Diff only pins the active file heading. Unlike show, it has no commit
//! context prefix because the command output is already focused on the patch.

use crate::command::{Binding, Command, KeyPattern, ViewCommand};
use crate::documents::StickyFileDocument;
use crate::jj::ViewSpec;

mod commands;
mod render;
mod state;

const TOGGLE_WRAP_KEYS: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('w')];
const SCROLL_LEFT_KEYS: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('h')];
const SCROLL_RIGHT_KEYS: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('l')];
const HORIZONTAL_SCROLL_AMOUNT: usize = 1;

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
    Binding::sequence(TOGGLE_WRAP_KEYS, Command::View(ViewCommand::ToggleWrap)),
    Binding::sequence(SCROLL_LEFT_KEYS, Command::View(ViewCommand::ScrollLeft)),
    Binding::sequence(SCROLL_RIGHT_KEYS, Command::View(ViewCommand::ScrollRight)),
    Binding::new(KeyPattern::char(']'), Command::View(ViewCommand::NextFile)),
    Binding::new(
        KeyPattern::char('['),
        Command::View(ViewCommand::PreviousFile),
    ),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenFiles)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenFiles),
    ),
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
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

/// Rendered `jj diff` output plus sticky file context and scroll state.
pub struct DiffView {
    /// View identity and revset navigation target for refresh and drill-down commands.
    spec: ViewSpec,
    /// Shared sticky document state for rendered lines, viewport, and file navigation.
    document: StickyFileDocument,
}

impl DiffView {
    #[cfg(test)]
    pub fn test_new(spec: ViewSpec) -> Self {
        Self {
            spec,
            document: StickyFileDocument::new(crate::documents::DocumentLines::new(Vec::new())),
        }
    }

    /// Returns the key bindings owned by the diff view.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Returns the view spec that identifies this diff surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the rendered line count of the underlying diff body.
    pub fn line_count(&self) -> usize {
        self.document.line_count()
    }

    /// Returns the current vertical scroll offset in rendered lines.
    pub fn scroll_offset(&self) -> usize {
        self.document.scroll_offset()
    }

    #[cfg(test)]
    pub fn horizontal_offset(&self) -> usize {
        self.document.horizontal_offset()
    }

    /// Restores a saved vertical scroll position, clamped to the current viewport.
    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        self.document
            .set_scroll_offset(viewport_height, scroll_offset);
    }
}

#[cfg(test)]
mod tests;
