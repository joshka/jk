//! Shared fixtures and service mocks for app orchestration tests.
//!
//! The behavior modules import this support surface so their tests can stay focused on the app
//! contract they exercise instead of rebuilding service seams and fixture views.

pub(super) use super::super::navigation::initial_view;
pub(super) use super::super::{APP_BINDINGS, App};
pub(super) use crate::action_menu::{ActionKind, FollowUp, RolePrompt, RolePromptOption};
pub(super) use crate::action_output::{ActionOutput, action_output_visible_lines};
pub(super) use crate::app::mode_input::{rebase_plan_from_prompt, squash_plan_from_prompt};
pub(super) use crate::app::services::AppServices;
pub(super) use crate::app_screen::{InteractionMode, ViewMenuAction};
pub(super) use crate::app_status::{StatusKind, StatusLine};
pub(super) use crate::command::{CommandContext, ViewCommand};
#[allow(unused_imports)]
pub(super) use crate::jj::{DiffFormat, JjCommand, LogViewMode, ViewSpec};
#[allow(unused_imports)]
pub(super) use crate::jj_actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan,
    JjBookmarkTarget, JjCommitPlan, JjDescribePlan, JjDescribeTarget, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery,
    JjOperationRecoveryKind, JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan,
    JjSplitPlan, JjSquashPlan, JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan,
};
#[allow(unused_imports)]
pub(super) use crate::jj_rows::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, FileListItem, LocalBookmarkRemoteState,
    LogItem, RemoteBookmarkTrackingState, ResolveEntry, WorkspaceContext, WorkspaceItem,
    document_plain_text, load_bookmark_entries, load_compact_log_context, load_entries,
    load_file_list_entries, load_resolve_entries, load_workspace_context,
};
pub(super) use crate::tui::Overlay;
pub(super) use crate::view_state::ViewState;
pub(super) use color_eyre::Result;
pub(super) use color_eyre::eyre::eyre;
pub(super) use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
pub(super) use ratatui::DefaultTerminal;
pub(super) use std::sync::atomic::{AtomicUsize, Ordering};
pub(super) use std::time::{Duration, Instant};

pub(super) static ABANDON_DRIFT_RECHECK_CALLS: AtomicUsize = AtomicUsize::new(0);
pub(super) static ABANDON_FAILED_RECHECK_CALLS: AtomicUsize = AtomicUsize::new(0);
pub(super) static NEW_TRUNK_CALLS: AtomicUsize = AtomicUsize::new(0);
pub(super) static OPERATION_RESTORE_REFRESH_CALLS: AtomicUsize = AtomicUsize::new(0);
pub(super) static OPERATION_REVERT_REFRESH_CALLS: AtomicUsize = AtomicUsize::new(0);

pub(super) fn mock_new_success(new_change: &JjNewPlan) -> Result<String> {
    Ok(format!("new parents: {}", new_change.parents().join(",")))
}

pub(super) fn mock_new_failure(_: &JjNewPlan) -> Result<String> {
    Err(eyre!("jj new failed: first line\nsecond line"))
}

pub(super) fn mock_duplicate_success(duplicate: &JjDuplicatePlan) -> Result<String> {
    Ok(format!("Duplicated source {}", duplicate.source()))
}

pub(super) fn mock_duplicate_failure(duplicate: &JjDuplicatePlan) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        duplicate.command_label()
    ))
}

pub(super) fn mock_rebase_success(_: &JjRebasePlan) -> Result<String> {
    Ok("rebased".to_owned())
}

pub(super) fn mock_rebase_failure(_: &JjRebasePlan) -> Result<String> {
    Err(eyre!("jj rebase failed: first line\nsecond line"))
}

pub(super) fn mock_split_success(split: &JjSplitPlan) -> Result<String> {
    Ok(split.success_result_message("exit status: 0"))
}

pub(super) fn mock_split_failure(split: &JjSplitPlan) -> Result<String> {
    assert_eq!(split.command_label(), "jj split");
    Err(eyre!("jj split failed with status exit status: 1"))
}

pub(super) fn mock_split_success_service(
    _terminal: Option<&mut DefaultTerminal>,
    split: &JjSplitPlan,
) -> Result<String> {
    mock_split_success(split)
}

pub(super) fn mock_split_failure_service(
    _terminal: Option<&mut DefaultTerminal>,
    split: &JjSplitPlan,
) -> Result<String> {
    mock_split_failure(split)
}

pub(super) fn mock_squash_success(_: &JjSquashPlan) -> Result<String> {
    Ok("squashed".to_owned())
}

pub(super) fn mock_squash_failure(_: &JjSquashPlan) -> Result<String> {
    Err(eyre!("jj squash failed: first line\nsecond line"))
}

