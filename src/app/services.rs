//! App-owned side-effect boundary.
//!
//! `App` decides when to run jj commands, refresh views, and update UI state. This module owns the
//! single side-effect seam for those effects so tests can replace them without spreading runner
//! fields or duplicate forwarding layers across the app state.

use color_eyre::Result;
use color_eyre::eyre::eyre;
use ratatui::DefaultTerminal;

use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationPlan, JjCommitPlan,
    JjDescribePlan, JjDuplicatePlan, JjFileMutationPlan, JjGitFetch, JjGitPush, JjNewPlan,
    JjOperationRecovery, JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan,
    JjSquashPlan, JjWorkingCopyNavigationPlan,
};
use crate::jj::{LogViewMode, ViewSpec, git_remotes, new_trunk, resolve_exact_change_id};
use crate::view_state::ViewState;

use super::App;

pub(in crate::app) type NewRun = fn(&JjNewPlan) -> Result<String>;
pub(in crate::app) type DuplicateRun = fn(&JjDuplicatePlan) -> Result<String>;
pub(in crate::app) type RebaseRun = fn(&JjRebasePlan) -> Result<String>;
pub(in crate::app) type SplitRun = fn(Option<&mut DefaultTerminal>, &JjSplitPlan) -> Result<String>;
pub(in crate::app) type SquashRun = fn(&JjSquashPlan) -> Result<String>;
pub(in crate::app) type AbsorbRun = fn(&JjAbsorbPlan) -> Result<String>;
pub(in crate::app) type RestoreRun = fn(&JjRestorePlan) -> Result<String>;
pub(in crate::app) type RevertRun = fn(&JjRevertPlan) -> Result<String>;
pub(in crate::app) type RestorePreviewLoad = fn(&JjRestorePlan) -> Result<String>;
pub(in crate::app) type RevertPreviewLoad = fn(&JjRevertPlan) -> Result<String>;
pub(in crate::app) type DescribeRun = fn(&JjDescribePlan) -> Result<String>;
pub(in crate::app) type CommitRun = fn(&JjCommitPlan) -> Result<String>;
pub(in crate::app) type BookmarkMutationRun = fn(&JjBookmarkMutationPlan) -> Result<String>;
pub(in crate::app) type FileMutationRun = fn(&JjFileMutationPlan) -> Result<String>;
pub(in crate::app) type AbandonPreviewLoad = fn(&JjAbandonPlan) -> Result<JjAbandonPreview>;
pub(in crate::app) type AbandonRun = fn(&JjAbandonPlan) -> Result<String>;
pub(in crate::app) type OperationRecoveryRun = fn(&JjOperationRecovery) -> Result<String>;
pub(in crate::app) type OperationTargetRun = fn(&JjOperationTarget) -> Result<String>;
pub(in crate::app) type WorkingCopyNavigationRun =
    fn(&JjWorkingCopyNavigationPlan) -> Result<String>;
pub(in crate::app) type ResolveRevision = fn(&str) -> Result<String>;
pub(in crate::app) type NewTrunkRun = fn() -> Result<String>;
pub(in crate::app) type GitFetchRun = fn(&JjGitFetch) -> Result<String>;
pub(in crate::app) type GitRemotesLoad = fn() -> Result<Vec<String>>;
pub(in crate::app) type PushPreviewRun = fn(&JjGitPush) -> Result<String>;
pub(in crate::app) type PushRun = fn(&JjGitPush) -> Result<String>;
pub(in crate::app) type RefreshView = fn(&mut ViewState) -> Result<()>;
pub(in crate::app) type RevealLogChange = fn(&mut ViewState, &str, LogViewMode) -> Result<bool>;
/// Load a fresh `ViewState` from a `ViewSpec`.
///
/// Startup, top-level navigation, and format switches use this seam so tests can replace view
/// loading without rebuilding the rest of the app runtime.
pub(in crate::app) type LoadView = fn(ViewSpec) -> Result<ViewState>;

