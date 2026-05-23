use ratatui::layout::Size;

use crate::actions::JjOperationRecoveryKind;
use crate::jj::JjCommand;
use crate::menus::{ActionMenu, CopyOption};
use crate::search::SearchQuery;

/// App-level dispatch vocabulary for global bindings and view-facing effects.
///
/// `App` matches these variants through top-level binding groups first. A
/// `Command::View` value is routed to the active view as a [`ViewCommand`].
/// Add a variant here only when app dispatch or a shared binding table needs a
/// stable command identity; feature-local policy still belongs in the owning
/// view or app submodule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Quit,
    Help,
    SearchPrompt,
    PromptLogRevset,
    OpenStatus,
    OpenResolve,
    OpenBookmarks,
    OpenWorkspaces,
    OpenOperationLog,
    OperationUndo,
    OperationRedo,
    Edit,
    NextEdit,
    PrevEdit,
    Describe,
    Commit,
    BookmarkCreate,
    BookmarkSet,
    BookmarkMove,
    BookmarkRename,
    BookmarkDelete,
    BookmarkForget,
    BookmarkTrack,
    BookmarkUntrack,
    Fetch,
    FetchRemote,
    Push,
    Copy,
    ViewFormat,
    Refresh,
    Back,
    SwitchLog,
    SwitchDefault,
    View(ViewCommand),
}

/// View-local commands that may inspect the current viewport and search state.
///
/// These commands stay on the presentation side of the boundary: they can
/// return a `ViewEffect`, but `App` owns the actual state transition that
/// follows. The active view may ignore commands it does not support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewCommand {
    CycleMode,
    NewTrunk,
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    MoveFirst,
    MoveLast,
    ToggleWrap,
    ScrollLeft,
    ScrollRight,
    NextFile,
    PreviousFile,
    OpenFiles,
    OpenItem,
    OpenShow,
    OpenDiff,
    StartSearch,
    NextSearchMatch,
    PreviousSearchMatch,
    ToggleSelect,
    OpenActionMenu,
    Copy,
}

impl Command {
    /// Return the jj operation recovery kind represented by this global command, if any.
    ///
    /// Recovery target availability belongs to the operation-log feature. This
    /// conversion only keeps the shared command identity connected to the app
    /// action flow once a recovery command has already been accepted.
    pub fn operation_recovery(self) -> Option<JjOperationRecoveryKind> {
        match self {
            Self::OperationUndo => Some(JjOperationRecoveryKind::Undo),
            Self::OperationRedo => Some(JjOperationRecoveryKind::Redo),
            Self::Quit
            | Self::Help
            | Self::SearchPrompt
            | Self::PromptLogRevset
            | Self::OpenStatus
            | Self::OpenResolve
            | Self::OpenBookmarks
            | Self::OpenWorkspaces
            | Self::OpenOperationLog
            | Self::Edit
            | Self::NextEdit
            | Self::PrevEdit
            | Self::Describe
            | Self::Commit
            | Self::BookmarkCreate
            | Self::BookmarkSet
            | Self::BookmarkMove
            | Self::BookmarkRename
            | Self::BookmarkDelete
            | Self::BookmarkForget
            | Self::BookmarkTrack
            | Self::BookmarkUntrack
            | Self::Fetch
            | Self::FetchRemote
            | Self::Push
            | Self::Copy
            | Self::ViewFormat
            | Self::Refresh
            | Self::Back
            | Self::SwitchLog
            | Self::SwitchDefault
            | Self::View(_) => None,
        }
    }
}

/// Snapshot of the active viewport and search state for one view dispatch.
///
/// Immediate key dispatch reuses the main viewport from the last completed
/// frame so height and width stay consistent for one event-loop turn. View
/// code must treat this as read-only input for the current dispatch instead of
/// retained state.
pub struct CommandContext<'a> {
    /// Current content viewport size in terminal cells.
    pub size: Size,
    /// Active search query, if search is currently scoped to the view.
    pub search: Option<&'a SearchQuery>,
}

impl CommandContext<'_> {
    /// Return the page jump size used by view-local page movement.
    ///
    /// Page movement keeps one row of overlap and never returns zero, even for
    /// very small terminal viewports.
    pub fn page_size(&self) -> usize {
        usize::from(self.size.height.saturating_sub(1).max(1))
    }
}

/// One-way output from a view command back to the app dispatcher.
///
/// The app interprets these effects and performs the resulting navigation,
/// refresh, status update, search update, copy menu, or action menu transition.
/// Views do not mutate app-owned state directly or run `jj` mutation flows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewEffect {
    /// The view does not support the command in the current state.
    Ignored,
    /// The view handled the command without further app work.
    Handled,
    /// Normal status text that app dispatch should surface.
    StatusMessage(String),
    /// Error status text that app dispatch should surface with error styling.
    StatusError(String),
    /// Request that the app open a new child change at trunk.
    RunNewTrunk,
    /// Request that the app open a detail view rooted at the selected target.
    OpenDetail(JjCommand, String),
    /// Request that the app open an already constructed top-level view spec.
    OpenView(crate::jj::ViewSpec),
    /// Search moved to the next or previous match.
    SearchMoved,
    SearchStarted {
        /// Number of matches found when search was first started.
        matches: usize,
    },
    /// Copyable options for the app-owned copy menu.
    CopyOptions(Vec<CopyOption>),
    /// Action menu for the currently selected row or path.
    OpenActionMenu(ActionMenu),
}
