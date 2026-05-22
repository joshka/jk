//! Git fetch and push action plans.
//!
//! This module owns git sync argv construction, dry-run preview labels, and
//! remote selection details. App prompt policy, row loading, and status wording
//! stay with their existing owners.

mod fetch;
mod push;

pub use fetch::JjGitFetch;
pub use push::{JjGitPush, JjGitPushTarget};

#[cfg(test)]
mod tests;
