//! Shared rendering and navigation for rendered file documents.
//!
//! Show, diff, status, and operation-detail surfaces use the same sticky
//! file-heading projection, search navigation, and file-to-file movement. This
//! root keeps `StickyFileDocument` as the feature-facing owner while child
//! modules hold the local mechanics for viewport state, rendering, vertical
//! scroll semantics, and file-jump behavior.

use color_eyre::Result;
use ratatui::text::Line;

use crate::documents::{DocumentLines, FileAnchor, PinnedDocument, project_with_active_file};
use crate::jj::ViewSpec;
use crate::search::SearchQuery;

mod file_navigation;
mod render;
mod scroll;
mod viewport;

pub use self::render::{
    lines_text, load_document, next_matching_line, previous_matching_line, render_document,
    render_document_with_viewport, search_matches,
};
use self::scroll::StickyScroll;
#[cfg(test)]
pub use self::viewport::DocumentDisplayMode;
pub use self::viewport::DocumentViewport;

/// Rendered file text plus file anchors and scroll state.
///
/// This type owns document navigation for file-oriented detail views. It
/// reloads from `jj` through `load_document`, then keeps scroll offsets clamped
/// to the rendered lines rather than to a reconstructed repository model.
pub struct StickyFileDocument {
    /// Rendered document lines loaded from `jj`.
    lines: DocumentLines,
    /// Detected file anchors reused for sticky headers and file navigation.
    anchors: Vec<FileAnchor>,
    /// Current vertical sticky-scroll state.
    scroll: StickyScroll,
    /// Current wrapping and horizontal-scroll viewport state.
    viewport: DocumentViewport,
}

impl StickyFileDocument {
    /// Load a rendered document plus sticky-file anchors for one `ViewSpec`.
    pub fn load(spec: &ViewSpec) -> Result<Self> {
        let lines = load_document(spec)?;
        Ok(Self::new(lines))
    }

    /// Build a sticky document from already-rendered lines.
    pub fn new(lines: DocumentLines) -> Self {
        let anchors = lines.file_anchors();
        Self {
            lines,
            anchors,
            scroll: StickyScroll::default(),
            viewport: DocumentViewport::default(),
        }
    }

    /// Reload the rendered document while recomputing file anchors.
    pub fn refresh(&mut self, spec: &ViewSpec) -> Result<()> {
        self.replace_lines(load_document(spec)?);
        Ok(())
    }

    /// Project the current scroll position into sticky fixed lines plus body.
    pub fn projection(&self, prefix: impl IntoIterator<Item = Line<'static>>) -> PinnedDocument {
        self.projection_at(self.scroll.offset(), prefix)
    }

    /// Return the total rendered line count.
    pub fn line_count(&self) -> usize {
        self.lines.line_count()
    }

    /// Return the current vertical scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    #[cfg(test)]
    pub fn horizontal_offset(&self) -> usize {
        self.viewport.horizontal_offset()
    }

    pub fn viewport(&self) -> DocumentViewport {
        self.viewport
    }

