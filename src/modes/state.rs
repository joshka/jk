use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan,
    JjBookmarkTarget, JjCommitPlan, JjDescribePlan, JjDescribeTarget, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery,
    JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};
use crate::app::actions::ActionPane;
use crate::menus::{ActionKind, ActionMenu, CopyOption, RolePrompt};

/// Transient screen state for help, prompts, menus, and action previews.
///
/// `App` owns the active mode and projects it into status-line and overlay
/// output on each draw; prompt data stays here until the dispatcher consumes
/// it.
pub enum InteractionMode {
    /// No modal overlay or prompt is active.
    Normal,
    /// Help overlay showing available bindings for the current view.
    Help,
    /// Search prompt carrying the user's in-progress query text.
    SearchPrompt(String),
    /// Log-revset prompt carrying the user's in-progress revset text.
    LogRevsetPrompt(String),
    /// Copy menu with app-provided copy options and the highlighted row.
    CopyMenu {
        /// Options surfaced by the active view for clipboard copying.
        options: Vec<CopyOption>,
        /// Currently highlighted menu row.
        selected: usize,
    },
    /// Global view-switching menu with the highlighted row.
    ViewMenu {
        /// Currently highlighted menu row.
        selected: usize,
    },
    /// Feature-specific action menu with the highlighted row.
    ActionMenu {
        /// Static or feature-built menu content.
        menu: ActionMenu,
        /// Currently highlighted menu row.
        selected: usize,
    },
    /// Role-selection prompt used to build rewrite plans.
    RolePrompt {
        /// Rewrite action whose roles are being assigned.
        action: ActionKind,
        /// Prompt model containing role options and selected revisions.
        prompt: RolePrompt,
        /// Currently highlighted role row.
        selected: usize,
    },
    /// Describe prompt carrying the target and in-progress description text.
    DescribePrompt {
        /// Describe target chosen before entering the prompt.
        target: JjDescribeTarget,
        /// User-typed description text.
        input: String,
    },
    /// Commit prompt carrying the in-progress commit description.
    CommitPrompt(String),
    /// Bookmark name prompt for create/set/move flows.
    BookmarkNamePrompt {
        /// Bookmark mutation kind selected before entering the prompt.
        kind: JjBookmarkMutationKind,
        /// Revision or bookmark target chosen before entering the prompt.
        target: JjBookmarkTarget,
        /// User-typed bookmark name.
        input: String,
    },
    /// Bookmark rename prompt carrying the old name and typed replacement.
    BookmarkRenamePrompt {
        /// Existing bookmark name being renamed.
        old_name: String,
        /// User-typed replacement name.
        input: String,
    },
    /// Describe preview/result pane.
    DescribePreview {
        /// Prepared describe plan being previewed or reported.
        describe: JjDescribePlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Commit preview/result pane.
    CommitPreview {
        /// Prepared commit plan being previewed or reported.
        commit: JjCommitPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Bookmark mutation preview/result pane.
    BookmarkMutationPreview {
        /// Prepared bookmark mutation being previewed or reported.
        mutation: JjBookmarkMutationPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// File mutation preview/result pane.
    FileMutationPreview {
        /// Prepared file mutation being previewed or reported.
        mutation: JjFileMutationPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// New-change preview/result pane.
    NewPreview {
        /// Prepared new-change plan being previewed or reported.
        new_change: JjNewPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Duplicate preview/result pane.
    DuplicatePreview {
        /// Prepared duplicate plan being previewed or reported.
        duplicate: JjDuplicatePlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Rebase preview/result pane.
    RebasePreview {
        /// Prepared rebase plan being previewed or reported.
        rebase: JjRebasePlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Split preview/result pane.
    SplitPreview {
        /// Prepared split plan being previewed or reported.
        split: JjSplitPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Restore preview/result pane.
    RestorePreview {
        /// Prepared restore plan being previewed or reported.
        restore: JjRestorePlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Revert preview/result pane.
    RevertPreview {
        /// Prepared revert plan being previewed or reported.
        revert: JjRevertPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Squash preview/result pane.
    SquashPreview {
        /// Prepared squash plan being previewed or reported.
        squash: JjSquashPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Absorb preview/result pane.
    AbsorbPreview {
        /// Prepared absorb plan being previewed or reported.
        absorb: JjAbsorbPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Abandon preview pane, including the empty/non-empty classification.
    AbandonPreview {
        /// Prepared abandon plan being previewed or reported.
        abandon: JjAbandonPlan,
        /// Preview classification used to decide whether confirmation is needed.
        preview: JjAbandonPreview,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Abandon confirmation prompt layered over the preview output.
    AbandonConfirm {
        /// Prepared abandon plan awaiting explicit confirmation.
        abandon: JjAbandonPlan,
        /// User-typed confirmation text.
        input: String,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Push-remote selection prompt.
    PushRemotePrompt {
        /// Push target chosen before entering remote selection.
        target: JjGitPushTarget,
        /// Candidate remotes to choose from.
        remotes: Vec<String>,
        /// Currently highlighted remote row.
        selected: usize,
    },
    /// Fetch-remote selection prompt.
    FetchRemotePrompt {
        /// Candidate remotes to choose from.
        remotes: Vec<String>,
        /// Currently highlighted remote row.
        selected: usize,
    },
    /// Fetch preview/result pane.
    FetchPreview {
        /// Prepared fetch action being previewed or reported.
        fetch: JjGitFetch,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Push preview/result pane.
    PushPreview {
        /// Prepared push action being previewed or reported.
        push: JjGitPush,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Operation recovery preview/result pane.
    OperationRecoveryPreview {
        /// Prepared undo/redo recovery action being previewed or reported.
        recovery: JjOperationRecovery,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Operation-target preview/result pane.
    OperationTargetPreview {
        /// Prepared restore/revert operation action being previewed or reported.
        target: JjOperationTarget,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
    /// Working-copy navigation preview/result pane.
    WorkingCopyNavigationPreview {
        /// Prepared edit/next/prev navigation action being previewed or reported.
        navigation: JjWorkingCopyNavigationPlan,
        /// Shared action-pane output body and status context.
        output: ActionPane,
    },
}
