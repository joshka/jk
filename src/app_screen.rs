//! App-level modal and prompt screen contracts.
//!
//! `app.rs` owns dispatch and side effects. This module owns the transient screen state and the
//! projection from that state to status-line text and shared TUI overlays.

use crate::action_menu::{ActionKind, ActionMenu, RolePrompt};
use crate::action_output::ActionOutput;
use crate::app_status::StatusLine;
use crate::command::{Binding, project_help};
use crate::copy::CopyOption;
use crate::jj::{
    DiffFormat, JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind,
    JjBookmarkMutationPlan, JjBookmarkTarget, JjCommand, JjCommitPlan, JjDescribePlan,
    JjDescribeTarget, JjGitFetch, JjGitPush, JjGitPushTarget, JjNewPlan, JjOperationRecovery,
    JjOperationTarget, JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};
use crate::tui::Overlay;
use crate::view_state::ViewState;

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
    NewPreview {
        new_change: JjNewPlan,
        output: ActionOutput,
    },
    RebasePreview {
        rebase: JjRebasePlan,
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
            Self::DescribePreview { output, .. } => Overlay::DescribePreview { output },
            Self::CommitPreview { output, .. } => Overlay::CommitPreview { output },
            Self::BookmarkMutationPreview { output, .. } => {
                Overlay::BookmarkMutationPreview { output }
            }
            Self::NewPreview { output, .. } => Overlay::NewPreview { output },
            Self::RebasePreview { output, .. } => Overlay::RebasePreview { output },
            Self::RestorePreview { output, .. } => Overlay::RestorePreview { output },
            Self::RevertPreview { output, .. } => Overlay::RevertPreview { output },
            Self::SquashPreview { output, .. } => Overlay::SquashPreview { output },
            Self::AbsorbPreview { output, .. } => Overlay::AbsorbPreview { output },
            Self::AbandonPreview { output, .. } => Overlay::AbandonPreview { output },
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
            Self::FetchPreview { output, .. } => Overlay::FetchPreview { output },
            Self::PushPreview { output, .. } => Overlay::PushPreview { output },
            Self::OperationRecoveryPreview { output, .. } => {
                Overlay::OperationRecoveryPreview { output }
            }
            Self::OperationTargetPreview { output, .. } => {
                Overlay::OperationTargetPreview { output }
            }
            Self::WorkingCopyNavigationPreview { navigation, output } => {
                Overlay::WorkingCopyNavigationPreview {
                    title: navigation.overlay_title(),
                    output,
                }
            }
            Self::Normal
            | Self::SearchPrompt(_)
            | Self::LogRevsetPrompt(_)
            | Self::DescribePrompt { .. }
            | Self::CommitPrompt(_)
            | Self::BookmarkNamePrompt { .. } => Overlay::None,
        }
    }
}

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewMenuAction {
    Open(JjCommand),
    DiffFormat(DiffFormat),
}

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
                ViewMenuAction::Open(JjCommand::OperationLog),
                ViewMenuAction::DiffFormat(DiffFormat::Default),
                ViewMenuAction::DiffFormat(DiffFormat::Git),
            ]
        );
        assert_eq!(options[6].label(), "show/diff format: default jj");
        assert_eq!(options[7].label(), "show/diff format: git (--git)");
    }
}
