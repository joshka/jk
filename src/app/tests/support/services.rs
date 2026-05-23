use super::{
    AppServices, DefaultTerminal, JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan,
    JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, JjDuplicatePlan, JjFileMutationPlan,
    JjGitFetch, JjGitPush, JjNewPlan, JjOperationRecovery, JjOperationRecoveryKind,
    JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan, LogViewMode, Ordering, Result,
    ViewState, eyre,
};

pub static ABANDON_DRIFT_RECHECK_CALLS: super::AtomicUsize = super::AtomicUsize::new(0);
pub static ABANDON_FAILED_RECHECK_CALLS: super::AtomicUsize = super::AtomicUsize::new(0);
pub static NEW_TRUNK_CALLS: super::AtomicUsize = super::AtomicUsize::new(0);
pub static OPERATION_RESTORE_REFRESH_CALLS: super::AtomicUsize = super::AtomicUsize::new(0);
pub static OPERATION_REVERT_REFRESH_CALLS: super::AtomicUsize = super::AtomicUsize::new(0);

pub fn mock_new_success(new_change: &JjNewPlan) -> Result<String> {
    Ok(format!("new parents: {}", new_change.parents().join(",")))
}

pub fn mock_new_failure(_: &JjNewPlan) -> Result<String> {
    Err(eyre!("jj new failed: first line\nsecond line"))
}

pub fn mock_duplicate_success(duplicate: &JjDuplicatePlan) -> Result<String> {
    Ok(format!("Duplicated source {}", duplicate.source()))
}

pub fn mock_duplicate_failure(duplicate: &JjDuplicatePlan) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        duplicate.command_label()
    ))
}

pub fn mock_rebase_success(_: &JjRebasePlan) -> Result<String> {
    Ok("rebased".to_owned())
}

pub fn mock_rebase_failure(_: &JjRebasePlan) -> Result<String> {
    Err(eyre!("jj rebase failed: first line\nsecond line"))
}

pub fn mock_split_success(split: &JjSplitPlan) -> Result<String> {
    Ok(split.success_result_message("exit status: 0"))
}

pub fn mock_split_failure(split: &JjSplitPlan) -> Result<String> {
    assert_eq!(split.command_label(), "jj split");
    Err(eyre!("jj split failed with status exit status: 1"))
}

pub fn mock_split_success_service(
    _terminal: Option<&mut DefaultTerminal>,
    split: &JjSplitPlan,
) -> Result<String> {
    mock_split_success(split)
}

pub fn mock_split_failure_service(
    _terminal: Option<&mut DefaultTerminal>,
    split: &JjSplitPlan,
) -> Result<String> {
    mock_split_failure(split)
}

pub fn mock_squash_success(_: &JjSquashPlan) -> Result<String> {
    Ok("squashed".to_owned())
}

pub fn mock_squash_failure(_: &JjSquashPlan) -> Result<String> {
    Err(eyre!("jj squash failed: first line\nsecond line"))
}

pub fn mock_absorb_success(_: &JjAbsorbPlan) -> Result<String> {
    Ok("absorbed".to_owned())
}

pub fn mock_absorb_failure(_: &JjAbsorbPlan) -> Result<String> {
    Err(eyre!("jj absorb failed: first line\nsecond line"))
}

pub fn mock_restore_success(restore: &JjRestorePlan) -> Result<String> {
    Ok(match restore.path() {
        Some(path) => format!("restored {} from {}", path, restore.revision()),
        None => format!("restored {}", restore.revision()),
    })
}

pub fn mock_restore_failure(_: &JjRestorePlan) -> Result<String> {
    Err(eyre!("jj restore failed: first line\nsecond line"))
}

