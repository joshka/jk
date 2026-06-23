//! State machine for semantic log navigation.
//!
//! `LogState` owns the relationship between semantic entries and rendered lines. It deliberately
//! keeps rendering concerns out, but it knows enough about line positions to preserve selection,
//! keep the selected change in view, and choose where inline details should be inserted.

use jk_core::{LogEntry, LogSnapshot};

use crate::ansi_text::strip_ansi;
use crate::chrome::title_or_default;

/// Semantic state behind the interactive log view.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LogState {
    title: String,
    rendered: String,
    entries: Vec<LogEntry>,
    selected: Option<usize>,
    expanded_change_id: Option<String>,
    scroll_offset: usize,
    viewport_height: usize,
    follow_selection: bool,
}

impl LogState {
    /// Creates state from a freshly loaded log snapshot.
    pub fn new(snapshot: LogSnapshot) -> Self {
        let (title, rendered, entries) = snapshot.into_parts();
        let selected = (!entries.is_empty()).then_some(0);
        Self {
            title: title_or_default(title),
            rendered,
            entries,
            selected,
            expanded_change_id: None,
            scroll_offset: 0,
            viewport_height: 10,
            follow_selection: true,
        }
    }

    /// Replaces the log snapshot while preserving selection when possible.
    pub fn refresh(&mut self, snapshot: LogSnapshot) {
        let selected_change_id = self
            .selected_entry()
            .map(|entry| entry.change_id().to_owned());
        let (title, rendered, entries) = snapshot.into_parts();

        self.title = title_or_default(title);
        self.rendered = rendered;
        self.entries = entries;
        self.selected = selected_change_id
            .and_then(|change_id| {
                self.entries
                    .iter()
                    .position(|entry| entry.change_id() == change_id)
            })
            .or_else(|| (!self.entries.is_empty()).then_some(0));

        if let Some(expanded_change_id) = &self.expanded_change_id {
            let expanded_still_exists_with_details = self
                .entries
                .iter()
                .any(|entry| entry.change_id() == expanded_change_id && entry_has_details(entry));
            if !expanded_still_exists_with_details {
                self.expanded_change_id = None;
            }
        }

        self.clamp_scroll_offset();
    }

    /// Returns the command context shown in the title bar.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the opaque rendered `jj` output.
    pub fn rendered(&self) -> &str {
        &self.rendered
    }

    /// Returns the currently selected semantic entry.
    pub fn selected_entry(&self) -> Option<&LogEntry> {
        self.selected.and_then(|index| self.entries.get(index))
    }

    /// Returns the first rendered line currently visible in the viewport.
    pub const fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scrolls one rendered line toward newer changes without changing selection.
    pub fn scroll_previous_line(&mut self) {
        self.follow_selection = false;
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scrolls one rendered line toward older changes without changing selection.
    pub fn scroll_next_line(&mut self) {
        self.follow_selection = false;
        self.scroll_offset = self.scroll_offset.saturating_add(1);
        self.clamp_scroll_offset();
    }

    /// Returns whether the selected change currently owns inline details.
    pub fn selected_is_expanded(&self) -> bool {
        let Some(entry) = self.selected_entry() else {
            return false;
        };
        self.expanded_change_id.as_deref() == Some(entry.change_id())
    }

    /// Moves selection to the previous semantic entry.
    pub fn select_previous(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };

        self.select_index(selected.saturating_sub(1));
    }

    /// Moves selection to the next semantic entry.
    pub fn select_next(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };

