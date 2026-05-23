use super::{JjAbsorbPlan, JjRebasePlan, JjSquashPlan};

#[test]
fn rebase_command_args_use_explicit_sources_and_destination() {
    let rebase = JjRebasePlan::new(
        vec!["source-a".to_owned(), "source-b".to_owned()],
        "dest".to_owned(),
    );

    assert_eq!(
        rebase.command_argv(),
        vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
    );
    assert_eq!(
        rebase.command_label(),
        "jj rebase -r source-a -r source-b -o dest"
    );
}

#[test]
fn rebase_preview_summary_includes_command_effect_and_undo_path() {
    let rebase = JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned());

    let preview = rebase.preview_summary();

    assert!(preview.contains("command: jj rebase -r source-a -o dest"));
    assert!(preview.contains("source revision: source-a"));
    assert!(preview.contains("destination revision: dest"));
    assert!(preview.contains("source rows are selected in jk"));
    assert!(preview.contains("destination is the current row"));
    assert!(preview.contains("semantics: jj rebase --revision <source> --onto <destination>"));
    assert!(preview.contains("only listed source revisions are rebased"));
    assert!(preview.contains("dependencies among listed sources are preserved"));
    assert!(preview.contains("descendants outside the selected set may be rebased"));
    assert!(preview.contains("destination descendants are not inserted or rebased by -o"));
    assert!(preview.contains("not a graph preview"));
    assert!(preview.contains("jk has not run jj and is not simulating the final graph"));
    assert!(preview.contains("review after run: jj op show -p"));
    assert!(preview.contains("undo path: jj undo"));
    assert!(preview.contains("confirmation: press Enter to run jj rebase"));
}

#[test]
fn squash_command_args_use_explicit_sources_destination_and_message_policy() {
    let squash = JjSquashPlan::new(
        vec!["source-a".to_owned(), "source-b".to_owned()],
        "dest".to_owned(),
    );

    assert_eq!(
        squash.command_argv(),
        vec![
            "squash",
            "--from",
            "source-a",
            "--from",
            "source-b",
            "--into",
            "dest",
            "--use-destination-message"
        ]
    );
    assert_eq!(
        squash.command_label(),
        "jj squash --from source-a --from source-b --into dest --use-destination-message"
    );
}

#[test]
fn squash_preview_summary_includes_roles_effect_message_policy_and_undo_path() {
    let squash = JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned());

    let preview = squash.preview_summary();

    assert!(
        preview
            .contains("command: jj squash --from source-a --into dest --use-destination-message")
    );
    assert!(preview.contains("source: source-a"));
    assert!(preview.contains("destination: dest"));
    assert!(preview.contains("graph effect: moves the selected source changes"));
    assert!(preview.contains("--use-destination-message keeps the destination description"));
    assert!(preview.contains("confirmation: press Enter to run jj squash"));
    assert!(preview.contains("undo path: jj undo"));
}

#[test]
fn absorb_command_args_use_exact_source_and_repeated_candidate_destinations() {
    let absorb = JjAbsorbPlan::new(
        "source-change",
        vec!["dest-a".to_owned(), "dest-b".to_owned()],
    );

    assert_eq!(
        absorb.command_argv(),
        vec![
            "absorb",
            "--from",
            "exactly(change_id(\"source-change\"), 1)",
            "--into",
            "exactly(change_id(\"dest-a\"), 1)",
            "--into",
            "exactly(change_id(\"dest-b\"), 1)",
        ]
    );
    assert_eq!(
        absorb.command_label(),
        "jj absorb --from exactly(change_id(\"source-change\"), 1) --into exactly(change_id(\"dest-a\"), 1) --into exactly(change_id(\"dest-b\"), 1)"
    );
}

#[test]
fn absorb_preview_summary_names_bounded_opacity_and_recovery_paths() {
    let absorb = JjAbsorbPlan::new("source-change", vec!["dest-a".to_owned()]);

    let preview = absorb.preview_summary();

    assert!(preview.contains("source: source-change"));
    assert!(preview.contains("candidate destination: dest-a"));
    assert!(preview.contains("selected revisions are candidate destinations"));
    assert!(preview.contains("only considers selected revisions that are ancestors"));
    assert!(preview.contains("jk does not simulate line-level placement"));
    assert!(preview.contains("changes remain in the source"));
    assert!(preview.contains("source may become empty or abandoned"));
    assert!(preview.contains("recovery: jj undo"));
    assert!(preview.contains("review: jj op show -p"));
}

#[test]
fn rebase_plan_argv_includes_repeated_sources_and_destination() {
    let rebase = JjRebasePlan::new(
        vec![
            "source-a".to_owned(),
            "source-b".to_owned(),
            "source-c".to_owned(),
        ],
        "dest".to_owned(),
    );

    assert_eq!(
        rebase.command_argv(),
        vec![
            "rebase", "-r", "source-a", "-r", "source-b", "-r", "source-c", "-o", "dest"
        ]
    );
}

#[test]
fn rebase_plan_argv_and_label_are_stable() {
    let rebase = JjRebasePlan::new(vec!["source-a".to_owned(), "source-b".to_owned()], "dest");

    assert_eq!(
        rebase.command_argv(),
        vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
    );
    assert_eq!(
        rebase.command_label(),
        "jj rebase -r source-a -r source-b -o dest"
    );
}