pub(super) fn mock_absorb_success(_: &JjAbsorbPlan) -> Result<String> {
    Ok("absorbed".to_owned())
}

pub(super) fn mock_absorb_failure(_: &JjAbsorbPlan) -> Result<String> {
    Err(eyre!("jj absorb failed: first line\nsecond line"))
}

pub(super) fn mock_restore_success(restore: &JjRestorePlan) -> Result<String> {
    Ok(match restore.path() {
        Some(path) => format!("restored {} from {}", path, restore.revision()),
        None => format!("restored {}", restore.revision()),
    })
}

pub(super) fn mock_restore_failure(_: &JjRestorePlan) -> Result<String> {
    Err(eyre!("jj restore failed: first line\nsecond line"))
}

pub(super) fn mock_restore_preview_success(restore: &JjRestorePlan) -> Result<String> {
    Ok(match restore.path() {
        Some(path) => format!(
            "target revision: {}\nselected path: {}\nexact fileset: root-file:\"{}\"\nundo path: jj undo",
            restore.revision(),
            path,
            path
        ),
        None => format!(
            "target revision: {}\nundo path: jj undo",
            restore.revision()
        ),
    })
}

pub(super) fn mock_revert_success(revert: &JjRevertPlan) -> Result<String> {
    Ok(format!("reverted {}", revert.revision()))
}

pub(super) fn mock_revert_failure(_: &JjRevertPlan) -> Result<String> {
    Err(eyre!("jj revert failed: first line\nsecond line"))
}

pub(super) fn mock_revert_preview_success(revert: &JjRevertPlan) -> Result<String> {
    Ok(format!(
        "target revision: {}\nforward diff:\nM src/main.rs\nundo path: jj undo",
        revert.revision()
    ))
}

pub(super) fn mock_describe_success(describe: &JjDescribePlan) -> Result<String> {
    Ok(format!("described {}", describe.target().label()))
}

pub(super) fn mock_describe_failure(_: &JjDescribePlan) -> Result<String> {
    Err(eyre!("jj describe failed: first line\nsecond line"))
}

pub(super) fn mock_commit_success(_: &JjCommitPlan) -> Result<String> {
    Ok("committed working copy".to_owned())
}

pub(super) fn mock_commit_failure(_: &JjCommitPlan) -> Result<String> {
    Err(eyre!("jj commit failed: first line\nsecond line"))
}

pub(super) fn mock_bookmark_mutation_success(mutation: &JjBookmarkMutationPlan) -> Result<String> {
    if let Some(new_name) = mutation.new_name() {
        return Ok(format!("bookmark rename {} -> {new_name}", mutation.name()));
    }

    Ok(format!(
        "bookmark {} {}",
        mutation.kind().label(),
        mutation.name()
    ))
}

pub(super) fn mock_bookmark_mutation_failure(_: &JjBookmarkMutationPlan) -> Result<String> {
    Err(eyre!("jj bookmark failed: first line\nsecond line"))
}

pub(super) fn mock_file_mutation_success(mutation: &JjFileMutationPlan) -> Result<String> {
    Ok(format!(
        "file {} {}",
        mutation.kind().label(),
        mutation.path()
    ))
}

pub(super) fn mock_file_mutation_failure(_: &JjFileMutationPlan) -> Result<String> {
    Err(eyre!("jj file failed: first line\nsecond line"))
}

pub(super) fn mock_bookmark_mutation_duplicate_name_failure(
    mutation: &JjBookmarkMutationPlan,
) -> Result<String> {
    let name = mutation.new_name().unwrap_or(mutation.name());
    Err(eyre!("Error: Bookmark already exists: {name}"))
}

pub(super) fn mock_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    Ok(JjAbandonPreview::new(
        abandon.revision().to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    ))
}

pub(super) fn mock_non_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    Ok(JjAbandonPreview::new(
        abandon.revision().to_owned(),
        Some("Edit change".to_owned()),
        "M src/main.rs\n".to_owned(),
    ))
}

pub(super) fn mock_abandon_preview_drifts_to_non_empty(
    abandon: &JjAbandonPlan,
) -> Result<JjAbandonPreview> {
    if ABANDON_DRIFT_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        mock_empty_abandon_preview(abandon)
    } else {
        mock_non_empty_abandon_preview(abandon)
    }
}

pub(super) fn mock_abandon_preview_recheck_failure(
    abandon: &JjAbandonPlan,
) -> Result<JjAbandonPreview> {
    if ABANDON_FAILED_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        mock_empty_abandon_preview(abandon)
    } else {
        Err(eyre!("jj diff -r change-a --summary failed: disappeared"))
    }
}

pub(super) fn mock_abandon_success(_: &JjAbandonPlan) -> Result<String> {
    Ok("abandoned".to_owned())
}

