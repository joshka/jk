//! Command registry metadata and safety classification.
//!
//! Terminology reference for execution modes and safety tiers:
//! `docs/glossary.md`.

mod overview;
mod spec;

pub use overview::{command_overview_lines, command_overview_lines_with_query};
pub use spec::{SafetyTier, command_safety};