/// Injectable app-side effect boundary used by dispatch and tests.
///
/// App submodules call this surface directly when they need jj, refresh, or alternate-view
/// effects. `App` keeps only the small wrappers that must couple the seam to current app-owned
/// state such as the active `ViewState`.
pub(in crate::app) struct AppServices {
    /// Run `jj new` for an already prepared new-change plan.
    pub(in crate::app) new_run: NewRun,
    /// Run `jj duplicate` for an already prepared duplicate plan.
    pub(in crate::app) duplicate_run: DuplicateRun,
    /// Run `jj rebase` for an already prepared rebase plan.
    pub(in crate::app) rebase_run: RebaseRun,
    /// Hand the terminal to an interactive split command.
    pub(in crate::app) split_run: SplitRun,
    /// Run `jj squash` for an already prepared squash plan.
    pub(in crate::app) squash_run: SquashRun,
    /// Run `jj absorb` for an already prepared absorb plan.
    pub(in crate::app) absorb_run: AbsorbRun,
    /// Run `jj restore` for an already prepared restore plan.
    pub(in crate::app) restore_run: RestoreRun,
    /// Run `jj revert` for an already prepared revert plan.
    pub(in crate::app) revert_run: RevertRun,
    /// Load preview output for a restore plan without applying it.
    pub(in crate::app) restore_preview_load: RestorePreviewLoad,
    /// Load preview output for a revert plan without applying it.
    pub(in crate::app) revert_preview_load: RevertPreviewLoad,
    /// Run `jj describe` for an already prepared describe plan.
    pub(in crate::app) describe_run: DescribeRun,
    /// Run `jj commit` for an already prepared commit plan.
    pub(in crate::app) commit_run: CommitRun,
    /// Run a bookmark mutation chosen by app-owned action flow.
    pub(in crate::app) bookmark_mutation_run: BookmarkMutationRun,
    /// Run a file mutation chosen by app-owned action flow.
    pub(in crate::app) file_mutation_run: FileMutationRun,
    /// Load the abandon preview classification before confirmation.
    pub(in crate::app) abandon_preview_load: AbandonPreviewLoad,
    /// Run `jj abandon` for an already prepared abandon plan.
    pub(in crate::app) abandon_run: AbandonRun,
    /// Run one operation-log recovery action such as undo or redo.
    pub(in crate::app) operation_recovery_run: OperationRecoveryRun,
    /// Run one operation-targeted action such as restore or revert.
    pub(in crate::app) operation_target_run: OperationTargetRun,
    /// Run working-copy navigation such as edit, next, or prev.
    pub(in crate::app) working_copy_navigation_run: WorkingCopyNavigationRun,
    /// Resolve a revset to one exact change id for follow-up reveal or status work.
    pub(in crate::app) resolve_revision: ResolveRevision,
    /// Create a new change from `trunk()` without additional plan state.
    pub(in crate::app) new_trunk_run: NewTrunkRun,
    /// Run `jj git fetch` for a chosen fetch configuration.
    pub(in crate::app) git_fetch_run: GitFetchRun,
    /// Load the available git remotes for push/fetch prompt routing.
    pub(in crate::app) git_remotes_load: GitRemotesLoad,
    /// Load preview output for a push action without applying it.
    pub(in crate::app) push_preview_run: PushPreviewRun,
    /// Run `jj git push` for a chosen push configuration.
    pub(in crate::app) push_run: PushRun,
    /// Refresh an already loaded `ViewState` in place.
    pub(in crate::app) refresh_view: RefreshView,
    /// Reveal one exact change id inside a log-capable `ViewState`.
    pub(in crate::app) reveal_log_change: RevealLogChange,
    /// Load a fresh `ViewState` from a `ViewSpec`.
    pub(in crate::app) load_view: LoadView,
}

/// Thin typed accessors over the injected function table.
///
/// These methods intentionally stay mechanical: the owning policy is which effect gets called, not
/// how to reinterpret that effect at each call site.
impl AppServices {
    pub(in crate::app) fn run_new_change(&self, new_change: &JjNewPlan) -> Result<String> {
        (self.new_run)(new_change)
    }

    pub(in crate::app) fn run_duplicate(&self, duplicate: &JjDuplicatePlan) -> Result<String> {
        (self.duplicate_run)(duplicate)
    }

    pub(in crate::app) fn run_rebase(&self, rebase: &JjRebasePlan) -> Result<String> {
        (self.rebase_run)(rebase)
    }

    pub(in crate::app) fn run_split(
        &self,
        terminal: Option<&mut DefaultTerminal>,
        split: &JjSplitPlan,
    ) -> Result<String> {
        (self.split_run)(terminal, split)
    }

    pub(in crate::app) fn run_squash(&self, squash: &JjSquashPlan) -> Result<String> {
        (self.squash_run)(squash)
    }

    pub(in crate::app) fn run_absorb(&self, absorb: &JjAbsorbPlan) -> Result<String> {
        (self.absorb_run)(absorb)
    }

