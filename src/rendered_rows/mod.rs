//! Shared rendered `jj` row helpers.
//!
//! Feature-specific row models should live with the feature that owns the user-visible behavior.
//! This subtree keeps only domain-neutral helpers used by those row models: plain-text flattening,
//! metadata drift handling, JSON field extraction, and graph-line detection.

mod graph;
mod metadata;
mod text;

pub use text::document_plain_text;

pub use graph::{first_content_char, is_standalone_graph_line};
pub use metadata::{
    RowMetadata, boolean_field, non_empty_string_field, optional_string_field, string_field,
};
pub use text::line_text;
