//! `jj file list` view state, rendering, and item-based navigation.
//!
//! The list stays path-first and keeps exact path identity alongside the
//! rendered row text. That lets refresh, copy, and drill-down behavior use the
//! selected path directly instead of reconstructing it from display labels.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

mod rows;

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, ViewSpec};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::{Selection, restore_by_key_or_index};
use crate::theme;

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

    /// Render the current file list with active-row styling and optional search highlighting.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(entry_list(&self.entries, search), area, &mut state);
    }

    /// Return the file-list-specific binding table.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Execute one view-local command against the file list.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::MoveDown => {
                self.selection.next(self.entries.len());
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.selection.previous();
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.selection.first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.selection.last(self.entries.len());
                ViewEffect::Handled
            }
            ViewCommand::OpenItem => self
                .selected_path()
                .map(|path| ViewEffect::OpenDetail(JjCommand::FileShow, path.to_owned()))
                .unwrap_or_else(|| {
                    ViewEffect::StatusMessage("selected file list is empty".to_owned())
                }),
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::ToggleSelect | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::PageDown
            | ViewCommand::PageUp
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff => ViewEffect::Ignored,
        }
    }

    /// Reload the file list while preserving the selected path when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_file_list_entries)
    }

    /// Clamp the current selection to the current entry count.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
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

    /// Count search matches across rendered file-list entries.
    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    /// Move to the next matching file entry, wrapping once without reselecting the current row.
    fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.entries.len())
            .chain(0..self.selection.index().min(self.entries.len()))
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    /// Move to the previous matching file entry, wrapping once without reselecting the current row.
    fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.entries.len()).rev())
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    /// Return copy options for the currently selected exact file path.
    fn copy_options(&self) -> Vec<CopyOption> {
        self.selected_path()
            .map(|path| vec![CopyOption::new("file path", path)])
            .unwrap_or_default()
    }

    /// Reload entries with a caller-supplied loader while restoring selection by exact path first.
    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<FileListItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_path = self.selected_path().map(str::to_owned);

        self.entries = load(&self.spec)?;
        restore_by_key_or_index(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_path.as_deref(),
            |entry| Some(entry.path()),
        );
        Ok(())
    }
}

/// Build the rendered file-entry list with active-row styling and search highlighting.
fn entry_list(entries: &[FileListItem], search: Option<&SearchQuery>) -> List<'static> {
    let items = entries
        .iter()
        .map(|entry| {
            let lines = entry
                .lines()
                .into_iter()
                .map(|line| highlight_line(line, search))
                .collect::<Vec<_>>();
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}

#[cfg(test)]
mod tests;
