//! `jj file show` document view state, rendering, and scroll navigation.
//!
//! This is a single-file document surface. It keeps the selected exact path alongside the rendered
//! text so copy and refresh behavior can stay tied to the same file without relying on displayed
//! labels.

mod commands;
mod render;
mod state;

use color_eyre::Result;

use crate::command::{Binding, Command, KeyPattern, ViewCommand};
#[cfg(test)]
use crate::documents::DocumentDisplayMode;
use crate::documents::{DocumentLines, DocumentViewport, load_document};
use crate::jj::ViewSpec;

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

/// Rendered `jj file show` output plus scroll state for one exact path.
pub struct FileShowView {
    /// View specification used to reload the current file show surface.
    spec: ViewSpec,
    /// Exact file path selected for this file show document.
    path: String,
    /// Rendered document lines for the selected file.
    document: DocumentLines,
    /// Current vertical scroll offset into the projected document.
    scroll_offset: usize,
    /// Current wrapping and horizontal-scroll viewport state.
    viewport: DocumentViewport,
}

impl FileShowView {
    /// Load the file-show document for the exact path owned by the `ViewSpec`.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let path = file_show_path(&spec);
        Ok(Self {
            path,
            document: load_document(&spec)?,
            spec,
            scroll_offset: 0,
            viewport: DocumentViewport::default(),
        })
    }

    #[cfg(test)]
    pub fn new(spec: ViewSpec, path: impl Into<String>, document: DocumentLines) -> Self {
        Self {
            spec,
            path: path.into(),
            document,
            scroll_offset: 0,
            viewport: DocumentViewport::default(),
        }
    }

    /// Return the file-show-specific binding table.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Return the `ViewSpec` that owns this file show surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Return the exact file path shown in this document.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Return the rendered document line count.
    pub fn line_count(&self) -> usize {
        self.document.line_count()
    }

    /// Return the current vertical scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    #[cfg(test)]
    pub fn horizontal_offset(&self) -> usize {
        self.viewport.horizontal_offset()
    }

    #[cfg(test)]
    pub fn display_mode(&self) -> DocumentDisplayMode {
        self.viewport.display_mode()
    }

    fn max_scroll_offset(&self) -> usize {
        self.line_count().saturating_sub(1)
    }

    fn max_line_width(&self) -> usize {
        self.document
            .lines()
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or_default()
    }
}

fn file_show_path(spec: &ViewSpec) -> String {
    spec.path()
        .map(str::to_owned)
        .or_else(|| spec.target().map(str::to_owned))
        .or_else(|| spec.args().last().cloned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
