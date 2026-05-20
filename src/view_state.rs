//! App-facing dispatcher over the concrete view slices.
//!
//! Feature modules own their state, rendering, bindings, and tests. `ViewState`
//! only routes rendering and command execution to the active slice.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::bookmarks::BookmarksView;
use crate::command::{Binding, CommandContext, HelpContext, ViewCommand, ViewEffect};
use crate::diff::DiffView;
use crate::file_list::FileListView;
use crate::file_show::FileShowView;
use crate::graph::GraphView;
use crate::jj::{JjCommand, JjGitPushTarget, LogViewMode, ViewSpec};
use crate::operation_detail::OperationDetailView;
use crate::operation_log::OperationLogView;
use crate::search::SearchQuery;
use crate::show::ShowView;
use crate::status::StatusView;
use crate::tui::StatusHints;

/// The currently active top-level view.
pub enum ViewState {
    Graph(GraphView),
    Show(ShowView),
    Diff(DiffView),
    Status(StatusView),
    FileList(FileListView),
    FileShow(FileShowView),
    Bookmarks(BookmarksView),
    OperationLog(OperationLogView),
    OperationDetail(OperationDetailView),
}

impl ViewState {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        match spec.command() {
            JjCommand::Default | JjCommand::Log => Ok(Self::Graph(GraphView::load(spec)?)),
            JjCommand::Show => Ok(Self::Show(ShowView::load(spec)?)),
            JjCommand::Diff => Ok(Self::Diff(DiffView::load(spec)?)),
            JjCommand::Status => Ok(Self::Status(StatusView::load(spec)?)),
            JjCommand::FileList => Ok(Self::FileList(FileListView::load(spec)?)),
            JjCommand::FileShow => Ok(Self::FileShow(FileShowView::load(spec)?)),
            JjCommand::Bookmarks => Ok(Self::Bookmarks(BookmarksView::load(spec)?)),
            JjCommand::OperationLog => Ok(Self::OperationLog(OperationLogView::load(spec)?)),
            JjCommand::OperationShow | JjCommand::OperationDiff => {
                Ok(Self::OperationDetail(OperationDetailView::load(spec)?))
            }
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        match self {
            Self::Graph(view) => view.render(frame, area, search),
            Self::Show(view) => view.render(frame, area, search),
            Self::Diff(view) => view.render(frame, area, search),
            Self::Status(view) => view.render(frame, area, search),
            Self::FileList(view) => view.render(frame, area, search),
            Self::FileShow(view) => view.render(frame, area, search),
            Self::Bookmarks(view) => view.render(frame, area, search),
            Self::OperationLog(view) => view.render(frame, area, search),
            Self::OperationDetail(view) => view.render(frame, area, search),
        }
    }

    pub fn bindings(&self) -> &'static [Binding] {
        match self {
            Self::Graph(view) => view.bindings(),
            Self::Show(view) => view.bindings(),
            Self::Diff(view) => view.bindings(),
            Self::Status(view) => view.bindings(),
            Self::FileList(view) => view.bindings(),
            Self::FileShow(view) => view.bindings(),
            Self::Bookmarks(view) => view.bindings(),
            Self::OperationLog(view) => view.bindings(),
            Self::OperationDetail(view) => view.bindings(),
        }
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match self {
            Self::Graph(view) => view.execute(command, context),
            Self::Show(view) => view.execute(command, context),
            Self::Diff(view) => view.execute(command, context),
            Self::Status(view) => view.execute(command, context),
            Self::FileList(view) => view.execute(command, context),
            Self::FileShow(view) => view.execute(command, context),
            Self::Bookmarks(view) => view.execute(command, context),
            Self::OperationLog(view) => view.execute(command, context),
            Self::OperationDetail(view) => view.execute(command, context),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        match self {
            Self::Graph(view) => view.refresh(),
            Self::Show(view) => view.refresh(),
            Self::Diff(view) => view.refresh(),
            Self::Status(view) => view.refresh(),
            Self::FileList(view) => view.refresh(),
            Self::FileShow(view) => view.refresh(),
            Self::Bookmarks(view) => view.refresh(),
            Self::OperationLog(view) => view.refresh(),
            Self::OperationDetail(view) => view.refresh(),
        }
    }

