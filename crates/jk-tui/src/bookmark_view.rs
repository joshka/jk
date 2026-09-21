//! Provider-neutral bookmark list view.

use ratatui::Frame;
use ratatui::prelude::{Line, Text};
use ratatui::widgets::Paragraph;

use crate::chrome::{ViewChrome, render_help_overlay};
use crate::selected_row::{interaction_areas, paint_cursor};

/// Snapshot consumed by [`BookmarkView`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BookmarkViewSnapshot {
    title: String,
    rows: Vec<BookmarkRow>,
}

impl BookmarkViewSnapshot {
    /// Creates a bookmark snapshot.
    #[must_use]
    pub fn new(rows: Vec<BookmarkRow>) -> Self {
        Self {
            title: "jj bookmark list".to_owned(),
            rows,
        }
    }

    /// Sets the title shown in the view chrome.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Returns the title shown in the view chrome.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns bookmark rows.
    #[must_use]
    pub fn rows(&self) -> &[BookmarkRow] {
        &self.rows
    }
}

/// A bookmark row rendered by the TUI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookmarkRow {
    /// Stable bookmark name.
    pub name: String,
    /// Optional remote name.
    pub remote: Option<String>,
    /// Full target IDs.
    pub targets: Vec<String>,
    /// Whether the reference is tracked.
    pub tracked: bool,
    /// Removed target IDs in a bookmark conflict.
    pub removed_targets: Vec<String>,
    /// Whether the target is a merge, even when only one added ID survives.
    pub conflicted: bool,
    /// Whether a tracked remote has exactly the same target as its local bookmark.
    pub synchronized: bool,
}

impl BookmarkRow {
    /// Creates a display row.
    #[must_use]
    pub fn new(name: impl Into<String>, targets: Vec<String>) -> Self {
        let conflicted = targets.len() > 1;
        Self {
            name: name.into(),
            remote: None,
            targets,
            tracked: false,
            removed_targets: Vec::new(),
            conflicted,
            synchronized: false,
        }
    }

    /// Sets the remote name.
    #[must_use]
    pub fn with_remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = Some(remote.into());
        self
    }

    /// Marks the row as tracked.
    #[must_use]
    pub const fn with_tracked(mut self, tracked: bool) -> Self {
        self.tracked = tracked;
        self
    }

    /// Sets synchronization state derived from jj's full target merge terms.
    #[must_use]
    pub const fn with_synchronized(mut self, synchronized: bool) -> Self {
        self.synchronized = synchronized;
        self
    }

    /// Retains conflict state separately from the number of added target IDs.
    #[must_use]
    pub fn with_conflict(mut self, conflicted: bool, removed_targets: Vec<String>) -> Self {
        self.conflicted = conflicted;
        self.removed_targets = removed_targets;
        self
    }

    /// Returns whether this is a local bookmark with one target.
    #[must_use]
    pub const fn can_mutate(&self) -> bool {
        self.remote.is_none() && !self.conflicted && self.targets.len() == 1
    }

    /// Returns the one target when inspection is unambiguous.
    #[must_use]
    pub fn normal_target(&self) -> Option<&str> {
        (!self.conflicted && self.targets.len() == 1).then(|| self.targets[0].as_str())
    }
}

/// The effect requested after applying bookmark input.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BookmarkActionResult {
    /// Keep the current view.
    Continue,
    /// Refresh from jj.
    Refresh,
    /// Open the selected target commit.
    OpenTarget {
        /// Commit ID to inspect.
        commit_id: String,
    },
    /// Start bookmark creation.
    Create,
    /// Start moving the selected bookmark.
    Move {
        /// Selected bookmark name.
        name: String,
    },
    /// Start deleting the selected bookmark.
    Delete {
        /// Selected bookmark name.
        name: String,
    },
    /// Return to the parent view.
    ReturnBack,
    /// Exit the application.
    Quit,
}

/// Input actions understood by the bookmark view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BookmarkAction {
    /// Keep the current view without changing state.
    Continue,
    /// Move selection up.
    Previous,
    /// Move selection down.
    Next,
    /// Move to first row.
    First,
    /// Move to last row.
    Last,
    /// Refresh the list.
    Refresh,
    /// Open the selected target.
    OpenTarget,
    /// Start create.
    Create,
    /// Start move.
    Move,
    /// Start delete.
    Delete,
    /// Toggle help.
    ToggleHelp,
    /// Return to parent.
    ReturnBack,
    /// Quit.
    Quit,
}

/// Interactive bookmark list.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BookmarkView {
    snapshot: BookmarkViewSnapshot,
    selected: Option<usize>,
    scroll_offset: usize,
    help_visible: bool,
    status_message: Option<String>,
}

