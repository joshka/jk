use super::*;

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
