use super::*;

#[test]
fn describe_plan_targets_exact_change_before_message() {
    let plan = JjDescribePlan::new(
        JjDescribeTarget::exact_change("abcdefghijklmnopqrstuvwxzyabcdef"),
        "New description",
    );

    assert_eq!(
        plan.command_argv(),
        vec![
            "describe",
            "exactly(change_id(\"abcdefghijklmnopqrstuvwxzyabcdef\"), 1)",
            "--message",
            "New description"
        ]
    );
    assert_eq!(
        plan.command_label(),
        "jj describe abcdefghijklmnopqrstuvwxzyabcdef --message New description"
    );
    assert!(
        plan.preview_summary()
            .contains("target: exact selected revision")
    );
    assert!(plan.preview_summary().contains("without opening an editor"));
}

#[test]
fn describe_plan_can_target_current_working_copy() {
    let plan = JjDescribePlan::new(JjDescribeTarget::current_working_copy(), "Describe @");

    assert_eq!(
        plan.command_argv(),
        vec!["describe", "@", "--message", "Describe @"]
    );
    assert_eq!(plan.command_label(), "jj describe @ --message Describe @");
    assert!(
        plan.preview_summary()
            .contains("current working-copy change (@)")
    );
}

#[test]
fn commit_plan_uses_message_without_revision_argument() {
    let plan = JjCommitPlan::new("Commit working copy");

    assert_eq!(
        plan.command_argv(),
        vec!["commit", "--message", "Commit working copy"]
    );
    assert_eq!(
        plan.command_label(),
        "jj commit --message Commit working copy"
    );
    assert!(
        plan.preview_summary()
            .contains("target: current working-copy change (@)")
    );
    assert!(
        plan.preview_summary()
            .contains("selected graph rows are not arguments")
    );
}
