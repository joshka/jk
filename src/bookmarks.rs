//! `jj bookmark list` view state, rendering, and item-based navigation.
//!
//! The first pass keeps bookmark rows close to rendered `jj` output while
//! carrying exact bookmark names and target ids separately for copy,
//! search, refresh, and open-show behavior.

mod action_targets;
pub(crate) mod actions;
mod rows;

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

use self::action_targets::BookmarkActionTargetResolver;
use self::actions::{JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkTrackingTarget};
pub(crate) use self::rows::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState, load_bookmark_entries,
};
use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, ViewSpec};
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
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenShow)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Enter),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(KeyPattern::char('x'), Command::BookmarkDelete),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Selectable bookmark output from `jj bookmark list`.
pub struct BookmarksView {
    spec: ViewSpec,
    entries: Vec<BookmarkItem>,
    selection: Selection,
}

impl BookmarksView {
    #[cfg(test)]
    pub(crate) fn test_new(entries: Vec<BookmarkItem>) -> Self {
        Self {
            entries,
            spec: ViewSpec::new(JjCommand::Bookmarks, Vec::new()),
            selection: Selection::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_new_with_args(entries: Vec<BookmarkItem>, args: Vec<String>) -> Self {
        Self {
            entries,
            spec: ViewSpec::new(JjCommand::Bookmarks, args),
            selection: Selection::default(),
        }
    }

    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: load_bookmark_entries(&spec)?,
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
            ViewCommand::OpenShow => self
                .selected_entry()
                .and_then(BookmarkItem::target_change_id)
                .map(|change_id| ViewEffect::OpenDetail(JjCommand::Show, change_id.to_owned()))
                .unwrap_or_else(|| {
                    ViewEffect::StatusMessage(
                        "selected bookmark has no target change id".to_owned(),
                    )
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
            | ViewCommand::OpenItem
            | ViewCommand::OpenDiff
            | ViewCommand::ToggleSelect
            | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_bookmark_entries)
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
        self.entries.iter().map(BookmarkItem::line_count).sum()
    }

    fn selected_entry(&self) -> Option<&BookmarkItem> {
        self.entries.get(self.selection.index())
    }

    pub fn selected_bookmark_name(&self) -> Option<&str> {
        self.selected_entry().map(BookmarkItem::bookmark_name)
    }

    pub fn selected_local_bookmark_name(&self) -> Option<&str> {
        self.action_targets().selected_local_bookmark_name()
    }

    pub fn selected_bookmark_forget_target(
        &self,
    ) -> Result<Option<(&str, JjBookmarkForgetTarget)>> {
        self.action_targets().selected_bookmark_forget_target()
    }

    pub fn selected_bookmark_tracking_target(
        &self,
        kind: JjBookmarkMutationKind,
    ) -> Result<Option<(&str, JjBookmarkTrackingTarget)>> {
        self.action_targets()
            .selected_bookmark_tracking_target(kind)
    }

    fn action_targets(&self) -> BookmarkActionTargetResolver<'_> {
        BookmarkActionTargetResolver::new(self.selected_entry(), &self.entries, self.spec.args())
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
        let Some(entry) = self.selected_entry() else {
            return Vec::new();
        };

        let mut options = Vec::new();
        options.push(CopyOption::new("bookmark name", entry.bookmark_name()));
        if let Some(change_id) = entry.target_change_id() {
            options.push(CopyOption::new("change id", change_id));
        }
        if let Some(commit_id) = entry.target_commit_id() {
            options.push(CopyOption::new("commit id", commit_id));
        }
        options.push(CopyOption::new("row text", entry.row_text()));
        options
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<BookmarkItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_bookmark_name = self
            .selected_entry()
            .map(|entry| entry.bookmark_name().to_owned());

        self.entries = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_bookmark_name,
        );
        Ok(())
    }
}

fn entry_list(entries: &[BookmarkItem], search: Option<&SearchQuery>) -> List<'static> {
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
    entries: &[BookmarkItem],
    previous_index: usize,
    previous_bookmark_name: Option<String>,
) {
    if let Some(bookmark_name) = previous_bookmark_name
        && let Some(index) = entries
            .iter()
            .position(|entry| entry.bookmark_name() == bookmark_name.as_str())
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
mod tests;
