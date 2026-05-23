use super::{JjGitFetch, JjGitPush, JjGitPushTarget};

#[test]
fn git_push_bookmark_args_include_dry_run_when_previewing() {
    let push = JjGitPush::for_bookmark("main".to_owned()).with_remote("origin".to_owned());

    assert_eq!(
        push.preview_command_argv(),
        vec![
            "git",
            "push",
            "--dry-run",
            "--remote",
            "origin",
            "--bookmark",
            "main"
        ]
    );
    assert_eq!(
        push.command_label(),
        "jj git push --remote origin --bookmark main"
    );
    assert_eq!(
        push.preview_command_label(),
        "jj git push --dry-run --remote origin --bookmark main"
    );
    assert_eq!(
        push.command_argv(),
        vec!["git", "push", "--remote", "origin", "--bookmark", "main"]
    );
}

#[test]
fn git_push_revision_args_follow_revision_target() {
    let push = JjGitPush::for_revision("main".to_owned()).with_remote("origin".to_owned());

    assert_eq!(
        push.preview_command_argv(),
        vec![
            "git",
            "push",
            "--dry-run",
            "--remote",
            "origin",
            "--revision",
            "main"
        ]
    );
}

#[test]
fn git_push_revision_can_use_jj_default_remote_resolution() {
    let push = JjGitPush::for_revision("main".to_owned());

    assert_eq!(
        push.command_argv(),
        vec!["git", "push", "--revision", "main"]
    );
    assert_eq!(
        push.preview_command_label(),
        "jj git push --dry-run --revision main"
    );
}

#[test]
fn git_push_status_default_uses_remote_only_target() {
    let push = JjGitPush::for_status().with_remote("origin".to_owned());

    assert_eq!(
        push.command_argv(),
        vec!["git", "push", "--remote", "origin"]
    );
    assert_eq!(
        push.preview_command_label(),
        "jj git push --dry-run --remote origin"
    );
}

#[test]
fn git_push_bookmark_can_use_jj_default_remote_resolution() {
    let push = JjGitPush::for_bookmark("main".to_owned());

    assert_eq!(
        push.preview_command_argv(),
        vec!["git", "push", "--dry-run", "--bookmark", "main"]
    );
}

#[test]
fn git_fetch_default_uses_jj_default_remote_resolution() {
    let fetch = JjGitFetch::default_remotes();

    assert_eq!(fetch.command_argv(), vec!["git", "fetch"]);
    assert_eq!(fetch.command_label(), "jj git fetch");
    assert!(fetch.exact_remote_pattern().is_none());
}

#[test]
fn git_fetch_remote_uses_exact_remote_pattern() {
    let fetch = JjGitFetch::for_remote("origin");

    assert_eq!(
        fetch.command_argv(),
        vec!["git", "fetch", "--remote", "exact:\"origin\""]
    );
    assert_eq!(
        fetch.command_label(),
        "jj git fetch --remote exact:\"origin\""
    );
    assert_eq!(
        fetch.exact_remote_pattern().as_deref(),
        Some("exact:\"origin\"")
    );
    assert!(
        fetch
            .preview_summary()
            .contains("remote pattern: exact:\"origin\"")
    );
}

#[test]
fn git_push_keeps_status_target_with_no_remote_optional() {
    assert_eq!(
        JjGitPush::for_status().preview_command_argv(),
        vec!["git", "push", "--dry-run"]
    );
}

#[test]
fn push_target_preserves_status_variant_shape() {
    assert_eq!(JjGitPushTarget::Status, JjGitPushTarget::Status);
}
