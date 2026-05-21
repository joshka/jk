//! App-level modal and prompt screen contracts.
//!
//! `app.rs` owns dispatch and side effects. This module owns the transient screen state and the
//! projection from that state to status-line text and shared TUI overlays.

use crate::action_menu::{ActionKind, ActionMenu, RolePrompt};
use crate::action_output::ActionOutput;
use crate::app_status::StatusLine;
use crate::command::{Binding, project_help};
use crate::copy::CopyOption;
use crate::jj::{DiffFormat, JjCommand};
use crate::jj_actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan,
    JjBookmarkTarget, JjCommitPlan, JjDescribePlan, JjDescribeTarget, JjDuplicatePlan,
    JjFileMutationPlan, JjGitFetch, JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery,
    JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};
use crate::tui::Overlay;
use crate::view_state::ViewState;

/// Transient screen state for help, prompts, menus, and action previews.
///
/// `App` owns the active mode and projects it into status-line and overlay
/// output on each draw; prompt data stays here until the dispatcher consumes
/// it.
pub(crate) enum InteractionMode {
    Normal,
    Help,
    SearchPrompt(String),
    LogRevsetPrompt(String),
    CopyMenu {
        options: Vec<CopyOption>,
        selected: usize,
    },
    ViewMenu {
        selected: usize,
    },
    ActionMenu {
        menu: ActionMenu,
        selected: usize,
    },
    RolePrompt {
        action: ActionKind,
        prompt: RolePrompt,
        selected: usize,
    },
    DescribePrompt {
        target: JjDescribeTarget,
        input: String,
    },
    CommitPrompt(String),
    BookmarkNamePrompt {
        kind: JjBookmarkMutationKind,
        target: JjBookmarkTarget,
        input: String,
    },
    BookmarkRenamePrompt {
        old_name: String,
        input: String,
    },
    DescribePreview {
        describe: JjDescribePlan,
        output: ActionOutput,
    },
    CommitPreview {
        commit: JjCommitPlan,
        output: ActionOutput,
    },
    BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan,
        output: ActionOutput,
    },
    FileMutationPreview {
        mutation: JjFileMutationPlan,
        output: ActionOutput,
    },
    NewPreview {
        new_change: JjNewPlan,
        output: ActionOutput,
    },
    DuplicatePreview {
        duplicate: JjDuplicatePlan,
        output: ActionOutput,
    },
    RebasePreview {
        rebase: JjRebasePlan,
        output: ActionOutput,
    },
    SplitPreview {
        split: JjSplitPlan,
        output: ActionOutput,
    },
    RestorePreview {
        restore: JjRestorePlan,
        output: ActionOutput,
    },
    RevertPreview {
        revert: JjRevertPlan,
        output: ActionOutput,
    },
    SquashPreview {
        squash: JjSquashPlan,
        output: ActionOutput,
    },
    AbsorbPreview {
        absorb: JjAbsorbPlan,
        output: ActionOutput,
    },
    AbandonPreview {
        abandon: JjAbandonPlan,
        preview: JjAbandonPreview,
        output: ActionOutput,
    },
    AbandonConfirm {
        abandon: JjAbandonPlan,
        input: String,
        output: ActionOutput,
    },
    PushRemotePrompt {
        target: JjGitPushTarget,
        remotes: Vec<String>,
        selected: usize,
    },
    FetchRemotePrompt {
        remotes: Vec<String>,
        selected: usize,
    },
    FetchPreview {
        fetch: JjGitFetch,
        output: ActionOutput,
    },
    PushPreview {
        push: JjGitPush,
        output: ActionOutput,
    },
    OperationRecoveryPreview {
        recovery: JjOperationRecovery,
        output: ActionOutput,
    },
    OperationTargetPreview {
        target: JjOperationTarget,
        output: ActionOutput,
    },
    WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan,
        output: ActionOutput,
    },
}

impl InteractionMode {
    /// Projects transient prompt state into the status line for the current draw.
    ///
    /// Only prompt-like modes override the stored status. Durable ready/error status belongs to
    /// `app_status`, and accepting, cancelling, or mutating prompt input belongs to app dispatch.
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
            Self::DescribePreview { output, .. } => Overlay::ActionOutput {
                title: "Describe",
                output,
            },
            Self::CommitPreview { output, .. } => Overlay::ActionOutput {
                title: "Commit",
                output,
            },
            Self::BookmarkMutationPreview { output, .. } => Overlay::ActionOutput {
                title: "Bookmark",
                output,
            },
            Self::FileMutationPreview { output, .. } => Overlay::ActionOutput {
                title: "File",
                output,
            },
            Self::NewPreview { output, .. } => Overlay::ActionOutput {
                title: "New change",
                output,
            },
            Self::DuplicatePreview { output, .. } => Overlay::ActionOutput {
                title: "Duplicate",
                output,
            },
            Self::RebasePreview { output, .. } => Overlay::ActionOutput {
                title: "Rebase",
                output,
            },
            Self::SplitPreview { output, .. } => Overlay::ActionOutput {
                title: "Split",
                output,
            },
            Self::RestorePreview { output, .. } => Overlay::ActionOutput {
                title: "Restore",
                output,
            },
            Self::RevertPreview { output, .. } => Overlay::ActionOutput {
                title: "Revert",
                output,
            },
            Self::SquashPreview { output, .. } => Overlay::ActionOutput {
                title: "Squash",
                output,
            },
            Self::AbsorbPreview { output, .. } => Overlay::ActionOutput {
                title: "Absorb",
                output,
            },
            Self::AbandonPreview { output, .. } => Overlay::ActionOutput {
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
            Self::FetchPreview { output, .. } => Overlay::ActionOutput {
                title: "Fetch",
                output,
            },
            Self::PushPreview { output, .. } => Overlay::ActionOutput {
                title: "Push",
                output,
            },
            Self::OperationRecoveryPreview { output, .. } => Overlay::ActionOutput {
                title: "Operation recovery",
                output,
            },
            Self::OperationTargetPreview { output, .. } => Overlay::ActionOutput {
                title: "Operation action",
                output,
            },
            Self::WorkingCopyNavigationPreview { navigation, output } => Overlay::ActionOutput {
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
    label: &'static str,
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
    Open(JjCommand),
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
