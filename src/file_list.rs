//! `jj file list` view state, rendering, and item-based navigation.
//!
//! The list stays path-first and keeps exact path identity alongside the
//! rendered row text. That lets refresh, copy, and drill-down behavior use the
//! selected path directly instead of reconstructing it from display labels.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{FileListItem, JjCommand, ViewSpec, load_file_list_entries};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;
use crate::theme;

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
    spec: ViewSpec,
    entries: Vec<FileListItem>,
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

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(entry_list(&self.entries, search), area, &mut state);
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

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

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_file_list_entries)
    }

    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    pub fn line_count(&self) -> usize {
        self.entries.iter().map(FileListItem::line_count).sum()
    }

    pub fn selected_path(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .map(FileListItem::path)
    }

    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

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

    fn copy_options(&self) -> Vec<CopyOption> {
        self.selected_path()
            .map(|path| vec![CopyOption::new("file path", path)])
            .unwrap_or_default()
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<FileListItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_path = self.selected_path().map(str::to_owned);

        self.entries = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_path,
        );
        Ok(())
    }
}

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

fn restore_selection(
    selection: &mut Selection,
    entries: &[FileListItem],
    previous_index: usize,
    previous_path: Option<String>,
) {
    if let Some(path) = previous_path
        && let Some(index) = entries.iter().position(|entry| entry.path() == path)
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
mod tests {
    use ratatui::text::Line;

    use super::*;

    fn file_item(path: &str) -> FileListItem {
        FileListItem::new(vec![Line::from(path.to_owned())], path.to_owned())
    }

    fn file_list_view(paths: &[&str]) -> FileListView {
        FileListView {
            spec: ViewSpec::file_list(None, None),
            entries: paths.iter().map(|path| file_item(path)).collect(),
            selection: Selection::default(),
        }
    }

    #[test]
    fn file_list_moves_by_path_item() {
        let mut view = file_list_view(&["alpha", "beta", "gamma"]);

        view.execute(
            ViewCommand::MoveLast,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 2);

        view.execute(
            ViewCommand::MoveUp,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 1);

        view.execute(
            ViewCommand::MoveFirst,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn file_list_search_wraps_without_reselecting_current_item() {
        let mut view = file_list_view(&["alpha", "target one", "beta", "target two"]);
        view.selection.set(1, view.item_count());
        let query = SearchQuery::new("target".to_owned()).unwrap();

        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 3);

        assert!(view.previous_match(&query));
        assert_eq!(view.selection.index(), 1);
    }

    #[test]
    fn file_list_copy_uses_exact_path() {
        let mut view = file_list_view(&["src/space file.txt", "docs/readme.md"]);
        view.selection.set(0, view.item_count());

        let options = view.copy_options();

        assert_eq!(
            options,
            vec![CopyOption::new("file path", "src/space file.txt")]
        );
    }

    #[test]
    fn file_list_refresh_preserves_selected_path_when_possible() {
        let mut view = file_list_view(&["alpha", "beta", "gamma"]);
        view.selection.set(1, view.item_count());

        view.refresh_with_loader(|_| {
            Ok(vec![
                file_item("gamma"),
                file_item("beta"),
                file_item("delta"),
            ])
        })
        .unwrap();

        assert_eq!(view.selection.index(), 1);
        assert_eq!(view.selected_path(), Some("beta"));
    }

    #[test]
    fn file_list_refresh_clamps_when_selected_path_disappears() {
        let mut view = file_list_view(&["alpha", "beta", "gamma"]);
        view.selection.set(2, view.item_count());

        view.refresh_with_loader(|_| Ok(vec![file_item("alpha")]))
            .unwrap();

        assert_eq!(view.selection.index(), 0);
        assert_eq!(view.selected_path(), Some("alpha"));
    }

    #[test]
    fn file_list_open_selected_file_uses_exact_path() {
        let mut view = file_list_view(&["src/space file.txt"]);

        let effect = view.execute(
            ViewCommand::OpenItem,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        );

        assert_eq!(
            effect,
            ViewEffect::OpenDetail(JjCommand::FileShow, "src/space file.txt".to_owned())
        );
    }
}
