//! Command registry metadata and safety classification.
//!
//! User-facing behavior labels are documented in `docs/glossary.md`.

mod overview;
mod spec;

pub use overview::{
    command_overview_lines, command_overview_lines_with_query,
    command_overview_lines_with_query_and_recent, command_overview_lines_with_recent,
    command_workflow_lines,
};
pub use spec::{SafetyTier, command_safety};
