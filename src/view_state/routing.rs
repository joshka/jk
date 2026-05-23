use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::{Rect, Size};

use super::ViewState;
use crate::command::{Binding, CommandContext, HelpContext, ViewCommand, ViewEffect};
use crate::jj::{JjCommand, ViewSpec};
use crate::search::SearchQuery;
use crate::tui::StatusHints;

impl ViewState {
    /// Materialize the concrete feature view for a previously parsed `ViewSpec`.
    ///
    /// Startup and app-level navigation both pass through here so the app can
    /// stay generic over the active slice until a specific `JjCommand`
    /// selects the owning view type.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        match spec.command() {
            JjCommand::Default | JjCommand::Log => Ok(Self::Log(crate::log::LogView::load(spec)?)),
            JjCommand::Show => Ok(Self::Show(crate::show::ShowView::load(spec)?)),
            JjCommand::Diff => Ok(Self::Diff(crate::diff::DiffView::load(spec)?)),
            JjCommand::Status => Ok(Self::Status(crate::status::StatusView::load(spec)?)),
            JjCommand::Resolve => Ok(Self::Resolve(crate::resolve::ResolveView::load(spec)?)),
            JjCommand::FileList => Ok(Self::FileList(crate::files::list::FileListView::load(
                spec,
            )?)),
            JjCommand::FileShow => Ok(Self::FileShow(crate::files::show::FileShowView::load(
                spec,
            )?)),
            JjCommand::Bookmarks => Ok(Self::Bookmarks(crate::bookmarks::BookmarksView::load(
                spec,
            )?)),
            JjCommand::Workspaces => Ok(Self::Workspaces(crate::workspaces::WorkspacesView::load(
                spec,
            )?)),
            JjCommand::OperationLog => Ok(Self::OperationLog(
                crate::operation_log::OperationLogView::load(spec)?,
            )),
            JjCommand::OperationShow | JjCommand::OperationDiff => Ok(Self::OperationDetail(
                crate::operation_log::detail::OperationDetailView::load(spec)?,
            )),
        }
    }

    /// Render the active view slice into the shared app layout.
    pub fn render(&self, frame: &mut Frame, area: Rect, search: Option<&SearchQuery>) {
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

    /// Clamp active-view scroll or selection state to the current viewport size.
    pub fn clamp(&mut self, viewport: Size) {
        match self {
            Self::Log(view) => view.clamp(),
            Self::Show(view) => view.clamp(viewport),
            Self::Diff(view) => view.clamp(viewport),
            Self::Status(view) => view.clamp(viewport.height),
            Self::Resolve(view) => view.clamp(),
            Self::FileList(view) => view.clamp(),
            Self::FileShow(view) => view.clamp(viewport),
            Self::Bookmarks(view) => view.clamp(),
            Self::Workspaces(view) => view.clamp(),
            Self::OperationLog(view) => view.clamp(),
            Self::OperationDetail(view) => view.clamp(viewport.height),
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
}