    /// Set the vertical scroll offset and clamp it against current bounds.
    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        self.scroll.set(scroll_offset, self.max_scroll_offset());
        self.clamp(viewport_height, u16::MAX);
    }

    /// Jump to the top of the rendered document.
    pub fn scroll_to_top(&mut self) {
        self.scroll.move_to_top();
    }

    /// Jump to the lowest meaningful scroll offset for the current viewport.
    pub fn scroll_to_bottom(
        &mut self,
        viewport_height: u16,
        prefix: impl Fn() -> Vec<Line<'static>>,
    ) {
        let max_offset = self.max_scroll_offset();
        let lines = &self.lines;
        let anchors = &self.anchors;
        self.scroll
            .move_to_bottom(max_offset, viewport_height, |offset| {
                project_with_active_file(lines, anchors, offset, prefix())
            });
    }

    /// Scroll down by a number of meaningful document offsets.
    pub fn scroll_down(
        &mut self,
        viewport_height: u16,
        amount: usize,
        prefix: impl Fn() -> Vec<Line<'static>>,
    ) {
        let max_offset = self.max_scroll_offset();
        let lines = &self.lines;
        let anchors = &self.anchors;
        self.scroll
            .down(amount, max_offset, viewport_height, |offset| {
                project_with_active_file(lines, anchors, offset, prefix())
            });
    }

    /// Scroll up by a number of meaningful document offsets.
    pub fn scroll_up(
        &mut self,
        viewport_height: u16,
        amount: usize,
        prefix: impl Fn() -> Vec<Line<'static>>,
    ) {
        let lines = &self.lines;
        let anchors = &self.anchors;
        self.scroll.up(amount, viewport_height, |offset| {
            project_with_active_file(lines, anchors, offset, prefix())
        });
    }

    /// Clamp vertical and horizontal viewport state to current content and width.
    pub fn clamp(&mut self, _viewport_height: u16, viewport_width: u16) {
        self.scroll.clamp(self.max_scroll_offset());
        self.viewport.clamp(viewport_width, self.max_line_width());
    }

    /// Toggle wrapped versus horizontal-scroll document presentation.
    pub fn toggle_wrap(&mut self, viewport_width: u16) {
        self.viewport.toggle_wrap();
        self.viewport.clamp(viewport_width, self.max_line_width());
    }

    /// Scroll horizontally left when no-wrap mode is active.
    pub fn scroll_left(&mut self, amount: usize) {
        self.viewport.scroll_left(amount);
    }

    /// Scroll horizontally right when no-wrap mode is active.
    pub fn scroll_right(&mut self, viewport_width: u16, amount: usize) {
        self.viewport
            .scroll_right(viewport_width, amount, self.max_line_width());
    }

    /// Count search matches across the rendered document.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        search_matches(&self.lines, query)
    }

    /// Move to the next matching document line, if any.
    pub fn next_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = next_matching_line(&self.lines, self.scroll.offset(), query) else {
            return false;
        };
        self.scroll.set(offset, self.max_scroll_offset());
        self.scroll.clamp(self.max_scroll_offset());
        true
    }

    /// Move to the previous matching document line, if any.
    pub fn previous_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = previous_matching_line(&self.lines, self.scroll.offset(), query) else {
            return false;
        };
        self.scroll.set(offset, self.max_scroll_offset());
        self.scroll.clamp(self.max_scroll_offset());
        true
    }

    /// Jump to the next file heading in the rendered document.
    pub fn next_file(&mut self) {
        if let Some(line_index) =
            file_navigation::next_file_offset(&self.lines, &self.anchors, self.scroll.offset())
        {
            self.scroll.set(line_index, self.max_scroll_offset());
        }
    }

    /// Jump to the previous file heading in the rendered document.
    pub fn previous_file(&mut self) {
        if let Some(line_index) =
            file_navigation::previous_file_offset(&self.lines, &self.anchors, self.scroll.offset())
        {
            self.scroll.set(line_index, self.max_scroll_offset());
        }
    }

    /// Return the current active file label, if any.
    pub fn current_file_label(&self) -> Option<&str> {
        file_navigation::current_file_label(&self.lines, &self.anchors, self.scroll.offset())
    }

    fn max_scroll_offset(&self) -> usize {
        self.line_count().saturating_sub(1)
    }

    fn max_line_width(&self) -> usize {
        render::max_line_width(self.lines.lines())
    }

    fn replace_lines(&mut self, lines: DocumentLines) {
        self.lines = lines;
        self.anchors = self.lines.file_anchors();
    }

    /// Project an arbitrary scroll offset into sticky fixed lines plus body.
    fn projection_at(
        &self,
        scroll_offset: usize,
        prefix: impl IntoIterator<Item = Line<'static>>,
    ) -> PinnedDocument {
        project_with_active_file(&self.lines, &self.anchors, scroll_offset, prefix)
    }
}

#[cfg(test)]
mod tests;
