//! State machine for semantic log navigation.
//!
//! `LogState` owns the relationship between semantic entries and rendered lines. It deliberately
//! keeps rendering concerns out, but it knows enough about line positions to preserve selection,
//! keep the selected change in view, and choose where inline details should be inserted.

use jk_core::{LogEntry, LogSnapshot};

use crate::ansi_text::strip_ansi;
use crate::chrome::title_or_default;

const REVSET_ID_PREFIX_LEN: usize = 8;

/// Ordered revision marks keyed by stable change id.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrderedRevisionMarks {
    change_ids: Vec<String>,
}

impl OrderedRevisionMarks {
    fn toggle(&mut self, change_id: &str) {
        if let Some(index) = self.index_for(change_id) {
            self.change_ids.remove(index);
        } else {
            self.change_ids.push(change_id.to_owned());
        }
    }

    fn clear(&mut self) -> bool {
        let had_marks = !self.change_ids.is_empty();
        self.change_ids.clear();
        had_marks
    }

    fn retain_visible(&mut self, entries: &[LogEntry]) {
        self.change_ids
            .retain(|change_id| entries.iter().any(|entry| entry.change_id() == change_id));
    }

    fn index_for(&self, change_id: &str) -> Option<usize> {
        self.change_ids
            .iter()
            .position(|marked| marked == change_id)
    }

    fn change_ids(&self) -> &[String] {
        &self.change_ids
    }
}

