use color_eyre::Result;
use color_eyre::eyre::eyre;
use ratatui::DefaultTerminal;

use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationPlan, JjCommitPlan,
    JjDescribePlan, JjDuplicatePlan, JjFileMutationPlan, JjGitFetch, JjGitPush, JjNewPlan,
    JjOperationRecovery, JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan,
    JjSquashPlan, JjWorkingCopyNavigationPlan,
};
use crate::app::services::AppServices;
use crate::jj::{LogViewMode, ViewSpec, git_remotes, new_trunk, resolve_exact_change_id};
use crate::view_state::ViewState;

pub fn build_default_services() -> AppServices {
    AppServices {
        new_run: default_new_run,
        duplicate_run: default_duplicate_run,
        rebase_run: default_rebase_run,
        split_run: default_split_run,
        squash_run: default_squash_run,
        absorb_run: default_absorb_run,
        restore_run: default_restore_run,
        revert_run: default_revert_run,
        restore_preview_load: default_restore_preview_load,
        revert_preview_load: default_revert_preview_load,
        describe_run: default_describe_run,
        commit_run: default_commit_run,
        bookmark_mutation_run: default_bookmark_mutation_run,
        file_mutation_run: default_file_mutation_run,
        abandon_preview_load: default_abandon_preview_load,
        abandon_run: default_abandon_run,
        operation_recovery_run: default_operation_recovery_run,
        operation_target_run: default_operation_target_run,
        working_copy_navigation_run: default_working_copy_navigation_run,
        resolve_revision: default_resolve_revision,
        new_trunk_run: default_new_trunk_run,
        git_fetch_run: default_git_fetch_run,
        git_remotes_load: default_git_remotes_load,
        push_preview_run: default_push_preview_run,
        push_run: default_push_run,
        refresh_view: default_refresh_view,
        reveal_log_change: default_reveal_log_change,
        load_view: default_load_view,
    }
}

fn default_new_run(new_change: &JjNewPlan) -> Result<String> {
    new_change.run().map(|output| output.message().to_owned())
}

fn default_duplicate_run(duplicate: &JjDuplicatePlan) -> Result<String> {
    duplicate.run().map(|output| output.message().to_owned())
}

fn default_rebase_run(rebase: &JjRebasePlan) -> Result<String> {
    rebase.run().map(|output| output.message().to_owned())
}

fn default_split_run(
    terminal: Option<&mut DefaultTerminal>,
    split: &JjSplitPlan,
) -> Result<String> {
    let Some(terminal) = terminal else {
        return Err(eyre!(
            "{} requires an interactive terminal handoff",
            split.command_label()
        ));
    };

    split
        .run_interactive(terminal)
        .map(|output| output.message().to_owned())
}

fn default_squash_run(squash: &JjSquashPlan) -> Result<String> {
    squash.run().map(|output| output.message().to_owned())
}

fn default_absorb_run(absorb: &JjAbsorbPlan) -> Result<String> {
    absorb.run().map(|output| output.message().to_owned())
}

fn default_restore_run(restore: &JjRestorePlan) -> Result<String> {
    restore.run().map(|output| output.message().to_owned())
}

fn default_revert_run(revert: &JjRevertPlan) -> Result<String> {
    revert.run().map(|output| output.message().to_owned())
}

fn default_restore_preview_load(restore: &JjRestorePlan) -> Result<String> {
    restore
        .run_preview()
        .map(|output| output.message().to_owned())
}

fn default_revert_preview_load(revert: &JjRevertPlan) -> Result<String> {
    revert
        .run_preview()
        .map(|output| output.message().to_owned())
}

fn default_describe_run(describe: &JjDescribePlan) -> Result<String> {
    describe.run().map(|output| output.message().to_owned())
}

fn default_commit_run(commit: &JjCommitPlan) -> Result<String> {
    commit.run().map(|output| output.message().to_owned())
}

fn default_bookmark_mutation_run(mutation: &JjBookmarkMutationPlan) -> Result<String> {
    mutation.run().map(|output| output.message().to_owned())
}

fn default_file_mutation_run(mutation: &JjFileMutationPlan) -> Result<String> {
    mutation.run().map(|output| output.message().to_owned())
}

fn default_abandon_preview_load(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    abandon.run_preview()
}

fn default_abandon_run(abandon: &JjAbandonPlan) -> Result<String> {
    abandon.run().map(|output| output.message().to_owned())
}

fn default_operation_recovery_run(recovery: &JjOperationRecovery) -> Result<String> {
    recovery.run().map(|output| output.message().to_owned())
}

fn default_operation_target_run(target: &JjOperationTarget) -> Result<String> {
    target.run().map(|output| output.message().to_owned())
}

fn default_working_copy_navigation_run(navigation: &JjWorkingCopyNavigationPlan) -> Result<String> {
    navigation.run().map(|output| output.message().to_owned())
}

fn default_resolve_revision(revset: &str) -> Result<String> {
    resolve_exact_change_id(revset)
}

fn default_new_trunk_run() -> Result<String> {
    new_trunk().map(|output| output.message().to_owned())
}

fn default_git_fetch_run(fetch: &JjGitFetch) -> Result<String> {
    fetch.run().map(|output| output.message().to_owned())
}

fn default_git_remotes_load() -> Result<Vec<String>> {
    git_remotes()
}

fn default_push_preview_run(push: &JjGitPush) -> Result<String> {
    push.run_preview().map(|output| output.message().to_owned())
}

fn default_push_run(push: &JjGitPush) -> Result<String> {
    push.run().map(|output| output.message().to_owned())
}

fn default_refresh_view(view: &mut ViewState) -> Result<()> {
    view.refresh()
}

fn default_reveal_log_change(
    view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    view.reveal_log_change(change_id, fallback_mode)
}

fn default_load_view(spec: ViewSpec) -> Result<ViewState> {
    ViewState::load(spec)
}
