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
    pub fn label(self) -> &'static str {
        self.label
    }

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
/// The labels are user-visible command names. Feature-specific view behavior and loading still
/// belong to `ViewState` and the individual view modules.
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
mod tests {
    use super::*;
    use crate::app_status::StatusKind;
    use crate::graph::GraphView;
    use crate::jj_actions::JjOperationRecoveryKind;
    use crate::tui::StatusHints;

    #[test]
    fn prompt_status_overrides_stored_status() {
        let view = ViewState::Graph(GraphView::test_new(vec![]));
        let stored = StatusLine::test("jk", "ready", StatusKind::Ready, StatusHints::Graph);
        let mode = InteractionMode::SearchPrompt("change".to_owned());

        let status = mode.status_line(&view, &stored);

        assert_eq!(status.message(), "/change");
    }

    #[test]
    fn normal_status_uses_stored_status() {
        let view = ViewState::Graph(GraphView::test_new(vec![]));
        let stored = StatusLine::test("jk", "ready", StatusKind::Ready, StatusHints::Graph);

        let status = InteractionMode::Normal.status_line(&view, &stored);

        assert_eq!(status.message(), "ready");
    }

    #[test]
    fn view_menu_projects_configured_view_options() {
        let view = ViewState::Graph(GraphView::test_new(vec![]));
        let mode = InteractionMode::ViewMenu { selected: 4 };

        let Overlay::ViewMenu { options, selected } = mode.overlay(&view, &[]) else {
            panic!("view menu mode should project a view menu overlay");
        };

        assert_eq!(selected, 4);
        assert_eq!(
            options[4].action(),
            ViewMenuAction::Open(JjCommand::Bookmarks)
        );
    }

    #[test]
    fn normal_mode_projects_no_overlay() {
        let view = ViewState::Graph(GraphView::test_new(vec![]));

        assert!(matches!(
            InteractionMode::Normal.overlay(&view, &[]),
            Overlay::None
        ));
    }