impl BookmarkView {
    /// Creates a view with the first row selected.
    #[must_use]
    pub fn new(snapshot: BookmarkViewSnapshot) -> Self {
        let selected = (!snapshot.rows.is_empty()).then_some(0);
        Self {
            snapshot,
            selected,
            scroll_offset: 0,
            help_visible: false,
            status_message: None,
        }
    }

    /// Replaces rows while preserving the selected `(name, remote)` identity.
    pub fn refresh(&mut self, snapshot: BookmarkViewSnapshot) {
        let selected_identity = self
            .selected_row()
            .map(|row| (row.name.clone(), row.remote.clone()));
        let previous = self.selected;
        self.snapshot = snapshot;
        let preserved = selected_identity.as_ref().and_then(|(name, remote)| {
            self.snapshot
                .rows
                .iter()
                .position(|row| &row.name == name && &row.remote == remote)
        });
        self.selected =
            preserved.or_else(|| clamp_index(previous.or(Some(0)), self.snapshot.rows.len()));
        self.scroll_offset = clamp_scroll(self.scroll_offset, self.snapshot.rows.len());
        self.status_message = if selected_identity.is_some() && preserved.is_none() {
            Some(
                "Selected bookmark disappeared; selection moved to the nearest remaining row."
                    .to_owned(),
            )
        } else {
            None
        };
    }

    /// Shows an error while keeping the last usable rows.
    pub fn show_error(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Shows a status message.
    pub fn show_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Returns the selected row.
    #[must_use]
    pub fn selected_row(&self) -> Option<&BookmarkRow> {
        self.selected
            .and_then(|index| self.snapshot.rows.get(index))
    }

    /// Returns whether help currently owns keyboard focus.
    #[must_use]
    pub const fn help_visible(&self) -> bool {
        self.help_visible
    }

    /// Applies one input action.
    #[must_use]
    pub fn apply(&mut self, action: BookmarkAction) -> BookmarkActionResult {
        if self.help_visible {
            if matches!(
                action,
                BookmarkAction::ToggleHelp | BookmarkAction::ReturnBack | BookmarkAction::Quit
            ) {
                self.help_visible = false;
            }
            return BookmarkActionResult::Continue;
        }
        match action {
            BookmarkAction::Continue => BookmarkActionResult::Continue,
            BookmarkAction::Previous => {
                if let Some(selected) = self.selected {
                    self.selected = Some(selected.saturating_sub(1));
                }
                BookmarkActionResult::Continue
            }
            BookmarkAction::Next => {
                if let Some(selected) = self.selected {
                    self.selected = Some(
                        selected
                            .saturating_add(1)
                            .min(self.snapshot.rows.len().saturating_sub(1)),
                    );
                }
                BookmarkActionResult::Continue
            }
            BookmarkAction::First => {
                self.selected = (!self.snapshot.rows.is_empty()).then_some(0);
                BookmarkActionResult::Continue
            }
            BookmarkAction::Last => {
                self.selected = self.snapshot.rows.len().checked_sub(1);
                BookmarkActionResult::Continue
            }
            BookmarkAction::Refresh => BookmarkActionResult::Refresh,
            BookmarkAction::OpenTarget => self
                .selected_row()
                .and_then(BookmarkRow::normal_target)
                .map(ToOwned::to_owned)
                .map_or_else(
                    || {
                        self.show_status("Selected bookmark has no single target to inspect.");
                        BookmarkActionResult::Continue
                    },
                    |commit_id| BookmarkActionResult::OpenTarget { commit_id },
                ),
            BookmarkAction::Create => BookmarkActionResult::Create,
            BookmarkAction::Move => match self.selected_row() {
                Some(row) if row.can_mutate() => BookmarkActionResult::Move {
                    name: row.name.clone(),
                },
                Some(_) => {
                    self.show_status("Only a local bookmark with one target can move.");
                    BookmarkActionResult::Continue
                }
                None => BookmarkActionResult::Continue,
            },
            BookmarkAction::Delete => match self.selected_row() {
                Some(row) if row.can_mutate() => BookmarkActionResult::Delete {
                    name: row.name.clone(),
                },
                Some(_) => {
                    self.show_status("Only a local bookmark with one target can delete.");
                    BookmarkActionResult::Continue
                }
                None => BookmarkActionResult::Continue,
            },
            BookmarkAction::ToggleHelp => {
                self.help_visible = !self.help_visible;
                BookmarkActionResult::Continue
            }
            BookmarkAction::ReturnBack => BookmarkActionResult::ReturnBack,
            BookmarkAction::Quit if self.help_visible => {
                self.help_visible = false;
                BookmarkActionResult::Continue
            }
            BookmarkAction::Quit => BookmarkActionResult::Quit,
        }
    }

    /// Renders the bookmark list.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let areas = ViewChrome::layout(area);
        let (gutter, content) = interaction_areas(areas.content);
        self.keep_selected_in_view(usize::from(areas.content.height));
        let fallback_status = if area.width >= 90 {
            "Enter inspect  c create  m move  x delete  F fetch  P dry-run  ? help  Esc back"
        } else {
            "Enter inspect  c create  F fetch  ? help  Esc back"
        };
        let status = self.status_message.as_deref().unwrap_or(fallback_status);
        ViewChrome::new(self.snapshot.title(), status).render(frame, areas);
        let body = self.visible_text();
        frame.render_widget(Paragraph::new(body), content);
        if let Some(selected) = self.selected {
            paint_cursor(
                frame,
                gutter,
                selected,
                self.scroll_offset,
                !self.help_visible,
            );
        }
        if self.help_visible {
            render_help_overlay(
                frame,
                areas.content,
                "Bookmarks keys",
                &[
                    "j/k or arrows  move selection".to_owned(),
                    "Enter          inspect selected target".to_owned(),
                    "c              create bookmark preview".to_owned(),
                    "m              move bookmark preview".to_owned(),
                    "x              delete bookmark preview".to_owned(),
                    "F              choose remote and preview fetch".to_owned(),
                    "P              choose remote and preview push dry-run".to_owned(),
                    "r              refresh".to_owned(),
                    "Esc / ? / q    close help".to_owned(),
                ],
            );
        }
    }