pub fn mock_restore_preview_success(restore: &JjRestorePlan) -> Result<String> {
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

pub fn mock_revert_success(revert: &JjRevertPlan) -> Result<String> {
    Ok(format!("reverted {}", revert.revision()))
}

pub fn mock_revert_failure(_: &JjRevertPlan) -> Result<String> {
    Err(eyre!("jj revert failed: first line\nsecond line"))
}

pub fn mock_revert_preview_success(revert: &JjRevertPlan) -> Result<String> {
    Ok(format!(
        "target revision: {}\nforward diff:\nM src/main.rs\nundo path: jj undo",
        revert.revision()
    ))
}

pub fn mock_describe_success(describe: &JjDescribePlan) -> Result<String> {
    Ok(format!("described {}", describe.target().label()))
}

pub fn mock_describe_failure(_: &JjDescribePlan) -> Result<String> {
    Err(eyre!("jj describe failed: first line\nsecond line"))
}

pub fn mock_commit_success(_: &JjCommitPlan) -> Result<String> {
    Ok("committed working copy".to_owned())
}

pub fn mock_commit_failure(_: &JjCommitPlan) -> Result<String> {
    Err(eyre!("jj commit failed: first line\nsecond line"))
}

pub fn mock_bookmark_mutation_success(mutation: &JjBookmarkMutationPlan) -> Result<String> {
    if let Some(new_name) = mutation.new_name() {
        return Ok(format!("bookmark rename {} -> {new_name}", mutation.name()));
    }

    Ok(format!(
        "bookmark {} {}",
        mutation.kind().label(),
        mutation.name()
    ))
}

pub fn mock_file_mutation_success(mutation: &JjFileMutationPlan) -> Result<String> {
    Ok(format!(
        "file {} {}",
        mutation.kind().label(),
        mutation.path()
    ))
}

pub fn mock_bookmark_mutation_failure(_: &JjBookmarkMutationPlan) -> Result<String> {
    Err(eyre!("jj bookmark failed: first line\nsecond line"))
}

pub fn mock_file_mutation_failure(_: &JjFileMutationPlan) -> Result<String> {
    Err(eyre!("jj file failed: first line\nsecond line"))
}

pub fn mock_bookmark_mutation_duplicate_name_failure(
    mutation: &JjBookmarkMutationPlan,
) -> Result<String> {
    let name = mutation.new_name().unwrap_or(mutation.name());
    Err(eyre!("Error: Bookmark already exists: {name}"))
}

pub fn mock_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    Ok(JjAbandonPreview::new(
        abandon.revision().to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    ))
}

pub fn mock_non_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    Ok(JjAbandonPreview::new(
        abandon.revision().to_owned(),
        Some("Edit change".to_owned()),
        "M src/main.rs\n".to_owned(),
    ))
}

pub fn mock_abandon_preview_drifts_to_non_empty(
    abandon: &JjAbandonPlan,
) -> Result<JjAbandonPreview> {
    if ABANDON_DRIFT_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        mock_empty_abandon_preview(abandon)
    } else {
        mock_non_empty_abandon_preview(abandon)
    }
}

pub fn mock_abandon_preview_recheck_failure(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    if ABANDON_FAILED_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        mock_empty_abandon_preview(abandon)
    } else {
        Err(eyre!("jj diff -r change-a --summary failed: disappeared"))
    }
}

pub fn mock_abandon_success(_: &JjAbandonPlan) -> Result<String> {
    Ok("abandoned".to_owned())
}

pub fn mock_abandon_failure(_: &JjAbandonPlan) -> Result<String> {
    Err(eyre!("jj abandon change-a failed: first line\nsecond line"))
}

pub fn mock_operation_recovery_success(recovery: &JjOperationRecovery) -> Result<String> {
    Ok(match recovery.kind() {
        JjOperationRecoveryKind::Undo => "undone operation".to_owned(),
        JjOperationRecoveryKind::Redo => "redone operation".to_owned(),
    })
}

pub fn mock_operation_recovery_failure(recovery: &JjOperationRecovery) -> Result<String> {
    Err(eyre!(
        "{} failed: no operation to {} available\nhint: run the opposite recovery command first",
        recovery.command_label(),
        recovery.status_action()
    ))
}

pub fn mock_operation_target_success(target: &JjOperationTarget) -> Result<String> {
    Ok(format!(
        "operation {} {}\nnew operation recorded",
        target.status_action(),
        target.operation_id()
    ))
}

pub fn mock_operation_target_failure(target: &JjOperationTarget) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        target.command_label()
    ))
}

pub fn mock_working_copy_navigation_success(
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

pub fn mock_working_copy_navigation_failure(
    navigation: &JjWorkingCopyNavigationPlan,
) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        navigation.command_label()
    ))
}

pub fn mock_no_remotes() -> Result<Vec<String>> {
    Ok(Vec::new())
}

pub fn mock_single_remote() -> Result<Vec<String>> {
    Ok(vec!["origin".to_owned()])
}

pub fn mock_multiple_remotes() -> Result<Vec<String>> {
    Ok(vec!["origin".to_owned(), "upstream".to_owned()])
}

pub fn mock_push_preview_success(push: &JjGitPush) -> Result<String> {
    Ok(format!("preview: {}", push.preview_command_label()))
}