    #[test]
    fn action_output_modes_project_common_overlay_titles() {
        let output = ActionOutput::pending("jj action".to_owned(), "preview".to_owned(), None);
        let modes = [
            (
                InteractionMode::NewPreview {
                    new_change: JjNewPlan::new(vec!["parent".to_owned()]),
                    output: output.clone(),
                },
                "New change",
            ),
            (
                InteractionMode::DuplicatePreview {
                    duplicate: JjDuplicatePlan::exact_change("change"),
                    output: output.clone(),
                },
                "Duplicate",
            ),
            (
                InteractionMode::DescribePreview {
                    describe: JjDescribePlan::new(
                        JjDescribeTarget::current_working_copy(),
                        "message",
                    ),
                    output: output.clone(),
                },
                "Describe",
            ),
            (
                InteractionMode::CommitPreview {
                    commit: JjCommitPlan::new("message"),
                    output: output.clone(),
                },
                "Commit",
            ),
            (
                InteractionMode::BookmarkMutationPreview {
                    mutation: JjBookmarkMutationPlan::create(
                        "name",
                        JjBookmarkTarget::current_working_copy(),
                    ),
                    output: output.clone(),
                },
                "Bookmark",
            ),
            (
                InteractionMode::FileMutationPreview {
                    mutation: JjFileMutationPlan::track("src/main.rs"),
                    output: output.clone(),
                },
                "File",
            ),
            (
                InteractionMode::RebasePreview {
                    rebase: JjRebasePlan::new(vec!["source".to_owned()], "destination"),
                    output: output.clone(),
                },
                "Rebase",
            ),
            (
                InteractionMode::SplitPreview {
                    split: JjSplitPlan::current_working_copy(),
                    output: output.clone(),
                },
                "Split",
            ),
            (
                InteractionMode::RestorePreview {
                    restore: JjRestorePlan::for_revision("change"),
                    output: output.clone(),
                },
                "Restore",
            ),
            (
                InteractionMode::RevertPreview {
                    revert: JjRevertPlan::new("change"),
                    output: output.clone(),
                },
                "Revert",
            ),
            (
                InteractionMode::SquashPreview {
                    squash: JjSquashPlan::new(vec!["source".to_owned()], "destination"),
                    output: output.clone(),
                },
                "Squash",
            ),
            (
                InteractionMode::AbsorbPreview {
                    absorb: JjAbsorbPlan::new("source", vec!["destination".to_owned()]),
                    output: output.clone(),
                },
                "Absorb",
            ),
            (
                InteractionMode::AbandonPreview {
                    abandon: JjAbandonPlan::new("change"),
                    preview: JjAbandonPreview::new("change".to_owned(), None, String::new()),
                    output: output.clone(),
                },
                "Abandon",
            ),
            (
                InteractionMode::PushPreview {
                    push: JjGitPush::for_status(),
                    output: output.clone(),
                },
                "Push",
            ),
            (
                InteractionMode::FetchPreview {
                    fetch: JjGitFetch::default_remotes(),
                    output: output.clone(),
                },
                "Fetch",
            ),
            (
                InteractionMode::OperationRecoveryPreview {
                    recovery: JjOperationRecovery::new(JjOperationRecoveryKind::Undo),
                    output: output.clone(),
                },
                "Operation recovery",
            ),
            (
                InteractionMode::OperationTargetPreview {
                    target: JjOperationTarget::restore("operation"),
                    output: output.clone(),
                },
                "Operation action",
            ),
            (
                InteractionMode::WorkingCopyNavigationPreview {
                    navigation: JjWorkingCopyNavigationPlan::edit("change"),
                    output: output.clone(),
                },
                "Edit",
            ),
            (
                InteractionMode::WorkingCopyNavigationPreview {
                    navigation: JjWorkingCopyNavigationPlan::next(),
                    output: output.clone(),
                },
                "Next",
            ),
            (
                InteractionMode::WorkingCopyNavigationPreview {
                    navigation: JjWorkingCopyNavigationPlan::prev(),
                    output: output.clone(),
                },
                "Prev",
            ),
        ];
        let view = ViewState::Graph(GraphView::test_new(vec![]));

        for (mode, expected_title) in modes {
            let Overlay::ActionOutput { title, output } = mode.overlay(&view, &[]) else {
                panic!("{expected_title} should project a common action output overlay");
            };

            assert_eq!(title, expected_title);
            assert_eq!(output.command_label(), "jj action");
        }
    }

    #[test]
    fn abandon_confirm_projects_dedicated_overlay() {
        let view = ViewState::Graph(GraphView::test_new(vec![]));
        let mode = InteractionMode::AbandonConfirm {
            abandon: JjAbandonPlan::new("change"),
            input: "change".to_owned(),
            output: ActionOutput::pending("jj abandon change".to_owned(), String::new(), None),
        };

        let Overlay::AbandonConfirm { input, output } = mode.overlay(&view, &[]) else {
            panic!("typed abandon confirmation should use the dedicated overlay");
        };

        assert_eq!(input, "change");
        assert_eq!(output.command_label(), "jj abandon change");
    }

    #[test]
    fn view_menu_options_include_shipped_entries_and_diff_formats() {
        let options = view_menu_options();
        let actions = options
            .iter()
            .map(|option| option.action())
            .collect::<Vec<_>>();

        assert_eq!(
            actions,
            vec![
                ViewMenuAction::Open(JjCommand::Log),
                ViewMenuAction::Open(JjCommand::Default),
                ViewMenuAction::Open(JjCommand::Status),
                ViewMenuAction::Open(JjCommand::Resolve),
                ViewMenuAction::Open(JjCommand::Bookmarks),
                ViewMenuAction::Open(JjCommand::Workspaces),
                ViewMenuAction::Open(JjCommand::OperationLog),
                ViewMenuAction::DiffFormat(DiffFormat::Default),
                ViewMenuAction::DiffFormat(DiffFormat::Git),
            ]
        );
        assert_eq!(options[7].label(), "show/diff format: default jj");
        assert_eq!(options[8].label(), "show/diff format: git (--git)");
    }
}
