//! Shared fixtures and service mocks for app orchestration tests.
//!
//! The behavior modules import this support surface so their tests can stay focused on the app
//! contract they exercise instead of rebuilding service seams and fixture views.

mod fixtures;
mod services;

pub(super) use super::super::{APP_BINDINGS, App};
#[allow(unused_imports)]
pub(super) use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan,
    JjBookmarkTarget, JjCommitPlan, JjDescribePlan, JjDescribeTarget, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery,
    JjOperationRecoveryKind, JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan,
    JjSplitPlan, JjSquashPlan, JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan,
};
pub(super) use crate::app::actions::{ActionPane, action_pane_visible_lines};
pub(super) use crate::app::input::{rebase_plan_from_prompt, squash_plan_from_prompt};
pub(super) use crate::app::navigation::startup::initial_view;
pub(super) use crate::app::services::AppServices;
pub(super) use crate::app::status_line::{StatusKind, StatusLine};
#[allow(unused_imports)]
pub(super) use crate::bookmarks::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState, load_bookmark_entries,
};
pub(super) use crate::command::{CommandContext, ViewCommand};
#[allow(unused_imports)]
pub(super) use crate::files::list::{FileListItem, load_file_list_entries};
#[allow(unused_imports)]
pub(super) use crate::jj::{DiffFormat, JjCommand, LogViewMode, ViewSpec};
#[allow(unused_imports)]
pub(super) use crate::log::{LogItem, load_compact_log_context, load_entries};
pub(super) use crate::menus::{ActionKind, FollowUp, RolePrompt, RolePromptOption};
pub(super) use crate::modes::{InteractionMode, ViewMenuAction};
#[allow(unused_imports)]
pub(super) use crate::rendered_rows::document_plain_text;
#[allow(unused_imports)]
pub(super) use crate::resolve::{ResolveEntry, load_resolve_entries};
pub(super) use crate::tui::Overlay;
pub(super) use crate::view_state::ViewState;
#[allow(unused_imports)]
pub(super) use crate::workspaces::{WorkspaceContext, WorkspaceItem, load_workspace_context};
pub(super) use color_eyre::Result;
pub(super) use color_eyre::eyre::eyre;
pub(super) use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
pub(super) use fixtures::{key, log_item, mock_load_view, test_app};
pub(super) use ratatui::DefaultTerminal;
pub(super) use services::*;
pub(super) use std::sync::atomic::{AtomicUsize, Ordering};
pub(super) use std::time::{Duration, Instant};
