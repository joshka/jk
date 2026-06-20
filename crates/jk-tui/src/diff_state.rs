//! State machine for selected-change diff inspection.

use std::collections::BTreeSet;

use jk_core::{DiffFileStat, DiffSnapshot};

use crate::ansi_text::strip_ansi;
use crate::chrome::title_or_default;

/// Semantic state behind a rendered selected-change diff.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiffState {
    title: String,
    change_id: String,
    rendered: String,
    sections: Vec<FileSection>,
    selected: Option<usize>,
    collapsed_paths: BTreeSet<String>,
    scroll_offset: usize,
    viewport_height: usize,
}

impl DiffState {
    /// Creates state from a freshly loaded diff snapshot.
    pub fn new(snapshot: DiffSnapshot) -> Self {
        let (title, change_id, rendered, file_stats) = snapshot.into_parts();
        let sections = file_sections(&rendered, &file_stats);
        let selected = (!sections.is_empty()).then_some(0);
        Self {
            title: title_or_default(title),
            change_id,
            rendered,
            sections,
            selected,
            collapsed_paths: BTreeSet::new(),
            scroll_offset: 0,
            viewport_height: 10,
        }
    }

    /// Replaces the diff output while preserving selected and collapsed file paths when possible.
    pub fn refresh(&mut self, snapshot: DiffSnapshot) {
        let selected_path = self.selected_section().map(|section| section.path.clone());
        let (title, change_id, rendered, file_stats) = snapshot.into_parts();

        self.title = title_or_default(title);
        self.change_id = change_id;
        self.rendered = rendered;
        self.sections = file_sections(&self.rendered, &file_stats);
        self.collapsed_paths
            .retain(|path| self.sections.iter().any(|section| section.path == *path));
        self.selected = selected_path
            .and_then(|path| {
                self.sections
                    .iter()
                    .position(|section| section.path == path)
            })
            .or_else(|| (!self.sections.is_empty()).then_some(0));

        self.clamp_scroll_offset();
    }

    /// Returns the command context shown in the title bar.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the target change identifier for refresh requests.
    pub fn change_id(&self) -> &str {
        &self.change_id
    }

    /// Returns the first rendered line currently visible in the viewport.
    pub const fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scrolls one visible line toward the start of the diff.
    pub fn scroll_previous_line(&mut self) {
        let old_offset = self.scroll_offset;
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
        if self.scroll_offset == old_offset {
            self.select_previous_file();
        } else {
            self.select_file_for_scroll_offset();
        }
    }

    /// Scrolls one visible line toward the end of the diff.
    pub fn scroll_next_line(&mut self) {
        let old_offset = self.scroll_offset;
        self.scroll_offset = self.scroll_offset.saturating_add(1);
        self.clamp_scroll_offset();
        if self.scroll_offset == old_offset {
            self.select_next_file();
        } else {
            self.select_file_for_scroll_offset();
        }
    }

    /// Scrolls one viewport toward the start of the diff.
    pub fn select_page_previous(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
        self.select_file_for_scroll_offset();
    }

    /// Scrolls one viewport toward the end of the diff.
    pub fn select_page_next(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(self.viewport_height);
        self.clamp_scroll_offset();
        self.select_file_for_scroll_offset();
    }

    /// Moves to the first visible diff line.
    pub fn select_first(&mut self) {
        self.scroll_offset = 0;
        self.select_file_for_scroll_offset();
    }

    /// Moves to the last visible diff page.
    pub fn select_last(&mut self) {
        self.scroll_offset = usize::MAX;
        self.clamp_scroll_offset();
        self.select_file_for_scroll_offset();
    }

    /// Jumps to the previous file section.
    pub fn select_previous_file(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };

