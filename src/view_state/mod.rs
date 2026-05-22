//! App-facing dispatcher over the concrete view slices.
//!
//! Feature modules own their state, rendering, bindings, and tests. This root
//! only names the active feature enum plus the three app-facing helper groups:
//!
//! - `routing`: load, render, execute, refresh, and generic view metadata
//! - `state`: scroll, line-count, and log-specific shared helpers
//! - `targets`: app action-target projection built from the current view

use crate::bookmarks::BookmarksView;
use crate::diff::DiffView;
use crate::files::list::FileListView;
use crate::files::show::FileShowView;
use crate::log::LogView;
use crate::operation_log::OperationLogView;
use crate::operation_log::detail::OperationDetailView;
use crate::resolve::ResolveView;
use crate::show::ShowView;
use crate::status::StatusView;
use crate::workspaces::WorkspacesView;

mod routing;
mod state;
mod targets;

/// The currently active top-level view.
pub enum ViewState {
    /// Default/log graph surface.
    Log(LogView),
    /// Show detail document.
    Show(ShowView),
    /// Diff detail document.
    Diff(DiffView),
    /// Working-copy status surface.
    Status(StatusView),
    /// Conflict-resolution surface.
    Resolve(ResolveView),
    /// File-list surface for one revision context.
    FileList(FileListView),
    /// File-show document surface.
    FileShow(FileShowView),
    /// Bookmark management surface.
    Bookmarks(BookmarksView),
    /// Workspace management surface.
    Workspaces(WorkspacesView),
    /// Operation-log surface.
    OperationLog(OperationLogView),
    /// Operation detail document, either show or diff flavored.
    OperationDetail(OperationDetailView),
}

#[cfg(test)]
mod tests;
