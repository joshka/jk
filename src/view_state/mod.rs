//! App-facing dispatcher over the concrete view slices.
//!
//! Feature modules own their state, rendering, bindings, and tests. `ViewState`
//! only routes rendering and command execution to the active slice.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::actions::{JjBookmarkForgetTarget, JjBookmarkTarget, JjGitPushTarget};
use crate::bookmarks::BookmarksView;
use crate::command::{Binding, CommandContext, HelpContext, ViewCommand, ViewEffect};
use crate::diff::DiffView;
use crate::files::list::FileListView;
use crate::files::show::FileShowView;
use crate::jj::{JjCommand, LogViewMode, ViewSpec};
use crate::log::LogView;
use crate::menus::ExactActionContext;
use crate::operation_log::OperationLogView;
use crate::operation_log::detail::OperationDetailView;
use crate::resolve::ResolveView;
use crate::search::SearchQuery;
use crate::show::ShowView;
use crate::status::StatusView;
use crate::tui::StatusHints;
use crate::view_action_targets::ViewActionTargets;
use crate::workspaces::WorkspacesView;

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

impl ViewState {
    /// Materialize the concrete feature view for a previously parsed `ViewSpec`.
    ///
    /// Startup and app-level navigation both pass through here so the app can stay generic over
    /// the active slice until a specific `JjCommand` selects the owning view type.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        match spec.command() {
            JjCommand::Default | JjCommand::Log => Ok(Self::Log(LogView::load(spec)?)),
            JjCommand::Show => Ok(Self::Show(ShowView::load(spec)?)),
            JjCommand::Diff => Ok(Self::Diff(DiffView::load(spec)?)),
            JjCommand::Status => Ok(Self::Status(StatusView::load(spec)?)),
            JjCommand::Resolve => Ok(Self::Resolve(ResolveView::load(spec)?)),
            JjCommand::FileList => Ok(Self::FileList(FileListView::load(spec)?)),
            JjCommand::FileShow => Ok(Self::FileShow(FileShowView::load(spec)?)),
            JjCommand::Bookmarks => Ok(Self::Bookmarks(BookmarksView::load(spec)?)),
            JjCommand::Workspaces => Ok(Self::Workspaces(WorkspacesView::load(spec)?)),
            JjCommand::OperationLog => Ok(Self::OperationLog(OperationLogView::load(spec)?)),
            JjCommand::OperationShow | JjCommand::OperationDiff => {
                Ok(Self::OperationDetail(OperationDetailView::load(spec)?))
            }
        }
    }

    /// Render the active view slice into the shared app layout.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        match self {
            Self::Log(view) => view.render(frame, area, search),
            Self::Show(view) => view.render(frame, area, search),
            Self::Diff(view) => view.render(frame, area, search),
            Self::Status(view) => view.render(frame, area, search),
            Self::Resolve(view) => view.render(frame, area, search),
            Self::FileList(view) => view.render(frame, area, search),
            Self::FileShow(view) => view.render(frame, area, search),
            Self::Bookmarks(view) => view.render(frame, area, search),
            Self::Workspaces(view) => view.render(frame, area, search),
            Self::OperationLog(view) => view.render(frame, area, search),
            Self::OperationDetail(view) => view.render(frame, area, search),
        }
    }

    /// Expose the bindings owned by the active view slice.
    pub fn bindings(&self) -> &'static [Binding] {
        match self {
            Self::Log(view) => view.bindings(),
            Self::Show(view) => view.bindings(),
            Self::Diff(view) => view.bindings(),
            Self::Status(view) => view.bindings(),
            Self::Resolve(view) => view.bindings(),
            Self::FileList(view) => view.bindings(),
            Self::FileShow(view) => view.bindings(),
            Self::Bookmarks(view) => view.bindings(),
            Self::Workspaces(view) => view.bindings(),
            Self::OperationLog(view) => view.bindings(),
            Self::OperationDetail(view) => view.bindings(),
        }
    }

    /// Route one view-local command into the active feature slice.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match self {
            Self::Log(view) => view.execute(command, context),
            Self::Show(view) => view.execute(command, context),
            Self::Diff(view) => view.execute(command, context),
            Self::Status(view) => view.execute(command, context),
            Self::Resolve(view) => view.execute(command, context),
            Self::FileList(view) => view.execute(command, context),
            Self::FileShow(view) => view.execute(command, context),
            Self::Bookmarks(view) => view.execute(command, context),
            Self::Workspaces(view) => view.execute(command, context),
            Self::OperationLog(view) => view.execute(command, context),
            Self::OperationDetail(view) => view.execute(command, context),
        }
    }

    /// Refresh the active view slice from its owning data source.
    pub fn refresh(&mut self) -> Result<()> {
        match self {
            Self::Log(view) => view.refresh(),
            Self::Show(view) => view.refresh(),
            Self::Diff(view) => view.refresh(),
            Self::Status(view) => view.refresh(),
            Self::Resolve(view) => view.refresh(),
            Self::FileList(view) => view.refresh(),
            Self::FileShow(view) => view.refresh(),
            Self::Bookmarks(view) => view.refresh(),
            Self::Workspaces(view) => view.refresh(),
            Self::OperationLog(view) => view.refresh(),
            Self::OperationDetail(view) => view.refresh(),
        }
    }

    /// Clamp active-view scroll or selection state to the current viewport.
    pub fn clamp(&mut self, viewport_height: u16, viewport_width: u16) {
        match self {
            Self::Log(view) => view.clamp(),
            Self::Show(view) => view.clamp(viewport_height, viewport_width),
            Self::Diff(view) => view.clamp(viewport_height, viewport_width),
            Self::Status(view) => view.clamp(viewport_height),
            Self::Resolve(view) => view.clamp(),
            Self::FileList(view) => view.clamp(),
            Self::FileShow(view) => view.clamp(viewport_height, viewport_width),
            Self::Bookmarks(view) => view.clamp(),
            Self::Workspaces(view) => view.clamp(),
            Self::OperationLog(view) => view.clamp(),
            Self::OperationDetail(view) => view.clamp(viewport_height),
        }
    }

    /// Borrow the original `ViewSpec` carried by the active slice.
    pub fn spec(&self) -> &ViewSpec {
        match self {
            Self::Log(view) => view.spec(),
            Self::Show(view) => view.spec(),
            Self::Diff(view) => view.spec(),
            Self::Status(view) => view.spec(),
            Self::Resolve(view) => view.spec(),
            Self::FileList(view) => view.spec(),
            Self::FileShow(view) => view.spec(),
            Self::Bookmarks(view) => view.spec(),
            Self::Workspaces(view) => view.spec(),
            Self::OperationLog(view) => view.spec(),
            Self::OperationDetail(view) => view.spec(),
        }
    }

    /// Report the shared status-hint flavor for the active slice.
    pub fn status_hints(&self) -> StatusHints {
        match self {
            Self::Log(_) => StatusHints::Log,
            Self::Show(_) => StatusHints::ShowDocument,
            Self::Diff(_) => StatusHints::DiffDocument,
            Self::Status(_) => StatusHints::Status,
            Self::Resolve(_) => StatusHints::Resolve,
            Self::FileList(_) => StatusHints::FileList,
            Self::FileShow(_) => StatusHints::FileShowDocument,
            Self::Bookmarks(_) => StatusHints::Bookmarks,
            Self::Workspaces(_) => StatusHints::Workspaces,
            Self::OperationLog(_) => StatusHints::OperationLog,
            Self::OperationDetail(_) => StatusHints::OperationDetailDocument,
        }
    }

    /// Report the help-surface grouping for the active slice.
    pub fn help_context(&self) -> HelpContext {
        match self {
            Self::Log(_) => HelpContext::Log,
            Self::Show(_) => HelpContext::Show,
            Self::Diff(_) => HelpContext::Diff,
            Self::Status(_) => HelpContext::Status,
            Self::Resolve(_) => HelpContext::Resolve,
            Self::FileList(_) => HelpContext::FileList,
            Self::FileShow(_) => HelpContext::FileShow,
            Self::Bookmarks(_) => HelpContext::Bookmarks,
            Self::Workspaces(_) => HelpContext::Workspaces,
            Self::OperationLog(_) => HelpContext::OperationLog,
            Self::OperationDetail(_) => HelpContext::OperationDetail,
        }
    }

    pub fn command(&self) -> JjCommand {
        self.spec().command()
    }

    pub fn scroll_offset(&self) -> usize {
        match self {
            Self::Log(_) => 0,
            Self::Show(view) => view.scroll_offset(),
            Self::Diff(view) => view.scroll_offset(),
            Self::Status(view) => view.scroll_offset(),
            Self::Resolve(_) => 0,
            Self::FileList(_) => 0,
            Self::FileShow(view) => view.scroll_offset(),
            Self::Bookmarks(_) => 0,
            Self::Workspaces(_) => 0,
            Self::OperationLog(_) => 0,
            Self::OperationDetail(view) => view.scroll_offset(),
        }
    }

    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        match self {
            Self::Log(_) => {}
            Self::Show(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Diff(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Status(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Resolve(_) => {}
            Self::FileList(_) => {}
            Self::FileShow(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Bookmarks(_) => {}
            Self::Workspaces(_) => {}
            Self::OperationLog(_) => {}
            Self::OperationDetail(view) => view.set_scroll_offset(viewport_height, scroll_offset),
        }
    }

    pub fn item_count(&self) -> Option<usize> {
        match self {
            Self::Log(view) => Some(view.item_count()),
            Self::Resolve(view) => Some(view.item_count()),
            Self::FileList(view) => Some(view.item_count()),
            Self::Bookmarks(view) => Some(view.item_count()),
            Self::Workspaces(view) => Some(view.item_count()),
            Self::OperationLog(view) => Some(view.item_count()),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::FileShow(_)
            | Self::OperationDetail(_) => None,
        }
    }

    pub fn log_mode_label(&self) -> Option<&str> {
        match self {
            Self::Log(view) => Some(view.mode_label()),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::Resolve(_)
            | Self::FileList(_)
            | Self::FileShow(_)
            | Self::Bookmarks(_)
            | Self::Workspaces(_)
            | Self::OperationLog(_)
            | Self::OperationDetail(_) => None,
        }
    }

    pub fn set_log_mode(&mut self, mode: LogViewMode) -> Result<()> {
        match self {
            Self::Log(view) => view.set_mode(mode),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::Resolve(_)
            | Self::FileList(_)
            | Self::FileShow(_)
            | Self::Bookmarks(_)
            | Self::Workspaces(_)
            | Self::OperationLog(_)
            | Self::OperationDetail(_) => Ok(()),
        }
    }

    pub fn reveal_log_change(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        match self {
            Self::Log(view) => view.reveal_change_id(change_id, fallback_mode),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::Resolve(_)
            | Self::FileList(_)
            | Self::FileShow(_)
            | Self::Bookmarks(_)
            | Self::Workspaces(_)
            | Self::OperationLog(_)
            | Self::OperationDetail(_) => Ok(false),
        }
    }

    pub fn document_line_count(&self) -> usize {
        match self {
            Self::Log(view) => view.line_count(),
            Self::Show(view) => view.line_count(),
            Self::Diff(view) => view.line_count(),
            Self::Status(view) => view.line_count(),
            Self::Resolve(view) => view.line_count(),
            Self::FileList(view) => view.line_count(),
            Self::FileShow(view) => view.line_count(),
            Self::Bookmarks(view) => view.line_count(),
            Self::Workspaces(view) => view.line_count(),
            Self::OperationLog(view) => view.line_count(),
            Self::OperationDetail(view) => view.line_count(),
        }
    }

    pub fn push_target(&self) -> Result<Option<JjGitPushTarget>> {
        ViewActionTargets::new(self).push_target()
    }

    pub fn bookmark_target(&self) -> Result<Option<JjBookmarkTarget>> {
        ViewActionTargets::new(self).bookmark_target()
    }

    pub fn selected_local_bookmark_name(&self) -> Result<Option<&str>> {
        ViewActionTargets::new(self).selected_local_bookmark_name()
    }

    pub fn selected_local_bookmark_name_for(&self, action: &str) -> Result<Option<&str>> {
        ViewActionTargets::new(self).selected_local_bookmark_name_for(action)
    }

    pub fn bookmark_forget_target(&self) -> Result<Option<(String, JjBookmarkForgetTarget)>> {
        ViewActionTargets::new(self).bookmark_forget_target()
    }

    pub fn exact_restore_revert_context(&self) -> Result<Option<ExactActionContext>> {
        ViewActionTargets::new(self).exact_restore_revert_context()
    }
}

#[cfg(test)]
mod tests;
