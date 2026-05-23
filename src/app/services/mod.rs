//! App-owned side-effect boundary.
//!
//! `App` decides when to run jj commands, refresh views, and update UI state. This module owns the
//! single side-effect seam for those effects so tests can replace them without spreading runner
//! fields or duplicate forwarding layers across the app state.

mod defaults;

use color_eyre::Result;
use ratatui::DefaultTerminal;

use super::App;
use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationPlan, JjCommitPlan,
    JjDescribePlan, JjDuplicatePlan, JjFileMutationPlan, JjGitFetch, JjGitPush, JjNewPlan,
    JjOperationRecovery, JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan,
    JjSquashPlan, JjWorkingCopyNavigationPlan,
};
use crate::jj::{LogViewMode, ViewSpec};
use crate::view_state::ViewState;

pub type NewRun = fn(&JjNewPlan) -> Result<String>;
pub type DuplicateRun = fn(&JjDuplicatePlan) -> Result<String>;
pub type RebaseRun = fn(&JjRebasePlan) -> Result<String>;
pub type SplitRun = fn(Option<&mut DefaultTerminal>, &JjSplitPlan) -> Result<String>;
pub type SquashRun = fn(&JjSquashPlan) -> Result<String>;
pub type AbsorbRun = fn(&JjAbsorbPlan) -> Result<String>;
pub type RestoreRun = fn(&JjRestorePlan) -> Result<String>;
pub type RevertRun = fn(&JjRevertPlan) -> Result<String>;
pub type RestorePreviewLoad = fn(&JjRestorePlan) -> Result<String>;
pub type RevertPreviewLoad = fn(&JjRevertPlan) -> Result<String>;
pub type DescribeRun = fn(&JjDescribePlan) -> Result<String>;
pub type CommitRun = fn(&JjCommitPlan) -> Result<String>;
pub type BookmarkMutationRun = fn(&JjBookmarkMutationPlan) -> Result<String>;
pub type FileMutationRun = fn(&JjFileMutationPlan) -> Result<String>;
pub type AbandonPreviewLoad = fn(&JjAbandonPlan) -> Result<JjAbandonPreview>;
pub type AbandonRun = fn(&JjAbandonPlan) -> Result<String>;
pub type OperationRecoveryRun = fn(&JjOperationRecovery) -> Result<String>;
pub type OperationTargetRun = fn(&JjOperationTarget) -> Result<String>;
pub type WorkingCopyNavigationRun = fn(&JjWorkingCopyNavigationPlan) -> Result<String>;
pub type ResolveRevision = fn(&str) -> Result<String>;
pub type NewTrunkRun = fn() -> Result<String>;
pub type GitFetchRun = fn(&JjGitFetch) -> Result<String>;
pub type GitRemotesLoad = fn() -> Result<Vec<String>>;
pub type PushPreviewRun = fn(&JjGitPush) -> Result<String>;
pub type PushRun = fn(&JjGitPush) -> Result<String>;
pub type RefreshView = fn(&mut ViewState) -> Result<()>;
pub type RevealLogChange = fn(&mut ViewState, &str, LogViewMode) -> Result<bool>;
/// Load a fresh `ViewState` from a `ViewSpec`.
///
/// Startup, top-level navigation, and format switches use this seam so tests can replace view
/// loading without rebuilding the rest of the app runtime.
pub type LoadView = fn(ViewSpec) -> Result<ViewState>;

