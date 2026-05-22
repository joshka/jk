//! Shared fixtures and service mocks for app orchestration tests.
//!
//! The behavior modules import this support surface so their tests can stay focused on the app
//! contract they exercise instead of rebuilding service seams and fixture views.

mod fixtures;
mod services;

pub use super::super::{APP_BINDINGS, App};
#[allow(unused_imports)]
pub use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan,
    JjBookmarkTarget, JjCommitPlan, JjDescribePlan, JjDescribeTarget, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery,
    JjOperationRecoveryKind, JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan,
    JjSplitPlan, JjSquashPlan, JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan,
};
pub use crate::app::actions::{ActionPane, action_pane_visible_lines};
pub use crate::app::input::{rebase_plan_from_prompt, squash_plan_from_prompt};
pub use crate::app::navigation::startup::initial_view;
pub use crate::app::services::AppServices;
pub use crate::app::status_line::{StatusKind, StatusLine};
#[allow(unused_imports)]
pub use crate::bookmarks::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState, load_bookmark_entries,
};
pub use crate::command::{CommandContext, ViewCommand};
#[allow(unused_imports)]
pub use crate::files::list::{FileListItem, load_file_list_entries};
#[allow(unused_imports)]
pub use crate::jj::{DiffFormat, JjCommand, LogViewMode, ViewSpec};
#[allow(unused_imports)]
pub use crate::log::{LogItem, load_compact_log_context, load_entries};
pub use crate::menus::{ActionKind, FollowUp, RolePrompt, RolePromptOption};
pub use crate::modes::{InteractionMode, ViewMenuAction};
#[allow(unused_imports)]
pub use crate::rendered_rows::document_plain_text;
#[allow(unused_imports)]
pub use crate::resolve::{ResolveEntry, load_resolve_entries};
pub use crate::tui::Overlay;
pub use crate::view_state::ViewState;
#[allow(unused_imports)]
pub use crate::workspaces::{WorkspaceContext, WorkspaceItem, load_workspace_context};
pub use color_eyre::Result;
pub use color_eyre::eyre::eyre;
pub use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
pub use fixtures::{key, log_item, mock_load_view, test_app};
pub use ratatui::DefaultTerminal;
pub use services::*;
pub use std::sync::atomic::{AtomicUsize, Ordering};
pub use std::time::{Duration, Instant};