pub(super) fn mock_abandon_failure(_: &JjAbandonPlan) -> Result<String> {
    Err(eyre!("jj abandon change-a failed: first line\nsecond line"))
}

pub(super) fn mock_operation_recovery_success(recovery: &JjOperationRecovery) -> Result<String> {
    Ok(match recovery.kind() {
        JjOperationRecoveryKind::Undo => "undone operation".to_owned(),
        JjOperationRecoveryKind::Redo => "redone operation".to_owned(),
    })
}

pub(super) fn mock_operation_recovery_failure(recovery: &JjOperationRecovery) -> Result<String> {
    Err(eyre!(
        "{} failed: no operation to {} available\nhint: run the opposite recovery command first",
        recovery.command_label(),
        recovery.status_action()
    ))
}

pub(super) fn mock_operation_target_success(target: &JjOperationTarget) -> Result<String> {
    Ok(format!(
        "operation {} {}\nnew operation recorded",
        target.status_action(),
        target.operation_id()
    ))
}

pub(super) fn mock_operation_target_failure(target: &JjOperationTarget) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        target.command_label()
    ))
}

pub(super) fn mock_working_copy_navigation_success(
    navigation: &JjWorkingCopyNavigationPlan,
) -> Result<String> {
    Ok(match navigation.kind() {
        JjWorkingCopyNavigationKind::Edit => format!(
            "editing {}",
            navigation
                .target_change_id()
                .expect("edit mock requires exact target change id")
        ),
        JjWorkingCopyNavigationKind::Next => "moved to next editable change".to_owned(),
        JjWorkingCopyNavigationKind::Prev => "moved to previous editable change".to_owned(),
    })
}

pub(super) fn mock_working_copy_navigation_failure(
    navigation: &JjWorkingCopyNavigationPlan,
) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        navigation.command_label()
    ))
}

pub(super) fn mock_no_remotes() -> Result<Vec<String>> {
    Ok(Vec::new())
}

pub(super) fn mock_single_remote() -> Result<Vec<String>> {
    Ok(vec!["origin".to_owned()])
}

pub(super) fn mock_multiple_remotes() -> Result<Vec<String>> {
    Ok(vec!["origin".to_owned(), "upstream".to_owned()])
}

pub(super) fn mock_push_preview_success(push: &JjGitPush) -> Result<String> {
    Ok(format!("preview: {}", push.command_label(true)))
}

pub(super) fn mock_push_success(push: &JjGitPush) -> Result<String> {
    Ok(format!("pushed: {}", push.command_label(false)))
}

pub(super) fn mock_resolve_current_change_id(revset: &str) -> Result<String> {
    assert_eq!(revset, "@");
    Ok("new-working-copy".to_owned())
}

pub(super) fn mock_resolve_trunk_and_current_change_id(revset: &str) -> Result<String> {
    match revset {
        "trunk()" => Ok("trunk-change".to_owned()),
        "@" => Ok("new-working-copy".to_owned()),
        other => panic!("unexpected revset: {other}"),
    }
}

pub(super) fn mock_new_trunk_success() -> Result<String> {
    NEW_TRUNK_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok("created new change from trunk".to_owned())
}

pub(super) fn mock_fetch_success(fetch: &JjGitFetch) -> Result<String> {
    Ok(match fetch.remote() {
        Some(remote) => format!("fetched {remote}"),
        None => "fetched".to_owned(),
    })
}

pub(super) fn mock_fetch_failure(fetch: &JjGitFetch) -> Result<String> {
    Err(eyre!("{} failed: denied", fetch.command_label()))
}

pub(super) fn mock_remotes_failure() -> Result<Vec<String>> {
    Err(eyre!("jj git remote list failed: denied"))
}

pub(super) fn mock_load_view(spec: ViewSpec) -> Result<ViewState> {
    let view = match spec.command() {
        JjCommand::Default | JjCommand::Log => {
            ViewState::Graph(crate::graph::GraphView::test_with_spec(spec, vec![]))
        }
        JjCommand::Show => ViewState::Show(crate::show::ShowView::test_new(spec)),
        JjCommand::Diff => ViewState::Diff(crate::diff::DiffView::test_new(spec)),
        JjCommand::Status => ViewState::Status(crate::status::StatusView::test_new(&[])),
        JjCommand::Resolve => ViewState::Resolve(crate::resolve::ResolveView::test_new(vec![])),
        JjCommand::FileList => {
            ViewState::FileList(crate::file_list::FileListView::test_new(vec![]))
        }
        JjCommand::FileShow => ViewState::FileShow(crate::file_show::FileShowView::new(
            spec,
            "src/main.rs",
            crate::rendered_jj::DocumentLines::new(Vec::new()),
        )),
        JjCommand::Bookmarks => {
            ViewState::Bookmarks(crate::bookmarks::BookmarksView::test_new(vec![]))
        }
        JjCommand::Workspaces => ViewState::Workspaces(
            crate::workspaces::WorkspacesView::test_new(crate::jj_rows::WorkspaceContext::default()),
        ),
        JjCommand::OperationLog => {
            ViewState::OperationLog(crate::operation_log::OperationLogView::test_new(vec![]))
        }
        JjCommand::OperationShow | JjCommand::OperationDiff => {
            ViewState::OperationDetail(crate::operation_detail::OperationDetailView::test_new(
                spec,
                crate::rendered_jj::DocumentLines::new(Vec::new()),
            ))
        }
    };
    Ok(view)
}

