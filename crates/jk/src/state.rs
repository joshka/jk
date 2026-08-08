use std::time::{Duration, Instant};

use jk_cli::{
    DiffQuery, EvologQuery, JjLog, LogTemplateSelection, OperationQuery, ShowQuery, StatusQuery,
    WorkspaceInspectionQuery,
};
use jk_core::CommandHistory;
use jk_tui::bookmark_view::BookmarkView;
use jk_tui::command_discovery::{ActionMenuAction, BindingContext};
use jk_tui::command_history_view::CommandHistoryView;
use jk_tui::diff_view::DiffView;
use jk_tui::log_view::LogView;
use jk_tui::operation_log_view::OperationLogView;
use jk_tui::rendered_view::RenderedView;
use jk_tui::workspaces_view::WorkspacesView;

use crate::mutation_preview::PendingCommandPreview;

const TOAST_DURATION: Duration = Duration::from_secs(3);

#[derive(Debug)]
struct Toast {
    message: String,
    expires_at: Instant,
}

/// Active top-level application view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppView {
    Bookmarks {
        view: BookmarkView,
    },
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
    log_source_stack: Vec<JjLog>,
    toast: Option<Toast>,
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
            log_source_stack: Vec::new(),
            toast: None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn command_history(&self) -> &CommandHistory {
        &self.history
    }

    pub(crate) fn push_log_source(&mut self, source: JjLog) {
        self.log_source_stack.push(source);
    }

    pub(crate) fn can_pop_log_drill(&self) -> bool {
        self.views.active_is_log_with_log_parent() && !self.log_source_stack.is_empty()
    }

    pub(crate) fn pop_log_drill(&mut self, source: &mut JjLog) -> bool {
        if !self.can_pop_log_drill() {
            return false;
        }
        let Some(previous_source) = self.log_source_stack.pop() else {
            return false;
        };
        if !self.views.pop() {
            self.log_source_stack.push(previous_source);
            return false;
        }
        *source = previous_source;
        true
    }

    /// Shows a short-lived completion notice without changing the active view's persistent chrome.
    pub(crate) fn show_toast(&mut self, message: impl Into<String>) {
        self.show_toast_until(message, Instant::now() + TOAST_DURATION);
    }

    /// Returns the active completion notice, removing it when it has expired.
    pub(crate) fn toast_message(&mut self) -> Option<&str> {
        self.toast_message_at(Instant::now())
    }

    /// Returns how long the terminal loop should wait before redrawing to remove the toast.
    pub(crate) fn toast_timeout(&self) -> Option<Duration> {
        self.toast
            .as_ref()
            .map(|toast| toast.expires_at.saturating_duration_since(Instant::now()))
    }

    fn show_toast_until(&mut self, message: impl Into<String>, expires_at: Instant) {
        self.toast = Some(Toast {
            message: message.into(),
            expires_at,
        });
    }

    fn toast_message_at(&mut self, now: Instant) -> Option<&str> {
        let expired = self
            .toast
            .as_ref()
            .is_some_and(|toast| toast.expires_at <= now);
        if expired {
            self.toast = None;
        }
        self.toast.as_ref().map(|toast| toast.message.as_str())
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

    fn active_is_log_with_log_parent(&self) -> bool {
        if self.views.len() < 2 {
            return false;
        }
        matches!(self.views.last(), Some(AppView::Log(_)))
            && matches!(
                self.views.get(self.views.len().saturating_sub(2)),
                Some(AppView::Log(_))
            )
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
    ActionMenu {
        context: BindingContext,
        selected: usize,
    },
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
        scroll_offset: usize,
    },
    DescribeMessage {
        rev: String,
        message: crate::description_editor::DescriptionEditor,
    },
    BookmarkMutation {
        kind: BookmarkMutationKind,
        name: String,
        revision: String,
        field: BookmarkMutationField,
    },
    RemotePicker {
        names: Vec<String>,
        selected: usize,
        bookmark: Option<String>,
    },
    CommandPreview {
        pending: PendingCommandPreview,
    },
    AbandonConfirmation {
        pending: PendingCommandPreview,
        dialog: Box<crate::abandon_confirmation::AbandonConfirmation>,
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

/// Bookmark mutation prompt kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BookmarkMutationKind {
    Create,
    Move,
}

/// Field edited by the bookmark mutation prompt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BookmarkMutationField {
    Name,
    Revision,
}

/// Whether an input-mode handler consumed a key event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputModeResult {
    Handled,
    Unhandled,
    Action(ActionMenuAction),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_expires_at_its_deadline() {
        let mut state = AppState::new(AppView::Log(LogView::default()));
        let now = Instant::now();
        state.show_toast_until("Created new change", now + TOAST_DURATION);

        assert_eq!(state.toast_message_at(now), Some("Created new change"));
        assert_eq!(state.toast_message_at(now + TOAST_DURATION), None);
    }
}
