use super::*;
use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjCommitPlan, JjDescribePlan, JjDescribeTarget,
    JjDuplicatePlan, JjFileMutationPlan, JjGitFetch, JjGitPush, JjNewPlan, JjOperationRecoveryKind,
    JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan,
    JjWorkingCopyNavigationPlan,
};
use crate::app::actions::ActionPane;
use crate::app::status_line::{StatusKind, StatusLine};
use crate::bookmarks::actions::{JjBookmarkMutationPlan, JjBookmarkTarget};
use crate::jj::{DiffFormat, JjCommand};
use crate::log::LogView;
use crate::operation_log::actions::{JjOperationRecovery, JjOperationTarget};
use crate::tui::{Overlay, StatusHints};
use crate::view_state::ViewState;

#[test]
fn prompt_status_overrides_stored_status() {
    let view = ViewState::Log(LogView::test_new(vec![]));
    let stored = StatusLine::test("jk", "ready", StatusKind::Ready, StatusHints::Log);
    let mode = InteractionMode::SearchPrompt("change".to_owned());

    let status = mode.status_line(&view, &stored);

    assert_eq!(status.message(), "/change");
}

#[test]
fn normal_status_uses_stored_status() {
    let view = ViewState::Log(LogView::test_new(vec![]));
    let stored = StatusLine::test("jk", "ready", StatusKind::Ready, StatusHints::Log);

    let status = InteractionMode::Normal.status_line(&view, &stored);

    assert_eq!(status.message(), "ready");
}

#[test]
fn view_menu_projects_configured_view_options() {
    let view = ViewState::Log(LogView::test_new(vec![]));
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
    let view = ViewState::Log(LogView::test_new(vec![]));

    assert!(matches!(
        InteractionMode::Normal.overlay(&view, &[]),
        Overlay::None
    ));
}

#[test]
fn action_pane_modes_project_common_overlay_titles() {
    let output = ActionPane::pending("jj action".to_owned(), "preview".to_owned(), None);
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
                describe: JjDescribePlan::new(JjDescribeTarget::current_working_copy(), "message"),
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
    let view = ViewState::Log(LogView::test_new(vec![]));

    for (mode, expected_title) in modes {
        let Overlay::ActionPane { title, output } = mode.overlay(&view, &[]) else {
            panic!("{expected_title} should project a common action output overlay");
        };

        assert_eq!(title, expected_title);
        assert_eq!(output.command_label(), "jj action");
    }
}

#[test]
fn abandon_confirm_projects_dedicated_overlay() {
    let view = ViewState::Log(LogView::test_new(vec![]));
    let mode = InteractionMode::AbandonConfirm {
        abandon: JjAbandonPlan::new("change"),
        input: "change".to_owned(),
        output: ActionPane::pending("jj abandon change".to_owned(), String::new(), None),
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
