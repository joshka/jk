use jk_cli::{
    DiffQuery, EvologQuery, LogTemplateSelection, OperationQuery, ShowQuery, StatusQuery,
    WorkspaceInspectionQuery,
};
use jk_core::CommandHistory;
use jk_tui::command_discovery::BindingContext;
use jk_tui::command_history_view::CommandHistoryView;
use jk_tui::diff_view::DiffView;
use jk_tui::log_view::LogView;
use jk_tui::operation_log_view::OperationLogView;
use jk_tui::rendered_view::RenderedView;
use jk_tui::workspaces_view::WorkspacesView;

use crate::mutation_preview::PendingCommandPreview;

/// Active top-level application view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppView {
    Log(LogView),
    Diff {
        view: DiffView,
        query: DiffQuery,
    },
    Show {
        view: RenderedView,
        query: ShowQuery,
    },
    Evolog {
        view: RenderedView,
        query: EvologQuery,
    },
    Status {
        view: RenderedView,
        query: StatusQuery,
    },
    Workspaces {
        view: WorkspacesView,
    },
    CommandHistory {
        view: CommandHistoryView,
    },
    CommandHistoryDetails {
        view: RenderedView,
    },
    CommandOutput {
        view: RenderedView,
        input: String,
    },
    OperationLog {
        view: OperationLogView,
    },
    OperationShow {
        view: RenderedView,
        query: OperationQuery,
    },
    OperationDiff {
        view: RenderedView,
        query: OperationQuery,
    },
    WorkspaceStatus {
        view: RenderedView,
        query: WorkspaceInspectionQuery,
    },
    WorkspaceLog {
        view: RenderedView,
        query: WorkspaceInspectionQuery,
    },
    WorkspaceDiff {
        view: RenderedView,
        query: WorkspaceInspectionQuery,
    },
}

/// Application state owned by the terminal loop.
#[derive(Debug)]
pub struct AppState {
    pub(crate) views: ViewStack,
    pub(crate) modes: ModeStack,
    pub(crate) history: CommandHistory,
}

impl AppState {
    #[cfg(test)]
    pub(crate) fn new(root: AppView) -> Self {
        Self::with_history(root, CommandHistory::default())
    }

    pub(crate) fn with_history(root: AppView, history: CommandHistory) -> Self {
        Self {
            views: ViewStack::new(root),
            modes: ModeStack::default(),
            history,
        }
    }

    #[cfg(test)]
    pub(crate) const fn command_history(&self) -> &CommandHistory {
        &self.history
    }
}

/// Non-empty stack of top-level views.
#[derive(Debug)]
pub struct ViewStack {
    views: Vec<AppView>,
}

impl ViewStack {
    pub(crate) fn new(root: AppView) -> Self {
        Self { views: vec![root] }
    }

    pub(crate) fn active(&self) -> &AppView {
        match self.views.last() {
            Some(view) => view,
            None => panic!("view stack always keeps one root view"),
        }
    }

    pub(crate) fn active_mut(&mut self) -> &mut AppView {
        match self.views.last_mut() {
            Some(view) => view,
            None => panic!("view stack always keeps one root view"),
        }
    }

    pub(crate) fn push(&mut self, view: AppView) {
        self.views.push(view);
    }

    pub(crate) fn pop(&mut self) -> bool {
        if self.views.len() == 1 {
            return false;
        }

        self.views.pop();
        true
    }

    #[cfg(test)]
    pub(crate) const fn len(&self) -> usize {
        self.views.len()
    }
}

/// Stack of transient prompt-like modes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModeStack {
    modes: Vec<InputMode>,
}

impl ModeStack {
    pub(crate) fn active(&self) -> Option<&InputMode> {
        self.modes.last()
    }

    pub(crate) fn active_mut(&mut self) -> Option<&mut InputMode> {
        self.modes.last_mut()
    }

    pub(crate) fn push(&mut self, mode: InputMode) {
        self.modes.push(mode);
    }

    pub(crate) fn pop(&mut self) -> Option<InputMode> {
        self.modes.pop()
    }
}

/// Transient input modes owned by the terminal loop.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputMode {
    ViewOptions {
        context: BindingContext,
        selected: usize,
    },
    DiffFileList {
        selected: usize,
    },
    DiffSearch {
        query: String,
    },
    InspectionSearch {
        query: String,
    },
    CommandDiscovery {
        context: BindingContext,
        query: String,
        selected: usize,
    },
    DescribeMessage {
        rev: String,
        message: String,
    },
    CommandPreview {
        pending: PendingCommandPreview,
    },
    JjCommand {
        input: String,
        error: Option<String>,
    },
    LogTemplate {
        options: Vec<LogTemplateSelection>,
        selected: usize,
    },
}

/// Whether an input-mode handler consumed a key event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputModeResult {
    Handled,
    Unhandled,
}
