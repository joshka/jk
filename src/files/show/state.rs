use color_eyre::Result;
use ratatui::layout::Size;

use crate::documents::{DocumentLines, load_document};
use crate::files::show::FileShowView;

impl FileShowView {
    /// Reload the document while preserving path identity and viewport state.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_document)
    }

    pub fn set_scroll_offset(&mut self, _viewport_height: u16, scroll_offset: usize) {
        self.scroll_offset = scroll_offset.min(self.max_scroll_offset());
    }

    /// Jump to the first document line.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Jump to the last visible document position.
    pub fn scroll_to_bottom(&mut self, _viewport_height: u16) {
        self.scroll_offset = self.max_scroll_offset();
    }

    /// Scroll down by a fixed number of lines.
    pub fn scroll_down(&mut self, _viewport_height: u16, amount: usize) {
        self.scroll_offset = self
            .scroll_offset
            .saturating_add(amount)
            .min(self.max_scroll_offset());
    }

    /// Scroll up by a fixed number of lines.
    pub fn scroll_up(&mut self, _viewport_height: u16, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Clamp vertical and horizontal viewport state to current document bounds.
    pub fn clamp(&mut self, viewport: Size) {
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
        self.viewport.clamp(viewport.width, self.max_line_width());
    }

    pub fn toggle_wrap(&mut self, viewport_width: u16) {
        self.viewport.toggle_wrap();
        self.viewport.clamp(viewport_width, self.max_line_width());
    }

    pub fn scroll_left(&mut self, amount: usize) {
        self.viewport.scroll_left(amount);
    }

    pub fn scroll_right(&mut self, viewport_width: u16, amount: usize) {
        self.viewport
            .scroll_right(viewport_width, amount, self.max_line_width());
    }

    pub fn refresh_with_loader(
        &mut self,
        load: impl Fn(&crate::jj::ViewSpec) -> Result<DocumentLines>,
    ) -> Result<()> {
        self.document = load(&self.spec)?;
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
        Ok(())
    }
}
