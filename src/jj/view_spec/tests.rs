use super::super::{ALL_REPO_REVSET, RECENT_WORK_REVSET, TRUNK_WORK_REVSET};
use super::*;

#[test]
fn bookmark_list_spec_uses_bookmark_labels() {
    let spec = ViewSpec::new(
        Command::Bookmarks,
        vec!["--revision".to_owned(), "main".to_owned()],
    );

    assert_eq!(spec.command(), Command::Bookmarks);
    assert_eq!(spec.args(), ["--revision", "main"]);
    assert_eq!(spec.label(), "jj bookmark list --revision main");
    assert_eq!(spec.app_label(), "jk bookmarks --revision main");
}

#[test]
fn workspace_spec_uses_workspace_labels() {
    let spec = ViewSpec::new(Command::Workspaces, Vec::new());

    assert_eq!(spec.command(), Command::Workspaces);
    assert!(spec.args().is_empty());
    assert_eq!(spec.label(), "jj workspace list");
    assert_eq!(spec.app_label(), "jk workspaces");
}

#[test]
fn file_list_spec_keeps_selected_path_out_of_args() {
    let spec = ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned()));

    assert_eq!(spec.command(), Command::FileList);
    assert_eq!(spec.args(), ["-r", "main"]);
    assert_eq!(spec.path(), Some("src/main.rs"));
    assert_eq!(spec.exact_change_target(), None);
    assert_eq!(spec.label(), "jj file list -r main");
    assert_eq!(spec.app_label(), "jk file list -r main");
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn file_show_spec_keeps_exact_path_identity() {
    let spec = ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned());

    assert_eq!(spec.command(), Command::FileShow);
    assert_eq!(spec.args(), ["-r", "main", "src/main.rs"]);
    assert_eq!(spec.path(), Some("src/main.rs"));
    assert_eq!(spec.exact_change_target(), None);
    assert_eq!(spec.label(), "jj file show -r main src/main.rs");
    assert_eq!(spec.app_label(), "jk file show -r main src/main.rs");
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn file_show_context_revset_defaults_to_current_revision() {
    let spec = ViewSpec::file_show(None, "src/main.rs".to_owned());

    assert_eq!(spec.show_context_revset(), "@");
    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
}

#[test]
fn resolve_spec_defaults_to_current_revision() {
    let spec = ViewSpec::resolve_current();

    assert_eq!(spec.command(), Command::Resolve);
    assert_eq!(spec.args(), ["-r", "@"]);
    assert_eq!(spec.label(), "jj resolve -r @");
    assert_eq!(spec.app_label(), "jk resolve -r @");
    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
    assert_eq!(spec.show_context_revset(), "@");
}

#[test]
fn resolve_spec_records_direct_revset_without_exact_target() {
    let spec = ViewSpec::resolve_revset("main".to_owned());

    assert_eq!(spec.command(), Command::Resolve);
    assert_eq!(spec.args(), ["-r", "main"]);
    assert_eq!(spec.exact_change_target(), None);
    assert_eq!(spec.label(), "jj resolve -r main");
    assert_eq!(spec.app_label(), "jk resolve -r main");
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn operation_show_spec_uses_positional_operation_id() {
    let spec = ViewSpec::operation_show(operation_id('a'));

    assert_eq!(spec.command(), Command::OperationShow);
    assert_eq!(spec.args(), [operation_id('a')]);
    assert_eq!(spec.app_label(), "jk operation show aaaaaaaa");
}

#[test]
fn operation_diff_spec_uses_operation_option() {
    let spec = ViewSpec::operation_diff(operation_id('b'));

    assert_eq!(spec.command(), Command::OperationDiff);
    assert_eq!(spec.args(), ["--operation", operation_id('b').as_str()]);
    assert_eq!(spec.app_label(), "jk operation diff --operation bbbbbbbb");
}

#[test]
fn file_views_ignore_diff_format_toggle() {
    let show_spec = ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned());
    let list_spec = ViewSpec::file_list(Some("main".to_owned()), None);

    assert_eq!(show_spec.with_diff_format(DiffFormat::Git), show_spec);
    assert_eq!(list_spec.with_diff_format(DiffFormat::Git), list_spec);
}

fn operation_id(digit: char) -> String {
    std::iter::repeat_n(digit, 128).collect()
}

#[test]
fn app_label_shortens_navigated_targets() {
    let spec = ViewSpec::show(
        "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
        DiffFormat::Default,
    );

    assert_eq!(
        spec.exact_change_target(),
        Some("tvykuurwpnwzzqulzrvwvmxxotnlywqw")
    );
    assert_eq!(spec.app_label(), "jk show tvykuurw");

    let spec = ViewSpec::diff(
        "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
        DiffFormat::Default,
    );

    assert_eq!(
        spec.exact_change_target(),
        Some("tvykuurwpnwzzqulzrvwvmxxotnlywqw")
    );
    assert_eq!(spec.app_label(), "jk diff -r tvykuurw");
}

#[test]
fn exact_change_target_provenance_is_explicit() {
    let direct = ViewSpec::new(Command::Show, vec!["main".to_owned()]);
    assert_eq!(direct.navigation_revset().as_deref(), Some("main"));
    assert_eq!(direct.exact_change_target(), None);

    let file_list =
        ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
            .with_exact_change_target();
    assert_eq!(file_list.exact_change_target(), Some("change-a"));

    let file_show = ViewSpec::file_show(Some("change-a".to_owned()), "src/main.rs".to_owned())
        .with_exact_change_target();
    assert_eq!(file_show.exact_change_target(), Some("change-a"));
}