    fn keep_selected_in_view(&mut self, height: usize) {
        self.scroll_offset = clamp_scroll(self.scroll_offset, self.snapshot.rows.len());
        let Some(selected) = self.selected else {
            return;
        };
        if height == 0 {
            return;
        }
        if selected < self.scroll_offset {
            self.scroll_offset = selected;
        } else if selected >= self.scroll_offset.saturating_add(height) {
            self.scroll_offset = selected.saturating_add(1).saturating_sub(height);
        }
    }

    fn visible_text(&self) -> Text<'_> {
        if self.snapshot.rows.is_empty() {
            return Text::from(vec![
                Line::from("No bookmarks found."),
                Line::from(""),
                Line::from("Press c to create a bookmark, F to fetch, or r to refresh."),
            ]);
        }
        Text::from(
            self.snapshot
                .rows
                .iter()
                .skip(self.scroll_offset)
                .map(bookmark_line)
                .collect::<Vec<_>>(),
        )
    }
}

fn bookmark_line(row: &BookmarkRow) -> Line<'static> {
    let name = row.remote.as_ref().map_or_else(
        || row.name.clone(),
        |remote| format!("{}@{remote}", row.name),
    );
    let state = match (row.remote.is_some(), row.tracked) {
        (false, _) => "local",
        (true, false) => "untracked",
        (true, true) if row.synchronized => "tracked, synced",
        (true, true) => "tracked, differs from local",
    };
    let target = if row.conflicted {
        format!(
            "conflict (+{}/-{})",
            row.targets.len(),
            row.removed_targets.len()
        )
    } else {
        match row.targets.as_slice() {
            [] => "(deleted)".to_owned(),
            [target] => target.chars().take(12).collect(),
            targets => format!("conflict ({} targets)", targets.len()),
        }
    };
    Line::from(format!("{name}: {target} ({state})"))
}

fn clamp_index(index: Option<usize>, len: usize) -> Option<usize> {
    let index = index?;
    (len != 0).then_some(index.min(len - 1))
}

