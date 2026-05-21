use std::ffi::OsStr;

use super::*;

#[test]
fn new_plan_uses_positional_parent_revsets() {
    let plan = JjNewPlan::new(vec!["parent-a".to_owned()]);

    assert_eq!(plan.command_argv(), vec!["new", "parent-a"]);
    assert_eq!(plan.command_label(), "jj new parent-a");
    assert!(plan.preview_summary().contains("parent: parent-a"));
    assert!(plan.preview_summary().contains("undo path: jj undo"));
}

#[test]
fn new_plan_preserves_multiple_parent_order() {
    let plan = JjNewPlan::new(vec![
        "parent-a".to_owned(),
        "parent-b".to_owned(),
        "parent-c".to_owned(),
    ]);

    assert_eq!(
        plan.command_argv(),
        vec!["new", "parent-a", "parent-b", "parent-c"]
    );
    assert_eq!(plan.command_label(), "jj new parent-a parent-b parent-c");
    assert_eq!(
        plan.preview_summary()
            .lines()
            .filter(|line| line.starts_with("parent: "))
            .collect::<Vec<_>>(),
        vec!["parent: parent-a", "parent: parent-b", "parent: parent-c"]
    );
}

#[test]
fn duplicate_plan_uses_single_exact_change_revset() {
    let duplicate = JjDuplicatePlan::exact_change("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

    assert_eq!(
        duplicate.command_argv(),
        vec![
            "duplicate",
            "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
        ]
    );
    assert_eq!(
        duplicate.command_label(),
        "jj duplicate exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
    );

    let preview = duplicate.preview_summary();
    assert!(preview.contains("source revision: tvykuurwpnwzzqulzrvwvmxxotnlywqw"));
    assert!(preview.contains("source count: 1 exact selected change"));
    assert!(preview.contains("multi-source duplicate is intentionally not exposed"));
    assert!(preview.contains("does not parse duplicate output for the new change id"));
    assert!(preview.contains("confirmation: press Enter to run jj duplicate"));
    assert!(preview.contains("recovery: jj undo"));
}

#[test]
fn split_current_working_copy_uses_bare_command() {
    let split = JjSplitPlan::current_working_copy();

    assert_eq!(split.command_argv(), vec!["split"]);
    assert_eq!(split.command_label(), "jj split");
    assert_eq!(split.target().exact_change_id(), None);

    let preview = split.preview_summary();
    assert!(preview.contains("target: current working-copy change (@)"));
    assert!(preview.contains("jj's diff editor"));
    assert!(preview.contains("jk is not an in-app patch editor"));
    assert!(preview.contains("does not choose hunks or lines"));
    assert!(preview.contains("jj op show -p"));
}

#[test]
fn split_exact_change_uses_exact_revision_option() {
    let split = JjSplitPlan::exact_change("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

    assert_eq!(
        split.command_argv(),
        vec![
            "split",
            "--revision",
            "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
        ]
    );
    assert_eq!(
        split.command_label(),
        "jj split --revision exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
    );
    assert_eq!(
        split.target().exact_change_id(),
        Some("tvykuurwpnwzzqulzrvwvmxxotnlywqw")
    );
    assert!(
        split
            .preview_summary()
            .contains("target: exact selected graph revision tvykuurwpnwzzqulzrvwvmxxotnlywqw")
    );
}

#[test]
fn split_interactive_command_inherits_stdio_and_keeps_no_pager() {
    let split = JjSplitPlan::exact_change("abc");
    let command = split.interactive_command();

    assert_eq!(command.program(), OsStr::new("jj"));
    assert_eq!(
        command.argv(),
        vec![
            OsStr::new("--no-pager"),
            OsStr::new("split"),
            OsStr::new("--revision"),
            OsStr::new("exactly(change_id(\"abc\"), 1)"),
        ]
    );
    assert_eq!(
        command.stdio_intent(),
        crate::interactive_process::StdioIntent::Inherit
    );
}

#[test]
fn split_result_messages_do_not_claim_captured_stderr() {
    let split = JjSplitPlan::current_working_copy();

    let success = split.success_result_message("exit status: 0");
    assert!(success.contains("child exit status: exit status: 0"));
    assert!(success.contains("live while jk's terminal was suspended"));
    assert!(success.contains("did not capture that output"));

    let failure = split.failure_result_message("jj split failed with status exit status: 1");
    assert!(failure.contains("result: split command failed or did not complete"));
    assert!(failure.contains("runner status: jj split failed with status exit status: 1"));
    assert!(failure.contains("did not capture stderr"));
    assert!(failure.contains("if jj recorded an operation, use jj undo"));
}

#[test]
fn edit_plan_uses_exact_change_id_revset() {
    let plan = JjWorkingCopyNavigationPlan::edit("change-a");

    assert_eq!(
        plan.command_argv(),
        vec!["edit", "exactly(change_id(\"change-a\"), 1)"]
    );
    assert_eq!(
        plan.command_label(),
        "jj edit exactly(change_id(\"change-a\"), 1)"
    );
    assert_eq!(plan.target_change_id(), Some("change-a"));
    assert!(
        plan.preview_summary()
            .contains("target: exact selected graph revision change-a")
    );
    assert!(
        plan.preview_summary()
            .contains("moves @ to edit that revision directly")
    );
}

#[test]
fn next_plan_uses_explicit_edit_flag_and_ignores_selection() {
    let plan = JjWorkingCopyNavigationPlan::next();

    assert_eq!(plan.command_argv(), vec!["next", "--edit"]);
    assert_eq!(plan.command_label(), "jj next --edit");
    assert_eq!(plan.target_change_id(), None);
    assert!(
        plan.preview_summary()
            .contains("highlighted graph row is not an argument to jj next --edit")
    );
    assert!(
        plan.preview_summary()
            .contains("runs jj topology movement relative to @")
    );
}

#[test]
fn prev_plan_uses_explicit_edit_flag_and_mentions_ambiguity() {
    let plan = JjWorkingCopyNavigationPlan::prev();

    assert_eq!(plan.command_argv(), vec!["prev", "--edit"]);
    assert_eq!(plan.command_label(), "jj prev --edit");
    assert_eq!(plan.target_change_id(), None);
    assert!(
        plan.preview_summary()
            .contains("highlighted graph row is not an argument to jj prev --edit")
    );
    assert!(
        plan.preview_summary()
            .contains("previous editable change is ambiguous or unavailable")
    );
}
