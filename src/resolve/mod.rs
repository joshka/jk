//! `jk resolve` conflict list view state and path-first navigation.
//!
//! The first pass stays read-only. It lists conflicted paths from a machine
//! template contract, preserves exact paths for refresh and copy behavior, and
//! opens `jj file show` for inspection without launching external resolvers or
//! mutating files.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, ViewSpec};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::{Selection, restore_by_key_or_index};
use crate::theme;

mod rows;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use rows::RESOLVE_CONFLICT_TEMPLATE;
pub(crate) use rows::{ResolveEntry, load_resolve_entries};

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
    spec: ViewSpec,
    entries: Vec<ResolveEntry>,
    selection: Selection,
}

impl ResolveView {
    #[cfg(test)]
    pub(crate) fn test_new(entries: Vec<ResolveEntry>) -> Self {
        Self {
            spec: ViewSpec::resolve(None),
            entries,
            selection: Selection::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_with_spec(spec: ViewSpec, entries: Vec<ResolveEntry>) -> Self {
        Self {
            spec,
            entries,
            selection: Selection::default(),
        }
    }

    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: load_resolve_entries(&spec)?,
            spec,
            selection: Selection::default(),
        })
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

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_resolve_entries)
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
        self.entries.iter().map(entry_line_count).sum()
    }

    pub fn selected_path(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(ResolveEntry::path)
    }

    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry_lines(entry), query))
            .count()
    }

    fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.entries.len())
            .chain(0..self.selection.index().min(self.entries.len()))
            .find(|index| entry_matches(&entry_lines(&self.entries[*index]), query))
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
            .find(|index| entry_matches(&entry_lines(&self.entries[*index]), query))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    fn copy_options(&self) -> Vec<CopyOption> {
        let Some(entry) = self.entries.get(self.selection.index()) else {
            return Vec::new();
        };

        let mut options = Vec::new();
        if let Some(path) = entry.path() {
            options.push(CopyOption::new("conflict path", path));
        }
        options.push(CopyOption::new("row text", entry_row_text(entry)));
        options
    }

    fn refresh_with_loader(
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

fn entry_list(entries: &[ResolveEntry], search: Option<&SearchQuery>) -> List<'static> {
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

fn entry_lines(entry: &ResolveEntry) -> Vec<Line<'static>> {
    entry.lines()
}

fn entry_line_count(entry: &ResolveEntry) -> usize {
    entry_lines(entry).len()
}

fn entry_row_text(entry: &ResolveEntry) -> String {
    entry_lines(entry)
        .into_iter()
        .map(|line| {
            line.spans
                .into_iter()
                .map(|span| span.content.into_owned())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

impl ResolveEntry {
    fn lines(&self) -> Vec<Line<'static>> {
        if let Some(raw_line) = self.raw_line() {
            return vec![
                Line::from("unparsed conflict metadata"),
                Line::from(raw_line.to_owned()),
            ];
        }

        vec![
            Line::from(self.path().unwrap_or("(path unavailable)").to_owned()),
            Line::from(format!(
                "type: {}  sides: {}",
                self.file_type().unwrap_or("unknown"),
                side_count_label(self.side_count()),
            )),
        ]
    }
}

fn side_count_label(side_count: Option<usize>) -> String {
    match side_count {
        Some(1) => "1".to_owned(),
        Some(count) => count.to_string(),
        None => "unknown".to_owned(),
    }
}
