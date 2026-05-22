//! Bookmark action plans and validation.
//!
//! This module owns bookmark mutation argv construction, exact-name quoting,
//! preview summaries, and rename validation for the bookmarks feature. App
//! prompt policy stays in the action lifecycle; selected-row target resolution
//! and bookmark row metadata stay in sibling bookmark modules.

mod plan;
mod targets;
mod validation;

pub use plan::{JjBookmarkMutationKind, JjBookmarkMutationPlan};
pub use targets::{JjBookmarkForgetTarget, JjBookmarkTarget, JjBookmarkTrackingTarget};
pub use validation::validate_bookmark_rename_new_name;

#[cfg(test)]
mod tests;
