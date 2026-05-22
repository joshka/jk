//! Rendered jj document models and sticky file-aware document projection.
//!
//! `rendered` owns lightweight structure over already-rendered jj lines. `sticky` owns viewport,
//! wrapping, search, and sticky file-heading projection for document-like screens.

mod rendered;
mod sticky;

pub use self::rendered::{DocumentLines, FileAnchor, PinnedDocument, project_with_active_file};
#[cfg(test)]
pub use self::sticky::DocumentDisplayMode;
pub use self::sticky::{
    DocumentViewport, StickyFileDocument, lines_text, load_document, next_matching_line,
    previous_matching_line, render_document, render_document_with_viewport, search_matches,
};
