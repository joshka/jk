use color_eyre::Result;

use crate::jj::LogViewMode;

use super::ViewState;

impl ViewState {
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
}
