use ratatui::Frame;
use ratatui::layout::Rect;

use crate::documents;
use crate::documents::PinnedDocument;
use crate::search::SearchQuery;

use super::DiffView;

impl DiffView {
    /// Renders the current sticky document projection into the active viewport.
    pub fn render(&self, frame: &mut Frame, area: Rect, search: Option<&SearchQuery>) {
        documents::render_document_with_viewport(
            frame,
            area,
            self.projection(),
            self.document.viewport(),
            search,
        );
    }

    /// Returns the rendered projection with sticky file context applied.
    pub fn projection(&self) -> PinnedDocument {
        self.document.projection([])
    }
}
