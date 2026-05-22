//! Rewrite action plans for graph-relative jj mutations.
//!
//! This module owns explicit rewrite source and destination roles, argv
//! construction, and preview wording for rebase, squash, and absorb. It
//! describes the selected revisions honestly, but it does not simulate jj's
//! line placement or final graph results.

mod absorb;
mod rebase;
mod squash;

pub use absorb::JjAbsorbPlan;
pub use rebase::JjRebasePlan;
pub use squash::JjSquashPlan;

#[cfg(test)]
mod tests;