fn clamp_scroll(scroll_offset: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        scroll_offset.min(len - 1)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn row(name: &str, target: &str) -> BookmarkRow {
        BookmarkRow::new(name, vec![target.to_owned()])
    }

    #[test]
    fn refresh_preserves_selected_name_and_remote() {
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(vec![
            row("one", "a"),
            row("two", "b").with_remote("origin"),
        ]));
        let _ = view.apply(BookmarkAction::Next);
        view.refresh(BookmarkViewSnapshot::new(vec![
            row("new", "c"),
            row("two", "b").with_remote("origin"),
        ]));
        assert_eq!(
            view.selected_row().map(|row| row.name.as_str()),
            Some("two")
        );
    }

    #[test]
    fn refresh_reports_when_selected_identity_disappears() {
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(vec![row("old", "a")]));
        view.refresh(BookmarkViewSnapshot::new(vec![row("new", "b")]));
        assert_eq!(
            view.selected_row().map(|row| row.name.as_str()),
            Some("new")
        );
        assert!(
            view.status_message
                .as_deref()
                .is_some_and(|message| message.contains("disappeared"))
        );
    }

    #[test]
    fn help_owns_selection_and_mutation_keys_until_closed() {
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(vec![
            row("one", "a"),
            row("two", "b"),
        ]));
        let _ = view.apply(BookmarkAction::ToggleHelp);
        for action in [
            BookmarkAction::Next,
            BookmarkAction::Create,
            BookmarkAction::Move,
            BookmarkAction::Delete,
            BookmarkAction::OpenTarget,
            BookmarkAction::Refresh,
        ] {
            assert_eq!(view.apply(action), BookmarkActionResult::Continue);
        }
        assert_eq!(
            view.selected_row().map(|row| row.name.as_str()),
            Some("one")
        );
        assert_eq!(
            view.apply(BookmarkAction::ReturnBack),
            BookmarkActionResult::Continue
        );
        assert!(!view.help_visible());
    }

    #[test]
    fn remote_rows_explain_tracking_and_sync_state() {
        let synced = row("topic", "a")
            .with_remote("origin")
            .with_tracked(true)
            .with_synchronized(true);
        let deleted_local = row("topic", "a").with_remote("origin").with_tracked(true);
        let untracked = row("topic", "a").with_remote("other");
        assert!(
            bookmark_line(&synced)
                .to_string()
                .contains("topic@origin: a (tracked, synced)")
        );
        assert!(
            bookmark_line(&deleted_local)
                .to_string()
                .contains("tracked, differs from local")
        );
        assert!(
            bookmark_line(&untracked)
                .to_string()
                .contains("(untracked)")
        );
    }

    #[test]
    fn remote_and_conflicted_rows_cannot_mutate() {
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(vec![
            row("remote", "a").with_remote("origin"),
            BookmarkRow::new("conflicted", vec!["a".to_owned(), "b".to_owned()]),
        ]));
        assert_eq!(
            view.apply(BookmarkAction::Move),
            BookmarkActionResult::Continue
        );
        let _ = view.apply(BookmarkAction::Next);
        assert_eq!(
            view.apply(BookmarkAction::Delete),
            BookmarkActionResult::Continue
        );
    }

    #[test]
    fn modify_delete_conflict_with_one_added_target_is_not_actionable() {
        let row = row("topic", "new").with_conflict(true, vec!["old".into()]);
        assert!(!row.can_mutate());
        assert_eq!(row.normal_target(), None);
        assert!(bookmark_line(&row).to_string().contains("conflict (+1/-1)"));
    }

    #[test]
    fn empty_view_renders_without_panicking() {
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(Vec::new()));
        let backend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal.draw(|frame| view.render(frame)).expect("render");
        let text = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(text.contains("No bookmarks found."));
    }

    #[test]
    fn selection_scrolls_into_a_short_viewport() {
        let rows = (0..5)
            .map(|index| row(&format!("bookmark-{index}"), "target"))
            .collect();
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(rows));
        let _ = view.apply(BookmarkAction::Last);
        let backend = TestBackend::new(80, 4);
        let mut terminal = Terminal::new(backend).expect("test terminal");

        terminal.draw(|frame| view.render(frame)).expect("render");

        let buffer = terminal.backend().buffer();
        let text = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(text.contains("bookmark-4"));
        assert!(!text.contains("bookmark-0"));
        assert_eq!(buffer[(0, 2)].symbol(), "›");
    }

    #[test]
    fn narrow_short_viewport_keeps_selected_row_visible() {
        let rows = (0..5)
            .map(|index| row(&index.to_string(), "target"))
            .collect();
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(rows));
        let _ = view.apply(BookmarkAction::Last);
        let backend = TestBackend::new(16, 3);
        let mut terminal = Terminal::new(backend).expect("test terminal");

        terminal.draw(|frame| view.render(frame)).expect("render");

        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 1)].symbol(), "›");
        assert_eq!(buffer[(3, 1)].symbol(), "4");
    }

    #[test]
    fn rendered_rows_distinguish_deleted_conflicted_and_tracked_bookmarks() {
        let mut view = BookmarkView::new(BookmarkViewSnapshot::new(vec![
            BookmarkRow::new("deleted", Vec::new()),
            BookmarkRow::new("conflicted", vec!["one".to_owned(), "two".to_owned()]),
            row("tracked", "target").with_tracked(true),
        ]));
        let backend = TestBackend::new(80, 8);
        let mut terminal = Terminal::new(backend).expect("test terminal");

        terminal.draw(|frame| view.render(frame)).expect("render");

        let text = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(text.contains("(deleted)"));
        assert!(text.contains("conflict (+2/-0)"));
        assert!(text.contains("tracked"));
    }
}