/// Semantic state behind the interactive log view.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LogState {
    title: String,
    rendered: String,
    entries: Vec<LogEntry>,
    elisions: Vec<LogElision>,
    selected: Option<LogSelection>,
    expanded_change_id: Option<String>,
    scroll_offset: usize,
    viewport_height: usize,
    follow_selection: bool,
    marks: OrderedRevisionMarks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LogSelection {
    Entry(usize),
    Elision(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LogElision {
    rendered_line: usize,
    before_entry: Option<usize>,
    after_entry: Option<usize>,
}

impl LogState {
    /// Creates state from a freshly loaded log snapshot.
    pub fn new(snapshot: LogSnapshot) -> Self {
        let (title, rendered, entries) = snapshot.into_parts();
        let elisions = log_elisions(&rendered, &entries);
        let selected = first_selection(&entries, &elisions);
        Self {
            title: title_or_default(title),
            rendered,
            entries,
            elisions,
            selected,
            expanded_change_id: None,
            scroll_offset: 0,
            viewport_height: 10,
            follow_selection: true,
            marks: OrderedRevisionMarks::default(),
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
        self.marks.retain_visible(&self.entries);
        self.elisions = log_elisions(&self.rendered, &self.entries);
        self.selected = selected_change_id
            .and_then(|change_id| {
                self.entries
                    .iter()
                    .position(|entry| entry.change_id() == change_id)
                    .map(LogSelection::Entry)
            })
            .or_else(|| first_selection(&self.entries, &self.elisions));

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
        let LogSelection::Entry(index) = self.selected? else {
            return None;
        };
        self.entries.get(index)
    }

    /// Returns the selected entry's revision identifier for follow-up commands.
    #[must_use]
    pub fn selected_revision_id(&self) -> Option<&str> {
        self.selected_entry()
            .map(|entry| revision_id_prefix(entry.change_id()))
    }

    /// Returns the visible change before the selected graph elision.
    #[must_use]
    pub fn selected_elision_before_change_id(&self) -> Option<&str> {
        let LogSelection::Elision(index) = self.selected? else {
            return None;
        };
        let elision = self.elisions.get(index)?;
        let before = elision.before_entry?;
        self.entries.get(before).map(LogEntry::change_id)
    }

    /// Selects the first entry rendered after the given visible change.
    #[must_use]
    pub fn select_first_entry_after_change_id(&mut self, change_id: &str) -> bool {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.change_id() == change_id)
        else {
            return false;
        };
        let Some(next_index) = index.checked_add(1) else {
            return false;
        };
        if next_index >= self.entries.len() {
            return false;
        }
        self.selected = Some(LogSelection::Entry(next_index));
        self.follow_selection = true;
        self.expanded_change_id = None;
        self.keep_selected_visible();
        true
    }

    /// Returns the revset that should reveal the selected graph elision.
    #[must_use]
    pub fn selected_elision_revset(&self) -> Option<String> {
        let LogSelection::Elision(index) = self.selected? else {
            return None;
        };
        let elision = self.elisions.get(index)?;
        let reveal_revset = match (elision.before_entry, elision.after_entry) {
            (Some(before), Some(after)) => {
                let before = self.entries.get(before)?;
                let after = self.entries.get(after)?;
                Some(format!(
                    "{}::{}",
                    revision_id_prefix(after.commit_id()),
                    revision_id_prefix(before.commit_id())
                ))
            }
            (Some(before), None) => {
                let before = self.entries.get(before)?;
                Some(format!("::{}", revision_id_prefix(before.commit_id())))
            }
            (None, Some(after)) => {
                let after = self.entries.get(after)?;
                Some(format!("{}::", revision_id_prefix(after.commit_id())))
            }
            (None, None) => None,
        }?;

        let visible_entries = self
            .entries
            .iter()
            .map(|entry| revision_id_prefix(entry.commit_id()))
            .collect::<Vec<_>>()
            .join(" | ");
        if visible_entries.is_empty() {
            Some(reveal_revset)
        } else {
            Some(format!("({reveal_revset}) | {visible_entries}"))
        }
    }

    /// Returns the first rendered line currently visible in the viewport.
    pub const fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scrolls one rendered line toward newer changes without changing selection.
    pub const fn scroll_previous_line(&mut self) {
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

        let Some(index) = self.selectable_index(selected) else {
            return;
        };
        self.select_selectable_index(index.saturating_sub(1));
    }

    /// Moves selection to the next semantic entry.
    pub fn select_next(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };

        let Some(index) = self.selectable_index(selected) else {
            return;
        };
        let last_index = self.selectable_len().saturating_sub(1);
        self.select_selectable_index((index + 1).min(last_index));
    }

    /// Moves selection one viewport toward newer visible rendered lines.
    pub fn select_page_previous(&mut self) {
        let Some(selected_line) = self.selected_rendered_line() else {
            return;
        };

        let target_line = selected_line.saturating_sub(self.viewport_height);
        let target_index = self
            .selectables()
            .iter()
            .rposition(|selection| self.selection_rendered_line(*selection) <= target_line)
            .unwrap_or(0);
        self.select_selectable_index(target_index);
    }

    /// Moves selection one viewport toward older visible rendered lines.
    pub fn select_page_next(&mut self) {
        let Some(selected_line) = self.selected_rendered_line() else {
            return;
        };

        let target_line = selected_line.saturating_add(self.viewport_height);
        let selectables = self.selectables();
        let last_index = selectables.len().saturating_sub(1);
        let target_index = selectables
            .iter()
            .position(|selection| self.selection_rendered_line(*selection) >= target_line)
            .unwrap_or(last_index);
        self.select_selectable_index(target_index);
    }

    /// Moves selection to the first semantic entry.
    pub fn select_first(&mut self) {
        if self.selectable_len() > 0 {
            self.select_selectable_index(0);
        }
    }

    /// Moves selection to the last semantic entry.
    pub fn select_last(&mut self) {
        let selectable_len = self.selectable_len();
        if selectable_len > 0 {
            self.select_selectable_index(selectable_len - 1);
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

    /// Toggles the selected change in the ordered revision marks.
    pub fn toggle_selected_mark(&mut self) {
        let Some(change_id) = self
            .selected_entry()
            .map(|entry| entry.change_id().to_owned())
        else {
            return;
        };

        self.marks.toggle(&change_id);
    }

    /// Clears all revision marks and returns whether anything changed.
    pub fn clear_marks(&mut self) -> bool {
        self.marks.clear()
    }

    /// Returns whether any revision marks are set.
    pub fn has_marks(&self) -> bool {
        !self.marks.change_ids().is_empty()
    }

    /// Returns marked change ids in insertion order.
    pub fn marked_change_ids(&self) -> &[String] {
        self.marks.change_ids()
    }

    /// Returns marked revision identifiers shortened for follow-up commands.
    pub fn marked_revision_ids(&self) -> Vec<String> {
        self.marks
            .change_ids()
            .iter()
            .map(|change_id| revision_id_prefix(change_id).to_owned())
            .collect()
    }

    /// Returns the selected change's zero-based mark index, if marked.
    pub fn selected_mark_index(&self) -> Option<usize> {
        self.selected_entry()
            .and_then(|entry| self.mark_index_for_change_id(entry.change_id()))
    }

    /// Returns the zero-based mark index for a change id.
    pub fn mark_index_for_change_id(&self, change_id: &str) -> Option<usize> {
        self.marks.index_for(change_id)
    }

    /// Returns the rendered line for a visible change id.
    pub fn rendered_line_for_change_id(&self, change_id: &str) -> Option<usize> {
        self.entries
            .iter()
            .find(|entry| entry.change_id() == change_id)
            .map(LogEntry::rendered_line)
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
        let LogSelection::Entry(selected) = self.selected? else {
            return None;
        };
        if !self.selected_is_expanded() {
            return None;
        }

        self.entry_content_end_line(selected)
    }

    /// Returns the selected entry's rendered line.
    pub fn selected_rendered_line(&self) -> Option<usize> {
        self.selected
            .map(|selection| self.selection_rendered_line(selection))
    }

    /// Returns the final rendered line that belongs to the selected entry.
    fn selected_entry_end_line(&self) -> Option<usize> {
        let LogSelection::Entry(selected) = self.selected? else {
            return None;
        };
        self.entry_content_end_line(selected)
    }

    /// Returns the last rendered line that is content for the entry.
    fn entry_content_end_line(&self, entry_index: usize) -> Option<usize> {
        let start = self.entries.get(entry_index)?.rendered_line();
        let end = self
            .entries
            .get(entry_index + 1)
            .map_or_else(|| self.rendered.lines().count(), LogEntry::rendered_line);
        content_end_line(&self.rendered, start, end)
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
    fn select_selectable_index(&mut self, index: usize) {
        let Some(selection) = self.selectables().get(index).copied() else {
            return;
        };

        let was_expanded = self.selected_is_expanded();
        self.selected = Some(selection);
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

    fn selectables(&self) -> Vec<LogSelection> {
        let mut selections = Vec::with_capacity(self.entries.len() + self.elisions.len());
        selections.extend((0..self.entries.len()).map(LogSelection::Entry));
        selections.extend((0..self.elisions.len()).map(LogSelection::Elision));
        selections.sort_by_key(|selection| self.selection_rendered_line(*selection));
        selections
    }

    const fn selectable_len(&self) -> usize {
        self.entries.len() + self.elisions.len()
    }

    fn selectable_index(&self, selected: LogSelection) -> Option<usize> {
        self.selectables()
            .iter()
            .position(|selection| *selection == selected)
    }

    fn selection_rendered_line(&self, selection: LogSelection) -> usize {
        match selection {
            LogSelection::Entry(index) => self.entries[index].rendered_line(),
            LogSelection::Elision(index) => self.elisions[index].rendered_line,
        }
    }
}

fn revision_id_prefix(id: &str) -> &str {
    id.char_indices()
        .nth(REVSET_ID_PREFIX_LEN)
        .map_or(id, |(index, _)| &id[..index])
}

/// Finds the final rendered line that belongs to a visible log entry.
fn content_end_line(rendered: &str, start: usize, end: usize) -> Option<usize> {
    rendered
        .lines()
        .enumerate()
        .skip(start)
        .take(end.saturating_sub(start))
        .filter(|(_, line)| !is_separator_line(line))
        .map(|(index, _)| index)
        .last()
}

/// Returns whether a rendered line is a separator after an entry instead of entry content.
fn is_separator_line(line: &str) -> bool {
    strip_ansi(line).trim().is_empty()
        || is_graph_elision_line(line)
        || is_graph_connector_line(line)
}

fn first_selection(entries: &[LogEntry], elisions: &[LogElision]) -> Option<LogSelection> {
    let first_entry = entries
        .first()
        .map(|entry| (entry.rendered_line(), LogSelection::Entry(0)));
    let first_elision = elisions
        .first()
        .map(|elision| (elision.rendered_line, LogSelection::Elision(0)));

    [first_entry, first_elision]
        .into_iter()
        .flatten()
        .min_by_key(|(line, _)| *line)
        .map(|(_, selection)| selection)
}

fn log_elisions(rendered: &str, entries: &[LogEntry]) -> Vec<LogElision> {
    let rendered_lines = rendered.lines().collect::<Vec<_>>();
    let entry_columns = entries
        .iter()
        .map(|entry| {
            rendered_lines
                .get(entry.rendered_line())
                .and_then(|line| graph_node_column(line))
        })
        .collect::<Vec<_>>();

    rendered
        .lines()
        .enumerate()
        .filter(|(_, line)| is_graph_elision_line(line))
        .map(|(rendered_line, line)| {
            let elision_column = graph_elision_column(line);
            let same_column = |entry_index: usize| {
                elision_column.is_none() || entry_columns.get(entry_index) == Some(&elision_column)
            };
            let before_entry = entries
                .iter()
                .enumerate()
                .rposition(|(index, entry)| {
                    entry.rendered_line() < rendered_line && same_column(index)
                })
                .or_else(|| {
                    entries
                        .iter()
                        .rposition(|entry| entry.rendered_line() < rendered_line)
                });
            let after_entry = entries
                .iter()
                .enumerate()
                .position(|(index, entry)| {
                    entry.rendered_line() > rendered_line && same_column(index)
                })
                .or_else(|| {
                    entries
                        .iter()
                        .position(|entry| entry.rendered_line() > rendered_line)
                });
            LogElision {
                rendered_line,
                before_entry,
                after_entry,
            }
        })
        .collect()
}

/// Returns whether a rendered line is jj's hidden-revision graph elision.
fn is_graph_elision_line(line: &str) -> bool {
    graph_elision_column(line).is_some()
}

/// Returns the zero-based graph column for jj's hidden-revision elision marker.
fn graph_elision_column(line: &str) -> Option<usize> {
    strip_ansi(line)
        .chars()
        .enumerate()
        .find_map(|(index, character)| (!is_graph_prefix(character)).then_some((index, character)))
        .and_then(|(index, character)| (character == '~').then_some(index))
}

/// Returns the zero-based graph column for a rendered commit node.
fn graph_node_column(line: &str) -> Option<usize> {
    strip_ansi(line)
        .chars()
        .enumerate()
        .find_map(|(index, character)| is_graph_node(character).then_some(index))
}

/// Returns whether a rendered line is a graph connector tail after an entry.
fn is_graph_connector_line(line: &str) -> bool {
    let line = strip_ansi(line);
    let mut seen_graph = false;
    let mut has_connector = false;

    for character in line.trim_end().chars() {
        if is_graph_prefix(character) {
            seen_graph = true;
            has_connector |= is_graph_connector(character);
            continue;
        }

        break;
    }

    seen_graph && has_connector
}

/// Returns whether a character can appear before jj's elision marker in a graph line.
const fn is_graph_prefix(character: char) -> bool {
    matches!(
        character,
        ' ' | '│' | '─' | '├' | '╭' | '╮' | '╯' | '╰' | '╲' | '╱'
    )
}

/// Returns whether a graph character marks a rendered commit row.
const fn is_graph_node(character: char) -> bool {
    matches!(character, '@' | '○' | '◆' | '×' | '◇')
}

/// Returns whether a graph-prefix character joins or closes lanes.
const fn is_graph_connector(character: char) -> bool {
    matches!(character, '─' | '├' | '╭' | '╮' | '╯' | '╰' | '╲' | '╱')
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
    fn selected_revision_id_uses_short_change_id() {
        let state = LogState::new(snapshot(["abcdefghijklmnop"]));

        assert_eq!(state.selected_revision_id(), Some("abcdefgh"));
        assert_eq!(
            state.selected_entry().map(LogEntry::change_id),
            Some("abcdefghijklmnop")
        );
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
    fn toggle_selected_mark_adds_change_id_in_order() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc"]));

        state.toggle_selected_mark();
        state.select_next();
        state.toggle_selected_mark();

        assert_eq!(state.marked_change_ids(), ["aaa", "bbb"]);
        assert_eq!(state.selected_mark_index(), Some(1));
    }

    #[test]
    fn toggle_selected_mark_removes_existing_mark_and_closes_gap() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc"]));

        state.toggle_selected_mark();
        state.select_next();
        state.toggle_selected_mark();
        state.select_previous();
        state.toggle_selected_mark();

        assert_eq!(state.marked_change_ids(), ["bbb"]);
        assert_eq!(state.mark_index_for_change_id("bbb"), Some(0));
    }

    #[test]
    fn toggle_selected_mark_does_not_duplicate_change_id() {
        let mut state = LogState::new(snapshot(["aaa", "bbb"]));

        state.toggle_selected_mark();
        state.toggle_selected_mark();
        state.toggle_selected_mark();

        assert_eq!(state.marked_change_ids(), ["aaa"]);
    }

    #[test]
    fn clear_marks_empties_marks_and_reports_change() {
        let mut state = LogState::new(snapshot(["aaa", "bbb"]));

        assert!(!state.clear_marks());
        state.toggle_selected_mark();
        assert!(state.has_marks());

        assert!(state.clear_marks());

        assert!(!state.has_marks());
        assert!(state.marked_change_ids().is_empty());
    }

    #[test]
    fn refresh_preserves_marks_when_change_ids_still_exist() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc"]));
        state.toggle_selected_mark();
        state.select_next();
        state.select_next();
        state.toggle_selected_mark();

        state.refresh(snapshot(["ccc", "xxx", "aaa"]));

        assert_eq!(state.marked_change_ids(), ["aaa", "ccc"]);
        assert_eq!(state.mark_index_for_change_id("aaa"), Some(0));
        assert_eq!(state.mark_index_for_change_id("ccc"), Some(1));
    }

    #[test]
    fn marked_revision_ids_use_short_change_ids() {
        let mut state = LogState::new(snapshot([
            "abcdefghijklmnop",
            "bbbbbbbbcccccccc",
            "zyxwvutsrqponmlk",
        ]));

        state.toggle_selected_mark();
        state.select_next();
        state.select_next();
        state.toggle_selected_mark();

        assert_eq!(
            state.marked_change_ids(),
            ["abcdefghijklmnop", "zyxwvutsrqponmlk"]
        );
        assert_eq!(state.marked_revision_ids(), ["abcdefgh", "zyxwvuts"]);
    }

    #[test]
    fn refresh_drops_marks_for_disappeared_changes() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc"]));
        state.toggle_selected_mark();
        state.select_next();
        state.toggle_selected_mark();
        state.select_next();
        state.toggle_selected_mark();

        state.refresh(snapshot(["ccc", "aaa"]));

        assert_eq!(state.marked_change_ids(), ["aaa", "ccc"]);
    }

    #[test]
    fn line_scroll_and_mark_toggle_do_not_change_selection_or_scroll() {
        let mut state = LogState::new(snapshot(["aaa", "bbb", "ccc", "ddd"]));
        state.select_next();
        state.scroll_next_line();
        let selected = state
            .selected_entry()
            .map(|entry| entry.change_id().to_owned());
        let scroll_offset = state.scroll_offset();

        state.toggle_selected_mark();

        assert_eq!(
            state
                .selected_entry()
                .map(|entry| entry.change_id().to_owned()),
            selected
        );
        assert_eq!(state.scroll_offset(), scroll_offset);
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
        assert_eq!(state.selected, Some(LogSelection::Entry(0)));

        state.select_next();
        state.select_next();
        assert_eq!(state.selected, Some(LogSelection::Entry(1)));
    }

    #[test]
    fn movement_can_select_graph_elision() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n~  (elided revisions)\n○  bbb second\n",
            vec![
                LogEntry::new("aaa", "111", "first").with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
            ],
        ));

        state.select_next();

        assert_eq!(state.selected, Some(LogSelection::Elision(0)));
        assert_eq!(state.selected_entry(), None);
        assert_eq!(state.selected_rendered_line(), Some(1));
        assert_eq!(
            state.selected_elision_revset(),
            Some("(222::111) | 111 | 222".to_owned())
        );
    }

    #[test]
    fn graph_elision_revset_includes_visible_boundaries() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  current\n~  (elided revisions)\n◆  root\n",
            vec![
                LogEntry::new("current", "111", "current").with_rendered_line(0),
                LogEntry::new("root", "000", "root").with_rendered_line(2),
            ],
        ));

        state.select_next();

        assert_eq!(
            state.selected_elision_revset(),
            Some("(000::111) | 111 | 000".to_owned())
        );
    }

    #[test]
    fn graph_elision_revset_uses_short_commit_ids() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  abcdefghijklmnop current\n~  (elided revisions)\n◆  zyxwvutsrqponmlk root\n",
            vec![
                LogEntry::new("abcdefghijklmnop", "1111222233334444", "current")
                    .with_rendered_line(0),
                LogEntry::new("zyxwvutsrqponmlk", "0000111122223333", "root").with_rendered_line(2),
            ],
        ));

        state.select_next();

        assert_eq!(
            state.selected_elision_revset(),
            Some("(00001111::11112222) | 11112222 | 00001111".to_owned())
        );
    }

    #[test]
    fn graph_elision_revset_avoids_divergent_change_ids() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  nvuwzuzn current\n~  (elided revisions)\n◆  nvuwzuzn root\n",
            vec![
                LogEntry::new("nvuwzuzn", "aaaabbbbccccdddd", "current").with_rendered_line(0),
                LogEntry::new("nvuwzuzn", "1111222233334444", "root").with_rendered_line(2),
            ],
        ));

        state.select_next();

        assert_eq!(
            state.selected_elision_revset(),
            Some("(11112222::aaaabbbb) | aaaabbbb | 11112222".to_owned())
        );
    }

    #[test]
    fn graph_elision_boundaries_follow_the_elision_column() {
        let mut state = LogState::new(LogSnapshot::new(
            "◆  main\n~  (elided revisions)\n│ ○  side\n╭─┤  merge edge\n◆ │  base\n",
            vec![
                LogEntry::new("main", "mmmmmmmm", "main").with_rendered_line(0),
                LogEntry::new("side", "ssssssss", "side").with_rendered_line(2),
                LogEntry::new("base", "bbbbbbbb", "base").with_rendered_line(4),
            ],
        ));

        state.select_next();

        assert_eq!(
            state.selected_elision_revset(),
            Some("(bbbbbbbb::mmmmmmmm) | mmmmmmmm | ssssssss | bbbbbbbb".to_owned())
        );
    }

    #[test]
    fn revision_id_prefix_keeps_short_ids() {
        assert_eq!(revision_id_prefix("abc123"), "abc123");
    }

    #[test]
    fn trailing_graph_elision_drills_into_ancestors_with_visible_context() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  aaa first\n~  (elided revisions)\n",
            vec![LogEntry::new("aaa", "111", "first").with_rendered_line(0)],
        ));

        state.select_next();

        assert_eq!(
            state.selected_elision_revset(),
            Some("(::111) | 111".to_owned())
        );
    }

    #[test]
    fn trailing_graph_elision_keeps_visible_stack_context() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  current\n○  parent\n◆  main\n~  (elided revisions)\n",
            vec![
                LogEntry::new("current", "333", "current").with_rendered_line(0),
                LogEntry::new("parent", "222", "parent").with_rendered_line(1),
                LogEntry::new("main", "111", "main").with_rendered_line(2),
            ],
        ));

        state.select_next();
        state.select_next();
        state.select_next();

        assert_eq!(
            state.selected_elision_revset(),
            Some("(::111) | 333 | 222 | 111".to_owned())
        );
    }

    #[test]
    fn can_select_first_revealed_entry_after_elision_boundary() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  current\n○  parent\n◆  main\n○  hidden\n◆  root\n",
            vec![
                LogEntry::new("current", "333", "current").with_rendered_line(0),
                LogEntry::new("parent", "222", "parent").with_rendered_line(1),
                LogEntry::new("main", "111", "main").with_rendered_line(2),
                LogEntry::new("hidden", "999", "hidden").with_rendered_line(3),
                LogEntry::new("root", "000", "root").with_rendered_line(4),
            ],
        ));

        assert!(state.select_first_entry_after_change_id("main"));

        assert_eq!(
            state.selected_entry().map(LogEntry::change_id),
            Some("hidden")
        );
        assert_eq!(state.selected_rendered_line(), Some(3));
    }

    #[test]
    fn selecting_after_final_entry_reports_no_target() {
        let mut state = LogState::new(LogSnapshot::new(
            "@  current\n",
            vec![LogEntry::new("current", "333", "current").with_rendered_line(0)],
        ));

        assert!(!state.select_first_entry_after_change_id("current"));

        assert_eq!(
            state.selected_entry().map(LogEntry::change_id),
            Some("current")
        );
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
            "@  aaa\n│  first\n\u{1b}[38;5;8m~  (elided revisions)\u{1b}[0m\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
            ],
        ));

        state.toggle_expanded();

        assert_eq!(state.expanded_insertion_line(), Some(1));
    }

    #[test]
    fn expanded_insertion_line_keeps_elision_separator_after_expansion() {
        let mut state = LogState::new(LogSnapshot::new(
            "◆  aaa\n│  first\n~\n\n○  bbb\n│  second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(4),
            ],
        ));

        state.toggle_expanded();

        assert_eq!(state.expanded_insertion_line(), Some(1));
    }

    #[test]
    fn expanded_insertion_line_keeps_parallel_elision_separator_after_expansion() {
        let mut state = LogState::new(LogSnapshot::new(
            "│ ◆  aaa\n│ │  first\n│ ~  (elided revisions)\n│ │ ○  bbb\n│ ├─╯  second\n│ ◆  ccc\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(3),
                LogEntry::new("ccc", "333", "third").with_rendered_line(5),
            ],
        ));

        state.toggle_expanded();

        assert_eq!(state.expanded_insertion_line(), Some(1));
    }

    #[test]
    fn expanded_insertion_line_keeps_connector_separator_after_expansion() {
        let mut state = LogState::new(LogSnapshot::new(
            "│ ◆  aaa\n├─╯  first\n│ ○  bbb\n│ │  second\n",
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody")
                    .with_details("body")
                    .with_rendered_line(0),
                LogEntry::new("bbb", "222", "second").with_rendered_line(2),
            ],
        ));

        state.toggle_expanded();

        assert_eq!(state.expanded_insertion_line(), Some(0));
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
