//! State machine for generic rendered-output inspection.

use jk_core::InspectionSnapshot;

use crate::ansi_text::strip_ansi;
use crate::chrome::title_or_default;

/// Semantic state behind rendered read-only inspection output.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RenderedState {
    title: String,
    target: String,
    rendered: String,
    search: Option<SearchState>,
    scroll_offset: usize,
    viewport_height: usize,
}

impl RenderedState {
    /// Creates state from a freshly loaded inspection snapshot.
    pub fn new(snapshot: InspectionSnapshot) -> Self {
        let (title, target, rendered) = snapshot.into_parts();
        Self {
            title: title_or_default(title),
            target,
            rendered,
            search: None,
            scroll_offset: 0,
            viewport_height: 10,
        }
    }

    /// Replaces rendered output while preserving the current search when practical.
    pub fn refresh(&mut self, snapshot: InspectionSnapshot) {
        let (title, target, rendered) = snapshot.into_parts();
        self.title = title_or_default(title);
        self.target = target;
        self.rendered = rendered;
        self.clamp_scroll_offset();
        self.refresh_search_matches();
    }

    /// Returns the command context shown in the title bar.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the first rendered line currently visible in the viewport.
    pub const fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the rendered viewport height and keeps scroll inside the body.
    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
        self.clamp_scroll_offset();
    }

    /// Returns the rendered body or a fallback message for empty output/error states.
    pub fn visible_body(&self, status_message: Option<&str>) -> String {
        if !self.rendered.trim().is_empty() {
            return self.rendered.clone();
        }

        if let Some(error) = status_message {
            return format!(
                "Unable to load inspection for {}.\n\n{error}\n\nPress r to retry or H/L to return to the log.\n",
                self.target
            );
        }

        format!(
            "No output for {}.\n\nThe jj command produced no visible text.\n",
            self.target
        )
    }

    /// Scrolls one visible line toward the start of the output.
    pub fn scroll_previous_line(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scrolls one visible line toward the end of the output.
    pub fn scroll_next_line(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
        self.clamp_scroll_offset();
    }

    /// Scrolls one viewport toward the start of the output.
    pub fn select_page_previous(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(self.viewport_height);
    }

    /// Scrolls one viewport toward the end of the output.
    pub fn select_page_next(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(self.viewport_height);
        self.clamp_scroll_offset();
    }

    /// Moves to the first rendered line.
    pub fn select_first(&mut self) {
        self.scroll_offset = 0;
    }

    /// Moves to the last visible page.
    pub fn select_last(&mut self) {
        self.scroll_offset = usize::MAX;
        self.clamp_scroll_offset();
    }

    /// Searches visible rendered lines.
    pub fn search(&mut self, query: &str, status_message: Option<&str>) {
        let query = query.trim();
        if query.is_empty() {
            self.search = None;
            return;
        }

        let matches = matching_lines(&self.visible_body(status_message), query);
        let selected = (!matches.is_empty()).then_some(0);
        self.search = Some(SearchState {
            query: query.to_owned(),
            matches,
            selected,
        });
        self.scroll_to_selected_match();
    }

    /// Selects the next search match.
    pub fn search_next(&mut self) {
        let Some(search) = &mut self.search else {
            return;
        };
        let Some(selected) = search.selected else {
            return;
        };
        search.selected = Some((selected + 1).min(search.matches.len().saturating_sub(1)));
        self.scroll_to_selected_match();
    }

    /// Selects the previous search match.
    pub fn search_previous(&mut self) {
        let Some(search) = &mut self.search else {
            return;
        };
        let Some(selected) = search.selected else {
            return;
        };
        search.selected = Some(selected.saturating_sub(1));
        self.scroll_to_selected_match();
    }

    /// Returns status-line text for the current search, if any.
    pub fn search_status(&self) -> Option<String> {
        let search = self.search.as_ref()?;
        match search.selected {
            Some(selected) => Some(format!(
                "/{}  {}/{}",
                search.query,
                selected + 1,
                search.matches.len()
            )),
            None => Some(format!("/{}  no matches", search.query)),
        }
    }

    fn refresh_search_matches(&mut self) {
        let Some(query) = self.search.as_ref().map(|search| search.query.clone()) else {
            return;
        };
        self.search(&query, None);
    }

    fn scroll_to_selected_match(&mut self) {
        let Some(line) = self.search.as_ref().and_then(SearchState::selected_line) else {
            return;
        };
        self.scroll_offset = line;
        self.clamp_scroll_offset();
    }

    fn clamp_scroll_offset(&mut self) {
        let line_count = self.visible_body(None).lines().count();
        let max_offset = line_count.saturating_sub(self.viewport_height.max(1));
        self.scroll_offset = self.scroll_offset.min(max_offset);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SearchState {
    query: String,
    matches: Vec<usize>,
    selected: Option<usize>,
}

impl SearchState {
    fn selected_line(&self) -> Option<usize> {
        self.selected
            .and_then(|index| self.matches.get(index).copied())
    }
}

fn matching_lines(rendered: &str, query: &str) -> Vec<usize> {
    let query = query.to_lowercase();
    strip_ansi(rendered)
        .lines()
        .enumerate()
        .filter_map(|(line, text)| text.to_lowercase().contains(&query).then_some(line))
        .collect()
}
