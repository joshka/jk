//! State machine for selected-change diff inspection.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use jk_core::{DiffFileStat, DiffSnapshot};

use crate::ansi_text::strip_ansi;
use crate::chrome::title_or_default;

const HORIZONTAL_SCROLL_STEP: usize = 8;
const DIFF_HORIZONTAL_STATUS: &str = "</> horizontal scroll";

/// Semantic state behind a rendered selected-change diff.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiffState {
    title: String,
    change_id: String,
    rendered: String,
    sections: Vec<FileSection>,
    hunks: Vec<HunkSection>,
    selected: Option<usize>,
    selected_hunk: Option<usize>,
    collapsed_paths: BTreeSet<String>,
    search: Option<SearchState>,
    scroll_offset: usize,
    horizontal_offset: usize,
    viewport_height: usize,
    viewport_width: usize,
}

impl DiffState {
    /// Creates state from a freshly loaded diff snapshot.
    pub fn new(snapshot: DiffSnapshot) -> Self {
        let (title, change_id, rendered, file_stats) = snapshot.into_parts();
        let sections = file_sections(&rendered, &file_stats);
        let hunks = hunk_sections(&rendered, &sections);
        let selected = (!sections.is_empty()).then_some(0);
        Self {
            title: title_or_default(title),
            change_id,
            rendered,
            sections,
            hunks,
            selected,
            selected_hunk: None,
            collapsed_paths: BTreeSet::new(),
            search: None,
            scroll_offset: 0,
            horizontal_offset: 0,
            viewport_height: 10,
            viewport_width: 80,
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
        self.hunks = hunk_sections(&self.rendered, &self.sections);
        self.selected_hunk = None;
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
        self.clamp_horizontal_offset();
        self.refresh_search_matches();
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

    /// Returns the first rendered column currently visible in the viewport.
    pub const fn horizontal_offset(&self) -> usize {
        self.horizontal_offset
    }

    /// Scrolls one visible line toward the start of the diff.
    pub fn scroll_previous_line(&mut self) {
        let old_offset = self.scroll_offset;
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
        if self.scroll_offset == old_offset {
            self.select_previous_file();
        } else {
            self.selected_hunk = None;
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
            self.selected_hunk = None;
            self.select_file_for_scroll_offset();
        }
    }

    /// Scrolls one viewport toward the start of the diff.
    pub fn select_page_previous(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
        self.selected_hunk = None;
        self.select_file_for_scroll_offset();
    }

    /// Scrolls one viewport toward the end of the diff.
    pub fn select_page_next(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(self.viewport_height);
        self.clamp_scroll_offset();
        self.selected_hunk = None;
        self.select_file_for_scroll_offset();
    }

    /// Moves to the first visible diff line.
    pub fn select_first(&mut self) {
        self.scroll_offset = 0;
        self.selected_hunk = None;
        self.select_file_for_scroll_offset();
    }

    /// Moves to the last visible diff page.
    pub fn select_last(&mut self) {
        self.scroll_offset = usize::MAX;
        self.clamp_scroll_offset();
        self.selected_hunk = None;
        self.select_file_for_scroll_offset();
    }

    /// Scrolls wide diff lines toward the start.
    pub const fn scroll_left(&mut self) {
        self.horizontal_offset = self
            .horizontal_offset
            .saturating_sub(HORIZONTAL_SCROLL_STEP);
    }

    /// Scrolls wide diff lines toward the end.
    pub fn scroll_right(&mut self) {
        self.horizontal_offset = self
            .horizontal_offset
            .saturating_add(HORIZONTAL_SCROLL_STEP);
        self.clamp_horizontal_offset();
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

    /// Jumps to the previous hunk.
    pub fn select_previous_hunk(&mut self) {
        let Some(index) = self.current_hunk_index() else {
            return;
        };

        self.select_hunk_index(index.saturating_sub(1));
    }

    /// Jumps to the next hunk.
    pub fn select_next_hunk(&mut self) {
        let Some(index) = self.current_hunk_index() else {
            return;
        };

        self.select_hunk_index((index + 1).min(self.hunks.len().saturating_sub(1)));
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

    /// Updates viewport width and clamps horizontal scrolling to visible content.
    pub fn set_viewport_width(&mut self, width: usize) {
        self.viewport_width = width.max(1);
        self.clamp_horizontal_offset();
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

        let selected_index = self.selected?;
        let selected = self.selected_section()?;
        let lines = self.rendered.split_inclusive('\n').collect::<Vec<_>>();
        let header = lines
            .get(selected.start_line)?
            .trim_end_matches('\n')
            .to_owned();
        let mut sticky_header = header.clone();
        if let Some(suffix) =
            selected.stat_suffix(self.folded_header_width(&lines), visible_width(&header))
        {
            sticky_header.push_str(&suffix);
        }
        let _ = write!(
            sticky_header,
            "  [file {}/{}]",
            selected_index + 1,
            self.sections.len()
        );
        Some(sticky_header)
    }

    /// Searches visible diff lines and moves to the first match at or below the viewport.
    pub fn search(&mut self, query: &str) {
        if query.is_empty() {
            self.search = None;
            return;
        }

        let matches = matching_visible_lines(&self.visible_rendered(), query);
        let selected = selected_match_at_or_after(&matches, self.scroll_offset);
        self.search = Some(SearchState {
            query: query.to_owned(),
            matches,
            selected,
        });
        self.scroll_to_selected_match();
    }

    /// Moves to the next search match, wrapping at the end of the diff.
    pub fn search_next(&mut self) {
        let Some(search) = &mut self.search else {
            return;
        };
        if search.matches.is_empty() {
            return;
        }

        let selected = search.selected.unwrap_or(0);
        search.selected = Some((selected + 1) % search.matches.len());
        self.scroll_to_selected_match();
    }

    /// Moves to the previous search match, wrapping at the beginning of the diff.
    pub fn search_previous(&mut self) {
        let Some(search) = &mut self.search else {
            return;
        };
        if search.matches.is_empty() {
            return;
        }

        let selected = search.selected.unwrap_or(0);
        search.selected = Some(selected.checked_sub(1).unwrap_or(search.matches.len() - 1));
        self.scroll_to_selected_match();
    }

    /// Returns status-line text for the current search, if any.
    pub fn search_status(&self) -> Option<String> {
        let search = self.search.as_ref()?;
        let Some(selected) = search.selected else {
            return Some(format!("/{}  no matches", search.query));
        };

        Some(format!(
            "/{}  {}/{}  n next  N previous",
            search.query,
            selected + 1,
            search.matches.len()
        ))
    }

    /// Returns status-line text for horizontal scroll state, if the view is shifted.
    pub fn horizontal_status(&self) -> Option<String> {
        (self.horizontal_offset > 0).then(|| {
            format!(
                "{DIFF_HORIZONTAL_STATUS}  col {}",
                self.horizontal_offset + 1
            )
        })
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
        self.selected_hunk = None;
        self.scroll_offset = self.visible_line_for_rendered_line(section.start_line);
        self.clamp_scroll_offset();
    }

    /// Selects a hunk by index and scrolls its header to the top.
    fn select_hunk_index(&mut self, index: usize) {
        let Some(hunk) = self.hunks.get(index) else {
            return;
        };

        self.selected = Some(hunk.file_index);
        self.selected_hunk = Some(index);
        self.scroll_offset = self.visible_line_for_rendered_line(hunk.start_line);
        self.clamp_scroll_offset();
    }

    /// Returns the hunk containing, or nearest before, the current scroll offset.
    fn current_hunk_index(&self) -> Option<usize> {
        if self.hunks.is_empty() {
            return None;
        }
        if let Some(selected_hunk) = self.selected_hunk {
            return Some(selected_hunk.min(self.hunks.len() - 1));
        }

        self.hunks
            .iter()
            .enumerate()
            .take_while(|(_, hunk)| {
                self.visible_line_for_rendered_line(hunk.start_line) <= self.scroll_offset
            })
            .map(|(index, _)| index)
            .last()
            .or(Some(0))
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

    /// Recomputes search matches after the rendered visible body changes.
    fn refresh_search_matches(&mut self) {
        let Some(query) = self.search.as_ref().map(|search| search.query.clone()) else {
            return;
        };

        let matches = matching_visible_lines(&self.visible_rendered(), &query);
        let selected = selected_match_at_or_after(&matches, self.scroll_offset);
        self.search = Some(SearchState {
            query,
            matches,
            selected,
        });
    }

    /// Scrolls to the selected search match and updates the current file from that line.
    fn scroll_to_selected_match(&mut self) {
        let Some(line) = self.search.as_ref().and_then(SearchState::selected_line) else {
            return;
        };

        self.scroll_offset = line;
        self.clamp_scroll_offset();
        self.selected_hunk = None;
        self.select_file_for_scroll_offset();
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

    /// Keeps horizontal offset within the widest visible line.
    fn clamp_horizontal_offset(&mut self) {
        let max_horizontal_offset = self
            .visible_rendered()
            .lines()
            .map(visible_width)
            .max()
            .unwrap_or_default()
            .saturating_sub(self.viewport_width);
        self.horizontal_offset = self.horizontal_offset.min(max_horizontal_offset);
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

/// State for the last submitted diff search.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SearchState {
    query: String,
    matches: Vec<usize>,
    selected: Option<usize>,
}

impl SearchState {
    /// Returns the visible line number of the selected search match.
    fn selected_line(&self) -> Option<usize> {
        self.selected
            .and_then(|index| self.matches.get(index).copied())
    }
}

/// A diff hunk header discovered in rendered `jj diff` output.
#[derive(Clone, Debug, Eq, PartialEq)]
struct HunkSection {
    file_index: usize,
    start_line: usize,
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
        self.stat_suffix(target_width, header_width)
            .unwrap_or_else(|| stat_padding(target_width, header_width) + "| folded")
    }

    /// Returns the stat suffix appended after a file section header.
    fn stat_suffix(&self, target_width: usize, header_width: usize) -> Option<String> {
        let padding = stat_padding(target_width, header_width);
        let stat = self.stat.as_ref()?;
        if !stat.rendered.is_empty() {
            let mut suffix = padding;
            suffix.push_str(&stat.rendered);
            return Some(suffix);
        }

        Some(format!(
            "{padding}| {:>3} \u{1b}[38;5;2m{}\u{1b}[38;5;1m{}\u{1b}[39m",
            stat.added + stat.removed,
            "+".repeat(stat.added.min(10)),
            "-".repeat(stat.removed.min(10)),
        ))
    }
}

/// Returns the spaces between a diff file header and its stat suffix.
fn stat_padding(target_width: usize, header_width: usize) -> String {
    " ".repeat(target_width.saturating_sub(header_width) + 1)
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

/// Finds diff hunk headers inside recognized file sections.
fn hunk_sections(rendered: &str, file_sections: &[FileSection]) -> Vec<HunkSection> {
    let lines = rendered.lines().collect::<Vec<_>>();
    let mut hunks = Vec::new();

    for (file_index, section) in file_sections.iter().enumerate() {
        for line_index in (section.start_line + 1)..section.end_line {
            let Some(line) = lines.get(line_index) else {
                continue;
            };
            if hunk_header_line(line) {
                hunks.push(HunkSection {
                    file_index,
                    start_line: line_index,
                });
            }
        }
    }

    hunks
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

/// Returns whether a rendered line looks like a unified diff hunk header.
fn hunk_header_line(line: &str) -> bool {
    let line = strip_ansi(line);
    let line = line.trim_start();
    line.starts_with("@@") && line.contains("@@")
}

/// Returns the terminal-visible width of text after removing ANSI color escapes.
fn visible_width(text: &str) -> usize {
    strip_ansi(text).chars().count()
}

/// Returns visible line numbers whose plain text contains `query`.
fn matching_visible_lines(rendered: &str, query: &str) -> Vec<usize> {
    let query = query.to_lowercase();
    rendered
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            let text = strip_ansi(text).to_lowercase();
            text.contains(&query).then_some(line)
        })
        .collect()
}

/// Selects the first match at or below the current viewport, wrapping to the first match.
fn selected_match_at_or_after(matches: &[usize], scroll_offset: usize) -> Option<usize> {
    if matches.is_empty() {
        return None;
    }

    matches
        .iter()
        .position(|line| *line >= scroll_offset)
        .or(Some(0))
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
            Some("Modified regular file src/a.rs:  [file 1/2]".to_owned())
        );

        state.select_first();

        assert_eq!(state.sticky_header(), None);
    }

    #[test]
    fn sticky_header_includes_stat_suffix_and_file_index() {
        let mut state = DiffState::new(snapshot_with_stats(
            "aaa",
            "Modified regular file src/a.rs:\n a1\n a2\nModified regular file src/b.rs:\n b\n",
            vec![
                DiffFileStat::new("src/a.rs", 2, 1)
                    .with_rendered("| 3 \u{1b}[38;5;2m++\u{1b}[38;5;1m-\u{1b}[39m"),
                DiffFileStat::new("src/b.rs", 1, 0).with_rendered("| 1 \u{1b}[38;5;2m+\u{1b}[39m"),
            ],
        ));
        state.keep_selected_in_view(2);

        state.scroll_next_line();

        let sticky_header = state
            .sticky_header()
            .map_or_else(String::new, |sticky_header| strip_ansi(&sticky_header));
        assert_eq!(
            sticky_header,
            "Modified regular file src/a.rs: | 3 ++-  [file 1/2]"
        );
    }

    #[test]
    fn search_jumps_to_matching_visible_line_and_repeats() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n alpha\n beta\nModified regular file src/b.rs:\n alphabet\n",
        ));
        state.keep_selected_in_view(2);

        state.search("alpha");

        assert_eq!(state.scroll_offset(), 1);
        assert_eq!(
            state.search_status(),
            Some("/alpha  1/2  n next  N previous".to_owned())
        );

        state.search_next();

        assert_eq!(state.scroll_offset(), 3);
        assert_eq!(state.selected_visible_line(), Some(3));
        assert_eq!(
            state.search_status(),
            Some("/alpha  2/2  n next  N previous".to_owned())
        );

        state.search_previous();

        assert_eq!(state.scroll_offset(), 1);
        assert_eq!(
            state.search_status(),
            Some("/alpha  1/2  n next  N previous".to_owned())
        );
    }

    #[test]
    fn search_reports_no_matches_without_moving_scroll() {
        let mut state =
            DiffState::new(snapshot("aaa", "Modified regular file src/a.rs:\n alpha\n"));
        state.keep_selected_in_view(2);

        state.search("missing");

        assert_eq!(state.scroll_offset(), 0);
        assert_eq!(
            state.search_status(),
            Some("/missing  no matches".to_owned())
        );
    }

    #[test]
    fn horizontal_scroll_moves_by_columns_and_clamps_to_wide_content() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            "Modified regular file src/a.rs:\n 12345678901234567890\n",
        ));
        state.set_viewport_width(10);

        state.scroll_right();

        assert_eq!(state.horizontal_offset(), 8);
        assert_eq!(
            state.horizontal_status(),
            Some("</> horizontal scroll  col 9".to_owned())
        );

        state.scroll_right();

        assert_eq!(state.horizontal_offset(), 16);

        state.scroll_left();

        assert_eq!(state.horizontal_offset(), 8);
    }

    #[test]
    fn brace_movement_selects_hunk_headers_across_files() {
        let mut state = DiffState::new(snapshot(
            "aaa",
            concat!(
                "Modified regular file src/a.rs:\n",
                "@@ -1,1 +1,1 @@\n",
                " a1\n",
                "@@ -8,1 +8,1 @@\n",
                " a2\n",
                "Modified regular file src/b.rs:\n",
                "@@ -1,1 +1,1 @@\n",
                " b1\n",
            ),
        ));
        state.keep_selected_in_view(3);

        state.select_next_hunk();

        assert_eq!(state.scroll_offset(), 3);
        assert_eq!(state.selected_visible_line(), Some(0));

        state.select_next_hunk();

        assert_eq!(state.scroll_offset(), 5);
        assert_eq!(state.selected_visible_line(), Some(5));

        state.select_previous_hunk();

        assert_eq!(state.scroll_offset(), 3);
        assert_eq!(state.selected_visible_line(), Some(0));
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