/// Injectable app-side effect boundary used by dispatch and tests.
///
/// App submodules call this surface directly when they need jj, refresh, or alternate-view
/// effects. `App` keeps only the small wrappers that must couple the seam to current app-owned
/// state such as the active `ViewState`.
pub struct AppServices {
    /// Run `jj new` for an already prepared new-change plan.
    pub new_run: NewRun,
    /// Run `jj duplicate` for an already prepared duplicate plan.
    pub duplicate_run: DuplicateRun,
    /// Run `jj rebase` for an already prepared rebase plan.
    pub rebase_run: RebaseRun,
    /// Hand the terminal to an interactive split command.
    pub split_run: SplitRun,
    /// Run `jj squash` for an already prepared squash plan.
    pub squash_run: SquashRun,
    /// Run `jj absorb` for an already prepared absorb plan.
    pub absorb_run: AbsorbRun,
    /// Run `jj restore` for an already prepared restore plan.
    pub restore_run: RestoreRun,
    /// Run `jj revert` for an already prepared revert plan.
    pub revert_run: RevertRun,
    /// Load preview output for a restore plan without applying it.
    pub restore_preview_load: RestorePreviewLoad,
    /// Load preview output for a revert plan without applying it.
    pub revert_preview_load: RevertPreviewLoad,
    /// Run `jj describe` for an already prepared describe plan.
    pub describe_run: DescribeRun,
    /// Run `jj commit` for an already prepared commit plan.
    pub commit_run: CommitRun,
    /// Run a bookmark mutation chosen by app-owned action flow.
    pub bookmark_mutation_run: BookmarkMutationRun,
    /// Run a file mutation chosen by app-owned action flow.
    pub file_mutation_run: FileMutationRun,
    /// Load the abandon preview classification before confirmation.
    pub abandon_preview_load: AbandonPreviewLoad,
    /// Run `jj abandon` for an already prepared abandon plan.
    pub abandon_run: AbandonRun,
    /// Run one operation-log recovery action such as undo or redo.
    pub operation_recovery_run: OperationRecoveryRun,
    /// Run one operation-targeted action such as restore or revert.
    pub operation_target_run: OperationTargetRun,
    /// Run working-copy navigation such as edit, next, or prev.
    pub working_copy_navigation_run: WorkingCopyNavigationRun,
    /// Resolve a revset to one exact change id for follow-up reveal or status work.
    pub resolve_revision: ResolveRevision,
    /// Create a new change from `trunk()` without additional plan state.
    pub new_trunk_run: NewTrunkRun,
    /// Run `jj git fetch` for a chosen fetch configuration.
    pub git_fetch_run: GitFetchRun,
    /// Load the available git remotes for push/fetch prompt routing.
    pub git_remotes_load: GitRemotesLoad,
    /// Load preview output for a push action without applying it.
    pub push_preview_run: PushPreviewRun,
    /// Run `jj git push` for a chosen push configuration.
    pub push_run: PushRun,
    /// Refresh an already loaded `ViewState` in place.
    pub refresh_view: RefreshView,
    /// Reveal one exact change id inside a log-capable `ViewState`.
    pub reveal_log_change: RevealLogChange,
    /// Load a fresh `ViewState` from a `ViewSpec`.
    pub load_view: LoadView,
}

/// Thin typed accessors over the injected function table.
///
/// These methods intentionally stay mechanical: the owning policy is which effect gets called, not
/// how to reinterpret that effect at each call site.
impl AppServices {
    pub fn run_new_change(&self, new_change: &JjNewPlan) -> Result<String> {
        (self.new_run)(new_change)
    }

    pub fn run_duplicate(&self, duplicate: &JjDuplicatePlan) -> Result<String> {
        (self.duplicate_run)(duplicate)
    }

    pub fn run_rebase(&self, rebase: &JjRebasePlan) -> Result<String> {
        (self.rebase_run)(rebase)
    }

    pub fn run_split(
        &self,
        terminal: Option<&mut DefaultTerminal>,
        split: &JjSplitPlan,
    ) -> Result<String> {
        (self.split_run)(terminal, split)
    }

    pub fn run_squash(&self, squash: &JjSquashPlan) -> Result<String> {
        (self.squash_run)(squash)
    }

    pub fn run_absorb(&self, absorb: &JjAbsorbPlan) -> Result<String> {
        (self.absorb_run)(absorb)
    }

