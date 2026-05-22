//! App-level modal and prompt screen contracts.
//!
//! `app.rs` owns dispatch and side effects. This module owns the transient screen state and the
//! projection from that state to status-line text and shared TUI overlays. It should stay free of
//! command execution and feature-specific availability rules.

use crate::action_pane::ActionPane;
use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan,
    JjBookmarkTarget, JjCommitPlan, JjDescribePlan, JjDescribeTarget, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery,
    JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};
use crate::command::{Binding, project_help};
use crate::copy::CopyOption;
use crate::jj::{DiffFormat, JjCommand};
use crate::menus::{ActionKind, ActionMenu, RolePrompt};
use crate::status_line::StatusLine;
use crate::tui::Overlay;
use crate::view_state::ViewState;

/// Transient screen state for help, prompts, menus, and action previews.
///
/// `App` owns the active mode and projects it into status-line and overlay
/// output on each draw; prompt data stays here until the dispatcher consumes
/// it.
pub(crate) enum InteractionMode {
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

impl InteractionMode {
    /// Projects transient prompt state into the status line for the current draw.
    ///
    /// Only prompt-like modes override the stored status. Durable ready/error status belongs to
    /// `status_line`, and accepting, cancelling, or mutating prompt input belongs to app dispatch.
    /// This method should stay side-effect free so lifecycle code can ask what the screen would
    /// show without changing the active mode.
    pub(crate) fn status_line(&self, view: &ViewState, stored_status: &StatusLine) -> StatusLine {
        match self {
            Self::SearchPrompt(input) => StatusLine::with_message(view, format!("/{input}")),
            Self::LogRevsetPrompt(input) => {
                StatusLine::with_message(view, format!("revset: {input}"))
            }
            Self::DescribePrompt { target, input } => {
                StatusLine::with_message(view, format!("describe {}: {input}", target.label()))
            }
            Self::CommitPrompt(input) => {
                StatusLine::with_message(view, format!("commit @: {input}"))
            }
            Self::BookmarkNamePrompt {
                kind,
                target,
                input,
            } => StatusLine::with_message(
                view,
                format!("bookmark {} {}: {input}", kind.label(), target.label()),
            ),
            Self::BookmarkRenamePrompt { old_name, input } => {
                StatusLine::with_message(view, format!("bookmark rename {old_name}: {input}"))
            }
            Self::AbandonConfirm { input, .. } => StatusLine::with_message(
                view,
                format!("type exact revision to confirm abandon: {input}"),
            ),
            _ => stored_status.clone(),
        }
    }

    /// Borrows the active screen state as the shared TUI overlay model.
    ///
    /// This is a projection boundary only: the returned `Overlay` borrows menu options, prompts,
    /// and action output owned by `InteractionMode` or static tables. Overlay chrome and layout
    /// belong to `tui`, while key handling, command execution, and mode transitions stay in app
    /// dispatch.
    pub(crate) fn overlay<'a>(
        &'a self,
        view: &'a ViewState,
        app_bindings: &'static [Binding],
    ) -> Overlay<'a> {
        match self {
            Self::Help => Overlay::Help {
                sections: project_help(app_bindings, view.bindings(), view.help_context()),
            },
            Self::CopyMenu { options, selected } => Overlay::CopyMenu {
                options,
                selected: *selected,
            },
            Self::ViewMenu { selected } => Overlay::ViewMenu {
                options: view_menu_options(),
                selected: *selected,
            },
            Self::ActionMenu { menu, selected } => Overlay::ActionMenu {
                menu,
                selected: *selected,
            },
            Self::RolePrompt {
                prompt, selected, ..
            } => Overlay::RolePrompt {
                prompt,
                selected: *selected,
            },
            Self::DescribePreview { output, .. } => Overlay::ActionPane {
                title: "Describe",
                output,
            },
            Self::CommitPreview { output, .. } => Overlay::ActionPane {
                title: "Commit",
                output,
            },
            Self::BookmarkMutationPreview { output, .. } => Overlay::ActionPane {
                title: "Bookmark",
                output,
            },
            Self::FileMutationPreview { output, .. } => Overlay::ActionPane {
                title: "File",
                output,
            },
            Self::NewPreview { output, .. } => Overlay::ActionPane {
                title: "New change",
                output,
            },
            Self::DuplicatePreview { output, .. } => Overlay::ActionPane {
                title: "Duplicate",
                output,
            },
            Self::RebasePreview { output, .. } => Overlay::ActionPane {
                title: "Rebase",
                output,
            },
            Self::SplitPreview { output, .. } => Overlay::ActionPane {
                title: "Split",
                output,
            },
            Self::RestorePreview { output, .. } => Overlay::ActionPane {
                title: "Restore",
                output,
            },
            Self::RevertPreview { output, .. } => Overlay::ActionPane {
                title: "Revert",
                output,
            },
            Self::SquashPreview { output, .. } => Overlay::ActionPane {
                title: "Squash",
                output,
            },
            Self::AbsorbPreview { output, .. } => Overlay::ActionPane {
                title: "Absorb",
                output,
            },
            Self::AbandonPreview { output, .. } => Overlay::ActionPane {
                title: "Abandon",
                output,
            },
            Self::AbandonConfirm { input, output, .. } => Overlay::AbandonConfirm { input, output },
            Self::PushRemotePrompt {
                remotes, selected, ..
            } => Overlay::PushRemotePrompt {
                remotes,
                selected: *selected,
            },
            Self::FetchRemotePrompt { remotes, selected } => Overlay::FetchRemotePrompt {
                remotes,
                selected: *selected,
            },
            Self::FetchPreview { output, .. } => Overlay::ActionPane {
                title: "Fetch",
                output,
            },
            Self::PushPreview { output, .. } => Overlay::ActionPane {
                title: "Push",
                output,
            },
            Self::OperationRecoveryPreview { output, .. } => Overlay::ActionPane {
                title: "Operation recovery",
                output,
            },
            Self::OperationTargetPreview { output, .. } => Overlay::ActionPane {
                title: "Operation action",
                output,
            },
            Self::WorkingCopyNavigationPreview { navigation, output } => Overlay::ActionPane {
                title: navigation.overlay_title(),
                output,
            },
            Self::Normal
            | Self::SearchPrompt(_)
            | Self::LogRevsetPrompt(_)
            | Self::DescribePrompt { .. }
            | Self::CommitPrompt(_)
            | Self::BookmarkNamePrompt { .. }
            | Self::BookmarkRenamePrompt { .. } => Overlay::None,
        }
    }
}

