//! File and content mutation command plans.
//!
//! These plans own argv, preview wording, and direct execution for `jj restore`, `jj revert`, and
//! `jj file` mutations after a view or menu has already selected the action target. Availability
//! policy stays with the feature surface that offers the action.

mod mutation;
mod restore;
mod revert;

pub use mutation::{JjFileChmodMode, JjFileMutationKind, JjFileMutationPlan, JjFileMutationTarget};
pub use restore::JjRestorePlan;
pub use revert::JjRevertPlan;

#[cfg(test)]
mod tests;
