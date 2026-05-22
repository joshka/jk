use std::ffi::OsStr;

use super::*;
use crate::actions::JjGitFetch;
use crate::operation_log::OPERATION_ID_TEMPLATE;
use crate::resolve::RESOLVE_CONFLICT_TEMPLATE;
use crate::workspaces::WORKSPACE_METADATA_TEMPLATE;

#[test]
fn bookmark_list_command_uses_bookmark_words_and_labels() {
    let spec = ViewSpec::bookmarks(vec!["--revision".to_owned(), "main".to_owned()]);

    assert_eq!(spec.command(), JjCommand::Bookmarks);
    assert_eq!(
        jj_command_args(&spec, None, false),
        vec!["bookmark", "list", "--revision", "main"]
    );
    assert_eq!(spec.label(), "jj bookmark list --revision main");
    assert_eq!(spec.app_label(), "jk bookmarks --revision main");
}

#[test]
fn workspace_commands_use_read_only_root_list_and_metadata_template() {
    let spec = ViewSpec::workspaces(Vec::new());

    assert_eq!(workspace_root_command_args(), vec!["root"]);
    assert_eq!(spec.command(), JjCommand::Workspaces);
    assert_eq!(
        jj_command_args(&spec, None, false),
        vec!["workspace", "list"]
    );
    assert_eq!(
        jj_command_args(&spec, Some(WORKSPACE_METADATA_TEMPLATE), false),
        vec!["workspace", "list", "-T", WORKSPACE_METADATA_TEMPLATE,]
    );
    assert!(!WORKSPACE_METADATA_TEMPLATE.contains("root"));
    assert_eq!(spec.label(), "jj workspace list");
    assert_eq!(spec.app_label(), "jk workspaces");
}

#[test]
fn file_list_command_uses_file_words_and_keeps_selected_path_out_of_args() {
    let spec = ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned()));

    assert_eq!(spec.command(), JjCommand::FileList);
    assert_eq!(spec.args(), ["-r", "main"]);
    assert_eq!(spec.path(), Some("src/main.rs"));
    assert_eq!(spec.exact_change_target(), None);
    assert_eq!(
        jj_command_args(&spec, None, false),
        vec!["file", "list", "-r", "main"]
    );
    assert_eq!(spec.label(), "jj file list -r main");
    assert_eq!(spec.app_label(), "jk file list -r main");
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn file_show_command_keeps_exact_path_identity() {
    let spec = ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned());

    assert_eq!(spec.command(), JjCommand::FileShow);
    assert_eq!(spec.args(), ["-r", "main", "src/main.rs"]);
    assert_eq!(spec.path(), Some("src/main.rs"));
    assert_eq!(spec.exact_change_target(), None);
    assert_eq!(
        jj_command_args(&spec, None, false),
        vec!["file", "show", "-r", "main", "src/main.rs"]
    );
    assert_eq!(spec.label(), "jj file show -r main src/main.rs");
    assert_eq!(spec.app_label(), "jk file show -r main src/main.rs");
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn resolve_command_defaults_to_current_revision() {
    let spec = ViewSpec::resolve(None);

    assert_eq!(spec.command(), JjCommand::Resolve);
    assert_eq!(spec.args(), ["-r", "@"]);
    assert_eq!(
        jj_command_args(&spec, Some(RESOLVE_CONFLICT_TEMPLATE), true),
        vec![
            "log",
            "--no-graph",
            "-T",
            RESOLVE_CONFLICT_TEMPLATE,
            "-r",
            "@",
        ]
    );
    assert_eq!(spec.label(), "jj resolve -r @");
    assert_eq!(spec.app_label(), "jk resolve -r @");
    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
    assert_eq!(spec.show_context_revset(), "@");
}

#[test]
fn resolve_command_uses_log_template_contract_without_graph() {
    let spec = ViewSpec::resolve(Some("main".to_owned()));

    assert_eq!(spec.command(), JjCommand::Resolve);
    assert_eq!(spec.args(), ["-r", "main"]);
    assert_eq!(spec.exact_change_target(), None);
    assert_eq!(
        jj_command_args(&spec, Some(RESOLVE_CONFLICT_TEMPLATE), true),
        vec![
            "log",
            "--no-graph",
            "-T",
            RESOLVE_CONFLICT_TEMPLATE,
            "-r",
            "main",
        ]
    );
    assert_eq!(spec.label(), "jj resolve -r main");
    assert_eq!(spec.app_label(), "jk resolve -r main");
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
    assert_eq!(spec.show_context_revset(), "main");
}

#[test]
fn operation_log_command_uses_at_op_prefix() {
    assert_eq!(
        jj_command_args(
            &ViewSpec::new(JjCommand::OperationLog, Vec::new()),
            None,
            false
        ),
        vec![
            "operation",
            "log",
            "--at-op=@",
            "--limit",
            OPERATION_LOG_LIMIT
        ]
    );
}

