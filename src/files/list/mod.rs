//! `jj file list` view state, rendering, and item-based navigation.
//!
//! The list stays path-first and keeps exact path identity alongside the rendered row text. That
//! lets refresh, copy, and drill-down behavior use the selected path directly instead of
//! reconstructing it from display labels.

mod commands;
mod render;
mod rows;
mod state;

use color_eyre::Result;

use crate::command::{Binding, Command, KeyPattern, ViewCommand};
#[cfg(test)]
use crate::jj::JjCommand;
use crate::jj::ViewSpec;
use crate::selection::Selection;

pub(crate) use rows::{FileListItem, load_file_list_entries};

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
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
];

/// Selectable file list output from `jj file list`.
pub struct FileListView {
    /// View specification used to reload the current file list.
    spec: ViewSpec,
    /// Rendered file rows plus exact path identity.
    entries: Vec<FileListItem>,
    /// Current selected file entry.
    selection: Selection,
}

impl FileListView {
    #[cfg(test)]
    pub(crate) fn test_new(entries: Vec<FileListItem>) -> Self {
        Self {
            entries,
            spec: ViewSpec::file_list(None, None),
            selection: Selection::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_with_spec(spec: ViewSpec, entries: Vec<FileListItem>) -> Self {
        Self {
            entries,
            spec,
            selection: Selection::default(),
        }
    }

    pub fn load(spec: ViewSpec) -> Result<Self> {
        let mut view = Self {
            entries: load_file_list_entries(&spec)?,
            spec,
            selection: Selection::default(),
        };
        if let Some(path) = view.spec.path()
            && let Some(index) = view.entries.iter().position(|entry| entry.path() == path)
        {
            view.selection.set(index, view.entries.len());
        }
        Ok(view)
    }

    /// Return the file-list-specific binding table.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Return the `ViewSpec` that owns this file list.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Return the number of selectable file entries.
    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    /// Return the total rendered line count across all file entries.
    pub fn line_count(&self) -> usize {
        self.entries.iter().map(FileListItem::line_count).sum()
    }

    /// Return the exact path of the currently selected entry, if any.
    pub fn selected_path(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .map(FileListItem::path)
    }
}

#[cfg(test)]
mod tests;