pub(super) fn panic_abandon_run(_: &JjAbandonPlan) -> Result<String> {
    panic!("abandon should not run without exact confirmation")
}

pub(super) fn mock_refresh_ok(_view: &mut ViewState) -> Result<()> {
    Ok(())
}

pub(super) fn mock_operation_restore_counting_refresh_ok(_view: &mut ViewState) -> Result<()> {
    OPERATION_RESTORE_REFRESH_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

pub(super) fn mock_operation_revert_second_refresh_failure(_view: &mut ViewState) -> Result<()> {
    if OPERATION_REVERT_REFRESH_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        Ok(())
    } else {
        Err(eyre!("view refresh failed"))
    }
}

pub(super) fn mock_refresh_failure(_view: &mut ViewState) -> Result<()> {
    Err(eyre!("view refresh failed"))
}

pub(super) fn mock_reveal_graph_change_error(
    _view: &mut ViewState,
    _change_id: &str,
    _fallback_mode: LogViewMode,
) -> Result<bool> {
    Err(eyre!(
        "refreshed graph did not include the new working-copy change"
    ))
}

pub(super) fn mock_reveal_new_change_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "new-working-copy");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub(super) fn mock_reveal_described_change_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(false)
}

pub(super) fn mock_reveal_rebased_source_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "source-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub(super) fn mock_reveal_duplicate_source_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub(super) fn mock_reveal_graph_change_unexpected(
    _view: &mut ViewState,
    _change_id: &str,
    _fallback_mode: LogViewMode,
) -> Result<bool> {
    panic!("unexpected graph reveal attempt from detail duplicate");
}

pub(super) fn mock_reveal_squash_destination_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "dest");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub(super) fn mock_reveal_edit_target_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(false)
}

pub(super) fn mock_reveal_current_working_copy_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "new-working-copy");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub(super) fn graph_item(change_id: &str) -> crate::jj_rows::LogItem {
    crate::jj_rows::LogItem::new(
        vec![ratatui::text::Line::from(change_id.to_owned())],
        Some(change_id.to_owned()),
        None,
    )
}

pub(super) fn default_reveal_graph_change(
    view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    view.reveal_graph_change(change_id, fallback_mode)
}

pub(super) fn test_services() -> AppServices {
    let mut services = AppServices::default();
    services.new_run = mock_new_success;
    services.duplicate_run = mock_duplicate_success;
    services.rebase_run = mock_rebase_success;
    services.split_run = mock_split_success_service;
    services.squash_run = mock_squash_success;
    services.absorb_run = mock_absorb_success;
    services.restore_run = mock_restore_success;
    services.revert_run = mock_revert_success;
    services.restore_preview_load = mock_restore_preview_success;
    services.revert_preview_load = mock_revert_preview_success;
    services.describe_run = mock_describe_success;
    services.commit_run = mock_commit_success;
    services.bookmark_mutation_run = mock_bookmark_mutation_success;
    services.file_mutation_run = mock_file_mutation_success;
    services.abandon_preview_load = mock_empty_abandon_preview;
    services.abandon_run = mock_abandon_success;
    services.operation_recovery_run = mock_operation_recovery_success;
    services.operation_target_run = mock_operation_target_success;
    services.working_copy_navigation_run = mock_working_copy_navigation_success;
    services.resolve_revision = mock_resolve_current_change_id;
    services.new_trunk_run = mock_new_trunk_success;
    services.git_fetch_run = mock_fetch_success;
    services.git_remotes_load = mock_multiple_remotes;
    services.push_preview_run = mock_push_preview_success;
    services.push_run = mock_push_success;
    services.refresh_view = mock_refresh_ok;
    services.reveal_graph_change = default_reveal_graph_change;
    services
}

pub(super) fn test_app(view: ViewState) -> App {
    App {
        status: StatusLine::ready(&view),
        view,
        stack: Vec::new(),
        startup_log_args: None,
        diff_format: DiffFormat::Default,
        mode: InteractionMode::Normal,
        pending_command: None,
        search: None,
        should_quit: false,
        services: test_services(),
    }
}

pub(super) fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}