    pub fn clamp(&mut self, viewport_height: u16) {
        match self {
            Self::Graph(view) => view.clamp(),
            Self::Show(view) => view.clamp(viewport_height),
            Self::Diff(view) => view.clamp(viewport_height),
            Self::Status(view) => view.clamp(viewport_height),
            Self::FileList(view) => view.clamp(),
            Self::FileShow(view) => view.clamp(viewport_height),
            Self::Bookmarks(view) => view.clamp(),
            Self::OperationLog(view) => view.clamp(),
            Self::OperationDetail(view) => view.clamp(viewport_height),
        }
    }

    pub fn spec(&self) -> &ViewSpec {
        match self {
            Self::Graph(view) => view.spec(),
            Self::Show(view) => view.spec(),
            Self::Diff(view) => view.spec(),
            Self::Status(view) => view.spec(),
            Self::FileList(view) => view.spec(),
            Self::FileShow(view) => view.spec(),
            Self::Bookmarks(view) => view.spec(),
            Self::OperationLog(view) => view.spec(),
            Self::OperationDetail(view) => view.spec(),
        }
    }

    pub fn status_hints(&self) -> StatusHints {
        match self {
            Self::Graph(_) => StatusHints::Graph,
            Self::Show(_) => StatusHints::ShowDocument,
            Self::Diff(_) => StatusHints::DiffDocument,
            Self::Status(_) => StatusHints::Status,
            Self::FileList(_) => StatusHints::FileList,
            Self::FileShow(_) => StatusHints::FileShowDocument,
            Self::Bookmarks(_) => StatusHints::Bookmarks,
            Self::OperationLog(_) => StatusHints::OperationLog,
            Self::OperationDetail(_) => StatusHints::OperationDetailDocument,
        }
    }

    pub fn help_context(&self) -> HelpContext {
        match self {
            Self::Graph(_) => HelpContext::Graph,
            Self::Show(_) => HelpContext::Show,
            Self::Diff(_) => HelpContext::Diff,
            Self::Status(_) => HelpContext::Status,
            Self::FileList(_) => HelpContext::FileList,
            Self::FileShow(_) => HelpContext::FileShow,
            Self::Bookmarks(_) => HelpContext::Bookmarks,
            Self::OperationLog(_) => HelpContext::OperationLog,
            Self::OperationDetail(_) => HelpContext::OperationDetail,
        }
    }

    pub fn command(&self) -> JjCommand {
        self.spec().command()
    }