pub fn mock_push_success(push: &JjGitPush) -> Result<String> {
    Ok(format!("pushed: {}", push.command_label()))
}

pub fn mock_resolve_current_change_id(revset: &str) -> Result<String> {
    assert_eq!(revset, "@");
    Ok("new-working-copy".to_owned())
}

pub fn mock_resolve_trunk_and_current_change_id(revset: &str) -> Result<String> {
    match revset {
        "trunk()" => Ok("trunk-change".to_owned()),
        "@" => Ok("new-working-copy".to_owned()),
        other => panic!("unexpected revset: {other}"),
    }
}

pub fn mock_new_trunk_success() -> Result<String> {
    NEW_TRUNK_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok("created new change from trunk".to_owned())
}

pub fn mock_fetch_success(fetch: &JjGitFetch) -> Result<String> {
    Ok(match fetch.remote() {
        Some(remote) => format!("fetched {remote}"),
        None => "fetched".to_owned(),
    })
}

pub fn mock_fetch_failure(fetch: &JjGitFetch) -> Result<String> {
    Err(eyre!("{} failed: denied", fetch.command_label()))
}

pub fn mock_remotes_failure() -> Result<Vec<String>> {
    Err(eyre!("jj git remote list failed: denied"))
}

pub fn panic_abandon_run(_: &JjAbandonPlan) -> Result<String> {
    panic!("abandon should not run without exact confirmation")
}

pub fn mock_refresh_ok(_view: &mut ViewState) -> Result<()> {
    Ok(())
}

pub fn mock_operation_restore_counting_refresh_ok(_view: &mut ViewState) -> Result<()> {
    OPERATION_RESTORE_REFRESH_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

pub fn mock_operation_revert_second_refresh_failure(_view: &mut ViewState) -> Result<()> {
    if OPERATION_REVERT_REFRESH_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        Ok(())
    } else {
        Err(eyre!("view refresh failed"))
    }
}

pub fn mock_refresh_failure(_view: &mut ViewState) -> Result<()> {
    Err(eyre!("view refresh failed"))
}

pub fn mock_reveal_log_change_error(
    _view: &mut ViewState,
    _change_id: &str,
    _fallback_mode: LogViewMode,
) -> Result<bool> {
    Err(eyre!(
        "refreshed log did not include the new working-copy change"
    ))
}

pub fn mock_reveal_new_change_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "new-working-copy");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub fn mock_reveal_described_change_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(false)
}

pub fn mock_reveal_rebased_source_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "source-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub fn mock_reveal_duplicate_source_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub fn mock_reveal_log_change_unexpected(
    _view: &mut ViewState,
    _change_id: &str,
    _fallback_mode: LogViewMode,
) -> Result<bool> {
    panic!("unexpected graph reveal attempt from detail duplicate");
}

pub fn mock_reveal_squash_destination_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "dest");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub fn mock_reveal_edit_target_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(false)
}

pub fn mock_reveal_current_working_copy_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "new-working-copy");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

pub fn test_services() -> AppServices {
    AppServices {
        new_run: mock_new_success,
        duplicate_run: mock_duplicate_success,
        rebase_run: mock_rebase_success,
        split_run: mock_split_success_service,
        squash_run: mock_squash_success,
        absorb_run: mock_absorb_success,
        restore_run: mock_restore_success,
        revert_run: mock_revert_success,
        restore_preview_load: mock_restore_preview_success,
        revert_preview_load: mock_revert_preview_success,
        describe_run: mock_describe_success,
        commit_run: mock_commit_success,
        bookmark_mutation_run: mock_bookmark_mutation_success,
        file_mutation_run: mock_file_mutation_success,
        abandon_preview_load: mock_empty_abandon_preview,
        abandon_run: mock_abandon_success,
        operation_recovery_run: mock_operation_recovery_success,
        operation_target_run: mock_operation_target_success,
        working_copy_navigation_run: mock_working_copy_navigation_success,
        resolve_revision: mock_resolve_current_change_id,
        new_trunk_run: mock_new_trunk_success,
        git_fetch_run: mock_fetch_success,
        git_remotes_load: mock_multiple_remotes,
        push_preview_run: mock_push_preview_success,
        push_run: mock_push_success,
        refresh_view: mock_refresh_ok,
        reveal_log_change: super::fixtures::default_reveal_log_change,
        ..Default::default()
    }
}