/// One static row in the view-switching menu.
///
/// The option is copied into overlay projection only; dispatch remains in `app.rs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ViewMenuOption {
    /// User-visible label rendered in the global view menu.
    label: &'static str,
    /// App-owned action requested when this row is accepted.
    action: ViewMenuAction,
}

impl ViewMenuOption {
    /// Returns the user-visible menu label without implying any dispatch behavior.
    ///
    /// `tui` renders this text as provided; changing wording here is a user-visible screen change.
    pub fn label(self) -> &'static str {
        self.label
    }

    /// Returns the app-owned action requested by this static menu row.
    ///
    /// The action is data for dispatch only. Opening views, changing diff format, refreshing, and
    /// surfacing errors remain in the app navigation/lifecycle code.
    pub fn action(self) -> ViewMenuAction {
        self.action
    }
}

/// Action selected from the global view menu.
///
/// Opening a view and changing diff format are app-owned effects; the menu only supplies the
/// user's requested target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewMenuAction {
    /// Open one shipped top-level view.
    Open(JjCommand),
    /// Apply one app-level show/diff format choice.
    DiffFormat(DiffFormat),
}

/// Static view menu entries shown by shared TUI chrome.
///
/// This table is the shared source for overlay rendering, selected-index clamping, and app
/// navigation lookup. The labels are user-visible command names. Feature-specific view behavior and
/// loading still belong to `ViewState` and the individual view modules.
pub fn view_menu_options() -> &'static [ViewMenuOption] {
    &[
        ViewMenuOption {
            label: "log",
            action: ViewMenuAction::Open(JjCommand::Log),
        },
        ViewMenuOption {
            label: "jj default",
            action: ViewMenuAction::Open(JjCommand::Default),
        },
        ViewMenuOption {
            label: "status",
            action: ViewMenuAction::Open(JjCommand::Status),
        },
        ViewMenuOption {
            label: "resolve",
            action: ViewMenuAction::Open(JjCommand::Resolve),
        },
        ViewMenuOption {
            label: "bookmarks",
            action: ViewMenuAction::Open(JjCommand::Bookmarks),
        },
        ViewMenuOption {
            label: "workspaces",
            action: ViewMenuAction::Open(JjCommand::Workspaces),
        },
        ViewMenuOption {
            label: "operation log",
            action: ViewMenuAction::Open(JjCommand::OperationLog),
        },
        ViewMenuOption {
            label: "show/diff format: default jj",
            action: ViewMenuAction::DiffFormat(DiffFormat::Default),
        },
        ViewMenuOption {
            label: "show/diff format: git (--git)",
            action: ViewMenuAction::DiffFormat(DiffFormat::Git),
        },
    ]
}

#[cfg(test)]
mod tests;
