//! Working-copy action plans.
//!
//! This module owns working-copy creation, duplication, split, and navigation
//! plans. These commands share the same boundary: log selection supplies an
//! exact change id only for exact-target actions, while `@` remains the stable
//! target for current-working-copy and topology-relative movement.

mod creation;
mod navigation;
mod split;

pub use creation::{JjDuplicatePlan, JjNewPlan};
pub use navigation::{JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan};
pub use split::{JjSplitPlan, JjSplitTarget};

#[cfg(test)]
mod tests;