        self.select_index(selected.saturating_sub(1));
    }

    /// Jumps to the next file section.
    pub fn select_next_file(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };

        let last_index = self.sections.len().saturating_sub(1);
        self.select_index((selected + 1).min(last_index));
    }

    /// Folds the selected file section.
    pub fn fold_selected_file(&mut self) {
        let Some(path) = self.selected_section().map(|section| section.path.clone()) else {
            return;
        };

        self.collapsed_paths.insert(path);
        self.clamp_scroll_offset();
        self.keep_selected_visible();
    }

    /// Unfolds the selected file section.
    pub fn unfold_selected_file(&mut self) {
        let Some(path) = self.selected_section().map(|section| section.path.clone()) else {
            return;
        };

        self.collapsed_paths.remove(&path);
        self.clamp_scroll_offset();
        self.keep_selected_visible();
    }

    /// Folds every file section.
    pub fn fold_all_files(&mut self) {
        self.collapsed_paths = self
            .sections
            .iter()
            .map(|section| section.path.clone())
            .collect();
        self.clamp_scroll_offset();
        self.keep_selected_visible();
    }

    /// Unfolds every file section.
    pub fn unfold_all_files(&mut self) {
        self.collapsed_paths.clear();
        self.clamp_scroll_offset();
        self.keep_selected_visible();
    }

    /// Updates viewport height without changing the user's current reading position.
    pub fn keep_selected_in_view(&mut self, height: usize) {
        self.viewport_height = height.max(1);
        self.clamp_scroll_offset();
    }

    /// Returns the visible diff body after applying collapsed file sections.
    pub fn visible_rendered(&self) -> String {
        if self.sections.is_empty() || self.collapsed_paths.is_empty() {
            return self.rendered.clone();
        }

        let lines = self.rendered.split_inclusive('\n').collect::<Vec<_>>();
        let folded_header_width = self.folded_header_width(&lines);
        let mut visible = String::new();
        let mut line_index = 0;

        for section in &self.sections {
            while line_index < section.start_line {
                if let Some(line) = lines.get(line_index) {
                    visible.push_str(line);
                }
                line_index += 1;
            }

            if !self.collapsed_paths.contains(&section.path) {
                while line_index < section.end_line {
                    if let Some(line) = lines.get(line_index) {
                        visible.push_str(line);
                    }
                    line_index += 1;
                }
                continue;
            }

            if let Some(header) = lines.get(section.start_line) {
                let header = header.trim_end_matches('\n');
                visible.push_str(header);
                visible
                    .push_str(&section.folded_suffix(folded_header_width, visible_width(header)));
                visible.push('\n');
            }
            line_index = section.end_line;
        }

        while let Some(line) = lines.get(line_index) {
            visible.push_str(line);
            line_index += 1;
        }

        visible
    }

    /// Returns the visible line for the selected file header after collapse is applied.
    pub fn selected_visible_line(&self) -> Option<usize> {
        let selected = self.selected_section()?;
        Some(self.visible_line_for_rendered_line(selected.start_line))
    }

    /// Returns the selected file header when it should be pinned above the scrolled diff body.
    pub fn sticky_header(&self) -> Option<String> {
        let selected_line = self.selected_visible_line()?;
        if selected_line >= self.scroll_offset {
            return None;
        }

        self.visible_rendered()
            .lines()
            .nth(selected_line)
            .map(ToOwned::to_owned)
    }

    /// Returns whether the selected file section is collapsed.
    #[cfg(test)]
    pub fn selected_file_is_collapsed(&self) -> bool {
        self.selected_section()
            .is_some_and(|section| self.collapsed_paths.contains(&section.path))
    }

    /// Returns the selected file section.
    fn selected_section(&self) -> Option<&FileSection> {
        self.selected.and_then(|index| self.sections.get(index))
    }

    /// Selects a file section by index.
    fn select_index(&mut self, index: usize) {
        let Some(section) = self.sections.get(index) else {
            return;
        };

        self.selected = Some(index);
        self.scroll_offset = self.visible_line_for_rendered_line(section.start_line);
        self.clamp_scroll_offset();
    }

    /// Selects the file section containing, or nearest before, the current scroll offset.
    fn select_file_for_scroll_offset(&mut self) {
        if self.sections.is_empty() {
            self.selected = None;
            return;
        }

        self.selected = self
            .sections
            .iter()
            .enumerate()
            .take_while(|(_, section)| {
                self.visible_line_for_rendered_line(section.start_line) <= self.scroll_offset
            })
            .map(|(index, _)| index)
            .last()
            .or(Some(0));
    }

    /// Scrolls upward or downward when selected content needs to remain visible after mutation.
    fn keep_selected_visible(&mut self) {
        let Some(selected_line) = self.selected_visible_line() else {
            self.clamp_scroll_offset();
            return;
        };

        if selected_line < self.scroll_offset {
            self.scroll_offset = selected_line;
            return;
        }

        let last_visible = self.scroll_offset + self.viewport_height.saturating_sub(1);
        if selected_line > last_visible {
            self.scroll_offset = selected_line + 1 - self.viewport_height;
        }
    }

    /// Keeps scroll offset within the currently visible diff body.
    fn clamp_scroll_offset(&mut self) {
        let max_scroll_offset = self
            .visible_rendered()
            .lines()
            .count()
            .saturating_sub(self.viewport_height);
        self.scroll_offset = self.scroll_offset.min(max_scroll_offset);
    }

    /// Maps an original rendered line number to its line number after collapsed sections are
    /// hidden.
    fn visible_line_for_rendered_line(&self, rendered_line: usize) -> usize {
        let mut hidden_lines = 0;
        for section in &self.sections {
            if section.start_line >= rendered_line {
                break;
            }
            if self.collapsed_paths.contains(&section.path) {
                hidden_lines += section.end_line.saturating_sub(section.start_line + 1);
            }
        }

        rendered_line.saturating_sub(hidden_lines)
    }

    /// Returns the widest visible file header in the rendered diff.
    fn folded_header_width(&self, lines: &[&str]) -> usize {
        self.sections
            .iter()
            .filter_map(|section| lines.get(section.start_line))
            .map(|header| visible_width(header.trim_end_matches('\n')))
            .max()
            .unwrap_or_default()
    }
}

