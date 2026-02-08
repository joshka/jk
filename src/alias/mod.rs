//! Alias normalization and alias-catalog rendering.

mod catalog;
mod normalize;

pub use catalog::{alias_overview_lines, alias_overview_lines_with_query};
pub use normalize::normalize_alias;
