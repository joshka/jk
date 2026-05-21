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

#[test]
fn abandon_plan_uses_exact_revision_command_shape() {
    let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

    assert_eq!(
        abandon.command_argv(),
        vec![
            "abandon",
            "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
        ]
    );
    assert_eq!(
        abandon.command_label(),
        "jj abandon tvykuurwpnwzzqulzrvwvmxxotnlywqw"
    );
}

#[test]
fn abandon_diff_summary_probe_uses_revision_summary() {
    let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

    assert_eq!(
        abandon.diff_summary_argv(),
        vec![
            "diff",
            "-r",
            "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)",
            "--summary"
        ]
    );
    assert_eq!(
        abandon.diff_summary_label(),
        "jj diff -r tvykuurwpnwzzqulzrvwvmxxotnlywqw --summary"
    );
}

#[test]
fn abandon_title_probe_uses_exact_change_id_revset() {
    let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

    assert_eq!(
        abandon.title_argv(),
        vec![
            "log",
            "-r",
            "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)",
            "--no-graph",
            "-T",
            DESCRIPTION_FIRST_LINE_TEMPLATE,
        ]
    );
}

#[test]
fn abandon_preview_classifies_empty_summary_as_empty_change() {
    let preview = JjAbandonPreview::new(
        "change-id".to_owned(),
        Some("Start feature".to_owned()),
        "\n".to_owned(),
    );

    assert!(preview.is_empty_change());
    assert_eq!(preview.revision(), "change-id");
    assert!(preview.preview_text().contains("title: Start feature"));
    assert!(
        preview
            .preview_text()
            .contains("diff summary:\nempty change")
    );
    assert!(
        preview
            .preview_text()
            .contains("press Enter to abandon this empty change")
    );
    assert!(preview.preview_text().contains("undo path: jj undo"));
}

#[test]
fn abandon_preview_classifies_non_empty_summary_as_strong_confirm() {
    let preview = JjAbandonPreview::new(
        "change-id".to_owned(),
        Some("Edit files".to_owned()),
        "M src/main.rs\nA README.md\n".to_owned(),
    );

    assert!(!preview.is_empty_change());
    let text = preview.preview_text();
    assert!(text.contains("change: change-id"));
    assert!(text.contains("title: Edit files"));
    assert!(text.contains("M src/main.rs\nA README.md"));
    assert!(text.contains("type exact revision 'change-id' before abandon runs"));
    assert!(text.contains("undo path: jj undo"));
}
