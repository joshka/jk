//! App-facing dispatcher over the concrete view slices.
//!
//! Feature modules own their state, rendering, bindings, and tests. `ViewState`
//! only routes rendering and command execution to the active slice.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::action_menu::ExactActionContext;
use crate::bookmarks::BookmarksView;
use crate::command::{Binding, CommandContext, HelpContext, ViewCommand, ViewEffect};
use crate::diff::DiffView;
use crate::file_list::FileListView;
use crate::file_show::FileShowView;
use crate::graph::GraphView;
use crate::jj::{JjCommand, LogViewMode, ViewSpec};
use crate::jj_actions::{JjBookmarkForgetTarget, JjBookmarkTarget, JjGitPushTarget};
use crate::operation_detail::OperationDetailView;
use crate::operation_log::OperationLogView;
use crate::resolve::ResolveView;
use crate::search::SearchQuery;
use crate::show::ShowView;
use crate::status::StatusView;
use crate::tui::StatusHints;
use crate::view_action_targets::ViewActionTargets;
use crate::workspaces::WorkspacesView;

/// The currently active top-level view.
pub enum ViewState {
    Graph(GraphView),
    Show(ShowView),
    Diff(DiffView),
    Status(StatusView),
    Resolve(ResolveView),
    FileList(FileListView),
    FileShow(FileShowView),
    Bookmarks(BookmarksView),
    Workspaces(WorkspacesView),
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

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        match self {
            Self::Graph(view) => view.render(frame, area, search),
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

    pub fn bindings(&self) -> &'static [Binding] {
        match self {
            Self::Graph(view) => view.bindings(),
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

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match self {
            Self::Graph(view) => view.execute(command, context),
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

    pub fn refresh(&mut self) -> Result<()> {
        match self {
            Self::Graph(view) => view.refresh(),
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

    pub fn clamp(&mut self, viewport_height: u16, viewport_width: u16) {
        match self {
            Self::Graph(view) => view.clamp(),
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

    pub fn spec(&self) -> &ViewSpec {
        match self {
            Self::Graph(view) => view.spec(),
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

    pub fn status_hints(&self) -> StatusHints {
        match self {
            Self::Graph(_) => StatusHints::Graph,
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

    pub fn help_context(&self) -> HelpContext {
        match self {
            Self::Graph(_) => HelpContext::Graph,
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
            Self::Graph(_) => 0,
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
            Self::Graph(_) => {}
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
            Self::Graph(view) => Some(view.item_count()),
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

    pub fn graph_mode_label(&self) -> Option<&str> {
        match self {
            Self::Graph(view) => Some(view.mode_label()),
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

    pub fn set_graph_mode(&mut self, mode: LogViewMode) -> Result<()> {
        match self {
            Self::Graph(view) => view.set_mode(mode),
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
            Self::Graph(view) => view.line_count(),
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
mod tests {
    use super::*;
    use crate::bookmarks;
    use crate::graph;

    #[test]
    fn push_target_from_graph_uses_exact_revision() {
        let view = ViewState::Graph(graph::GraphView::test_new(vec![
            crate::jj_rows::LogItem::new(Vec::new(), Some("abcdefg".to_owned()), None),
        ]));

        assert_eq!(
            view.push_target().unwrap(),
            Some(JjGitPushTarget::Revision("abcdefg".to_owned()))
        );
    }

    #[test]
    fn push_target_from_graph_requires_exact_revision() {
        let view = ViewState::Graph(graph::GraphView::test_new(vec![
            crate::jj_rows::LogItem::new(Vec::new(), None, None),
        ]));

        assert_eq!(
            view.push_target().unwrap_err().to_string(),
            "push from graph requires a selected row with an exact revision"
        );
    }

    #[test]
    fn push_target_from_bookmarks_uses_selected_name() {
        let view = ViewState::Bookmarks(bookmarks::BookmarksView::test_new(vec![
            crate::jj_rows::BookmarkItem::new(Vec::new(), "main".to_owned(), None, None),
        ]));

        assert_eq!(
            view.push_target().unwrap(),
            Some(JjGitPushTarget::Bookmark("main".to_owned()))
        );
    }

    #[test]
    fn bookmark_target_from_graph_and_status_is_exact() {
        let view = ViewState::Graph(graph::GraphView::test_new(vec![
            crate::jj_rows::LogItem::new(Vec::new(), Some("abcdefg".to_owned()), None),
        ]));

        assert_eq!(
            view.bookmark_target().unwrap(),
            Some(JjBookmarkTarget::exact_change("abcdefg"))
        );

        let view = ViewState::Status(crate::status::StatusView::test_new(&[]));

        assert_eq!(
            view.bookmark_target().unwrap(),
            Some(JjBookmarkTarget::current_working_copy())
        );
    }

    #[test]
    fn bookmark_target_from_graph_requires_exact_revision() {
        let view = ViewState::Graph(graph::GraphView::test_new(vec![
            crate::jj_rows::LogItem::new(Vec::new(), None, None),
        ]));

        assert_eq!(
            view.bookmark_target().unwrap_err().to_string(),
            "bookmark mutation from graph requires a selected row with an exact revision"
        );
    }

    #[test]
    fn selected_local_bookmark_name_rejects_nonlocal_rows() {
        let view = ViewState::Bookmarks(bookmarks::BookmarksView::test_new(vec![
            crate::jj_rows::BookmarkItem::new(Vec::new(), "@origin".to_owned(), None, None)
                .with_local(false),
        ]));

        assert_eq!(
            view.selected_local_bookmark_name().unwrap_err().to_string(),
            "delete requires a selected exact local bookmark"
        );
    }

    #[test]
    fn exact_restore_revert_context_uses_graph_derived_detail_target_and_path() {
        let show = ViewState::Show(crate::show::ShowView::test_new(ViewSpec::show(
            "abcdefg".to_owned(),
            crate::jj::DiffFormat::Default,
        )));
        let file_list = ViewState::FileList(crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("abcdefg".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj_rows::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ));
        let file_show = ViewState::FileShow(crate::file_show::FileShowView::new(
            ViewSpec::file_show(Some("abcdefg".to_owned()), "src/main.rs".to_owned())
                .with_exact_change_target(),
            "src/main.rs",
            crate::rendered_jj::DocumentLines::new(Vec::new()),
        ));

        assert_eq!(
            show.exact_restore_revert_context().unwrap(),
            Some(ExactActionContext::detail("abcdefg"))
        );
        assert_eq!(
            file_list.exact_restore_revert_context().unwrap(),
            Some(ExactActionContext::detail("abcdefg").with_selected_path("src/main.rs"))
        );
        assert_eq!(
            file_show.exact_restore_revert_context().unwrap(),
            Some(ExactActionContext::detail("abcdefg").with_selected_path("src/main.rs"))
        );
    }

    #[test]
    fn exact_restore_revert_context_uses_status_selected_path_at_working_copy() {
        let mut status =
            crate::status::StatusView::test_new(&["Working copy changes:", "M src/status.rs"]);
        status.scroll_down(4, 1);
        let view = ViewState::Status(status);

        assert_eq!(
            view.exact_restore_revert_context().unwrap(),
            Some(ExactActionContext::status_path("src/status.rs"))
        );
    }

    #[test]
    fn exact_restore_revert_context_rejects_ambiguous_status_row() {
        let view = ViewState::Status(crate::status::StatusView::test_new(&[
            "R {old.rs => new.rs}",
        ]));

        assert_eq!(
            view.exact_restore_revert_context().unwrap_err().to_string(),
            "status file action unavailable: renamed status rows contain multiple paths"
        );
    }

    #[test]
    fn exact_restore_revert_context_rejects_direct_startup_detail_revsets() {
        let show = ViewState::Show(crate::show::ShowView::test_new(ViewSpec::new(
            JjCommand::Show,
            vec!["main".to_owned()],
        )));
        let diff = ViewState::Diff(crate::diff::DiffView::test_new(ViewSpec::new(
            JjCommand::Diff,
            vec!["-r".to_owned(), "main".to_owned()],
        )));
        let file_list = ViewState::FileList(crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned())),
            vec![crate::jj_rows::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ));
        let file_show = ViewState::FileShow(crate::file_show::FileShowView::new(
            ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned()),
            "src/main.rs",
            crate::rendered_jj::DocumentLines::new(Vec::new()),
        ));

        assert_eq!(
            show.exact_restore_revert_context().unwrap_err().to_string(),
            "restore/revert from jk show main requires an exact graph-derived revision target"
        );
        assert_eq!(
            diff.exact_restore_revert_context().unwrap_err().to_string(),
            "restore/revert from jk diff -r main requires an exact graph-derived revision target"
        );
        assert_eq!(
            file_list
                .exact_restore_revert_context()
                .unwrap_err()
                .to_string(),
            "file actions from jk file list -r main require a working-copy file list or exact graph-derived revision target"
        );
        assert_eq!(
            file_show
                .exact_restore_revert_context()
                .unwrap_err()
                .to_string(),
            "file actions from jk file show -r main src/main.rs require a working-copy file show or exact graph-derived revision target"
        );
    }
}