        let last_index = self.entries.len().saturating_sub(1);
        self.select_index((selected + 1).min(last_index));
    }

    /// Moves selection one viewport toward newer visible rendered lines.
    pub fn select_page_previous(&mut self) {
        let Some(selected_line) = self.selected_rendered_line() else {
            return;
        };

        let target_line = selected_line.saturating_sub(self.viewport_height);
        let target_index = self
            .entries
            .iter()
            .rposition(|entry| entry.rendered_line() <= target_line)
            .unwrap_or(0);
        self.select_index(target_index);
    }

    /// Moves selection one viewport toward older visible rendered lines.
    pub fn select_page_next(&mut self) {
        let Some(selected_line) = self.selected_rendered_line() else {
            return;
        };

        let target_line = selected_line.saturating_add(self.viewport_height);
        let last_index = self.entries.len().saturating_sub(1);
        let target_index = self
            .entries
            .iter()
            .position(|entry| entry.rendered_line() >= target_line)
            .unwrap_or(last_index);
        self.select_index(target_index);
    }

    /// Moves selection to the first semantic entry.
    pub fn select_first(&mut self) {
        if !self.entries.is_empty() {
            self.select_index(0);
        }
    }

    /// Moves selection to the last semantic entry.
    pub fn select_last(&mut self) {
        if !self.entries.is_empty() {
            self.select_index(self.entries.len() - 1);
        }
    }

    /// Toggles inline details for the selected entry.
    pub fn toggle_expanded(&mut self) {
        let Some(entry) = self.selected_entry() else {
            return;
        };

        if self.selected_is_expanded() {
            self.expanded_change_id = None;
        } else if entry_has_details(entry) {
            self.expanded_change_id = Some(entry.change_id().to_owned());
        } else {
            self.expanded_change_id = None;
        }
    }

    /// Collapses any inline details.
    pub fn collapse_expanded(&mut self) {
        self.expanded_change_id = None;
    }

    /// Updates viewport height and scrolls enough to keep selection visible.
    pub fn keep_selected_in_view(&mut self, height: usize) {
        self.viewport_height = height.max(1);
        if !self.follow_selection {
            self.clamp_scroll_offset();
            return;
        }

        let Some(selected_line) = self.selected_rendered_line() else {
            self.scroll_offset = 0;
            return;
        };

        if height == 0 || selected_line < self.scroll_offset {
            self.scroll_offset = selected_line;
            return;
        }

        let last_visible = self.scroll_offset + height.saturating_sub(1);
        let selected_end_line = self.selected_entry_end_line().unwrap_or(selected_line);
        if selected_end_line > last_visible {
            let scroll_for_entry_end = selected_end_line + 1 - height;
            self.scroll_offset = scroll_for_entry_end.min(selected_line);
            self.clamp_scroll_offset();
        }
    }

    /// Returns non-empty inline details for the expanded entry.
    pub fn expanded_details(&self) -> Option<&str> {
        let Some(expanded_change_id) = &self.expanded_change_id else {
            return None;
        };

        self.entries
            .iter()
            .find(|entry| entry.change_id() == expanded_change_id)
            .and_then(|entry| entry_has_details(entry).then_some(entry.details()))
    }

    /// Returns the rendered line after which inline details should be inserted.
    pub fn expanded_insertion_line(&self) -> Option<usize> {
        let selected = self.selected?;
        if !self.selected_is_expanded() {
            return None;
        }

        self.entries
            .get(selected + 1)
            .map(|entry| entry.rendered_line().saturating_sub(1))
            .or_else(|| last_entry_line(&self.rendered))
    }

    /// Returns the selected entry's rendered line.
    pub fn selected_rendered_line(&self) -> Option<usize> {
        self.selected_entry().map(LogEntry::rendered_line)
    }

    /// Returns the final rendered line that belongs to the selected entry.
    fn selected_entry_end_line(&self) -> Option<usize> {
        let selected = self.selected?;
        self.entries
            .get(selected + 1)
            .map(|entry| entry.rendered_line().saturating_sub(1))
            .or_else(|| last_entry_line(&self.rendered))
    }

    /// Scrolls upward when selection moves above the current viewport.
    fn keep_selected_visible(&mut self) {
        let Some(selected_line) = self.selected_rendered_line() else {
            self.scroll_offset = 0;
            return;
        };

        if selected_line < self.scroll_offset {
            self.scroll_offset = selected_line;
        }
    }

    /// Selects an entry and carries expansion forward when movement expects it.
    fn select_index(&mut self, index: usize) {
        let Some(_) = self.entries.get(index) else {
            return;
        };

        let was_expanded = self.selected_is_expanded();
        self.selected = Some(index);
        self.follow_selection = true;
        self.apply_expansion_to_selected(was_expanded);
        self.keep_selected_visible();
    }

    /// Applies the previous expansion mode to the currently selected entry.
    fn apply_expansion_to_selected(&mut self, expanded: bool) {
        self.expanded_change_id = if expanded {
            self.selected_entry()
                .map(|entry| entry.change_id().to_owned())
        } else {
            None
        };
    }

    /// Keeps scroll offset within lines that can still contain selected entries.
    fn clamp_scroll_offset(&mut self) {
        if self.entries.is_empty() {
            self.scroll_offset = 0;
            return;
        }

        let max_scroll_offset = self.rendered.lines().count().saturating_sub(1);
        self.scroll_offset = self.scroll_offset.min(max_scroll_offset);
    }
}

