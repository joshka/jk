//! Search query parsing, matching, and highlight styling.

mod highlight;
mod query;

pub use self::highlight::{entry_matches, highlight_line, line_matches};
pub use self::query::SearchQuery;

#[cfg(test)]
mod tests;
