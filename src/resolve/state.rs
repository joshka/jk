use color_eyre::Result;

use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::jj::{JjCommand, ViewSpec};
use crate::menus::CopyOption;
use crate::search::{SearchQuery, entry_matches};
use crate::selection::restore_by_key_or_index;

use super::{ResolveEntry, ResolveView, load_resolve_entries};

impl ResolveView {
    /// Loads resolve rows and initializes selection at the first row.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: load_resolve_entries(&spec)?,
            spec,
            selection: crate::selection::Selection::default(),
        })
    }

    /// Applies selection, search, copy, and file-inspection commands to the resolve list.
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
            ViewCommand::OpenItem => match self.entries.get(self.selection.index()) {
                None => ViewEffect::StatusMessage("resolve list is empty".to_owned()),
                Some(entry) => entry
                    .path()
                    .map(|path| ViewEffect::OpenDetail(JjCommand::FileShow, path.to_owned()))
                    .unwrap_or_else(|| {
                        ViewEffect::StatusMessage(
                            "resolve inspect unavailable: selected conflict has no exact path"
                                .to_owned(),
                        )
                    }),
            },
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
            | ViewCommand::OpenDiff
            | ViewCommand::ToggleSelect
            | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
        }
    }

    /// Reloads resolve rows while preserving the selected exact path when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_resolve_entries)
    }

    /// Clamps the current selection to the available rows.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    /// Counts rows whose rendered text matches the current search query.
    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    /// Advances selection to the next matching row if one exists.
    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.entries.len())
            .chain(0..self.selection.index().min(self.entries.len()))
            .find(|index| entry_matches(&self.entries[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    /// Moves selection to the previous matching row if one exists.
    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
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

    /// Returns copyable exact paths and rendered row text for the selected conflict.
    pub fn copy_options(&self) -> Vec<CopyOption> {
        let Some(entry) = self.entries.get(self.selection.index()) else {
            return Vec::new();
        };

        let mut options = Vec::new();
        if let Some(path) = entry.path() {
            options.push(CopyOption::new("conflict path", path));
        }
        options.push(CopyOption::new("row text", entry.row_text()));
        options
    }

    /// Reloads rows and restores selection by exact path before falling back to index.
    pub fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<ResolveEntry>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_path = self.selected_path().map(str::to_owned);

        self.entries = load(&self.spec)?;
        restore_by_key_or_index(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_path.as_deref(),
            ResolveEntry::path,
        );
        Ok(())
    }
}