    pub(in crate::app) fn run_restore(&self, restore: &JjRestorePlan) -> Result<String> {
        (self.restore_run)(restore)
    }

    pub(in crate::app) fn run_revert(&self, revert: &JjRevertPlan) -> Result<String> {
        (self.revert_run)(revert)
    }

    pub(in crate::app) fn load_restore_preview(&self, restore: &JjRestorePlan) -> Result<String> {
        (self.restore_preview_load)(restore)
    }

    pub(in crate::app) fn load_revert_preview(&self, revert: &JjRevertPlan) -> Result<String> {
        (self.revert_preview_load)(revert)
    }

    pub(in crate::app) fn run_describe(&self, describe: &JjDescribePlan) -> Result<String> {
        (self.describe_run)(describe)
    }

    pub(in crate::app) fn run_commit(&self, commit: &JjCommitPlan) -> Result<String> {
        (self.commit_run)(commit)
    }

    pub(in crate::app) fn run_bookmark_mutation(
        &self,
        mutation: &JjBookmarkMutationPlan,
    ) -> Result<String> {
        (self.bookmark_mutation_run)(mutation)
    }

    pub(in crate::app) fn run_file_mutation(
        &self,
        mutation: &JjFileMutationPlan,
    ) -> Result<String> {
        (self.file_mutation_run)(mutation)
    }

    pub(in crate::app) fn load_abandon_preview(
        &self,
        abandon: &JjAbandonPlan,
    ) -> Result<JjAbandonPreview> {
        (self.abandon_preview_load)(abandon)
    }

    pub(in crate::app) fn run_abandon(&self, abandon: &JjAbandonPlan) -> Result<String> {
        (self.abandon_run)(abandon)
    }

    pub(in crate::app) fn run_operation_recovery(
        &self,
        recovery: &JjOperationRecovery,
    ) -> Result<String> {
        (self.operation_recovery_run)(recovery)
    }

    pub(in crate::app) fn run_operation_target(
        &self,
        target: &JjOperationTarget,
    ) -> Result<String> {
        (self.operation_target_run)(target)
    }

    pub(in crate::app) fn run_working_copy_navigation(
        &self,
        navigation: &JjWorkingCopyNavigationPlan,
    ) -> Result<String> {
        (self.working_copy_navigation_run)(navigation)
    }

    pub(in crate::app) fn resolve_revision(&self, revset: &str) -> Result<String> {
        (self.resolve_revision)(revset)
    }

    pub(in crate::app) fn run_new_trunk(&self) -> Result<String> {
        (self.new_trunk_run)()
    }

    pub(in crate::app) fn run_git_fetch(&self, fetch: &JjGitFetch) -> Result<String> {
        (self.git_fetch_run)(fetch)
    }

    pub(in crate::app) fn load_git_remotes(&self) -> Result<Vec<String>> {
        (self.git_remotes_load)()
    }

    pub(in crate::app) fn load_push_preview(&self, push: &JjGitPush) -> Result<String> {
        (self.push_preview_run)(push)
    }

    pub(in crate::app) fn run_push(&self, push: &JjGitPush) -> Result<String> {
        (self.push_run)(push)
    }

    pub(in crate::app) fn refresh_view(&self, view: &mut ViewState) -> Result<()> {
        (self.refresh_view)(view)
    }

    pub(in crate::app) fn reveal_log_change(
        &self,
        view: &mut ViewState,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        (self.reveal_log_change)(view, change_id, fallback_mode)
    }

    /// Load a fresh view state for startup or navigation.
    pub(in crate::app) fn load_view(&self, spec: ViewSpec) -> Result<ViewState> {
        (self.load_view)(spec)
    }
}

impl App {
    /// Refresh the active view through the injected seam.
    ///
    /// This wrapper stays on `App` because callers mean "refresh the current app-owned view", not
    /// "run an arbitrary refresh function against any `ViewState`".
    pub(in crate::app) fn refresh_view_state(&mut self) -> Result<()> {
        self.services.refresh_view(&mut self.view)
    }

    /// Ask the injected seam to reveal a change in the current app-owned view.
    ///
    /// The service seam owns how reveal works; `App` owns which `ViewState` instance is current.
    pub(in crate::app) fn reveal_log_change(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        self.services
            .reveal_log_change(&mut self.view, change_id, fallback_mode)
    }
}

/// Production wiring for the app side-effect seam.
impl Default for AppServices {
    /// Build the production service table backed by real jj and view-loading functions.
    fn default() -> Self {
        Self {
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
