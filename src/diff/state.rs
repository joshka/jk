use color_eyre::Result;

use crate::documents::StickyFileDocument;
use crate::jj::ViewSpec;

use super::DiffView;

impl DiffView {
    /// Loads rendered `jj diff` output into the shared sticky document model.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let document = StickyFileDocument::load(&spec)?;
        Ok(Self { spec, document })
    }

    /// Reloads the rendered diff body while preserving the same view identity.
    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)?;
        Ok(())
    }

    /// Moves the viewport to the first rendered line.
    pub fn scroll_to_top(&mut self) {
        self.document.scroll_to_top();
    }

    /// Moves the viewport to the last reachable rendered line.
    pub fn scroll_to_bottom(&mut self, viewport_height: u16) {
        self.document.scroll_to_bottom(viewport_height, Vec::new)
    }

    /// Scrolls down by `amount` rendered lines while preserving sticky projection rules.
    pub fn scroll_down(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document.scroll_down(viewport_height, 1, Vec::new);
        }
    }

    /// Scrolls up by `amount` rendered lines while preserving sticky projection rules.
    pub fn scroll_up(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document.scroll_up(viewport_height, 1, Vec::new);
        }
    }

    /// Clamps vertical and horizontal offsets to the current viewport dimensions.
    pub fn clamp(&mut self, viewport_height: u16, viewport_width: u16) {
        self.document.clamp(viewport_height, viewport_width);
    }

    /// Toggles wrapped rendering for the current viewport width.
    pub fn toggle_wrap(&mut self, viewport_width: u16) {
        self.document.toggle_wrap(viewport_width);
    }

    /// Moves the horizontal offset left by `amount` columns.
    pub fn scroll_left(&mut self, amount: usize) {
        self.document.scroll_left(amount);
    }

    /// Moves the horizontal offset right by `amount` columns within the viewport width.
    pub fn scroll_right(&mut self, viewport_width: u16, amount: usize) {
        self.document.scroll_right(viewport_width, amount);
    }
}
