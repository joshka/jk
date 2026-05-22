use super::*;

#[test]
fn restore_plan_uses_exact_change_revset_for_revision_restore() {
    let restore = JjRestorePlan::for_revision("change-a");

    assert_eq!(
        restore.command_argv(),
        vec![
            "restore",
            "--changes-in",
            "exactly(change_id(\"change-a\"), 1)"
        ]
    );
    assert_eq!(
        restore.preview_diff_argv(),
        vec!["diff", "-r", "exactly(change_id(\"change-a\"), 1)"]
    );
    assert_eq!(
        restore.command_label(),
        "jj restore --changes-in exactly(change_id(\"change-a\"), 1)"
    );

    let preview = restore.preview_summary("M src/main.rs\n");
    assert!(preview.contains("target revision: change-a"));
    assert!(preview.contains("effect: restore removes the selected revision's forward diff"));
    assert!(preview.contains("preview source: jj diff -r exactly(change_id(\"change-a\"), 1)"));
    assert!(preview.contains("jk is not simulating the final graph"));
    assert!(preview.contains("confirmation: press Enter to run jj restore"));
    assert!(preview.contains("undo path: jj undo"));
    assert!(preview.contains("forward diff:\nM src/main.rs"));
}

#[test]
fn restore_plan_uses_root_file_fileset_for_exact_paths() {
    let restore = JjRestorePlan::for_path("change-a", "dir/with spaces/quo\"te\\[glob]?*");

    assert_eq!(
        restore.command_argv(),
        vec![
            "restore",
            "--changes-in",
            "exactly(change_id(\"change-a\"), 1)",
            "root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\""
        ]
    );
    assert_eq!(
        restore.preview_diff_argv(),
        vec![
            "diff",
            "-r",
            "exactly(change_id(\"change-a\"), 1)",
            "root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\""
        ]
    );

    let preview = restore.preview_summary("A dir/with spaces/quo\"te\\[glob]?*\n");
    assert!(preview.contains("selected path: dir/with spaces/quo\"te\\[glob]?*"));
    assert!(preview.contains("exact fileset: root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\""));
    assert!(preview.contains("effect: restore removes the selected path's forward diff"));
}

#[test]
fn restore_plan_uses_default_working_copy_restore_for_status_paths() {
    let restore = JjRestorePlan::for_working_copy_path("src/status.rs");

    assert_eq!(
        restore.command_argv(),
        vec!["restore", "root-file:\"src/status.rs\""]
    );
    assert_eq!(
        restore.preview_diff_argv(),
        vec!["diff", "root-file:\"src/status.rs\""]
    );
    assert_eq!(
        restore.command_label(),
        "jj restore root-file:\"src/status.rs\""
    );

    let preview = restore.preview_summary("M src/status.rs\n");
    assert!(preview.contains("target revision: @"));
    assert!(preview.contains("selected path: src/status.rs"));
    assert!(preview.contains("working-copy diff from @"));
    assert!(preview.contains("preview source: jj diff root-file:\"src/status.rs\""));
}

#[test]
fn revert_plan_uses_exact_change_revset_and_working_copy_destination() {
    let revert = JjRevertPlan::new("change-a");

    assert_eq!(
        revert.command_argv(),
        vec![
            "revert",
            "-r",
            "exactly(change_id(\"change-a\"), 1)",
            "-o",
            "@"
        ]
    );
    assert_eq!(
        revert.preview_diff_argv(),
        vec!["diff", "-r", "exactly(change_id(\"change-a\"), 1)"]
    );
    assert_eq!(
        revert.command_label(),
        "jj revert -r exactly(change_id(\"change-a\"), 1) -o @"
    );

    let preview = revert.preview_summary("M src/main.rs\n");
    assert!(preview.contains("target revision: change-a"));
    assert!(preview.contains("reverse-applies the selected revision's forward diff into @"));
    assert!(preview.contains("preview source: jj diff -r exactly(change_id(\"change-a\"), 1)"));
    assert!(preview.contains("jk is not simulating the final graph"));
    assert!(preview.contains("confirmation: press Enter to run jj revert"));
    assert!(preview.contains("undo path: jj undo"));
    assert!(preview.contains("forward diff:\nM src/main.rs"));
}

#[test]
fn file_track_uses_root_file_fileset_after_double_dash() {
    let plan = JjFileMutationPlan::track("-leading dir/quo\"te\\[glob]?*.rs");

    assert_eq!(
        plan.command_argv(),
        vec![
            "file",
            "track",
            "--",
            "root-file:\"-leading dir/quo\\\"te\\\\[glob]?*.rs\""
        ]
    );
    let preview = plan.preview_summary();
    assert!(preview.contains("selected path: -leading dir/quo\"te\\[glob]?*.rs"));
    assert!(preview.contains("exact fileset: root-file:\"-leading dir/quo\\\"te\\\\[glob]?*.rs\""));
    assert!(preview.contains("effect: starts tracking this exact untracked working-copy path"));
    assert!(preview.contains("output: jj stdout and stderr are preserved"));
    assert!(preview.contains("recovery: jj undo"));
    assert!(preview.contains("review: jj status; jj op show -p"));
}

#[test]
fn file_untrack_uses_root_file_fileset_and_mentions_ignore_requirement() {
    let plan = JjFileMutationPlan::untrack("dir/file.rs");

    assert_eq!(
        plan.command_argv(),
        vec!["file", "untrack", "--", "root-file:\"dir/file.rs\""]
    );
    assert!(
        plan.preview_summary()
            .contains("jj requires the path to already be ignored")
    );
}

#[test]
fn file_chmod_modes_use_installed_jj_mode_args() {
    let executable =
        JjFileMutationPlan::chmod_working_copy("src/main.rs", JjFileChmodMode::Executable);
    let normal = JjFileMutationPlan::chmod_working_copy("src/main.rs", JjFileChmodMode::Normal);

    assert_eq!(
        executable.command_argv(),
        vec!["file", "chmod", "x", "--", "root-file:\"src/main.rs\""]
    );
    assert_eq!(
        normal.command_argv(),
        vec!["file", "chmod", "n", "--", "root-file:\"src/main.rs\""]
    );
    assert!(
        executable
            .preview_summary()
            .contains("chmod mode: x (executable)")
    );
    assert!(normal.preview_summary().contains("chmod mode: n (normal)"));
}

#[test]
fn exact_revision_file_chmod_uses_exact_change_revset_before_mode_and_fileset() {
    let plan = JjFileMutationPlan::chmod_exact_revision(
        "change-a",
        "dir/space file.rs",
        JjFileChmodMode::Executable,
    );

    assert_eq!(
        plan.command_argv(),
        vec![
            "file",
            "chmod",
            "-r",
            "exactly(change_id(\"change-a\"), 1)",
            "x",
            "--",
            "root-file:\"dir/space file.rs\""
        ]
    );
    assert_eq!(
        plan.command_label(),
        "jj file chmod -r exactly(change_id(\"change-a\"), 1) x -- root-file:\"dir/space file.rs\""
    );
    assert!(
        plan.preview_summary()
            .contains("target scope: exact selected revision change-a")
    );
}
