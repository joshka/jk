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
        output: ActionPane,
    },
    CommitPreview {
        commit: JjCommitPlan,
        output: ActionPane,
    },
    BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan,
        output: ActionPane,
    },
    FileMutationPreview {
        mutation: JjFileMutationPlan,
        output: ActionPane,
    },
    NewPreview {
        new_change: JjNewPlan,
        output: ActionPane,
    },
    DuplicatePreview {
        duplicate: JjDuplicatePlan,
        output: ActionPane,
    },
    RebasePreview {
        rebase: JjRebasePlan,
        output: ActionPane,
    },
    SplitPreview {
        split: JjSplitPlan,
        output: ActionPane,
    },
    RestorePreview {
        restore: JjRestorePlan,
        output: ActionPane,
    },
    RevertPreview {
        revert: JjRevertPlan,
        output: ActionPane,
    },
    SquashPreview {
        squash: JjSquashPlan,
        output: ActionPane,
    },
    AbsorbPreview {
        absorb: JjAbsorbPlan,
        output: ActionPane,
    },
    AbandonPreview {
        abandon: JjAbandonPlan,
        preview: JjAbandonPreview,
        output: ActionPane,
    },
    AbandonConfirm {
        abandon: JjAbandonPlan,
        input: String,
        output: ActionPane,
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
        output: ActionPane,
    },
    PushPreview {
        push: JjGitPush,
        output: ActionPane,
    },
    OperationRecoveryPreview {
        recovery: JjOperationRecovery,
        output: ActionPane,
    },
    OperationTargetPreview {
        target: JjOperationTarget,
        output: ActionPane,
    },
    WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan,
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
