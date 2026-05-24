use color_eyre::Result;

use crate::documents::{self, DocumentLines, FileAnchor, PinnedDocument, project_with_active_file};
use crate::jj::ViewSpec;
use crate::search::SearchQuery;

pub struct PlainDocument {
    /// Preserved rendered operation detail lines from `jj operation show` or `diff`.
    lines: DocumentLines,
    /// Current top-of-viewport offset within the preserved rendered lines.
    scroll_offset: usize,
}

impl PlainDocument {
    /// Loads one plain rendered document from the `jj` boundary.
    pub fn load(spec: &ViewSpec) -> Result<Self> {
        let lines = documents::load_document(spec)?;
        Ok(Self::new(lines))
    }

    /// Builds scroll state around already-rendered document lines.
    pub fn new(lines: DocumentLines) -> Self {
        Self {
            lines,
            scroll_offset: 0,
        }
    }

    /// Reloads the rendered lines and clamps the old scroll position to the new size.
    pub fn refresh(&mut self, spec: &ViewSpec) -> Result<()> {
        self.lines = documents::load_document(spec)?;
        self.clamp();
        Ok(())
    }

    /// Projects the plain document without any sticky-file anchors.
    pub fn projection(&self) -> PinnedDocument {
        let anchors: &[FileAnchor] = &[];
        project_with_active_file(&self.lines, anchors, self.scroll_offset, [])
    }

    /// Returns the number of rendered lines in the current document.
    pub fn line_count(&self) -> usize {
        self.lines.line_count()
    }

    /// Returns the current vertical scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the scroll offset, clamped to the last reachable line.
    pub fn set_scroll_offset(&mut self, scroll_offset: usize) {
        self.scroll_offset = scroll_offset.min(self.max_scroll_offset());
    }

    /// Moves the viewport to the first rendered line.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Moves the viewport to the last rendered line.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    /// Scrolls down by `amount` lines.
    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self
            .scroll_offset
            .saturating_add(amount)
            .min(self.max_scroll_offset());
    }

    /// Scrolls up by `amount` lines.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Reapplies the current offset through clamping after document changes.
    pub fn clamp(&mut self) {
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
    }

    /// Counts rendered matches for the current query.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        documents::search_matches(&self.lines, query)
    }

    /// Advances to the next rendered search match if one exists.
    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(offset) = documents::next_matching_line(&self.lines, self.scroll_offset, query)
        else {
            return false;
        };
        self.set_scroll_offset(offset);
        true
    }

    /// Moves to the previous rendered search match if one exists.
    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(offset) =
            documents::previous_matching_line(&self.lines, self.scroll_offset, query)
        else {
            return false;
        };
        self.set_scroll_offset(offset);
        true
    }

    /// Returns the last reachable vertical offset for the current document size.
    fn max_scroll_offset(&self) -> usize {
        self.line_count().saturating_sub(1)
    }
}