    pub fn run_restore(&self, restore: &JjRestorePlan) -> Result<String> {
        (self.restore_run)(restore)
    }

    pub fn run_revert(&self, revert: &JjRevertPlan) -> Result<String> {
        (self.revert_run)(revert)
    }

    pub fn load_restore_preview(&self, restore: &JjRestorePlan) -> Result<String> {
        (self.restore_preview_load)(restore)
    }

    pub fn load_revert_preview(&self, revert: &JjRevertPlan) -> Result<String> {
        (self.revert_preview_load)(revert)
    }

    pub fn run_describe(&self, describe: &JjDescribePlan) -> Result<String> {
        (self.describe_run)(describe)
    }

    pub fn run_commit(&self, commit: &JjCommitPlan) -> Result<String> {
        (self.commit_run)(commit)
    }

    pub fn run_bookmark_mutation(&self, mutation: &JjBookmarkMutationPlan) -> Result<String> {
        (self.bookmark_mutation_run)(mutation)
    }

    pub fn run_file_mutation(&self, mutation: &JjFileMutationPlan) -> Result<String> {
        (self.file_mutation_run)(mutation)
    }

    pub fn load_abandon_preview(&self, abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
        (self.abandon_preview_load)(abandon)
    }

    pub fn run_abandon(&self, abandon: &JjAbandonPlan) -> Result<String> {
        (self.abandon_run)(abandon)
    }

    pub fn run_operation_recovery(&self, recovery: &JjOperationRecovery) -> Result<String> {
        (self.operation_recovery_run)(recovery)
    }

    pub fn run_operation_target(&self, target: &JjOperationTarget) -> Result<String> {
        (self.operation_target_run)(target)
    }

    pub fn run_working_copy_navigation(
        &self,
        navigation: &JjWorkingCopyNavigationPlan,
    ) -> Result<String> {
        (self.working_copy_navigation_run)(navigation)
    }

    pub fn resolve_revision(&self, revset: &str) -> Result<String> {
        (self.resolve_revision)(revset)
    }

    pub fn run_new_trunk(&self) -> Result<String> {
        (self.new_trunk_run)()
    }

    pub fn run_git_fetch(&self, fetch: &JjGitFetch) -> Result<String> {
        (self.git_fetch_run)(fetch)
    }

    pub fn load_git_remotes(&self) -> Result<Vec<String>> {
        (self.git_remotes_load)()
    }

    pub fn load_push_preview(&self, push: &JjGitPush) -> Result<String> {
        (self.push_preview_run)(push)
    }

    pub fn run_push(&self, push: &JjGitPush) -> Result<String> {
        (self.push_run)(push)
    }

    pub fn refresh_view(&self, view: &mut ViewState) -> Result<()> {
        (self.refresh_view)(view)
    }

    pub fn reveal_log_change(
        &self,
        view: &mut ViewState,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        (self.reveal_log_change)(view, change_id, fallback_mode)
    }

    /// Load a fresh view state for startup or navigation.
    pub fn load_view(&self, spec: ViewSpec) -> Result<ViewState> {
        (self.load_view)(spec)
    }
}

impl App {
    /// Refresh the active view through the injected seam.
    ///
    /// This wrapper stays on `App` because callers mean "refresh the current app-owned view", not
    /// "run an arbitrary refresh function against any `ViewState`".
    pub fn refresh_view_state(&mut self) -> Result<()> {
        self.services.refresh_view(&mut self.view)
    }

    /// Ask the injected seam to reveal a change in the current app-owned view.
    ///
    /// The service seam owns how reveal works; `App` owns which `ViewState` instance is current.
    pub fn reveal_log_change(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        self.services
            .reveal_log_change(&mut self.view, change_id, fallback_mode)
    }
}

impl Default for AppServices {
    /// Build the production service table backed by real jj and view-loading functions.
    fn default() -> Self {
        defaults::build_default_services()
    }
}
