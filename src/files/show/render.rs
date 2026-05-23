use ratatui::Frame;
use ratatui::layout::Rect;

use super::FileShowView;
use crate::documents;
use crate::documents::{PinnedDocument, project_with_active_file};
use crate::search::SearchQuery;

impl FileShowView {
    /// Render the current file document with the active viewport projection.
    pub fn render(&self, frame: &mut Frame, area: Rect, search: Option<&SearchQuery>) {
        documents::render_document_with_viewport(
            frame,
            area,
            self.projection(),
            self.viewport,
            search,
        );
    }

    /// Project the rendered document with the active file pinned for sticky context.
    pub fn projection(&self) -> PinnedDocument {
        project_with_active_file(&self.document, &[], self.scroll_offset, std::iter::empty())
    }
}
