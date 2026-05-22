//! Startup parsing, view-stack navigation, and global view selection.
//!
//! The event loop decides when navigation happens. This root only names the
//! three app-owned navigation concerns:
//!
//! - `startup`: process arguments become the first `ViewSpec` and initial
//!   `App` state
//! - `stack`: detail specs, pushed views, and shipped top-level surface
//!   transitions
//! - `view_menu`: top-level view menu selection plus show/diff format toggles

mod stack;
pub(super) mod startup;
mod view_menu;