#[test]
fn show_context_revset_prefers_navigation_target() {
    let spec = ViewSpec::show("abc".to_owned(), DiffFormat::Default);

    assert_eq!(spec.show_context_revset(), "abc");
}

#[test]
fn show_context_revset_uses_direct_revset() {
    let spec = ViewSpec::new(Command::Show, vec!["main".to_owned()]);

    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn show_context_revset_skips_option_values() {
    let spec = ViewSpec::new(
        Command::Show,
        vec![
            "--template".to_owned(),
            "description".to_owned(),
            "--summary".to_owned(),
        ],
    );

    assert_eq!(spec.show_context_revset(), "@");
}

#[test]
fn show_context_revset_finds_revset_after_options() {
    let spec = ViewSpec::new(
        Command::Show,
        vec![
            "--template=description".to_owned(),
            "--summary".to_owned(),
            "main".to_owned(),
        ],
    );

    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn show_context_revset_defaults_to_current_revision() {
    let spec = ViewSpec::new(Command::Show, Vec::new());

    assert_eq!(spec.show_context_revset(), "@");
}

#[test]
fn git_diff_format_adds_git_argument_to_navigated_views() {
    let spec = ViewSpec::show("abc".to_owned(), DiffFormat::Git);

    assert_eq!(spec.args(), ["--git", "abc"]);
    assert_eq!(spec.app_label(), "jk show --git abc");

    let spec = ViewSpec::diff("abc".to_owned(), DiffFormat::Git);

    assert_eq!(spec.args(), ["--git", "-r", "abc"]);
    assert_eq!(spec.app_label(), "jk diff --git -r abc");
}

#[test]
fn diff_format_can_be_replaced_without_duplicating_git_flag() {
    let spec = ViewSpec::new(
        Command::Diff,
        vec!["--git".to_owned(), "-r".to_owned(), "abc".to_owned()],
    );

    assert_eq!(spec.diff_format(), DiffFormat::Git);
    assert_eq!(
        spec.with_diff_format(DiffFormat::Git).args(),
        ["--git", "-r", "abc"]
    );
    assert_eq!(
        spec.with_diff_format(DiffFormat::Default).args(),
        ["-r", "abc"]
    );
}

#[test]
fn navigation_revset_uses_direct_show_startup_revset() {
    let spec = ViewSpec::new(Command::Show, vec!["main".to_owned()]);

    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
}

#[test]
fn navigation_revset_defaults_direct_show_to_current_revision() {
    let spec = ViewSpec::new(Command::Show, Vec::new());

    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
}

#[test]
fn navigation_revset_uses_direct_diff_startup_revset() {
    let spec = ViewSpec::new(
        Command::Diff,
        vec!["--git".to_owned(), "-r".to_owned(), "main".to_owned()],
    );

    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
}

#[test]
fn navigation_revset_ignores_direct_diff_filesets() {
    let spec = ViewSpec::new(Command::Diff, vec!["src/main.rs".to_owned()]);

    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
}

#[test]
fn navigation_revset_uses_direct_diff_to_revision() {
    let spec = ViewSpec::new(
        Command::Diff,
        vec!["--from".to_owned(), "main".to_owned(), "--to=@".to_owned()],
    );

    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
}

#[test]
fn navigation_revset_defaults_direct_diff_from_revision_to_current_revision() {
    let spec = ViewSpec::new(Command::Diff, vec!["--from".to_owned(), "main".to_owned()]);

    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
}

#[test]
fn navigation_revset_uses_long_direct_diff_revision_option() {
    let spec = ViewSpec::new(
        Command::Diff,
        vec!["--revisions=main".to_owned(), "src/main.rs".to_owned()],
    );

    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
}

#[test]
fn tool_git_is_passthrough_not_view_format_state() {
    let spec = ViewSpec::new(Command::Diff, vec!["--tool=:git".to_owned()]);

    assert_eq!(spec.diff_format(), DiffFormat::Default);
    assert_eq!(spec.args(), ["--tool=:git"]);
}

#[test]
fn log_view_mode_uses_plain_default_command() {
    let spec = ViewSpec::for_log_mode(Command::Default, &LogViewMode::Default);

    assert_eq!(spec.command(), Command::Default);
    assert!(spec.args().is_empty());
}

#[test]
fn log_view_mode_uses_explicit_revset_for_named_modes() {
    let spec = ViewSpec::for_log_mode(Command::Default, &LogViewMode::Trunk);

    assert_eq!(spec.command(), Command::Log);
    assert_eq!(spec.args(), ["-r", TRUNK_WORK_REVSET]);
}

#[test]
fn log_view_mode_uses_recent_revset_for_recent_mode() {
    let spec = ViewSpec::for_log_mode(Command::Default, &LogViewMode::Recent);

    assert_eq!(spec.command(), Command::Log);
    assert_eq!(spec.args(), ["-r", RECENT_WORK_REVSET]);
}

#[test]
fn log_view_mode_uses_all_revset_for_all_mode() {
    let spec = ViewSpec::for_log_mode(Command::Default, &LogViewMode::All);

    assert_eq!(spec.command(), Command::Log);
    assert_eq!(spec.args(), ["-r", ALL_REPO_REVSET]);
}