/// A file section discovered in `jj diff` output.
#[derive(Clone, Debug, Eq, PartialEq)]
struct FileSection {
    path: String,
    start_line: usize,
    end_line: usize,
    stat: Option<FileStat>,
}

impl FileSection {
    /// Returns the folded suffix appended to this file section's header.
    fn folded_suffix(&self, target_width: usize, header_width: usize) -> String {
        let padding = " ".repeat(target_width.saturating_sub(header_width) + 1);
        match &self.stat {
            Some(stat) if !stat.rendered.is_empty() => {
                let mut suffix = padding;
                suffix.push_str(&stat.rendered);
                suffix
            }
            Some(stat) => format!(
                "{padding}| {:>3} \u{1b}[38;5;2m{}\u{1b}[38;5;1m{}\u{1b}[39m",
                stat.added + stat.removed,
                "+".repeat(stat.added.min(10)),
                "-".repeat(stat.removed.min(10)),
            ),
            None => format!("{padding}| folded"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FileStat {
    added: usize,
    removed: usize,
    rendered: String,
}

/// Finds rendered file sections from the narrow header shape emitted by `jj diff`.
fn file_sections(rendered: &str, file_stats: &[DiffFileStat]) -> Vec<FileSection> {
    let headers = rendered
        .lines()
        .enumerate()
        .filter_map(|(line, text)| file_header_path(text).map(|path| (line, path)))
        .collect::<Vec<_>>();
    let mut sections = Vec::with_capacity(headers.len());

    for (index, (start_line, path)) in headers.iter().enumerate() {
        let end_line = headers
            .get(index + 1)
            .map_or_else(|| rendered.lines().count(), |(line, _)| *line);
        sections.push(FileSection {
            path: path.clone(),
            start_line: *start_line,
            end_line,
            stat: file_stats
                .iter()
                .find(|stat| stat.path() == path)
                .map(|stat| FileStat {
                    added: stat.added(),
                    removed: stat.removed(),
                    rendered: stat.rendered().to_owned(),
                }),
        });
    }

    sections
}

/// Extracts a stable file path from a visible `jj diff` file header line.
fn file_header_path(line: &str) -> Option<String> {
    let line = strip_ansi(line);
    let line = line.trim();
    let path = [
        "Modified regular file ",
        "Added regular file ",
        "Removed regular file ",
        "Renamed regular file ",
        "Copied regular file ",
        "Modified executable file ",
        "Added executable file ",
        "Removed executable file ",
        "Modified symlink ",
        "Added symlink ",
        "Removed symlink ",
    ]
    .into_iter()
    .find_map(|prefix| line.strip_prefix(prefix))?;

    path.strip_suffix(':').map(ToOwned::to_owned)
}

/// Returns the terminal-visible width of text after removing ANSI color escapes.
fn visible_width(text: &str) -> usize {
    strip_ansi(text).chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_collapse_survives_refresh_by_path() {
        let mut state = DiffState::new(snapshot_with_stats(
            "aaa",
            "Modified regular file src/a.rs:\n old\n new\nModified regular file src/b.rs:\n b\n",
            vec![
                DiffFileStat::new("src/a.rs", 1, 1),
                DiffFileStat::new("src/b.rs", 1, 0),
            ],
        ));
        state.fold_selected_file();

        state.refresh(snapshot_with_stats(
            "aaa",
            "Modified regular file src/a.rs:\n newer\nModified regular file src/c.rs:\n c\n",
            vec![
                DiffFileStat::new("src/a.rs", 1, 0),
                DiffFileStat::new("src/c.rs", 1, 0),
            ],
        ));

        assert!(state.selected_file_is_collapsed());
        assert!(state.visible_rendered().contains(
            "Modified regular file src/a.rs: |   1 \u{1b}[38;5;2m+\u{1b}[38;5;1m\u{1b}[39m"
        ));
        assert!(!state.visible_rendered().contains("newer"));
    }

    #[test]
    fn refresh_drops_collapsed_paths_that_disappear() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n old\nModified regular file src/b.rs:\n b\n",
        ));
        state.fold_selected_file();

        state.refresh(snapshot("aaa", "Modified regular file src/b.rs:\n b\n"));

        assert!(!state.selected_file_is_collapsed());
        assert!(!state.visible_rendered().contains(" | folded"));
    }

    #[test]
    fn bracket_movement_selects_file_sections() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a\nModified regular file src/b.rs:\n b\n",
        ));
        state.keep_selected_in_view(2);

        state.select_next_file();

        assert_eq!(state.scroll_offset(), 2);
        assert_eq!(state.selected_visible_line(), Some(2));
    }

    #[test]
    fn fold_all_and_unfold_all_update_every_section() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a\nModified regular file src/b.rs:\n b\n",
        ));

        state.fold_all_files();

        let rendered = state.visible_rendered();
        assert!(rendered.contains("Modified regular file src/a.rs:"));
        assert!(rendered.contains("Modified regular file src/b.rs:"));
        assert!(!rendered.contains("\n a\n"));
        assert!(!rendered.contains("\n b\n"));

        state.unfold_all_files();

        let rendered = state.visible_rendered();
        assert!(rendered.contains("\n a\n"));
        assert!(rendered.contains("\n b\n"));
    }

    #[test]
    fn folded_stat_suffix_aligns_after_full_diff_header() {
        let mut state = DiffState::new(snapshot_with_stats(
            "aaa",
            concat!(
                "Added regular file src/short.rs:\n",
                " a\n",
                "Modified regular file crates/jk-tui/src/diff_state.rs:\n",
                " b\n",
            ),
            vec![
                DiffFileStat::new("src/short.rs", 1, 0)
                    .with_rendered("| 1 \u{1b}[38;5;2m+\u{1b}[39m"),
                DiffFileStat::new("crates/jk-tui/src/diff_state.rs", 1, 0)
                    .with_rendered("| 1 \u{1b}[38;5;2m+\u{1b}[39m"),
            ],
        ));

        state.fold_all_files();

        let rendered = strip_ansi(&state.visible_rendered());
        let pipe_columns = rendered
            .lines()
            .filter_map(|line| line.find('|'))
            .collect::<Vec<_>>();
        assert_eq!(pipe_columns.len(), 2);
        assert_eq!(pipe_columns[0], pipe_columns[1]);
        assert!(pipe_columns[0] > "Added regular file src/short.rs:".len());
    }

    #[test]
    fn sticky_header_tracks_current_file_after_header_scrolls_offscreen() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a1\n a2\nModified regular file src/b.rs:\n b\n",
        ));
        state.keep_selected_in_view(2);

        state.scroll_next_line();

        assert_eq!(
            state.sticky_header(),
            Some("Modified regular file src/a.rs:".to_owned())
        );

        state.select_first();

        assert_eq!(state.sticky_header(), None);
    }

    #[test]
    fn unfold_selected_file_keeps_other_files_folded() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a\nModified regular file src/b.rs:\n b\n",
        ));
        state.fold_all_files();

        state.unfold_selected_file();

        let rendered = state.visible_rendered();
        assert!(rendered.contains("\n a\n"));
        assert!(!rendered.contains("\n b\n"));
    }

    #[test]
    fn line_scroll_moves_between_folded_files_when_content_fits() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a\nModified regular file src/b.rs:\n b\n",
        ));
        state.keep_selected_in_view(10);
        state.fold_all_files();

        state.scroll_next_line();

        assert_eq!(state.scroll_offset(), 0);
        assert_eq!(state.selected_visible_line(), Some(1));

        state.scroll_previous_line();

        assert_eq!(state.scroll_offset(), 0);
        assert_eq!(state.selected_visible_line(), Some(0));
    }

    #[test]
    fn selected_visible_line_accounts_for_collapsed_sections() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a1\n a2\nModified regular file src/b.rs:\n b\n",
        ));
        state.fold_selected_file();
        state.select_next_file();

        assert_eq!(state.selected_visible_line(), Some(1));
    }

    #[test]
    fn line_scroll_updates_current_file_after_header_reaches_top() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n a1\n a2\nModified regular file src/b.rs:\n b\n",
        ));
        state.keep_selected_in_view(2);

        state.scroll_next_line();
        state.scroll_next_line();
        state.scroll_next_line();

        assert_eq!(state.scroll_offset(), 3);
        assert_eq!(state.selected_visible_line(), Some(3));
    }

    fn snapshot(change_id: &str, rendered: &str) -> DiffSnapshot {
        DiffSnapshot::new(change_id, rendered).with_title(format!("jj diff -r {change_id}"))
    }

    fn snapshot_with_stats(
        change_id: &str,
        rendered: &str,
        stats: Vec<DiffFileStat>,
    ) -> DiffSnapshot {
        snapshot(change_id, rendered).with_file_stats(stats)
    }
}