    pub fn scroll_offset(&self) -> usize {
        match self {
            Self::Graph(_) => 0,
            Self::Show(view) => view.scroll_offset(),
            Self::Diff(view) => view.scroll_offset(),
            Self::Status(view) => view.scroll_offset(),
            Self::FileList(_) => 0,
            Self::FileShow(view) => view.scroll_offset(),
            Self::Bookmarks(_) => 0,
            Self::OperationLog(_) => 0,
            Self::OperationDetail(view) => view.scroll_offset(),
        }
    }

    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        match self {
            Self::Graph(_) => {}
            Self::Show(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Diff(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Status(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::FileList(_) => {}
            Self::FileShow(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Bookmarks(_) => {}
            Self::OperationLog(_) => {}
            Self::OperationDetail(view) => view.set_scroll_offset(viewport_height, scroll_offset),
        }
    }

    pub fn item_count(&self) -> Option<usize> {
        match self {
            Self::Graph(view) => Some(view.item_count()),
            Self::FileList(view) => Some(view.item_count()),
            Self::Bookmarks(view) => Some(view.item_count()),
            Self::OperationLog(view) => Some(view.item_count()),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::FileShow(_)
            | Self::OperationDetail(_) => None,
        }
    }

    pub fn graph_mode_label(&self) -> Option<&str> {
        match self {
            Self::Graph(view) => Some(view.mode_label()),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::FileList(_)
            | Self::FileShow(_)
            | Self::Bookmarks(_)
            | Self::OperationLog(_)
            | Self::OperationDetail(_) => None,
        }
    }

    pub fn set_graph_mode(&mut self, mode: LogViewMode) -> Result<()> {
        match self {
            Self::Graph(view) => view.set_mode(mode),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::FileList(_)
            | Self::FileShow(_)
            | Self::Bookmarks(_)
            | Self::OperationLog(_)
            | Self::OperationDetail(_) => Ok(()),
        }
    }

    pub fn reveal_graph_change(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        match self {
            Self::Graph(view) => view.reveal_change_id(change_id, fallback_mode),
            Self::Show(_)
            | Self::Diff(_)
            | Self::Status(_)
            | Self::FileList(_)
            | Self::FileShow(_)
            | Self::Bookmarks(_)
            | Self::OperationLog(_)
            | Self::OperationDetail(_) => Ok(false),
        }
    }

    pub fn document_line_count(&self) -> usize {
        match self {
            Self::Graph(view) => view.line_count(),
            Self::Show(view) => view.line_count(),
            Self::Diff(view) => view.line_count(),
            Self::Status(view) => view.line_count(),
            Self::FileList(view) => view.line_count(),
            Self::FileShow(view) => view.line_count(),
            Self::Bookmarks(view) => view.line_count(),
            Self::OperationLog(view) => view.line_count(),
            Self::OperationDetail(view) => view.line_count(),
        }
    }

    pub fn push_target(&self) -> Result<Option<JjGitPushTarget>> {
        match self {
            Self::Graph(view) => view
                .selected_revision()
                .map(|revision| JjGitPushTarget::Revision(revision.to_owned()))
                .map_or_else(
                    || {
                        Err(color_eyre::eyre::eyre!(
                            "push from graph requires a selected row with an exact revision"
                        ))
                    },
                    |target| Ok(Some(target)),
                ),
            Self::Bookmarks(view) => view
                .selected_bookmark_name()
                .map(|name| JjGitPushTarget::Bookmark(name.to_owned()))
                .map_or_else(
                    || {
                        Err(color_eyre::eyre::eyre!(
                            "selected bookmark has no target name for push"
                        ))
                    },
                    |target| Ok(Some(target)),
                ),
            Self::Status(_) => Ok(Some(JjGitPushTarget::Status)),
            Self::Show(_)
            | Self::Diff(_)
            | Self::FileList(_)
            | Self::FileShow(_)
            | Self::OperationLog(_)
            | Self::OperationDetail(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookmarks;
    use crate::graph;

    #[test]
    fn push_target_from_graph_uses_exact_revision() {
        let view = ViewState::Graph(graph::GraphView::test_new(vec![crate::jj::LogItem::new(
            Vec::new(),
            Some("abcdefg".to_owned()),
            None,
        )]));

        assert_eq!(
            view.push_target().unwrap(),
            Some(JjGitPushTarget::Revision("abcdefg".to_owned()))
        );
    }

    #[test]
    fn push_target_from_graph_requires_exact_revision() {
        let view = ViewState::Graph(graph::GraphView::test_new(vec![crate::jj::LogItem::new(
            Vec::new(),
            None,
            None,
        )]));

        assert_eq!(
            view.push_target().unwrap_err().to_string(),
            "push from graph requires a selected row with an exact revision"
        );
    }

    #[test]
    fn push_target_from_bookmarks_uses_selected_name() {
        let view = ViewState::Bookmarks(bookmarks::BookmarksView::test_new(vec![
            crate::jj::BookmarkItem::new(Vec::new(), "main".to_owned(), None, None),
        ]));

        assert_eq!(
            view.push_target().unwrap(),
            Some(JjGitPushTarget::Bookmark("main".to_owned()))
        );
    }
}
