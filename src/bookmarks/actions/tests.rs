use super::*;

#[test]
fn bookmark_create_and_set_target_exact_changes_or_current_working_copy() {
    let create = JjBookmarkMutationPlan::create(
        "feature/name",
        JjBookmarkTarget::exact_change("abcdefghijklmnopqrstuvwxzyabcdef"),
    );
    assert_eq!(
        create.command_argv(),
        vec![
            "bookmark",
            "create",
            "--revision",
            "exactly(change_id(\"abcdefghijklmnopqrstuvwxzyabcdef\"), 1)",
            "feature/name"
        ]
    );
    assert!(create.preview_summary().contains("exact selected revision"));
    assert!(create.preview_summary().contains("undo path: jj undo"));

    let set = JjBookmarkMutationPlan::set("feature/name", JjBookmarkTarget::current_working_copy());
    assert_eq!(
        set.command_argv(),
        vec!["bookmark", "set", "--revision", "@", "feature/name"]
    );
    assert!(
        set.preview_summary()
            .contains("current working-copy change (@)")
    );
}

#[test]
fn bookmark_move_and_delete_use_exact_string_patterns() {
    let move_plan = JjBookmarkMutationPlan::move_to(
        "feature/\"quote\\tab",
        JjBookmarkTarget::exact_change("abcdefghijklmnopqrstuvwxzyabcdef"),
    );

    assert_eq!(
        move_plan.command_argv(),
        vec![
            "bookmark",
            "move",
            "--to",
            "exactly(change_id(\"abcdefghijklmnopqrstuvwxzyabcdef\"), 1)",
            "exact:\"feature/\\\"quote\\\\tab\""
        ]
    );
    assert!(
        move_plan
            .command_label()
            .contains("exact:\"feature/\\\"quote\\\\tab\"")
    );

    let delete = JjBookmarkMutationPlan::delete("feature/name");
    assert_eq!(
        delete.command_argv(),
        vec!["bookmark", "delete", "exact:\"feature/name\""]
    );
    assert!(delete.preview_summary().contains("delete, not forget"));
    assert!(
        delete
            .preview_summary()
            .contains("track/untrack stay disabled")
    );
}

#[test]
fn bookmark_forget_uses_exact_local_or_include_remote_patterns() {
    let local = JjBookmarkMutationPlan::forget(
        "feature/name",
        JjBookmarkForgetTarget::local("tracked local bookmark"),
    );

    assert_eq!(
        local.command_argv(),
        vec!["bookmark", "forget", "exact:\"feature/name\""]
    );
    assert!(local.preview_summary().contains("tracked local bookmark"));
    assert!(local.preview_summary().contains("forget, not delete"));

    let remote_only = JjBookmarkMutationPlan::forget(
        "feature/name",
        JjBookmarkForgetTarget::remote_only("origin", "untracked remote bookmark"),
    );

    assert_eq!(
        remote_only.command_argv(),
        vec![
            "bookmark",
            "forget",
            "--include-remotes",
            "exact:\"feature/name\""
        ]
    );
    assert!(
        remote_only
            .preview_summary()
            .contains("remote-only bookmark on origin")
    );
}

#[test]
fn bookmark_forget_exact_pattern_quotes_special_characters() {
    let forget = JjBookmarkMutationPlan::forget(
        "feature/\"quote\\tab",
        JjBookmarkForgetTarget::local("tracked local bookmark"),
    );

    assert_eq!(
        forget.command_argv(),
        vec!["bookmark", "forget", "exact:\"feature/\\\"quote\\\\tab\""]
    );
    assert!(
        forget
            .command_label()
            .contains("exact:\"feature/\\\"quote\\\\tab\"")
    );
}

#[test]
fn bookmark_track_and_untrack_are_exact_remote_scoped() {
    let target = JjBookmarkTrackingTarget::local(
        "feature/name",
        "feature/name",
        "origin",
        "local bookmark with one remote peer",
    );
    let track = JjBookmarkMutationPlan::track("feature/name", target.clone());

    assert_eq!(
        track.command_argv(),
        vec![
            "bookmark",
            "track",
            "--remote",
            "exact:\"origin\"",
            "exact:\"feature/name\"",
        ]
    );
    assert_eq!(
        track.command_label(),
        "jj bookmark track --remote exact:\"origin\" exact:\"feature/name\""
    );
    let preview = track.preview_summary();
    assert!(preview.contains("local bookmark: feature/name"));
    assert!(preview.contains("remote bookmark: feature/name"));
    assert!(preview.contains("remote: origin"));
    assert!(preview.contains("confirmation: press Enter to run jj bookmark track"));
    assert!(preview.contains("recovery: jj undo; review: jj op show -p"));

    let untrack = JjBookmarkMutationPlan::untrack("feature/name", target);
    assert_eq!(
        untrack.command_argv(),
        vec![
            "bookmark",
            "untrack",
            "--remote",
            "exact:\"origin\"",
            "exact:\"feature/name\"",
        ]
    );
    assert!(
        untrack
            .preview_summary()
            .contains("does not delete the local or remote bookmark")
    );
}

#[test]
fn bookmark_track_quotes_remote_and_bookmark_patterns() {
    let target = JjBookmarkTrackingTarget::remote_only(
        "feature/\"quote\\tab",
        "origin/\"remote",
        "remote-only bookmark",
    );
    let track = JjBookmarkMutationPlan::track("feature/\"quote\\tab", target);

    assert_eq!(
        track.command_argv(),
        vec![
            "bookmark",
            "track",
            "--remote",
            "exact:\"origin/\\\"remote\"",
            "exact:\"feature/\\\"quote\\\\tab\"",
        ]
    );
}

#[test]
fn bookmark_rename_uses_old_and_new_names_as_argv() {
    let rename = JjBookmarkMutationPlan::rename("feature/\"old name\"", "feature/new'special");

    assert_eq!(
        rename.command_argv(),
        vec![
            "bookmark",
            "rename",
            "feature/\"old name\"",
            "feature/new'special"
        ]
    );
    assert_eq!(
        rename.command_label(),
        "jj bookmark rename feature/\"old name\" feature/new'special"
    );
    let preview = rename.preview_summary();
    assert!(preview.contains("old name: feature/\"old name\""));
    assert!(preview.contains("new name: feature/new'special"));
    assert!(preview.contains("without --overwrite-existing"));
    assert!(preview.contains("confirmation: press Enter to run jj bookmark rename"));
}

#[test]
fn bookmark_rename_new_name_validation_rejects_obvious_invalid_inputs() {
    assert_eq!(
        validate_bookmark_rename_new_name("feature/name", "").unwrap_err(),
        "empty bookmark name"
    );
    assert_eq!(
        validate_bookmark_rename_new_name("feature/name", "feature/name").unwrap_err(),
        "new bookmark name is unchanged"
    );
    assert_eq!(
        validate_bookmark_rename_new_name("feature/name", "bad name").unwrap_err(),
        "bookmark name must not contain whitespace or control characters"
    );
    assert_eq!(
        validate_bookmark_rename_new_name("feature/name", " feature/renamed ").unwrap_err(),
        "bookmark name must not contain whitespace or control characters"
    );
    assert_eq!(
        validate_bookmark_rename_new_name("feature/name", "feature@origin").unwrap_err(),
        "bookmark name contains a reserved ref character"
    );
    assert_eq!(
        validate_bookmark_rename_new_name("feature/name", "feature//name").unwrap_err(),
        "bookmark name must not contain empty path components"
    );
    assert!(validate_bookmark_rename_new_name("feature/name", "feature/renamed").is_ok());
}