/// Finds the final rendered line that belongs to a visible log entry.
fn last_entry_line(rendered: &str) -> Option<usize> {
    let mut last_entry_line = None;
    for (index, line) in rendered.lines().enumerate() {
        if !is_graph_elision_line(line) {
            last_entry_line = Some(index);
        }
    }
    last_entry_line
}

/// Returns whether a rendered line is jj's hidden-revision graph elision.
fn is_graph_elision_line(line: &str) -> bool {
    strip_ansi(line).trim() == "~"
}

/// Returns whether an entry has detail text worth expanding inline.
fn entry_has_details(entry: &LogEntry) -> bool {
    !entry.details().trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_keeps_selected_change_when_still_visible() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc"]));
        state.select_next();

        state.refresh(snapshot(["xxx", "bbb", "yyy"]));

        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("bbb"));
    }

    #[test]
    fn refresh_selects_first_change_when_selected_change_disappears() {
        let mut state = LogState::new(snapshot(["aaa", "bbb"]));
        state.select_next();

        state.refresh(snapshot(["ccc", "ddd"]));

        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("ccc"));
    }

    #[test]
    fn refresh_preserves_scroll_offset_when_still_in_bounds() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc", "ddd"]));
        state.select_next();
        state.select_next();
        state.keep_selected_in_view(2);

        state.refresh(snapshot(["aaa", "bbb", "ccc", "ddd"]));

        assert_eq!(state.scroll_offset(), 1);
    }

    #[test]
    fn refresh_preserves_expansion_when_selected_change_still_has_details() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second")
                    .with_details("old body")
                    .with_rendered_line(1),
            ],
        ));
        state.select_next();
        state.toggle_expanded();

        state.refresh(LogSnapshot::new(
            "@  xxx new\n○  bbb second\n◆  yyy newer\n",
            vec![
                LogEntry::new("xxx", "333", "new").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second")
                    .with_details("new body")
                    .with_rendered_line(1),
                LogEntry::new("yyy", "444", "newer").with_rendered_line(2),
            ],
        ));

        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("bbb"));
        assert!(state.selected_is_expanded());
        assert_eq!(state.expanded_details(), Some("new body"));
    }

    #[test]
    fn refresh_collapses_expansion_when_expanded_change_disappears() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second")
                    .with_details("body")
                    .with_rendered_line(1),
            ],
        ));
        state.select_next();
        state.toggle_expanded();

        state.refresh(snapshot(["ccc", "ddd"]));

        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("ccc"));
        assert!(!state.selected_is_expanded());
        assert_eq!(state.expanded_details(), None);
        assert_eq!(state.expanded_insertion_line(), None);
    }

    #[test]
    fn toggle_expanded_ignores_entries_without_details() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second")
                    .with_details("body")
                    .with_rendered_line(1),
            ],
        ));

        state.toggle_expanded();
        state.select_next();

        assert!(!state.selected_is_expanded());
        assert_eq!(state.expanded_details(), None);
    }

    #[test]
    fn scroll_offset_uses_rendered_lines_not_entry_indexes() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n│  first body\n○  bbb second\n│  second body\n◆  ccc third\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
                LogEntry::new("ccc", "333", "third").with_rendered_line(4),
            ],
        ));

        state.select_next();
        state.select_next();
        state.keep_selected_in_view(2);

        assert_eq!(state.scroll_offset(), 3);
    }

    #[test]
    fn selected_multiline_entry_keeps_whole_message_visible_when_it_fits() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n│  first body\n○  bbb second\n│  second body\n│  more body\n◆  ccc third\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
                LogEntry::new("ccc", "333", "third").with_rendered_line(5),
            ],
        ));

        state.select_next();
        state.keep_selected_in_view(3);

        assert_eq!(state.scroll_offset(), 2);
    }

    #[test]
    fn selected_tall_entry_keeps_commit_row_and_body_visible() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n│  first body\n○  bbb second\n│  line one\n│  line two\n│  line three\n│  line four\n◆  ccc third\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
                LogEntry::new("ccc", "333", "third").with_rendered_line(7),
            ],
        ));

        state.select_next();
        state.keep_selected_in_view(3);

        assert_eq!(state.scroll_offset(), 2);
    }

    #[test]
    fn line_scroll_moves_view_without_changing_selection() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n│  first body\n○  bbb second\n│  second body\n◆  ccc third\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
                LogEntry::new("ccc", "333", "third").with_rendered_line(4),
            ],
        ));

        state.scroll_next_line();
        state.scroll_next_line();
        state.scroll_previous_line();
        state.keep_selected_in_view(2);

        assert_eq!(state.scroll_offset(), 1);
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("aaa"));
    }

    #[test]
    fn selection_movement_resumes_following_selection_after_line_scroll() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n│  first body\n○  bbb second\n│  second body\n◆  ccc third\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
                LogEntry::new("ccc", "333", "third").with_rendered_line(4),
            ],
        ));

        state.scroll_next_line();
        state.scroll_next_line();
        state.keep_selected_in_view(2);
        state.select_next();
        state.keep_selected_in_view(2);

        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("bbb"));
        assert_eq!(state.scroll_offset(), 2);
    }

    #[test]
    fn movement_stays_inside_log_bounds() {
        let mut state = LogState::new(snapshot(["aaa", "bbb"]));

        state.select_previous();
        assert_eq!(state.selected, Some(0));

        state.select_next();
        state.select_next();
        assert_eq!(state.selected, Some(1));
    }

    #[test]
    fn page_movement_uses_viewport_height_and_rendered_lines() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n│  first body\n○  bbb second\n│  second body\n◆  ccc third\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
                LogEntry::new("ccc", "333", "third").with_rendered_line(4),
            ],
        ));

        state.keep_selected_in_view(2);
        state.select_page_next();
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("bbb"));

        state.select_page_next();
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("ccc"));

        state.select_page_previous();
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("bbb"));
    }

    #[test]
    fn page_movement_stays_inside_log_bounds() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc"]));

        state.select_page_previous();
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("aaa"));

        state.select_page_next();
        state.select_page_next();
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("ccc"));
    }

    #[test]
    fn edge_movement_selects_first_and_last_entries() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc"]));

        state.select_last();
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("ccc"));

        state.select_first();
        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("aaa"));
    }

    #[test]
    fn selected_item_toggles_inline_details() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa summary\n",
            vec![LogEntry::new("aaa", "111", "summary\n\nbody").with_details("body")],
        ));

        state.toggle_expanded();
        assert!(state.selected_is_expanded());

        state.toggle_expanded();
        assert!(!state.selected_is_expanded());
    }

    #[test]
    fn movement_applies_inline_details_to_new_selection() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody").with_details("body"),
                LogEntry::new("bbb", "222", "second\n\nsecond body").with_details("second body"),
            ],
        ));

        state.toggle_expanded();
        state.select_next();

        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("bbb"));
        assert!(state.selected_is_expanded());
        assert_eq!(state.expanded_details(), Some("second body"));
    }

    #[test]
    fn page_movement_applies_inline_details_to_new_selection() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second\n\nsecond body")
                    .with_details("second body")
                    .with_rendered_line(1),
            ],
        ));

        state.keep_selected_in_view(1);
        state.toggle_expanded();
        state.select_page_next();

        assert_eq!(state.selected_entry().map(LogEntry::change_id), Some("bbb"));
        assert!(state.selected_is_expanded());
        assert_eq!(state.expanded_details(), Some("second body"));
    }

    #[test]
    fn movement_keeps_collapsed_rows_collapsed() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody").with_details("body"),
                LogEntry::new("bbb", "222", "second\n\nsecond body").with_details("second body"),
            ],
        ));

        state.select_next();

        assert!(!state.selected_is_expanded());
    }

    #[test]
    fn expanded_details_come_from_log_entry_details() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n",
            vec![LogEntry::new("aaa", "111", "first\n\nbody\nmore").with_details("body\nmore")],
        ));

        state.toggle_expanded();

        assert_eq!(state.expanded_details(), Some("body\nmore"));
    }

    #[test]
    fn expanded_insertion_line_uses_end_of_rendered_entry() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa\n│  first\n○  bbb\n│  second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
            ],
        ));

        state.toggle_expanded();

        assert_eq!(state.expanded_insertion_line(), Some(1));
    }

    #[test]
    fn expanded_insertion_line_skips_trailing_graph_elision() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa\n│  first\n\u{1b}[38;5;8m~\u{1b}[0m\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
            ],
        ));

        state.toggle_expanded();

        assert_eq!(state.expanded_insertion_line(), Some(1));
    }

    fn snapshot<const N: usize>(change_ids: [&str; N]) -> LogSnapshot {
        let entries = change_ids
            .into_iter()
            .enumerate()
            .map(|(index, change_id)| {
                LogEntry::new(change_id, "commit", format!("{change_id} summary"))
                    .with_rendered_line(index)
            })
            .collect::<Vec<_>>();
        let mut rendered = String::new();
        for entry in &entries {
            rendered.push_str("○  ");
            rendered.push_str(entry.summary());
            rendered.push('\n');
        }
        LogSnapshot::new(rendered, entries)
    }
}
