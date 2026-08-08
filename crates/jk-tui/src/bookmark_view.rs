//! Provider-neutral bookmark list view.

use ratatui::Frame;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::Paragraph;

use crate::chrome::{ViewChrome, render_help_overlay};
use crate::selected_row::paint_subtle_selected_row;

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
}

impl BookmarkRow {
    /// Creates a display row.
    #[must_use]
    pub fn new(name: impl Into<String>, targets: Vec<String>) -> Self {
        Self {
            name: name.into(),
            remote: None,
            targets,
            tracked: false,
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

    /// Returns whether this is a local bookmark with one target.
    #[must_use]
    pub const fn can_mutate(&self) -> bool {
        self.remote.is_none() && self.targets.len() == 1
    }

    /// Returns the one target when inspection is unambiguous.
    #[must_use]
    pub fn normal_target(&self) -> Option<&str> {
        (self.targets.len() == 1).then(|| self.targets[0].as_str())
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
        self.selected = selected_identity
            .and_then(|(name, remote)| {
                self.snapshot
                    .rows
                    .iter()
                    .position(|row| row.name == name && row.remote == remote)
            })
            .or_else(|| clamp_index(previous.or(Some(0)), self.snapshot.rows.len()));
        self.scroll_offset = clamp_scroll(self.scroll_offset, self.snapshot.rows.len());
        self.status_message = None;
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

    /// Applies one input action.
    #[must_use]
    pub fn apply(&mut self, action: BookmarkAction) -> BookmarkActionResult {
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
        self.keep_selected_in_view(usize::from(areas.content.height));
        let fallback_status = "c create  m move  x delete  F fetch  P push  r refresh  Esc back";
        let status = self.status_message.as_deref().unwrap_or(fallback_status);
        ViewChrome::new(self.snapshot.title(), status).render(frame, areas);
        let body = self.visible_text();
        frame.render_widget(Paragraph::new(body), areas.content);
        if let Some(selected) = self.selected {
            paint_subtle_selected_row(frame, areas.content, selected, self.scroll_offset);
        }
        if self.help_visible {
            render_help_overlay(
                frame,
                areas.content,
                "Bookmarks keys",
                &[
                    "j/k or arrows  move selection".to_owned(),
                    "enter          inspect selected target".to_owned(),
                    "c              create bookmark preview".to_owned(),
                    "m              move bookmark preview".to_owned(),
                    "x              delete bookmark preview".to_owned(),
                    "F              fetch selected remote (confirm)".to_owned(),
                    "P              push dry-run for selected remote".to_owned(),
                    "r              refresh    Esc back".to_owned(),
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
                Line::from(Span::styled(
                    "No bookmarks found.",
                    Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("Press r to refresh."),
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
    let marker = if row.remote.is_some() { "R" } else { "L" };
    let remote = row.remote.as_deref().unwrap_or("local");
    let tracking = if row.tracked { " tracked" } else { "" };
    let target = match row.targets.as_slice() {
        [] => "(deleted)".to_owned(),
        [target] => target.chars().take(12).collect(),
        targets => format!("conflict ({} targets)", targets.len()),
    };
    Line::from(format!(
        "{marker} {remote:<10} {name:<32} {target}{tracking}",
        name = row.name
    ))
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
        assert_eq!(
            buffer[(0, 2)].bg,
            Color::Rgb(34, 40, 44),
            "the selected final row remains highlighted in the viewport"
        );
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
        assert_eq!(buffer[(0, 1)].bg, Color::Rgb(34, 40, 44));
        assert_eq!(buffer[(13, 1)].symbol(), "4");
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
        assert!(text.contains("conflict (2 targets)"));
        assert!(text.contains("tracked"));
    }
}