#[test]
fn operation_log_id_command_disables_graph_for_template_output() {
    assert_eq!(
        jj_command_args(
            &ViewSpec::new(JjCommand::OperationLog, Vec::new()),
            Some(OPERATION_ID_TEMPLATE),
            true
        ),
        vec![
            "operation",
            "log",
            "--at-op=@",
            "--limit",
            OPERATION_LOG_LIMIT,
            "--no-graph",
            "-T",
            OPERATION_ID_TEMPLATE,
        ]
    );
}

#[test]
fn operation_show_command_uses_positional_operation_id() {
    let spec = ViewSpec::operation_show(operation_id('a'));

    assert_eq!(spec.command(), JjCommand::OperationShow);
    assert_eq!(spec.args(), [operation_id('a')]);
    assert_eq!(
        jj_command_args(&spec, None, false),
        vec!["operation", "show", operation_id('a').as_str()]
    );
    assert_eq!(spec.app_label(), "jk operation show aaaaaaaa");
}

#[test]
fn operation_diff_command_uses_operation_option() {
    let spec = ViewSpec::operation_diff(operation_id('b'));

    assert_eq!(spec.command(), JjCommand::OperationDiff);
    assert_eq!(spec.args(), ["--operation", operation_id('b').as_str()]);
    assert_eq!(
        jj_command_args(&spec, None, false),
        vec![
            "operation",
            "diff",
            "--operation",
            operation_id('b').as_str()
        ]
    );
    assert_eq!(spec.app_label(), "jk operation diff --operation bbbbbbbb");
}

fn operation_id(digit: char) -> String {
    std::iter::repeat_n(digit, 128).collect()
}

#[test]
fn log_view_mode_parses_custom_revset_from_log_spec() {
    let spec = ViewSpec::new(JjCommand::Log, vec!["-r".to_owned(), "::".to_owned()]);

    assert_eq!(
        LogViewMode::from_spec(&spec),
        LogViewMode::CustomRevset("::".to_owned())
    );
}

#[test]
fn log_view_mode_recognizes_named_recent_revset() {
    let spec = ViewSpec::new(
        JjCommand::Log,
        vec!["-r".to_owned(), RECENT_WORK_REVSET.to_owned()],
    );

    assert_eq!(LogViewMode::from_spec(&spec), LogViewMode::Recent);
}

#[test]
fn fetch_command_args_are_stable() {
    assert_eq!(
        JjGitFetch::default_remotes().command_argv(),
        vec!["git", "fetch"]
    );
    assert_eq!(
        JjGitFetch::for_remote("origin").command_argv(),
        vec!["git", "fetch", "--remote", "exact:\"origin\""]
    );
}

#[test]
fn interactive_jj_command_inherits_stdio_and_keeps_no_pager() {
    let command = interactive_jj_command(
        vec![
            "split".to_owned(),
            "--revision".to_owned(),
            "exactly(change_id(\"abc\"), 1)".to_owned(),
        ],
        "jj split",
    );

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
        crate::terminal_process::StdioIntent::Inherit
    );
}

#[test]
fn new_trunk_command_args_are_stable() {
    assert_eq!(NEW_TRUNK_ARGS, ["new", "trunk()"]);
}

#[test]
fn parses_git_remotes_from_jj_remote_list_output() {
    let stdout = "origin https://example.com/repo.git\nupstream git@github.com:org/repo.git\n";
    assert_eq!(parse_git_remotes(stdout), vec!["origin", "upstream"]);
}

#[test]
fn summarize_output_prefers_real_output_over_fallback() {
    assert_eq!(
        summarize_output(b"fetched origin\n", b"", "fetched"),
        "fetched origin"
    );
    assert_eq!(summarize_output(b"", b"warning\n", "fetched"), "warning");
    assert_eq!(summarize_output(b"", b"", "fetched"), "fetched");
}

#[test]
fn parse_exact_change_id_requires_exactly_one_result() {
    assert_eq!(parse_exact_change_id("abc\n").unwrap(), "abc");
    assert!(parse_exact_change_id("").is_err());
    assert!(parse_exact_change_id("abc\ndef\n").is_err());
}

#[test]
fn parse_exact_change_id_rejects_graph_like_output() {
    assert!(parse_exact_change_id("@ abcdefghijkl\n│  some graph suffix").is_err());
}

#[test]
fn resolve_exact_change_id_command_uses_no_graph_contract() {
    assert_eq!(
        resolve_exact_change_id_command_argv("main"),
        vec!["log", "--no-graph", "-r", "main", "-T", CHANGE_ID_TEMPLATE,]
    );
}
