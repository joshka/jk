//! File-oriented views.
//!
//! This feature root owns user-visible file list and file show behavior. Shared file mutation
//! command plans still live with action planning; this module is the starting point for view state,
//! row interpretation, selection, search, copy, and file-surface tests. Treat this root as a
//! table of contents: `list` owns selectable file rows and drill-down into one file, while `show`
//! owns the single-file document surface once a path has been chosen.

pub(crate) mod list;
pub(crate) mod show;
