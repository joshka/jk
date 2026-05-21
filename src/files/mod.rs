//! File-oriented views.
//!
//! This feature root owns user-visible file list and file show behavior. Shared file mutation
//! command plans still live with action planning; this module is the starting point for view state,
//! row interpretation, selection, search, copy, and file-surface tests.

pub(crate) mod list;
pub(crate) mod show;
