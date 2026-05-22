use super::*;
use crate::menus::RolePromptOption;

fn role_prompt(options: Vec<RolePromptOption>) -> RolePrompt {
    RolePrompt::new("confirm role assignment", options, "Preview required.")
}

#[test]
fn role_prompt_accept_builds_rebase_plan() {
    let prompt = role_prompt(vec![
        RolePromptOption::new("source", "source-a"),
        RolePromptOption::new("destination", "dest"),
        RolePromptOption::new("source", "source-b"),
    ]);

    let RolePromptDecision::Rebase(rebase) = reduce_role_prompt_accept(ActionKind::Rebase, &prompt)
    else {
        panic!("expected rebase decision");
    };

    assert_eq!(rebase.sources(), &["source-a", "source-b"]);
    assert_eq!(rebase.destination(), "dest");
}

#[test]
fn role_prompt_accept_builds_squash_plan() {
    let prompt = role_prompt(vec![
        RolePromptOption::new("source", "source-a"),
        RolePromptOption::new("source", "source-b"),
        RolePromptOption::new("destination", "dest"),
    ]);

    let RolePromptDecision::Squash(squash) = reduce_role_prompt_accept(ActionKind::Squash, &prompt)
    else {
        panic!("expected squash decision");
    };

    assert_eq!(squash.sources(), &["source-a", "source-b"]);
    assert_eq!(squash.destination(), "dest");
}

#[test]
fn role_prompt_accept_reports_rewrite_prompt_error_without_a_plan() {
    let prompt = role_prompt(vec![RolePromptOption::new("source", "source-a")]);

    assert_eq!(
        reduce_role_prompt_accept(ActionKind::Rebase, &prompt),
        RolePromptDecision::StatusError("source: source-a\nPreview required.".to_owned())
    );
}

#[test]
fn role_prompt_accept_reports_unsupported_action_as_status_message() {
    let prompt = role_prompt(vec![
        RolePromptOption::new("source", "source-a"),
        RolePromptOption::new("destination", "dest"),
    ]);

    assert_eq!(
        reduce_role_prompt_accept(ActionKind::Absorb, &prompt),
        RolePromptDecision::StatusMessage(
            "source: source-a\ndestination: dest\nPreview required.".to_owned(),
        )
    );
}

#[test]
fn describe_prompt_accept_trims_message_and_builds_plan() {
    let target = JjDescribeTarget::exact_change("abc123");

    let PromptAcceptDecision::Preview(plan) =
        reduce_describe_prompt_accept(&target, "  new description  ")
    else {
        panic!("expected describe preview decision");
    };

    assert_eq!(plan.target(), &target);
    assert_eq!(
        plan.command_label(),
        "jj describe abc123 --message new description"
    );
}

#[test]
fn describe_prompt_accept_reports_empty_description_cancellation() {
    assert_eq!(
        reduce_describe_prompt_accept(&JjDescribeTarget::current_working_copy(), "   "),
        PromptAcceptDecision::StatusMessage("describe cancelled: empty description".to_owned())
    );
}

#[test]
fn commit_prompt_accept_trims_message_and_builds_plan() {
    let PromptAcceptDecision::Preview(plan) = reduce_commit_prompt_accept("  commit message  ")
    else {
        panic!("expected commit preview decision");
    };

    assert_eq!(plan.command_label(), "jj commit --message commit message");
}

#[test]
fn commit_prompt_accept_reports_empty_description_cancellation() {
    assert_eq!(
        reduce_commit_prompt_accept("\t"),
        PromptAcceptDecision::StatusMessage("commit cancelled: empty description".to_owned())
    );
}

#[test]
fn bookmark_name_prompt_accept_trims_name_and_builds_plan() {
    let target = JjBookmarkTarget::exact_change("abc123");

    let PromptAcceptDecision::Preview(plan) = reduce_bookmark_name_prompt_accept(
        JjBookmarkMutationKind::Create,
        &target,
        "  feature/name  ",
    ) else {
        panic!("expected bookmark mutation preview decision");
    };

    assert_eq!(plan.kind(), JjBookmarkMutationKind::Create);
    assert_eq!(plan.name(), "feature/name");
    assert_eq!(plan.target(), Some(&target));
}

#[test]
fn bookmark_name_prompt_accept_reports_empty_name_with_kind_label() {
    assert_eq!(
        reduce_bookmark_name_prompt_accept(
            JjBookmarkMutationKind::Move,
            &JjBookmarkTarget::current_working_copy(),
            "  ",
        ),
        PromptAcceptDecision::StatusMessage(
            "bookmark move cancelled: empty bookmark name".to_owned(),
        )
    );
}

#[test]
fn bookmark_rename_prompt_accept_builds_rename_plan() {
    let PromptAcceptDecision::Preview(plan) =
        reduce_bookmark_rename_prompt_accept("old/name", "new/name")
    else {
        panic!("expected bookmark rename preview decision");
    };

    assert_eq!(plan.kind(), JjBookmarkMutationKind::Rename);
    assert_eq!(plan.name(), "old/name");
    assert_eq!(plan.new_name(), Some("new/name"));
}

#[test]
fn bookmark_rename_prompt_accept_reports_validation_reason() {
    assert_eq!(
        reduce_bookmark_rename_prompt_accept("old/name", ""),
        PromptAcceptDecision::StatusMessage(
            "bookmark rename cancelled: empty bookmark name".to_owned()
        )
    );
}
