//! `jj bookmark list` view state, rendering, and item-based navigation.
//!
//! The first pass keeps bookmark rows close to rendered `jj` output while
//! carrying exact bookmark names and target ids separately for copy,
//! search, refresh, and open-show behavior.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{BookmarkItem, BookmarkRowState, JjCommand, ViewSpec, load_bookmark_entries};
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
        self.selected_entry()
            .filter(|entry| matches!(entry.state(), BookmarkRowState::Local { .. }))
            .map(BookmarkItem::bookmark_name)
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
    if let Some(bookmark_name) = previous_bookmark_name {
        if let Some(index) = entries
            .iter()
            .position(|entry| entry.bookmark_name() == bookmark_name.as_str())
        {
            selection.set(index, entries.len());
            return;
        }
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use ratatui::text::Line;

    use super::*;
    use crate::command::{Command, find_binding};
    use crate::jj::JjCommand;

    fn bookmark_item(
        text: &[&str],
        bookmark_name: &str,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkItem {
        BookmarkItem::new(
            text.iter()
                .map(|line| Line::from((*line).to_owned()))
                .collect::<Vec<_>>(),
            bookmark_name.to_owned(),
            target_change_id.map(str::to_owned),
            target_commit_id.map(str::to_owned),
        )
    }

    fn bookmarks_view(entries: Vec<BookmarkItem>) -> BookmarksView {
        BookmarksView {
            spec: ViewSpec::new(JjCommand::Bookmarks, Vec::new()),
            entries,
            selection: Selection::default(),
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn movement_is_bookmark_item_based() {
        let mut view = bookmarks_view(vec![
            bookmark_item(
                &["@  feature", "│  target change"],
                "feature",
                Some("a"),
                Some("aa"),
            ),
            bookmark_item(&["○  trunk"], "trunk", Some("b"), Some("bb")),
        ]);

        view.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );

        assert_eq!(view.selection.index(), 1);
        view.execute(
            ViewCommand::MoveUp,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 0);
        view.execute(
            ViewCommand::MoveLast,
            CommandContext {
                viewport_height: 10,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 1);
    }

    #[test]
    fn copy_options_include_exact_name_and_target_ids_when_known() {
        let view = bookmarks_view(vec![bookmark_item(
            &["@  feature", "│  target change"],
            "feature",
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            Some("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
        )]);

        let options = view.copy_options();

        assert_eq!(options.len(), 4);
        assert_eq!(options[0].label(), "bookmark name");
        assert_eq!(options[0].value(), "feature");
        assert_eq!(options[1].label(), "change id");
        assert_eq!(options[1].value().len(), 64);
        assert_eq!(options[2].label(), "commit id");
        assert_eq!(options[2].value().len(), 64);
        assert_eq!(options[3].label(), "row text");
        assert_eq!(options[3].value(), "@  feature\n│  target change");
    }

    #[test]
    fn refresh_preserves_selected_bookmark_name() {
        let mut view = bookmarks_view(vec![
            bookmark_item(&["@  first"], "first", Some("a"), Some("aa")),
            bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
        ]);
        view.selection.set(1, view.entries.len());

        view.refresh_with_loader(|_| {
            Ok(vec![
                bookmark_item(&["@  second"], "second", Some("b2"), Some("bb2")),
                bookmark_item(&["○  third"], "third", Some("c"), Some("cc")),
            ])
        })
        .unwrap();

        assert_eq!(view.selection.index(), 0);
        assert_eq!(view.entries[0].bookmark_name(), "second");
    }

    #[test]
    fn refresh_clamps_when_selected_bookmark_disappears() {
        let mut view = bookmarks_view(vec![
            bookmark_item(&["@  first"], "first", Some("a"), Some("aa")),
            bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
        ]);
        view.selection.set(1, view.entries.len());

        view.refresh_with_loader(|_| {
            Ok(vec![bookmark_item(
                &["@  first"],
                "first",
                Some("a"),
                Some("aa"),
            )])
        })
        .unwrap();

        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn selected_bookmark_name_returns_current_row() {
        let view = bookmarks_view(vec![
            bookmark_item(
                &["@  feature", "│  target"],
                "feature",
                Some("a"),
                Some("aa"),
            ),
            bookmark_item(&["○  second"], "second", Some("b"), Some("bb")),
        ]);

        assert_eq!(view.selected_bookmark_name(), Some("feature"));
    }

    #[test]
    fn selected_local_bookmark_name_ignores_nonlocal_rows() {
        let remote = bookmark_item(&["  @origin: abc"], "@origin", None, None).with_local(false);
        let view = bookmarks_view(vec![remote]);

        assert_eq!(view.selected_bookmark_name(), Some("@origin"));
        assert_eq!(view.selected_local_bookmark_name(), None);
    }

    #[test]
    fn selected_local_bookmark_name_ignores_unknown_metadata_rows() {
        let unknown = bookmark_item(&["maybe-local: abc"], "maybe-local", None, None)
            .with_state(BookmarkRowState::Unknown);
        let view = bookmarks_view(vec![unknown]);

        assert_eq!(view.selected_bookmark_name(), Some("maybe-local"));
        assert_eq!(view.selected_local_bookmark_name(), None);
    }

    #[test]
    fn search_wraps_by_bookmark_item() {
        let mut view = bookmarks_view(vec![
            bookmark_item(&["@  alpha", "│  target"], "alpha", Some("a"), Some("aa")),
            bookmark_item(&["○  beta"], "beta", Some("b"), Some("bb")),
            bookmark_item(&["○  alpha"], "gamma", Some("c"), Some("cc")),
        ]);
        view.selection.set(1, view.entries.len());
        let query = SearchQuery::new("alpha".to_owned()).unwrap();

        assert_eq!(view.search_matches(&query), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 0);
        assert!(view.previous_match(&query));
        assert_eq!(view.selection.index(), 2);
    }

    #[test]
    fn open_show_uses_target_change_id_and_reports_missing_targets() {
        let view = bookmarks_view(vec![bookmark_item(
            &["@  feature", "│  target change"],
            "feature",
            Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
            Some("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210"),
        )]);

        assert_eq!(
            find_binding(view.bindings(), key(KeyCode::Enter)).map(Binding::command),
            Some(Command::View(ViewCommand::OpenShow))
        );
        assert_eq!(
            find_binding(view.bindings(), key(KeyCode::Char('s'))).map(Binding::command),
            Some(Command::View(ViewCommand::OpenShow))
        );

        let mut view = view;
        assert_eq!(
            view.execute(
                ViewCommand::OpenShow,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::OpenDetail(
                JjCommand::Show,
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned()
            )
        );

        let mut view = bookmarks_view(vec![bookmark_item(&["○  feature"], "feature", None, None)]);
        assert_eq!(
            view.execute(
                ViewCommand::OpenShow,
                CommandContext {
                    viewport_height: 10,
                    viewport_width: 80,
                    search: None,
                },
            ),
            ViewEffect::StatusMessage("selected bookmark has no target change id".to_owned())
        );
    }
}
