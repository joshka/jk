use color_eyre::Result;
use ratatui::layout::Size;

use crate::documents::StickyFileDocument;
use crate::jj::ViewSpec;
use crate::log::load_compact_log_context;
use crate::show::ShowView;

impl ShowView {
    /// Loads rendered `jj show` output and the compact log context pinned above file content.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let document = StickyFileDocument::load(&spec)?;
        let compact_context = load_compact_log_context(&spec.show_context_revset())?;
        Ok(Self {
            spec,
            document,
            compact_context,
        })
    }

    /// Reloads the rendered body and recomputes the compact context pinned above file content.
    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)?;
        self.compact_context = load_compact_log_context(&self.spec.show_context_revset())?;
        Ok(())
    }

    /// Moves the viewport to the first rendered line.
    pub fn scroll_to_top(&mut self) {
        self.document.scroll_to_top();
    }

    /// Moves the viewport to the last reachable rendered line.
    pub fn scroll_to_bottom(&mut self, viewport_height: u16) {
        self.document
            .scroll_to_bottom(viewport_height, || self.compact_context.clone());
    }

    /// Scrolls down by `amount` rendered lines while preserving sticky projection rules.
    pub fn scroll_down(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document
                .scroll_down(viewport_height, 1, || self.compact_context.clone());
        }
    }

    /// Scrolls up by `amount` rendered lines while preserving sticky projection rules.
    pub fn scroll_up(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document
                .scroll_up(viewport_height, 1, || self.compact_context.clone());
        }
    }

    /// Clamp vertical and horizontal offsets to the current viewport size.
    pub fn clamp(&mut self, viewport: Size) {
        self.document.clamp(viewport.height, viewport.width);
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
