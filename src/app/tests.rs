use super::navigation::initial_view;
use super::*;
use crate::action_menu::FollowUp;
use crate::action_menu::RolePromptOption;
use crate::action_menu::{ActionKind, RolePrompt};
use crate::action_output::ActionOutput;
use crate::action_output::action_output_visible_lines;
use crate::app::mode_input::{rebase_plan_from_prompt, squash_plan_from_prompt};
use crate::jj::{
    JjBookmarkTarget, JjDescribeTarget, JjGitFetch, JjGitPushTarget, JjOperationRecoveryKind,
};
use crate::tui::Overlay;
use color_eyre::eyre::eyre;
use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use std::sync::atomic::{AtomicUsize, Ordering};

static ABANDON_DRIFT_RECHECK_CALLS: AtomicUsize = AtomicUsize::new(0);
static ABANDON_FAILED_RECHECK_CALLS: AtomicUsize = AtomicUsize::new(0);
static NEW_TRUNK_CALLS: AtomicUsize = AtomicUsize::new(0);
static OPERATION_RESTORE_REFRESH_CALLS: AtomicUsize = AtomicUsize::new(0);
static OPERATION_REVERT_REFRESH_CALLS: AtomicUsize = AtomicUsize::new(0);

fn mock_new_success(new_change: &JjNewPlan) -> Result<String> {
    Ok(format!("new parents: {}", new_change.parents().join(",")))
}

fn mock_new_failure(_: &JjNewPlan) -> Result<String> {
    Err(eyre!("jj new failed: first line\nsecond line"))
}

fn mock_rebase_success(_: &JjRebasePlan) -> Result<String> {
    Ok("rebased".to_owned())
}

fn mock_rebase_failure(_: &JjRebasePlan) -> Result<String> {
    Err(eyre!("jj rebase failed: first line\nsecond line"))
}

fn mock_split_success(split: &JjSplitPlan) -> Result<String> {
    Ok(split.success_result_message("exit status: 0"))
}

fn mock_split_failure(split: &JjSplitPlan) -> Result<String> {
    assert_eq!(split.command_label(), "jj split");
    Err(eyre!("jj split failed with status exit status: 1"))
}

fn mock_split_success_service(
    _terminal: Option<&mut DefaultTerminal>,
    split: &JjSplitPlan,
) -> Result<String> {
    mock_split_success(split)
}

fn mock_split_failure_service(
    _terminal: Option<&mut DefaultTerminal>,
    split: &JjSplitPlan,
) -> Result<String> {
    mock_split_failure(split)
}

fn mock_squash_success(_: &JjSquashPlan) -> Result<String> {
    Ok("squashed".to_owned())
}

fn mock_squash_failure(_: &JjSquashPlan) -> Result<String> {
    Err(eyre!("jj squash failed: first line\nsecond line"))
}

fn mock_absorb_success(_: &JjAbsorbPlan) -> Result<String> {
    Ok("absorbed".to_owned())
}

fn mock_absorb_failure(_: &JjAbsorbPlan) -> Result<String> {
    Err(eyre!("jj absorb failed: first line\nsecond line"))
}

fn mock_restore_success(restore: &JjRestorePlan) -> Result<String> {
    Ok(match restore.path() {
        Some(path) => format!("restored {} from {}", path, restore.revision()),
        None => format!("restored {}", restore.revision()),
    })
}

fn mock_restore_failure(_: &JjRestorePlan) -> Result<String> {
    Err(eyre!("jj restore failed: first line\nsecond line"))
}

fn mock_restore_preview_success(restore: &JjRestorePlan) -> Result<String> {
    Ok(match restore.path() {
        Some(path) => format!(
            "target revision: {}\nselected path: {}\nexact fileset: root-file:\"{}\"\nundo path: jj undo",
            restore.revision(),
            path,
            path
        ),
        None => format!(
            "target revision: {}\nundo path: jj undo",
            restore.revision()
        ),
    })
}

fn mock_revert_success(revert: &JjRevertPlan) -> Result<String> {
    Ok(format!("reverted {}", revert.revision()))
}

fn mock_revert_failure(_: &JjRevertPlan) -> Result<String> {
    Err(eyre!("jj revert failed: first line\nsecond line"))
}

fn mock_revert_preview_success(revert: &JjRevertPlan) -> Result<String> {
    Ok(format!(
        "target revision: {}\nforward diff:\nM src/main.rs\nundo path: jj undo",
        revert.revision()
    ))
}

fn mock_describe_success(describe: &JjDescribePlan) -> Result<String> {
    Ok(format!("described {}", describe.target().label()))
}

fn mock_describe_failure(_: &JjDescribePlan) -> Result<String> {
    Err(eyre!("jj describe failed: first line\nsecond line"))
}

fn mock_commit_success(_: &JjCommitPlan) -> Result<String> {
    Ok("committed working copy".to_owned())
}

fn mock_commit_failure(_: &JjCommitPlan) -> Result<String> {
    Err(eyre!("jj commit failed: first line\nsecond line"))
}

fn mock_bookmark_mutation_success(mutation: &JjBookmarkMutationPlan) -> Result<String> {
    Ok(format!(
        "bookmark {} {}",
        mutation.kind().label(),
        mutation.name()
    ))
}

fn mock_bookmark_mutation_failure(_: &JjBookmarkMutationPlan) -> Result<String> {
    Err(eyre!("jj bookmark failed: first line\nsecond line"))
}

fn mock_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    Ok(JjAbandonPreview::new(
        abandon.revision().to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    ))
}

fn mock_non_empty_abandon_preview(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    Ok(JjAbandonPreview::new(
        abandon.revision().to_owned(),
        Some("Edit change".to_owned()),
        "M src/main.rs\n".to_owned(),
    ))
}

fn mock_abandon_preview_drifts_to_non_empty(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    if ABANDON_DRIFT_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        mock_empty_abandon_preview(abandon)
    } else {
        mock_non_empty_abandon_preview(abandon)
    }
}

fn mock_abandon_preview_recheck_failure(abandon: &JjAbandonPlan) -> Result<JjAbandonPreview> {
    if ABANDON_FAILED_RECHECK_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        mock_empty_abandon_preview(abandon)
    } else {
        Err(eyre!("jj diff -r change-a --summary failed: disappeared"))
    }
}

fn mock_abandon_success(_: &JjAbandonPlan) -> Result<String> {
    Ok("abandoned".to_owned())
}

fn mock_abandon_failure(_: &JjAbandonPlan) -> Result<String> {
    Err(eyre!("jj abandon change-a failed: first line\nsecond line"))
}

fn mock_operation_recovery_success(recovery: &JjOperationRecovery) -> Result<String> {
    Ok(match recovery.kind() {
        JjOperationRecoveryKind::Undo => "undone operation".to_owned(),
        JjOperationRecoveryKind::Redo => "redone operation".to_owned(),
    })
}

fn mock_operation_recovery_failure(recovery: &JjOperationRecovery) -> Result<String> {
    Err(eyre!(
        "{} failed: no operation to {} available\nhint: run the opposite recovery command first",
        recovery.command_label(),
        recovery.status_action()
    ))
}

fn mock_operation_target_success(target: &JjOperationTarget) -> Result<String> {
    Ok(format!(
        "operation {} {}\nnew operation recorded",
        target.status_action(),
        target.operation_id()
    ))
}

fn mock_operation_target_failure(target: &JjOperationTarget) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        target.command_label()
    ))
}

fn mock_working_copy_navigation_success(
    navigation: &JjWorkingCopyNavigationPlan,
) -> Result<String> {
    Ok(match navigation.kind() {
        JjWorkingCopyNavigationKind::Edit => format!(
            "editing {}",
            navigation
                .target_change_id()
                .expect("edit mock requires exact target change id")
        ),
        JjWorkingCopyNavigationKind::Next => "moved to next editable change".to_owned(),
        JjWorkingCopyNavigationKind::Prev => "moved to previous editable change".to_owned(),
    })
}

fn mock_working_copy_navigation_failure(
    navigation: &JjWorkingCopyNavigationPlan,
) -> Result<String> {
    Err(eyre!(
        "{} failed: first line\nsecond line",
        navigation.command_label()
    ))
}

fn mock_no_remotes() -> Result<Vec<String>> {
    Ok(Vec::new())
}

fn mock_single_remote() -> Result<Vec<String>> {
    Ok(vec!["origin".to_owned()])
}

fn mock_multiple_remotes() -> Result<Vec<String>> {
    Ok(vec!["origin".to_owned(), "upstream".to_owned()])
}

fn mock_push_preview_success(push: &JjGitPush) -> Result<String> {
    Ok(format!("preview: {}", push.command_label(true)))
}

fn mock_push_success(push: &JjGitPush) -> Result<String> {
    Ok(format!("pushed: {}", push.command_label(false)))
}

fn mock_resolve_current_change_id(revset: &str) -> Result<String> {
    assert_eq!(revset, "@");
    Ok("new-working-copy".to_owned())
}

fn mock_resolve_trunk_and_current_change_id(revset: &str) -> Result<String> {
    match revset {
        "trunk()" => Ok("trunk-change".to_owned()),
        "@" => Ok("new-working-copy".to_owned()),
        other => panic!("unexpected revset: {other}"),
    }
}

fn mock_new_trunk_success() -> Result<String> {
    NEW_TRUNK_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok("created new change from trunk".to_owned())
}

fn mock_fetch_success(fetch: &JjGitFetch) -> Result<String> {
    Ok(match fetch.remote() {
        Some(remote) => format!("fetched {remote}"),
        None => "fetched".to_owned(),
    })
}

fn mock_fetch_failure(fetch: &JjGitFetch) -> Result<String> {
    Err(eyre!("{} failed: denied", fetch.command_label()))
}

fn mock_remotes_failure() -> Result<Vec<String>> {
    Err(eyre!("jj git remote list failed: denied"))
}

fn mock_load_view(spec: ViewSpec) -> Result<ViewState> {
    let view = match spec.command() {
        JjCommand::Default | JjCommand::Log => {
            ViewState::Graph(crate::graph::GraphView::test_with_spec(spec, vec![]))
        }
        JjCommand::Show => ViewState::Show(crate::show::ShowView::test_new(spec)),
        JjCommand::Diff => ViewState::Diff(crate::diff::DiffView::test_new(spec)),
        JjCommand::Status => ViewState::Status(crate::status::StatusView::test_new(&[])),
        JjCommand::Resolve => ViewState::Resolve(crate::resolve::ResolveView::test_new(vec![])),
        JjCommand::FileList => {
            ViewState::FileList(crate::file_list::FileListView::test_new(vec![]))
        }
        JjCommand::FileShow => ViewState::FileShow(crate::file_show::FileShowView::new(
            spec,
            "src/main.rs",
            crate::rendered_jj::DocumentLines::new(Vec::new()),
        )),
        JjCommand::Bookmarks => {
            ViewState::Bookmarks(crate::bookmarks::BookmarksView::test_new(vec![]))
        }
        JjCommand::OperationLog => {
            ViewState::OperationLog(crate::operation_log::OperationLogView::test_new(vec![]))
        }
        JjCommand::OperationShow | JjCommand::OperationDiff => {
            ViewState::OperationDetail(crate::operation_detail::OperationDetailView::test_new(
                spec,
                crate::rendered_jj::DocumentLines::new(Vec::new()),
            ))
        }
    };
    Ok(view)
}

fn panic_abandon_run(_: &JjAbandonPlan) -> Result<String> {
    panic!("abandon should not run without exact confirmation")
}

fn mock_refresh_ok(_view: &mut ViewState) -> Result<()> {
    Ok(())
}

fn mock_operation_restore_counting_refresh_ok(_view: &mut ViewState) -> Result<()> {
    OPERATION_RESTORE_REFRESH_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

fn mock_operation_revert_second_refresh_failure(_view: &mut ViewState) -> Result<()> {
    if OPERATION_REVERT_REFRESH_CALLS.fetch_add(1, Ordering::SeqCst) == 0 {
        Ok(())
    } else {
        Err(eyre!("view refresh failed"))
    }
}

fn mock_refresh_failure(_view: &mut ViewState) -> Result<()> {
    Err(eyre!("view refresh failed"))
}

fn mock_reveal_graph_change_error(
    _view: &mut ViewState,
    _change_id: &str,
    _fallback_mode: LogViewMode,
) -> Result<bool> {
    Err(eyre!(
        "refreshed graph did not include the new working-copy change"
    ))
}

fn mock_reveal_new_change_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "new-working-copy");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

fn mock_reveal_described_change_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(false)
}

fn mock_reveal_rebased_source_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "source-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

fn mock_reveal_squash_destination_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "dest");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

fn mock_reveal_edit_target_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "change-a");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(false)
}

fn mock_reveal_current_working_copy_in_recent(
    _view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    assert_eq!(change_id, "new-working-copy");
    assert_eq!(fallback_mode, LogViewMode::Recent);
    Ok(true)
}

fn graph_item(change_id: &str) -> crate::jj::LogItem {
    crate::jj::LogItem::new(
        vec![ratatui::text::Line::from(change_id.to_owned())],
        Some(change_id.to_owned()),
        None,
    )
}

fn default_reveal_graph_change(
    view: &mut ViewState,
    change_id: &str,
    fallback_mode: LogViewMode,
) -> Result<bool> {
    view.reveal_graph_change(change_id, fallback_mode)
}

fn test_services() -> AppServices {
    let mut services = AppServices::default();
    services.new_run = mock_new_success;
    services.rebase_run = mock_rebase_success;
    services.split_run = mock_split_success_service;
    services.squash_run = mock_squash_success;
    services.absorb_run = mock_absorb_success;
    services.restore_run = mock_restore_success;
    services.revert_run = mock_revert_success;
    services.restore_preview_load = mock_restore_preview_success;
    services.revert_preview_load = mock_revert_preview_success;
    services.describe_run = mock_describe_success;
    services.commit_run = mock_commit_success;
    services.bookmark_mutation_run = mock_bookmark_mutation_success;
    services.abandon_preview_load = mock_empty_abandon_preview;
    services.abandon_run = mock_abandon_success;
    services.operation_recovery_run = mock_operation_recovery_success;
    services.operation_target_run = mock_operation_target_success;
    services.working_copy_navigation_run = mock_working_copy_navigation_success;
    services.resolve_revision = mock_resolve_current_change_id;
    services.new_trunk_run = mock_new_trunk_success;
    services.git_fetch_run = mock_fetch_success;
    services.git_remotes_load = mock_multiple_remotes;
    services.push_preview_run = mock_push_preview_success;
    services.push_run = mock_push_success;
    services.refresh_view = mock_refresh_ok;
    services.reveal_graph_change = default_reveal_graph_change;
    services
}

fn test_app(view: ViewState) -> App {
    App {
        status: StatusLine::ready(&view),
        view,
        stack: Vec::new(),
        startup_log_args: None,
        diff_format: DiffFormat::Default,
        mode: InteractionMode::Normal,
        pending_command: None,
        search: None,
        should_quit: false,
        services: test_services(),
    }
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

#[test]
fn parses_default_startup_view() {
    let spec = initial_view(Vec::new()).unwrap();

    assert_eq!(spec.command(), JjCommand::Default);
    assert!(spec.args().is_empty());
}

#[test]
fn parses_passthrough_startup_view() {
    let spec = initial_view(vec!["log".into(), "-r".into(), "::".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::Log);
    assert_eq!(spec.args(), ["-r", "::"]);
}

#[test]
fn parses_show_startup_view() {
    let spec = initial_view(vec!["show".into(), "--git".into(), "main".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::Show);
    assert_eq!(spec.args(), ["--git", "main"]);
    assert_eq!(spec.diff_format(), DiffFormat::Git);
}

#[test]
fn parses_diff_startup_view() {
    let spec = initial_view(vec!["diff".into(), "-r".into(), "main".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::Diff);
    assert_eq!(spec.args(), ["-r", "main"]);
}

#[test]
fn parses_status_startup_view() {
    let spec = initial_view(vec!["status".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::Status);
    assert!(spec.args().is_empty());
}

#[test]
fn parses_resolve_startup_view() {
    let spec = initial_view(vec!["resolve".into(), "-r".into(), "main".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::Resolve);
    assert_eq!(spec.args(), ["-r", "main"]);
    assert_eq!(spec.navigation_revset().as_deref(), Some("main"));
}

#[test]
fn parses_default_resolve_startup_view() {
    let spec = initial_view(vec!["resolve".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::Resolve);
    assert_eq!(spec.args(), ["-r", "@"]);
    assert_eq!(spec.navigation_revset().as_deref(), Some("@"));
}

#[test]
fn open_resolve_uses_default_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.open_resolve().unwrap();

    assert_eq!(app.view.spec().command(), JjCommand::Resolve);
    assert_eq!(app.view.spec().args(), ["-r", "@"]);
    assert_eq!(app.view.spec().navigation_revset().as_deref(), Some("@"));
}

#[test]
fn parses_operation_log_startup_view() {
    let spec = initial_view(vec!["operation-log".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::OperationLog);
    assert!(spec.args().is_empty());
}

#[test]
fn parses_bookmarks_startup_view() {
    let spec = initial_view(vec!["bookmarks".into()]).unwrap();

    assert_eq!(spec.command(), JjCommand::Bookmarks);
    assert!(spec.args().is_empty());
}

#[test]
fn rejects_unknown_startup_command() {
    assert!(initial_view(vec!["bookmark".into()]).is_err());
}

#[test]
fn direct_view_entry_keys_open_shipped_top_level_views() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('S'), KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), JjCommand::Status);
    assert!(app.pending_command.is_none());

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('B'), KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), JjCommand::Bookmarks);
    assert!(app.pending_command.is_none());

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('O'), KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), JjCommand::OperationLog);
    assert!(app.pending_command.is_none());
}

#[test]
fn direct_log_key_reuses_startup_args_and_clears_stack() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[])));
    app.services.load_view = mock_load_view;
    app.startup_log_args = Some(vec!["-r".to_owned(), "mine()".to_owned()]);
    app.stack.push(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![]),
    ));

    app.handle_normal_key(key(KeyCode::Char('L'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(app.view.command(), JjCommand::Log);
    assert_eq!(app.view.spec().args(), ["-r", "mine()"]);
    assert!(app.stack.is_empty());
    assert!(app.pending_command.is_none());
}

#[test]
fn direct_default_key_loads_default_view_and_clears_stack() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[])));
    app.services.load_view = mock_load_view;
    app.startup_log_args = Some(vec!["-r".to_owned(), "mine()".to_owned()]);
    app.stack.push(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![]),
    ));

    app.handle_normal_key(key(KeyCode::Char('J'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(app.view.command(), JjCommand::Default);
    assert!(app.view.spec().args().is_empty());
    assert!(app.stack.is_empty());
    assert!(app.pending_command.is_none());
}

#[test]
fn generated_help_uses_same_multikey_and_view_entry_bindings_as_dispatch() {
    let sections = crate::command::project_help(
        APP_BINDINGS,
        crate::graph::BINDINGS,
        crate::command::HelpContext::Graph,
    );
    let rows = sections
        .iter()
        .flat_map(|section| section.rows())
        .map(|row| (row.keys(), row.action()))
        .collect::<Vec<_>>();

    assert!(rows.contains(&("S", "status")));
    assert!(rows.contains(&("B", "bookmarks")));
    assert!(rows.contains(&("O", "operation log")));
    assert!(rows.contains(&("b, bc", "create bookmark here")));
    assert!(rows.contains(&("f", "fetch")));
    assert!(rows.contains(&("gf", "fetch")));
    assert!(rows.contains(&("F", "fetch remote")));
    assert!(rows.contains(&("gr", "fetch remote")));
    assert!(rows.contains(&("v", "view menu")));

    let status_sections = crate::command::project_help(
        APP_BINDINGS,
        crate::status::BINDINGS,
        crate::command::HelpContext::Status,
    );
    let status_rows = status_sections
        .iter()
        .flat_map(|section| section.rows())
        .map(|row| (row.keys(), row.action()))
        .collect::<Vec<_>>();
    assert!(status_rows.contains(&("f", "fetch")));
    assert!(status_rows.contains(&("F", "fetch remote")));
    assert!(!status_rows.contains(&("f, gf", "fetch")));
}

#[test]
fn help_menu_executes_listed_command_and_closes() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('?'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(matches!(app.mode, InteractionMode::Help));

    app.handle_mode_key(KeyCode::Char('S'), 12).unwrap();

    assert_eq!(app.view.command(), JjCommand::Status);
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert!(app.pending_command.is_none());
}

#[test]
fn help_menu_close_key_closes_without_executing() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('?'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert_eq!(app.view.command(), JjCommand::Default);
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert!(app.pending_command.is_none());
}

#[test]
fn help_menu_does_not_execute_hidden_commands() {
    let show =
        crate::show::ShowView::test_new(ViewSpec::show("change-a".to_owned(), DiffFormat::Default));
    let mut app = test_app(ViewState::Show(show));

    app.handle_normal_key(key(KeyCode::Char('?'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('D'), 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Help));
    assert_eq!(app.view.command(), JjCommand::Show);
    assert_eq!(app.status.message(), "not available from help menu");
}

#[test]
fn help_menu_supports_multikey_options_and_fallbacks() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_fetch_run = mock_fetch_success;

    app.handle_normal_key(key(KeyCode::Char('?'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('g'), 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Help));
    assert!(app.pending_command.is_some());

    app.handle_mode_key(KeyCode::Char('f'), 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::FetchPreview { .. }));
    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "fetch: fetched");
}

#[test]
fn expired_help_prefix_runs_fallback_before_routing_next_key_to_opened_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        graph_item("change-a"),
    ])));

    app.handle_normal_key(key(KeyCode::Char('?'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('b'), 12).unwrap();
    app.pending_command.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);

    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();

    assert!(app.pending_command.is_none());
    let InteractionMode::BookmarkNamePrompt { input, .. } = &app.mode else {
        panic!("expired bare b fallback should open bookmark prompt");
    };
    assert_eq!(input, "x");
}

#[test]
fn help_prefix_nonmatching_suffix_runs_fallback_then_routes_suffix() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        graph_item("first"),
        graph_item("second"),
    ])));

    app.handle_normal_key(key(KeyCode::Char('?'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('g'), 12).unwrap();
    app.handle_mode_key(KeyCode::Char('j'), 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert!(app.pending_command.is_none());
    assert_eq!(app.graph_selected_revision().as_deref(), Some("second"));
}

#[test]
fn help_menu_projection_groups_commands_by_user_operation() {
    let view = ViewState::Graph(crate::graph::GraphView::test_new(vec![]));
    let mode = InteractionMode::Help;

    let Overlay::Help { sections } = mode.overlay(&view, APP_BINDINGS) else {
        panic!("help mode should project a help overlay");
    };

    let titles = sections
        .iter()
        .map(crate::command::HelpSection::title)
        .collect::<Vec<_>>();

    assert_eq!(
        titles,
        vec![
            "Navigation",
            "View Switching",
            "Search / Copy",
            "Repository Actions",
            "Action Previews",
            "App",
        ]
    );
}

#[test]
fn view_menu_selects_shipped_top_level_views() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('v'), KeyModifiers::NONE), 12)
        .unwrap();
    for _ in 0..3 {
        app.handle_mode_key(KeyCode::Down, 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    assert_eq!(app.view.command(), JjCommand::Bookmarks);
    assert!(matches!(app.mode, InteractionMode::Normal));

    app.handle_normal_key(key(KeyCode::Char('v'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Down, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    assert_eq!(app.view.command(), JjCommand::OperationLog);
}

#[test]
fn view_menu_diff_format_status_names_show_diff_scope() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.apply_view_menu_action(ViewMenuAction::DiffFormat(DiffFormat::Git), 12)
        .unwrap();

    assert_eq!(app.status.message(), "show/diff format: git");
}

#[test]
fn multi_key_bookmark_create_dispatches_without_typing_prefix_suffix() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(app.pending_command.is_some());
    assert_eq!(app.status.message(), "prefix: b");

    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();
    match &app.mode {
        InteractionMode::BookmarkNamePrompt { input, .. } => assert_eq!(input, ""),
        _ => panic!("expected bookmark name prompt"),
    }
    assert!(app.pending_command.is_none());
}

#[test]
fn multi_key_fetch_dispatches_from_git_prefix() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.handle_normal_key(key(KeyCode::Char('g'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(app.pending_command.is_some());

    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(app.status.message(), "fetch: fetched");
    assert!(app.pending_command.is_none());
    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), None);
            output
        }
        _ => panic!("expected fetch result mode"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git fetch");
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("default fetch uses jj git fetch remote resolution")
    );
}

#[test]
fn default_fetch_runs_immediately_and_keeps_result_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.handle_normal_key(key(KeyCode::Char('f'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "fetch: fetched");
    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), None);
            output
        }
        _ => panic!("expected fetch result mode"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git fetch");
    assert_eq!(
        output.body_lines(),
        [
            "command: jj git fetch",
            "context: default fetch uses jj git fetch remote resolution",
            "output:",
            "  fetched",
        ]
    );
}

#[test]
fn graph_remote_fetch_key_opens_remote_prompt_for_multiple_remotes() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.handle_normal_key(key(KeyCode::Char('g'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::NONE), 12)
        .unwrap();

    match &app.mode {
        InteractionMode::FetchRemotePrompt { remotes, selected } => {
            assert_eq!(remotes, &["origin".to_owned(), "upstream".to_owned()]);
            assert_eq!(*selected, 0);
        }
        _ => panic!("expected fetch remote prompt"),
    }
}

#[test]
fn fetch_remote_prompt_selects_remote_for_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.open_fetch_remote_prompt();

    app.handle_mode_key(crossterm::event::KeyCode::Down, 12)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), Some("upstream"));
            output
        }
        _ => panic!("expected fetch preview"),
    };
    assert!(!output.completed());
    assert_eq!(
        output.command_label(),
        "jj git fetch --remote exact:upstream"
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("fetch targets exact remote 'upstream' with pattern exact:upstream")
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("remote pattern: exact:upstream")
    );
}

#[test]
fn fetch_remote_skips_prompt_for_single_remote() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_remotes_load = mock_single_remote;

    app.open_fetch_remote_prompt();

    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), Some("origin"));
            output
        }
        _ => panic!("expected fetch preview"),
    };
    assert!(!output.completed());
    assert_eq!(output.command_label(), "jj git fetch --remote exact:origin");
}

#[test]
fn fetch_remote_reports_no_remotes_with_readable_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_remotes_load = mock_no_remotes;

    app.open_fetch_remote_prompt();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(
        app.status.message(),
        "no git remotes found; run default fetch or add a remote before choosing one"
    );
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch output"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git remote list");
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("fetch remote selection found no remotes")
    );
}

#[test]
fn fetch_remote_reports_remote_list_errors_with_readable_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_remotes_load = mock_remotes_failure;

    app.open_fetch_remote_prompt();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(app.status.message(), "jj git remote list failed: denied");
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch output"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git remote list");
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj git remote list failed: denied")
    );
}

#[test]
fn fetch_preview_enter_runs_remote_fetch_and_keeps_result_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.open_fetch_preview("origin".to_owned());

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    assert_eq!(app.status.message(), "fetch origin: fetched origin");
    let output = match &app.mode {
        InteractionMode::FetchPreview { fetch, output } => {
            assert_eq!(fetch.remote(), Some("origin"));
            output
        }
        _ => panic!("expected fetch result"),
    };
    assert!(output.completed());
    assert_eq!(output.command_label(), "jj git fetch --remote exact:origin");
    assert!(output.body_lines().join("\n").contains("fetched origin"));
}

#[test]
fn fetch_failure_keeps_error_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.git_fetch_run = mock_fetch_failure;
    app.open_fetch_preview("origin".to_owned());

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(
        app.status.message(),
        "jj git fetch --remote exact:origin failed: denied"
    );
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj git fetch --remote exact:origin failed: denied")
    );
}

#[test]
fn fetch_success_with_refresh_error_keeps_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.refresh_view = mock_refresh_failure;
    app.open_fetch_preview("origin".to_owned());

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(app.status.message(), "view refresh failed");
    let output = match &app.mode {
        InteractionMode::FetchPreview { output, .. } => output,
        _ => panic!("expected fetch result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("fetched origin"));
    assert!(body.contains("refresh failed: view refresh failed"));
}

#[test]
fn git_fetch_prefix_does_not_delay_non_graph_g_navigation() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "working copy changes:",
        "M src/app.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('g'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "1/2 lines");
}

#[test]
fn expired_bookmark_prefix_runs_fallback_before_next_key() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.pending_command.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);
    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();

    match &app.mode {
        InteractionMode::BookmarkNamePrompt { input, .. } => assert_eq!(input, "c"),
        _ => panic!("expected bookmark name prompt"),
    }
    assert!(app.pending_command.is_none());
}

#[test]
fn idle_command_prefix_timeout_runs_exact_fallback_and_refreshes_status() {
    let mut graph = crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::Graph(graph));

    app.handle_normal_key(key(KeyCode::Char('g'), KeyModifiers::NONE), 12)
        .unwrap();
    app.pending_command.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);
    app.flush_expired_pending_command(12).unwrap();

    let ViewState::Graph(graph) = &app.view else {
        panic!("expected graph view");
    };
    assert_eq!(graph.selected_revision(), Some("first"));
    assert!(app.pending_command.is_none());
    assert_eq!(app.status.message(), "2 items | default work");
}

#[test]
fn command_prefix_cancel_does_not_run_global_escape() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Esc, KeyModifiers::NONE), 12)
        .unwrap();

    assert!(!app.should_quit);
    assert!(app.pending_command.is_none());
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "prefix cancelled");
}

#[test]
fn right_and_l_open_expandable_detail_and_h_or_left_backs_out() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('l'), KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), JjCommand::Show);

    app.handle_normal_key(key(KeyCode::Char('h'), KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), JjCommand::Default);

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::jj::BookmarkItem::new(
            Vec::new(),
            "main".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Right, KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), JjCommand::Show);

    app.handle_normal_key(key(KeyCode::Left, KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(app.view.command(), JjCommand::Bookmarks);
}

#[test]
fn operation_log_l_opens_operation_detail() {
    let operation_id = "op123".to_owned();
    let mut app = test_app(ViewState::OperationLog(
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some(operation_id),
        )]),
    ));
    app.services.load_view = mock_load_view;

    app.handle_normal_key(key(KeyCode::Char('l'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(app.view.command(), JjCommand::OperationShow);
}

#[test]
fn open_push_prompt_requires_exact_graph_revision() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));

    assert!(!app.open_push_prompt().unwrap());
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "push from graph requires a selected row with an exact revision"
    );
}

#[test]
fn open_push_prompt_skips_remote_prompt_for_single_remote() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.git_remotes_load = mock_single_remote;

    assert!(!app.open_push_prompt().unwrap());

    let output = match &app.mode {
        InteractionMode::PushPreview { push, output } => {
            assert_eq!(push.remote(), Some("origin"));
            output
        }
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.command_label(),
        "jj git push --dry-run --remote origin --revision abcdef"
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("graph push targets exact selected revision 'abcdef' on remote origin")
    );
}

#[test]
fn open_push_prompt_keeps_remote_prompt_for_multiple_remotes() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.git_remotes_load = mock_multiple_remotes;

    assert!(!app.open_push_prompt().unwrap());

    match &app.mode {
        InteractionMode::PushRemotePrompt {
            target,
            remotes,
            selected,
        } => {
            assert_eq!(target, &JjGitPushTarget::Revision("abcdef".to_owned()));
            assert_eq!(remotes, &["origin".to_owned(), "upstream".to_owned()]);
            assert_eq!(*selected, 0);
        }
        _ => panic!("expected push remote prompt"),
    }
}

#[test]
fn open_push_prompt_reports_no_remote_error() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.git_remotes_load = mock_no_remotes;

    assert!(!app.open_push_prompt().unwrap());

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "no git remotes found; add a remote before pushing"
    );
}

#[test]
fn open_push_prompt_reports_unsupported_view_error() {
    let mut app = test_app(ViewState::OperationLog(
        crate::operation_log::OperationLogView::test_new(Vec::new()),
    ));

    assert!(!app.open_push_prompt().unwrap());

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "push is only available from graph, status, or bookmarks views"
    );
}

#[test]
fn push_preview_context_names_status_default_resolution() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));

    app.open_push_preview(JjGitPushTarget::Status, "origin".to_owned());

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("status push uses jj default target resolution for remote origin")
    );
    assert_eq!(
        output.command_label(),
        "jj git push --dry-run --remote origin"
    );
}

#[test]
fn push_preview_context_names_exact_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::jj::BookmarkItem::new(
            Vec::new(),
            "feature".to_owned(),
            Some("abcdef".to_owned()),
            None,
        )]),
    ));

    app.open_push_preview(
        JjGitPushTarget::Bookmark("feature".to_owned()),
        "origin".to_owned(),
    );

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark push targets exact bookmark 'feature' on remote origin")
    );
    assert_eq!(
        output.command_label(),
        "jj git push --dry-run --remote origin --bookmark feature"
    );
}

#[test]
fn push_result_keeps_context_until_closed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_revision("abcdef".to_owned()).with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --dry-run --remote origin --revision abcdef".to_owned(),
            "preview only".to_owned(),
            Some("graph push targets exact selected revision 'abcdef' on remote origin".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push result mode"),
    };
    assert!(output.completed());
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("graph push targets exact selected revision 'abcdef' on remote origin")
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("pushed: jj git push --remote origin --revision abcdef")
    );

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
}

#[test]
fn push_preview_entering_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --remote origin --revision abcdef".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "push cancelled");
}

#[test]
fn push_confirm_success_with_refresh_error_keeps_output() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_revision("abcdef".to_owned()).with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --dry-run --remote origin --revision abcdef".to_owned(),
            "preview only".to_owned(),
            Some("graph push targets exact selected revision 'abcdef' on remote origin".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("pushed: jj git push --remote origin --revision abcdef"));
    assert!(body.contains("refresh failed: view refresh failed"));
    assert_eq!(app.status.message(), "view refresh failed");
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn push_preview_completion_stays_until_closed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::finished(
            "jj git push --remote origin".to_owned(),
            "pushed".to_owned(),
            Some("status push uses jj default target resolution for remote origin".to_owned()),
        ),
    };
    app.status = StatusLine::with_message(&app.view, "pushed");

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "pushed");
}

#[test]
fn action_output_scroll_keys_clamp_to_visible_body() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --preview --remote origin".to_owned(),
            (0..8)
                .map(|line| format!("line {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Char('j'), 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::PageDown, 4)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(
        output.scroll(),
        output.max_scroll(action_output_visible_lines(4))
    );

    app.handle_mode_key(crossterm::event::KeyCode::PageUp, 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Char('k'), 4)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Char('g'), 4)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::PushPreview { output, .. } => output,
        _ => panic!("expected push preview mode"),
    };
    assert_eq!(output.scroll(), 0);
}

#[test]
fn closing_action_output_preserves_graph_selection() {
    let mut graph = crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::Graph(graph));
    app.mode = InteractionMode::PushPreview {
        push: JjGitPush::for_status().with_remote("origin"),
        output: ActionOutput::pending(
            "jj git push --preview --remote origin".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    let ViewState::Graph(graph) = &app.view else {
        panic!("expected graph view");
    };
    assert_eq!(graph.selected_revision(), Some("second"));
    assert!(matches!(app.mode, InteractionMode::Normal));
}

#[test]
fn bookmark_create_prompt_uses_exact_graph_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "feature/name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Create);
    assert_eq!(mutation.name(), "feature/name");
    assert_eq!(
        output.command_label(),
        "jj bookmark create --revision exactly(change_id(\"change-a\"), 1) feature/name"
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("destination: exact selected revision change-a"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark create"));
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark create 'feature/name' targets change-a from jk")
    );
}

#[test]
fn bookmark_set_prompt_uses_status_current_working_copy_target() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "working copy changes:",
        "M src/app.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('='), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "feature/name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Set);
    assert_eq!(
        output.command_label(),
        "jj bookmark set --revision @ feature/name"
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("destination: current working-copy change (@)")
    );
}

#[test]
fn bookmark_move_prompt_uses_exact_pattern_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('m'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "feature/name".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark preview"),
    };
    assert_eq!(
        output.command_label(),
        "jj bookmark move --to exactly(change_id(\"change-a\"), 1) exact:\"feature/name\""
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("source/current: exact pattern exact:\"feature/name\"")
    );
}

#[test]
fn bookmark_prompt_cancel_and_empty_input_do_not_open_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark create cancelled");

    app.handle_normal_key(key(KeyCode::Char('='), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark set cancelled: empty bookmark name"
    );
}

#[test]
fn bookmark_mutation_rejects_unsupported_and_inexact_contexts() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('b'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "bookmark mutation from graph requires a selected row with an exact revision"
    );

    let mut app = test_app(ViewState::OperationLog(
        crate::operation_log::OperationLogView::test_new(Vec::new()),
    ));
    app.handle_normal_key(key(KeyCode::Char('m'), KeyModifiers::NONE), 12)
        .unwrap();
    assert_eq!(
        app.status.message(),
        "bookmark move is only available from graph or status views"
    );
}

#[test]
fn bookmark_delete_preview_uses_selected_exact_local_bookmark() {
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::jj::BookmarkItem::new(
            Vec::new(),
            "feature/name".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('x'), KeyModifiers::NONE), 12)
        .unwrap();

    let (mutation, output) = match &app.mode {
        InteractionMode::BookmarkMutationPreview { mutation, output } => (mutation, output),
        _ => panic!("expected bookmark delete preview"),
    };
    assert_eq!(mutation.kind(), JjBookmarkMutationKind::Delete);
    assert_eq!(
        output.command_label(),
        "jj bookmark delete exact:\"feature/name\""
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("effect: deletes one local bookmark; this is delete, not forget"));
    assert!(body.contains("track/untrack stay disabled"));
    assert!(body.contains("confirmation: press Enter to run jj bookmark delete"));
}

#[test]
fn bookmark_delete_rejects_nonlocal_bookmark_rows() {
    let remote = crate::jj::BookmarkItem::new(Vec::new(), "@origin".to_owned(), None, None)
        .with_local(false);
    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![remote]),
    ));

    app.handle_normal_key(key(KeyCode::Char('x'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "delete requires a selected exact local bookmark"
    );
}

#[test]
fn file_list_x_is_not_bookmark_delete() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_new(vec![crate::jj::FileListItem::new(
            vec![ratatui::text::Line::from("src/lib.rs")],
            "src/lib.rs".to_owned(),
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('x'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "1 files");
}

#[test]
fn bookmark_mutation_confirm_success_failure_and_cancel_are_inspectable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::create(
            "feature/name",
            JjBookmarkTarget::exact_change("change-a"),
        ),
        output: ActionOutput::pending(
            "jj bookmark create --revision exactly(change_id(\"change-a\"), 1) feature/name"
                .to_owned(),
            "preview only".to_owned(),
            Some("bookmark create context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("bookmark create feature/name | jj undo")
    );
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("bookmark create context")
    );
    assert_eq!(
        app.status.message(),
        "bookmark create feature/name | jj undo"
    );

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.bookmark_mutation_run = mock_bookmark_mutation_failure;
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::set(
            "feature/name",
            JjBookmarkTarget::exact_change("change-a"),
        ),
        output: ActionOutput::pending(
            "jj bookmark set --revision exactly(change_id(\"change-a\"), 1) feature/name"
                .to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::BookmarkMutationPreview { output, .. } => output,
        _ => panic!("expected bookmark result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj bookmark failed: first line")
    );
    assert_eq!(
        app.status.message(),
        "jj bookmark failed: first line\nsecond line"
    );

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::BookmarkMutationPreview {
        mutation: JjBookmarkMutationPlan::move_to(
            "feature/name",
            JjBookmarkTarget::exact_change("change-a"),
        ),
        output: ActionOutput::pending(
            "jj bookmark move --to exactly(change_id(\"change-a\"), 1) exact:\"feature/name\""
                .to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "bookmark move cancelled");
}

#[test]
fn describe_prompt_types_backspaces_and_opens_preview_for_exact_graph_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "Mesx".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Backspace, 12).unwrap();
    app.handle_mode_key(KeyCode::Char('g'), 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (describe, output) = match &app.mode {
        InteractionMode::DescribePreview { describe, output } => (describe, output),
        _ => panic!("expected describe preview"),
    };
    assert_eq!(
        describe.target(),
        &JjDescribeTarget::ExactChange("change-a".to_owned())
    );
    assert_eq!(
        output.command_label(),
        "jj describe change-a --message Mesg"
    );
    let body = output.body_lines().join("\n");
    assert!(body.contains("target: exact selected revision change-a"));
    assert!(body.contains("message: Mesg"));
    assert!(body.contains("without opening an editor"));
}

#[test]
fn describe_prompt_types_and_opens_preview_for_status_target() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "working copy changes:",
        "M src/app.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "Message".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (describe, output) = match &app.mode {
        InteractionMode::DescribePreview { describe, output } => (describe, output),
        _ => panic!("expected describe preview"),
    };
    assert_eq!(describe.target(), &JjDescribeTarget::CurrentWorkingCopy);
    assert_eq!(output.command_label(), "jj describe @ --message Message");
    let body = output.body_lines().join("\n");
    assert!(body.contains("target: current working-copy change (@)"));
    assert!(body.contains("message: Message"));
    assert!(body.contains("without opening an editor"));
}

#[test]
fn describe_prompt_cancel_and_empty_input_do_not_open_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "describe cancelled");

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "describe cancelled: empty description"
    );
}

#[test]
fn describe_requires_exact_graph_target_and_rejects_unsupported_context() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "describe from graph requires a selected row with an exact revision"
    );

    let mut app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::jj::BookmarkItem::new(
            Vec::new(),
            "main".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    app.handle_normal_key(key(KeyCode::Char('D'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(
        app.status.message(),
        "describe is only available from graph or status views"
    );
}

#[test]
fn describe_confirm_success_refreshes_reveals_and_keeps_undo_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.reveal_graph_change = mock_reveal_described_change_in_recent;
    app.mode = InteractionMode::DescribePreview {
        describe: JjDescribePlan::new(
            JjDescribeTarget::exact_change("change-a"),
            "New description",
        ),
        output: ActionOutput::pending(
            "jj describe change-a --message New description".to_owned(),
            "preview only".to_owned(),
            Some("describe change-a from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DescribePreview { output, .. } => output,
        _ => panic!("expected describe result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("described change-a | jj undo")
    );
    assert_eq!(app.status.message(), "described change-a | jj undo");
}

#[test]
fn describe_failure_and_refresh_failure_remain_inspectable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.describe_run = mock_describe_failure;
    app.mode = InteractionMode::DescribePreview {
        describe: JjDescribePlan::new(JjDescribeTarget::exact_change("change-a"), "New"),
        output: ActionOutput::pending(
            "jj describe change-a --message New".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DescribePreview { output, .. } => output,
        _ => panic!("expected describe result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj describe failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj describe failed: first line\nsecond line"
    );

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::DescribePreview {
        describe: JjDescribePlan::new(JjDescribeTarget::exact_change("change-a"), "New"),
        output: ActionOutput::pending(
            "jj describe change-a --message New".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::DescribePreview { output, .. } => output,
        _ => panic!("expected describe result"),
    };
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("described change-a | refresh failed: view refresh failed | jj undo")
    );
    assert_eq!(app.status.message(), "view refresh failed");
}

#[test]
fn commit_prompt_is_honest_about_current_working_copy_target() {
    let mut graph = crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("historical".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("selected-row".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::Graph(graph));

    app.handle_normal_key(key(KeyCode::Char('C'), KeyModifiers::NONE), 12)
        .unwrap();
    for character in "Commitx".chars() {
        app.handle_mode_key(KeyCode::Char(character), 12).unwrap();
    }
    app.handle_mode_key(KeyCode::Backspace, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj commit --message Commit");
    assert!(body.contains("target: current working-copy change (@)"));
    assert!(body.contains("selected graph rows are not arguments"));
    assert!(!body.contains("selected-row"));
}

#[test]
fn commit_prompt_cancel_and_empty_input_do_not_open_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('C'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('x'), 12).unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "commit cancelled");

    app.handle_normal_key(key(KeyCode::Char('C'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "commit cancelled: empty description");
}

#[test]
fn commit_confirm_success_and_failure_keep_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::CommitPreview {
        commit: JjCommitPlan::new("Commit"),
        output: ActionOutput::pending(
            "jj commit --message Commit".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(
        body.contains("committed working copy | new working-copy change created on top | jj undo")
    );
    assert_eq!(
        app.status.message(),
        "committed working copy | new working-copy change created on top | jj undo"
    );

    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.commit_run = mock_commit_failure;
    app.mode = InteractionMode::CommitPreview {
        commit: JjCommitPlan::new("Commit"),
        output: ActionOutput::pending(
            "jj commit --message Commit".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj commit failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj commit failed: first line\nsecond line"
    );
}

#[test]
fn commit_refresh_failure_keeps_undo_and_new_working_copy_effect_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::CommitPreview {
        commit: JjCommitPlan::new("Commit"),
        output: ActionOutput::pending(
            "jj commit --message Commit".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::CommitPreview { output, .. } => output,
        _ => panic!("expected commit result"),
    };
    assert!(output.body_lines().join("\n").contains(
        "committed working copy | refresh failed: view refresh failed | new working-copy change created on top | jj undo"
    ));
    assert_eq!(app.status.message(), "view refresh failed");
}

#[test]
fn rebase_plan_from_prompt_respects_explicit_roles() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "bbbbbbbb1111111111111111111111111111111111"),
            RolePromptOption::new("destination", "cccccccc2222222222222222222222222222222222"),
            RolePromptOption::new("source", "aaaaaaaa3333333333333333333333333333333333"),
        ],
        "Preview required before execution.",
    );

    let rebase =
        rebase_plan_from_prompt(&prompt).expect("role prompt should include a destination");

    assert_eq!(
        rebase.sources(),
        &[
            "bbbbbbbb1111111111111111111111111111111111",
            "aaaaaaaa3333333333333333333333333333333333"
        ]
    );
    assert_eq!(
        rebase.destination(),
        "cccccccc2222222222222222222222222222222222"
    );
}

#[test]
fn squash_plan_from_prompt_respects_explicit_roles() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "bbbbbbbb1111111111111111111111111111111111"),
            RolePromptOption::new("destination", "cccccccc2222222222222222222222222222222222"),
            RolePromptOption::new("source", "aaaaaaaa3333333333333333333333333333333333"),
        ],
        "Preview required before execution.",
    );

    let squash =
        squash_plan_from_prompt(&prompt).expect("role prompt should include a destination");

    assert_eq!(
        squash.sources(),
        &[
            "bbbbbbbb1111111111111111111111111111111111",
            "aaaaaaaa3333333333333333333333333333333333"
        ]
    );
    assert_eq!(
        squash.destination(),
        "cccccccc2222222222222222222222222222222222"
    );
}

#[test]
fn new_action_menu_enter_opens_preview_with_exact_parents() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("current")
                .with_sources(["parent-a", "parent-b"]),
        ),
        selected: 0,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (parents, command_label, body) = match &app.mode {
        InteractionMode::NewPreview { new_change, output } => (
            new_change.parents().to_vec(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected new preview mode"),
    };
    assert_eq!(parents, ["parent-a", "parent-b"]);
    assert_eq!(command_label, "jj new parent-a parent-b");
    assert!(body.contains("parent: parent-a"));
    assert!(body.contains("parent: parent-b"));
    assert!(body.contains("undo path: jj undo"));
}

#[test]
fn action_menu_shortcut_opens_item_without_moving_selection() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("current")
                .with_sources(["parent-a", "parent-b"]),
        ),
        selected: 3,
    };

    app.handle_mode_key(KeyCode::Char('n'), 12).unwrap();

    let parents = match &app.mode {
        InteractionMode::NewPreview { new_change, .. } => new_change.parents().to_vec(),
        _ => panic!("expected new preview mode"),
    };
    assert_eq!(parents, ["parent-a", "parent-b"]);
}

#[test]
fn action_menu_close_key_preserves_normal_context() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        ),
        selected: 4,
    };

    app.handle_mode_key(KeyCode::Char('q'), 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.graph_selected_revision().as_deref(), Some("change-a"));
}

#[test]
fn edit_action_menu_enter_opens_preview_with_exact_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        ),
        selected: 0,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (navigation, command_label, body) = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { navigation, output } => (
            navigation.clone(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected working-copy navigation preview"),
    };
    assert_eq!(navigation.kind(), JjWorkingCopyNavigationKind::Edit);
    assert_eq!(navigation.target_change_id(), Some("change-a"));
    assert_eq!(command_label, "jj edit exactly(change_id(\"change-a\"), 1)");
    assert!(body.contains("target: exact selected graph revision change-a"));
    assert!(body.contains("moves @ to edit that revision directly"));
}

#[test]
fn edit_direct_key_requires_exact_selected_graph_row() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char('e'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "edit from graph requires a selected row with an exact revision"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn next_direct_key_opens_preview_without_selected_row_targeting() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));

    app.handle_normal_key(key(KeyCode::Char(']'), KeyModifiers::NONE), 12)
        .unwrap();

    let (navigation, command_label, body) = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { navigation, output } => (
            navigation.clone(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected working-copy navigation preview"),
    };
    assert_eq!(navigation.kind(), JjWorkingCopyNavigationKind::Next);
    assert_eq!(command_label, "jj next --edit");
    assert!(body.contains("highlighted graph row is not an argument to jj next --edit"));
    assert!(body.contains("runs jj topology movement relative to @"));
}

#[test]
fn working_copy_navigation_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::edit("change-a"),
        output: ActionOutput::pending(
            "jj edit exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("edit preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "edit cancelled");
}

#[test]
fn edit_confirm_success_refreshes_and_reveals_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.reveal_graph_change = mock_reveal_edit_target_in_recent;
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::edit("change-a"),
        output: ActionOutput::pending(
            "jj edit exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("edit preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { output, .. } => output,
        _ => panic!("expected working-copy navigation result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("editing change-a | jj undo"));
    assert_eq!(app.status.message(), "editing change-a | jj undo");
}

#[test]
fn split_action_menu_enter_opens_exact_target_preview() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(
            vec![ratatui::text::Line::from("○  change")],
            Some("change-a".to_owned()),
            None,
        ),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        ),
        selected: 2,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (target, command_label, body) = match &app.mode {
        InteractionMode::SplitPreview { split, output } => (
            split.target().exact_change_id().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected split preview"),
    };
    assert_eq!(target.as_deref(), Some("change-a"));
    assert_eq!(
        command_label,
        "jj split --revision exactly(change_id(\"change-a\"), 1)"
    );
    assert!(body.contains("target: exact selected graph revision change-a"));
    assert!(body.contains("jj's diff editor"));
    assert!(body.contains("jk is not an in-app patch editor"));
}

#[test]
fn split_visible_working_copy_uses_bare_command() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("current-change".to_owned()),
            None,
        ),
    ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Char('s'), 12).unwrap();

    let (target, command_label, body) = match &app.mode {
        InteractionMode::SplitPreview { split, output } => (
            split.target().exact_change_id().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected split preview"),
    };
    assert_eq!(target, None);
    assert_eq!(command_label, "jj split");
    assert!(body.contains("target: current working-copy change (@)"));
    assert!(body.contains("fileset: no fileset is passed"));
}

#[test]
fn split_preview_cancel_preserves_graph_selection() {
    let mut graph = crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::Graph(graph));
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::exact_change("second"),
        output: ActionOutput::pending(
            "jj split --revision exactly(change_id(\"second\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("split preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    let ViewState::Graph(graph) = &app.view else {
        panic!("expected graph view");
    };
    assert_eq!(graph.selected_revision(), Some("second"));
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "split cancelled");
}

#[test]
fn split_confirm_success_refreshes_reveals_and_keeps_recovery_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.reveal_graph_change = mock_reveal_edit_target_in_recent;
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::exact_change("change-a"),
        output: ActionOutput::pending(
            "jj split --revision exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("split exact graph revision change-a from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::SplitPreview { output, .. } => output,
        _ => panic!("expected split result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("child exit status: exit status: 0"));
    assert!(body.contains("did not capture that output"));
    assert!(body.contains("refresh: split completed | jj undo | jj op show -p"));
    assert_eq!(
        app.status.message(),
        "split completed | jj undo | jj op show -p"
    );
}

#[test]
fn split_current_confirm_success_reveals_current_working_copy_when_possible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("current-change".to_owned()), None),
    ])));
    app.services.reveal_graph_change = mock_reveal_current_working_copy_in_recent;
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::current_working_copy(),
        output: ActionOutput::pending(
            "jj split".to_owned(),
            "preview only".to_owned(),
            Some("split current working-copy change (@) from jk".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::SplitPreview { output, .. } => output,
        _ => panic!("expected split result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(
        body.contains("refresh: split completed | showing recent work | jj undo | jj op show -p")
    );
    assert_eq!(
        app.status.message(),
        "split completed | showing recent work | jj undo | jj op show -p"
    );
}

#[test]
fn split_failure_keeps_app_owned_result_without_claiming_captured_stderr() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("current-change".to_owned()), None),
    ])));
    app.services.split_run = mock_split_failure_service;
    app.mode = InteractionMode::SplitPreview {
        split: JjSplitPlan::current_working_copy(),
        output: ActionOutput::pending("jj split".to_owned(), "preview only".to_owned(), None),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::SplitPreview { output, .. } => output,
        _ => panic!("expected split result"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj split");
    assert!(output.completed());
    assert!(body.contains("result: split command failed or did not complete"));
    assert!(body.contains("runner status: jj split failed with status exit status: 1"));
    assert!(body.contains("did not capture stderr"));
    assert!(body.contains("if jj recorded an operation, use jj undo"));
    assert!(body.contains("review: jj op show -p"));
    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert!(
        app.status
            .message()
            .contains("runner status: jj split failed")
    );
}

#[test]
fn prev_confirm_success_resolves_current_working_copy_and_reveals_recent() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));
    app.services.reveal_graph_change = mock_reveal_current_working_copy_in_recent;
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::prev(),
        output: ActionOutput::pending(
            "jj prev --edit".to_owned(),
            "preview only".to_owned(),
            Some("prev preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { output, .. } => output,
        _ => panic!("expected working-copy navigation result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("moved to previous editable change | showing recent work | jj undo"));
    assert_eq!(
        app.status.message(),
        "moved to previous editable change | showing recent work | jj undo"
    );
}

#[test]
fn working_copy_navigation_failure_keeps_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), None, None),
    ])));
    app.services.working_copy_navigation_run = mock_working_copy_navigation_failure;
    app.mode = InteractionMode::WorkingCopyNavigationPreview {
        navigation: JjWorkingCopyNavigationPlan::next(),
        output: ActionOutput::pending("jj next --edit".to_owned(), "preview only".to_owned(), None),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::WorkingCopyNavigationPreview { output, .. } => output,
        _ => panic!("expected working-copy navigation result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj next --edit");
    assert!(output.completed());
    assert!(body.contains("jj next --edit failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj next --edit failed: first line\nsecond line"
    );
}

#[test]
fn new_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::NewPreview {
        new_change: JjNewPlan::new(vec!["parent-a".to_owned()]),
        output: ActionOutput::pending(
            "jj new parent-a".to_owned(),
            "preview only".to_owned(),
            Some("new preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "new change cancelled");
}

#[test]
fn new_confirm_success_refreshes_and_reveals_working_copy() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.services.reveal_graph_change = mock_reveal_new_change_in_recent;
    app.mode = InteractionMode::NewPreview {
        new_change: JjNewPlan::new(vec!["parent-a".to_owned(), "parent-b".to_owned()]),
        output: ActionOutput::pending(
            "jj new parent-a parent-b".to_owned(),
            "preview only".to_owned(),
            Some("new preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::NewPreview { output, .. } => output,
        _ => panic!("expected new result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj new parent-a parent-b");
    assert!(output.completed());
    assert!(body.contains("new parents: parent-a,parent-b | showing recent work | jj undo"));
    assert_eq!(
        app.status.message(),
        "new parents: parent-a,parent-b | showing recent work | jj undo"
    );
}

#[test]
fn graph_new_trunk_uses_test_service_and_reveals_working_copy() {
    NEW_TRUNK_CALLS.store(0, Ordering::SeqCst);
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![])));
    app.services.resolve_revision = mock_resolve_trunk_and_current_change_id;
    app.services.reveal_graph_change = mock_reveal_new_change_in_recent;

    app.handle_normal_key(key(KeyCode::Char('c'), KeyModifiers::NONE), 12)
        .unwrap();

    assert_eq!(NEW_TRUNK_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(
        app.status.message(),
        "created new change from trunk | showing recent work | jj undo"
    );
}

#[test]
fn new_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("parent-a".to_owned()), None),
    ])));
    app.services.new_run = mock_new_failure;
    app.mode = InteractionMode::NewPreview {
        new_change: JjNewPlan::new(vec!["parent-a".to_owned()]),
        output: ActionOutput::pending(
            "jj new parent-a".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::NewPreview { output, .. } => output,
        _ => panic!("expected new result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj new parent-a");
    assert!(output.completed());
    assert!(body.contains("jj new failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj new failed: first line\nsecond line"
    );
}

#[test]
fn detail_action_menu_from_exact_show_offers_restore_and_revert() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let actions = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu
            .items()
            .iter()
            .map(|item| item.action())
            .collect::<Vec<_>>(),
        _ => panic!("expected detail action menu"),
    };
    assert_eq!(actions, vec![ActionKind::Restore, ActionKind::Revert]);
}

#[test]
fn detail_action_menu_from_exact_file_list_offers_path_restore_first() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let menu = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu,
        _ => panic!("expected detail action menu"),
    };
    let actions = menu
        .items()
        .iter()
        .map(|item| item.action())
        .collect::<Vec<_>>();
    assert_eq!(
        actions,
        vec![ActionKind::Restore, ActionKind::Restore, ActionKind::Revert]
    );
    assert!(matches!(
        menu.items()[0].follow_up(),
        FollowUp::RestoreExactTarget { revision, path }
            if revision == "change-a" && path.as_deref() == Some("src/main.rs")
    ));
}

#[test]
fn open_action_menu_rejects_direct_show_startup_revset() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]),
    )));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "restore/revert from jk show main requires an exact graph-derived revision target"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn open_action_menu_rejects_bookmark_derived_show() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default).without_exact_change_target(),
    )));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "restore/revert from jk show change-a requires an exact graph-derived revision target"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn detail_navigation_marks_graph_targets_exact() {
    let app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(
        Vec::new(),
    )));

    let show = app
        .detail_spec(JjCommand::Show, "change-a".to_owned())
        .unwrap();
    let diff = app
        .detail_spec(JjCommand::Diff, "change-a".to_owned())
        .unwrap();

    assert_eq!(show.exact_change_target(), Some("change-a"));
    assert_eq!(diff.exact_change_target(), Some("change-a"));
}

#[test]
fn detail_navigation_from_bookmarks_is_not_exact() {
    let app = test_app(ViewState::Bookmarks(
        crate::bookmarks::BookmarksView::test_new(vec![crate::jj::BookmarkItem::new(
            Vec::new(),
            "feature".to_owned(),
            Some("change-a".to_owned()),
            None,
        )]),
    ));

    let show = app
        .detail_spec(JjCommand::Show, "change-a".to_owned())
        .unwrap();
    let diff = app
        .detail_spec(JjCommand::Diff, "change-a".to_owned())
        .unwrap();

    assert_eq!(show.exact_change_target(), None);
    assert_eq!(diff.exact_change_target(), None);
}

#[test]
fn detail_navigation_preserves_inexact_direct_startup_revsets() {
    let app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]),
    )));

    let diff = app.detail_spec(JjCommand::Diff, "main".to_owned()).unwrap();

    assert_eq!(diff.navigation_revset().as_deref(), Some("main"));
    assert_eq!(diff.exact_change_target(), None);
}

#[test]
fn file_show_navigation_preserves_source_exactness_only() {
    let exact_app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));
    let direct_app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned())),
            vec![crate::jj::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));

    let exact = exact_app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();
    let direct = direct_app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();

    assert_eq!(exact.exact_change_target(), Some("change-a"));
    assert_eq!(direct.navigation_revset().as_deref(), Some("main"));
    assert_eq!(direct.exact_change_target(), None);
}

#[test]
fn file_show_navigation_from_resolve_uses_resolve_revision() {
    let app = test_app(ViewState::Resolve(
        crate::resolve::ResolveView::test_with_spec(
            ViewSpec::resolve(Some("main".to_owned())),
            vec![crate::jj::ResolveEntry::parsed(
                Some("src/main.rs".to_owned()),
                Some("file".to_owned()),
                Some(3),
            )],
        ),
    ));

    let file_show = app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();

    assert_eq!(file_show.command(), JjCommand::FileShow);
    assert_eq!(file_show.args(), ["-r", "main", "src/main.rs"]);
    assert_eq!(file_show.navigation_revset().as_deref(), Some("main"));
    assert_eq!(file_show.exact_change_target(), None);
}

#[test]
fn file_show_navigation_from_default_resolve_uses_current_revision() {
    let app = test_app(ViewState::Resolve(
        crate::resolve::ResolveView::test_with_spec(
            ViewSpec::resolve(None),
            vec![crate::jj::ResolveEntry::parsed(
                Some("src/main.rs".to_owned()),
                Some("file".to_owned()),
                Some(3),
            )],
        ),
    ));

    let file_show = app
        .detail_spec(JjCommand::FileShow, "src/main.rs".to_owned())
        .unwrap();

    assert_eq!(file_show.command(), JjCommand::FileShow);
    assert_eq!(file_show.args(), ["-r", "@", "src/main.rs"]);
    assert_eq!(file_show.navigation_revset().as_deref(), Some("@"));
    assert_eq!(file_show.exact_change_target(), None);
}

#[test]
fn open_action_menu_rejects_status_without_exact_path() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[])));

    app.open_action_menu(12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "status file action unavailable: status output is empty"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn status_action_menu_opens_working_copy_path_restore_preview() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "Working copy changes:",
        "M src/status.rs",
    ])));

    app.handle_normal_key(key(KeyCode::Char('j'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    let menu = match &app.mode {
        InteractionMode::ActionMenu { menu, .. } => menu,
        _ => panic!("expected status action menu"),
    };
    assert_eq!(menu.items().len(), 1);
    assert!(matches!(
        menu.items()[0].follow_up(),
        FollowUp::RestoreWorkingCopyPath { path } if path == "src/status.rs"
    ));

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (path, command_label, body) = match &app.mode {
        InteractionMode::RestorePreview { restore, output } => (
            restore.path().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected restore preview mode"),
    };
    assert_eq!(path.as_deref(), Some("src/status.rs"));
    assert_eq!(command_label, "jj restore root-file:\"src/status.rs\"");
    assert!(body.contains("target revision: @"));
    assert!(body.contains("selected path: src/status.rs"));
}

#[test]
fn status_action_menu_reports_disabled_ambiguous_row() {
    let mut app = test_app(ViewState::Status(crate::status::StatusView::test_new(&[
        "R {old.rs => new.rs}",
    ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "status file action unavailable: renamed status rows contain multiple paths"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn restore_action_menu_enter_opens_path_preview() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::detail("change-a")
                .with_selected_path("src/main.rs"),
        ),
        selected: 0,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (path, command_label, body) = match &app.mode {
        InteractionMode::RestorePreview { restore, output } => (
            restore.path().map(str::to_owned),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected restore preview mode"),
    };
    assert_eq!(path.as_deref(), Some("src/main.rs"));
    assert_eq!(
        command_label,
        "jj restore --changes-in exactly(change_id(\"change-a\"), 1) root-file:\"src/main.rs\""
    );
    assert!(body.contains("target revision: change-a"));
    assert!(body.contains("selected path: src/main.rs"));
    assert!(body.contains("undo path: jj undo"));
}

#[test]
fn restore_action_menu_path_shortcut_opens_path_preview() {
    let mut app = test_app(ViewState::FileList(
        crate::file_list::FileListView::test_with_spec(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("src/main.rs".to_owned()))
                .with_exact_change_target(),
            vec![crate::jj::FileListItem::new(
                Vec::new(),
                "src/main.rs".to_owned(),
            )],
        ),
    ));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::detail("change-a")
                .with_selected_path("src/main.rs"),
        ),
        selected: 1,
    };

    app.handle_mode_key(KeyCode::Char('p'), 12).unwrap();

    let path = match &app.mode {
        InteractionMode::RestorePreview { restore, .. } => restore.path().map(str::to_owned),
        _ => panic!("expected restore preview mode"),
    };
    assert_eq!(path.as_deref(), Some("src/main.rs"));
}

#[test]
fn restore_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::RestorePreview {
        restore: JjRestorePlan::for_revision("change-a"),
        output: ActionOutput::pending(
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("restore preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "restore cancelled");
}

#[test]
fn restore_confirm_success_and_failure_keep_output_readable() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::RestorePreview {
        restore: JjRestorePlan::for_path("change-a", "src/main.rs"),
        output: ActionOutput::pending(
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1) root-file:\"src/main.rs\""
                .to_owned(),
            "preview only".to_owned(),
            Some("restore preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RestorePreview { output, .. } => output,
        _ => panic!("expected restore result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("restored src/main.rs from change-a | jj undo"));
    assert_eq!(
        app.status.message(),
        "restored src/main.rs from change-a | jj undo"
    );

    app.services.restore_run = mock_restore_failure;
    app.mode = InteractionMode::RestorePreview {
        restore: JjRestorePlan::for_revision("change-a"),
        output: ActionOutput::pending(
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RestorePreview { output, .. } => output,
        _ => panic!("expected restore result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj restore failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj restore failed: first line\nsecond line"
    );
}

#[test]
fn revert_action_menu_enter_opens_preview_and_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::detail("change-a"),
        ),
        selected: 1,
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let (command_label, body) = match &app.mode {
        InteractionMode::RevertPreview { output, .. } => (
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected revert preview mode"),
    };
    assert_eq!(
        command_label,
        "jj revert -r exactly(change_id(\"change-a\"), 1) -o @"
    );
    assert!(body.contains("target revision: change-a"));

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "revert cancelled");
}

#[test]
fn revert_confirm_success_and_failure_keep_output_readable() {
    let mut app = test_app(ViewState::Show(crate::show::ShowView::test_new(
        ViewSpec::show("change-a".to_owned(), DiffFormat::Default),
    )));
    app.mode = InteractionMode::RevertPreview {
        revert: JjRevertPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj revert -r exactly(change_id(\"change-a\"), 1) -o @".to_owned(),
            "preview only".to_owned(),
            Some("revert preview context".to_owned()),
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RevertPreview { output, .. } => output,
        _ => panic!("expected revert result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("reverted change-a | jj undo"));
    assert_eq!(app.status.message(), "reverted change-a | jj undo");

    app.services.revert_run = mock_revert_failure;
    app.mode = InteractionMode::RevertPreview {
        revert: JjRevertPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj revert -r exactly(change_id(\"change-a\"), 1) -o @".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::RevertPreview { output, .. } => output,
        _ => panic!("expected revert result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj revert failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj revert failed: first line\nsecond line"
    );
}

#[test]
fn rebase_preview_entering_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            Some("rebase preview context".to_owned()),
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "rebase cancelled");
}

#[test]
fn rebase_preview_completion_stays_until_closed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::finished(
            "jj rebase -r source-a -o dest".to_owned(),
            "rebased".to_owned(),
            None,
        ),
    };
    app.status = StatusLine::with_message(&app.view, "rebased");

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "rebased");
}

#[test]
fn rebase_confirm_success_with_reveal_failure_stays_completed() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.rebase_run = mock_rebase_success;
    app.services.refresh_view = mock_refresh_ok;
    app.services.reveal_graph_change = mock_reveal_graph_change_error;
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            Some("rebase preview context".to_owned()),
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );

    let output = match app.mode {
        InteractionMode::RebasePreview { ref output, .. } => output,
        _ => panic!("expected rebase preview mode"),
    };
    assert_eq!(output.command_label(), "jj rebase -r source-a -o dest");
    assert_eq!(
        output.status_context().map(String::as_str),
        Some("rebase preview context")
    );
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("reveal failed: refreshed graph did not include the new working-copy change")
    );
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("jj undo | jj op show -p")
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
    assert_eq!(
        app.status.message(),
        "refreshed graph did not include the new working-copy change"
    );

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "refreshed graph did not include the new working-copy change"
    );
}

#[test]
fn rebase_confirm_success_keeps_review_and_undo_paths_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("source-a".to_owned()), None),
    ])));
    app.services.rebase_run = mock_rebase_success;
    app.services.refresh_view = mock_refresh_ok;
    app.services.reveal_graph_change = mock_reveal_rebased_source_in_recent;
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            Some("rebase preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::RebasePreview { output, .. } => output,
        _ => panic!("expected rebase result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("rebased | showing recent work | jj undo | jj op show -p"));
    assert_eq!(
        app.status.message(),
        "rebased | showing recent work | jj undo | jj op show -p"
    );
}

#[test]
fn rebase_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("source-a".to_owned()), None),
    ])));
    app.services.rebase_run = mock_rebase_failure;
    app.mode = InteractionMode::RebasePreview {
        rebase: JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj rebase -r source-a -o dest".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::RebasePreview { output, .. } => output,
        _ => panic!("expected rebase result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj rebase -r source-a -o dest");
    assert!(output.completed());
    assert!(body.contains("jj rebase failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj rebase failed: first line\nsecond line"
    );
}

#[test]
fn rebase_role_prompt_enters_preview_with_explicit_plan() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "source-a".to_owned()),
            RolePromptOption::new("destination", "dest".to_owned()),
        ],
        "Preview required before execution.",
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RolePrompt {
        action: ActionKind::Rebase,
        prompt,
        selected: 0,
    };

    let result = app.handle_mode_key(crossterm::event::KeyCode::Enter, 12);
    assert!(result.is_ok());
    let (command_label, status_context, preview_output) = match app.mode {
        InteractionMode::RebasePreview { ref output, .. } => (
            output.command_label().to_owned(),
            output.status_context().cloned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected rebase preview mode"),
    };
    assert_eq!(command_label, "jj rebase -r source-a -o dest");
    assert_eq!(
        status_context.as_deref(),
        Some("rebase from 1 source(s) into dest from jk | source(s): source-a")
    );
    assert!(preview_output.contains("command: jj rebase -r source-a -o dest"));
    assert!(preview_output.contains("source revision: source-a"));
    assert!(preview_output.contains("destination revision: dest"));
    assert!(preview_output.contains("current graph context:"));
    assert!(preview_output.contains("source rows are selected in jk"));
    assert!(preview_output.contains("destination is the current row"));
    assert!(preview_output.contains("expected jj effect:"));
    assert!(
        preview_output.contains("semantics: jj rebase --revision <source> --onto <destination>")
    );
    assert!(preview_output.contains("not a graph preview"));
    assert!(preview_output.contains("review after run: jj op show -p"));
    assert!(preview_output.contains("undo path: jj undo"));
    assert!(preview_output.contains("confirmation: press Enter to run jj rebase"));
}

#[test]
fn squash_role_prompt_enters_preview_with_explicit_plan() {
    let prompt = RolePrompt::new(
        "confirm role assignment",
        vec![
            RolePromptOption::new("source", "source-a".to_owned()),
            RolePromptOption::new("source", "source-b".to_owned()),
            RolePromptOption::new("destination", "dest".to_owned()),
        ],
        "Preview required before execution.",
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::RolePrompt {
        action: ActionKind::Squash,
        prompt,
        selected: 0,
    };

    let result = app.handle_mode_key(crossterm::event::KeyCode::Enter, 12);
    assert!(result.is_ok());
    let (command_label, status_context, preview_output) = match app.mode {
        InteractionMode::SquashPreview { ref output, .. } => (
            output.command_label().to_owned(),
            output.status_context().cloned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected squash preview mode"),
    };
    assert_eq!(
        command_label,
        "jj squash --from source-a --from source-b --into dest --use-destination-message"
    );
    assert_eq!(
        status_context.as_deref(),
        Some("squash from 2 source(s) into dest from jk | source(s): source-a, source-b")
    );
    assert!(preview_output.contains("source: source-a"));
    assert!(preview_output.contains("source: source-b"));
    assert!(preview_output.contains("destination: dest"));
    assert!(preview_output.contains("--use-destination-message keeps the destination description"));
    assert!(preview_output.contains("confirmation: press Enter to run jj squash"));
    assert!(preview_output.contains("undo path: jj undo"));
}

#[test]
fn squash_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            Some("squash preview context".to_owned()),
        ),
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "squash cancelled");
}

#[test]
fn squash_confirm_success_refreshes_and_reveals_destination() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.reveal_graph_change = mock_reveal_squash_destination_in_recent;
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            Some("squash preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::SquashPreview { output, .. } => output,
        _ => panic!("expected squash result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        "jj squash --from source-a --into dest --use-destination-message"
    );
    assert!(output.completed());
    assert!(body.contains("squashed | showing recent work | jj undo"));
    assert_eq!(
        app.status.message(),
        "squashed | showing recent work | jj undo"
    );
}

#[test]
fn squash_confirm_refresh_failure_keeps_undo_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.refresh_view = mock_refresh_failure;
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            Some("squash preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::SquashPreview { output, .. } => output,
        _ => panic!("expected squash result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("squashed | refresh failed: view refresh failed | jj undo"));
    assert_eq!(app.status.message(), "view refresh failed");
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn squash_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.services.squash_run = mock_squash_failure;
    app.mode = InteractionMode::SquashPreview {
        squash: JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned()),
        output: ActionOutput::pending(
            "jj squash --from source-a --into dest --use-destination-message".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::SquashPreview { output, .. } => output,
        _ => panic!("expected squash result mode"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        "jj squash --from source-a --into dest --use-destination-message"
    );
    assert!(output.completed());
    assert!(body.contains("jj squash failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj squash failed: first line\nsecond line"
    );
}

#[test]
fn absorb_action_menu_enter_opens_preview_with_current_source_and_candidates() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("source".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("dest-a".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("dest-b".to_owned()), None),
    ])));
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("source")
                .with_sources(["source", "dest-a", "dest-b"]),
        ),
        selected: 3,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (source, destinations, command_label, body) = match &app.mode {
        InteractionMode::AbsorbPreview { absorb, output } => (
            absorb.source().to_owned(),
            absorb.destinations().to_vec(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected absorb preview mode"),
    };
    assert_eq!(source, "source");
    assert_eq!(destinations, ["dest-a", "dest-b"]);
    assert_eq!(
        command_label,
        "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1) --into exactly(change_id(\"dest-b\"), 1)"
    );
    assert!(body.contains("source: source"));
    assert!(body.contains("candidate destination: dest-a"));
    assert!(body.contains("candidate destination: dest-b"));
    assert!(body.contains("only considers selected revisions that are ancestors"));
    assert!(body.contains("jj op show -p"));
}

#[test]
fn absorb_preview_cancel_restores_normal_mode() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));
    app.mode = InteractionMode::AbsorbPreview {
        absorb: JjAbsorbPlan::new("source", vec!["dest-a".to_owned()]),
        output: ActionOutput::pending(
            "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("absorb preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "absorb cancelled");
}

#[test]
fn absorb_confirm_success_keeps_undo_and_operation_review_visible() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));
    app.mode = InteractionMode::AbsorbPreview {
        absorb: JjAbsorbPlan::new("source", vec!["dest-a".to_owned()]),
        output: ActionOutput::pending(
            "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            Some("absorb preview context".to_owned()),
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbsorbPreview { output, .. } => output,
        _ => panic!("expected absorb result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("absorbed | jj undo | jj op show -p"));
    assert_eq!(app.status.message(), "absorbed | jj undo | jj op show -p");
}

#[test]
fn absorb_failure_keeps_full_error_output_readable() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));
    app.services.absorb_run = mock_absorb_failure;
    app.mode = InteractionMode::AbsorbPreview {
        absorb: JjAbsorbPlan::new("source", vec!["dest-a".to_owned()]),
        output: ActionOutput::pending(
            "jj absorb --from exactly(change_id(\"source\"), 1) --into exactly(change_id(\"dest-a\"), 1)".to_owned(),
            "preview only".to_owned(),
            None,
        ),
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbsorbPreview { output, .. } => output,
        _ => panic!("expected absorb result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj absorb failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj absorb failed: first line\nsecond line"
    );
}

#[test]
fn absorb_without_candidates_returns_clear_status() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("source".to_owned()), None),
    ])));

    app.open_absorb_preview(JjAbsorbPlan::new("source", Vec::new()));

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "absorb requires at least one selected exact candidate destination"
    );
    assert!(matches!(app.status.kind(), StatusKind::Error));
}

#[test]
fn abandon_action_menu_enter_opens_preview_with_exact_target() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_preview_load = mock_non_empty_abandon_preview;
    app.mode = InteractionMode::ActionMenu {
        menu: crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        ),
        selected: 3,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (revision, command_label, body) = match &app.mode {
        InteractionMode::AbandonPreview {
            abandon, output, ..
        } => (
            abandon.revision().to_owned(),
            output.command_label().to_owned(),
            output.body_lines().join("\n"),
        ),
        _ => panic!("expected abandon preview mode"),
    };
    assert_eq!(revision, "change-a");
    assert_eq!(command_label, "jj abandon change-a");
    assert!(body.contains("change: change-a"));
    assert!(body.contains("title: Edit change"));
}

#[test]
fn empty_abandon_preview_enter_runs_and_keeps_undo_visible() {
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            Some("abandon exact revision change-a from jk".to_owned()),
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("abandoned | jj undo")
    );
    assert_eq!(app.status.message(), "abandoned | jj undo");
}

#[test]
fn empty_abandon_rechecks_before_running_and_requires_confirmation_after_drift() {
    ABANDON_DRIFT_RECHECK_CALLS.store(1, Ordering::SeqCst);
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_preview_load = mock_abandon_preview_drifts_to_non_empty;
    app.services.abandon_run = panic_abandon_run;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            Some("abandon exact revision change-a from jk".to_owned()),
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let (input, body) = match &app.mode {
        InteractionMode::AbandonConfirm { input, output, .. } => {
            (input.as_str(), output.body_lines().join("\n"))
        }
        _ => panic!("expected abandon confirmation after recheck drift"),
    };
    assert_eq!(input, "");
    assert!(body.contains("change is no longer empty"));
    assert!(body.contains("M src/main.rs"));
    assert_eq!(
        app.status.message(),
        "change is no longer empty; type exact revision to confirm abandon"
    );

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert_eq!(
        app.status.message(),
        "confirmation did not match; abandon not run"
    );

    app.services.abandon_run = mock_abandon_success;
    for character in "change-a".chars() {
        app.handle_mode_key(crossterm::event::KeyCode::Char(character), 12)
            .unwrap();
    }
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("abandoned | jj undo")
    );
}

#[test]
fn empty_abandon_recheck_failure_stays_readable_without_running() {
    ABANDON_FAILED_RECHECK_CALLS.store(1, Ordering::SeqCst);
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Empty change".to_owned()),
        String::new(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_preview_load = mock_abandon_preview_recheck_failure;
    app.services.abandon_run = panic_abandon_run;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            None,
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected readable abandon recheck failure"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj diff -r change-a --summary failed: disappeared"));
    assert_eq!(
        app.status.message(),
        "jj diff -r change-a --summary failed: disappeared"
    );
}

#[test]
fn non_empty_abandon_requires_exact_typed_revision() {
    let preview = JjAbandonPreview::new(
        "change-a".to_owned(),
        Some("Edit change".to_owned()),
        "M src/main.rs\n".to_owned(),
    );
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_run = panic_abandon_run;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            Some("abandon exact revision change-a from jk".to_owned()),
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert!(matches!(app.mode, InteractionMode::AbandonConfirm { .. }));

    app.handle_mode_key(crossterm::event::KeyCode::Char('x'), 12)
        .unwrap();
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();
    assert_eq!(
        app.status.message(),
        "confirmation did not match; abandon not run"
    );

    app.services.abandon_run = mock_abandon_success;
    app.handle_mode_key(crossterm::event::KeyCode::Backspace, 12)
        .unwrap();
    for character in "change-a".chars() {
        app.handle_mode_key(crossterm::event::KeyCode::Char(character), 12)
            .unwrap();
    }
    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("abandoned | jj undo")
    );
}

#[test]
fn abandon_cancel_restores_normal_mode_and_selection() {
    let mut graph = crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("first".to_owned()), None),
        crate::jj::LogItem::new(Vec::new(), Some("second".to_owned()), None),
    ]);
    graph.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let preview = JjAbandonPreview::new("second".to_owned(), None, String::new());
    let mut app = test_app(ViewState::Graph(graph));
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("second"),
        output: ActionOutput::pending("jj abandon second".to_owned(), preview.preview_text(), None),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Esc, 12)
        .unwrap();

    let ViewState::Graph(graph) = &app.view else {
        panic!("expected graph view");
    };
    assert_eq!(graph.selected_revision(), Some("second"));
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "abandon cancelled");
}

#[test]
fn abandon_failure_keeps_full_error_output_readable() {
    let preview = JjAbandonPreview::new("change-a".to_owned(), None, String::new());
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("change-a".to_owned()), None),
    ])));
    app.services.abandon_run = mock_abandon_failure;
    app.mode = InteractionMode::AbandonPreview {
        abandon: JjAbandonPlan::new("change-a"),
        output: ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            preview.preview_text(),
            None,
        ),
        preview,
    };

    app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::AbandonPreview { output, .. } => output,
        _ => panic!("expected abandon result mode"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains("jj abandon change-a failed: first line"));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        "jj abandon change-a failed: first line\nsecond line"
    );
}

#[test]
fn push_remote_prompt_without_selection_stays_ready() {
    let mut app = test_app(ViewState::Graph(crate::graph::GraphView::test_new(vec![
        crate::jj::LogItem::new(Vec::new(), Some("abcdef".to_owned()), None),
    ])));
    app.mode = InteractionMode::PushRemotePrompt {
        target: JjGitPushTarget::Revision("abcdef".to_owned()),
        remotes: Vec::new(),
        selected: 0,
    };

    assert!(
        app.handle_mode_key(crossterm::event::KeyCode::Enter, 12)
            .is_ok()
    );
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "no remote selected for push");
}

#[test]
fn operation_log_undo_key_opens_global_preview_without_selected_operation_id() {
    let selected_operation_id = "b".repeat(128);
    let mut operation_log = crate::operation_log::OperationLogView::test_new(vec![
        crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("a".repeat(128)),
        ),
        crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("○  selected")],
            Some(selected_operation_id.clone()),
        ),
    ]);
    operation_log.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 12,
            viewport_width: 80,
            search: None,
        },
    );
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();

    let output = match &app.mode {
        InteractionMode::OperationRecoveryPreview { recovery, output } => {
            assert_eq!(recovery.kind(), JjOperationRecoveryKind::Undo);
            output
        }
        _ => panic!("expected operation recovery preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj undo");
    assert!(body.contains("global current-repo undo from jk operation log"));
    assert!(body.contains("selected operation-log row is not an argument"));
    assert!(!body.contains(&selected_operation_id));
}

#[test]
fn operation_recovery_preview_can_cancel_or_confirm_success() {
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("a".repeat(128)),
        )]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "undo cancelled");

    app.handle_normal_key(key(KeyCode::Char('u'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationRecoveryPreview { output, .. } => output,
        _ => panic!("expected operation recovery result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("undone operation | jj redo")
    );
    assert_eq!(app.status.message(), "undone operation | jj redo");
}

#[test]
fn operation_redo_failure_keeps_command_output_readable() {
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some("a".repeat(128)),
        )]);
    let mut app = test_app(ViewState::OperationLog(operation_log));
    app.services.operation_recovery_run = mock_operation_recovery_failure;

    app.handle_normal_key(key(KeyCode::Char('r'), KeyModifiers::CONTROL), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationRecoveryPreview { recovery, output } => {
            assert_eq!(recovery.kind(), JjOperationRecoveryKind::Redo);
            output
        }
        _ => panic!("expected operation recovery result"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(output.command_label(), "jj redo");
    assert!(output.completed());
    assert!(body.contains("jj redo failed: no operation to redo available"));
    assert!(body.contains("hint: run the opposite recovery command first"));
    assert_eq!(
        app.status.message(),
        "jj redo failed: no operation to redo available\nhint: run the opposite recovery command first"
    );
}

#[test]
fn operation_action_menu_requires_exact_operation_id() {
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            None,
        )]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();

    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(
        app.status.message(),
        "operation recovery actions unavailable: selected row has no operation id"
    );
}

#[test]
fn operation_restore_preview_can_cancel_or_confirm_success() {
    let operation_id = "e".repeat(128);
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        )]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { target, output } => {
            assert_eq!(target.kind(), crate::jj::JjOperationTargetKind::Restore);
            assert_eq!(target.operation_id(), operation_id.as_str());
            output
        }
        _ => panic!("expected operation target preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        format!("jj operation restore {operation_id}")
    );
    assert!(body.contains(&format!("operation id: {operation_id}")));
    assert!(body.contains(&format!("command: jj operation restore {operation_id}")));
    assert!(body.contains(&format!(
        "confirmation: press Enter to run jj operation restore {operation_id}"
    )));

    app.handle_mode_key(KeyCode::Esc, 12).unwrap();
    assert!(matches!(app.mode, InteractionMode::Normal));
    assert_eq!(app.status.message(), "operation restore cancelled");

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation restore result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains(&format!("operation restore {operation_id}")));
    assert!(body.contains("new operation recorded | jj undo"));
    assert_eq!(
        app.status.message(),
        format!("operation restore {operation_id}\nnew operation recorded | jj undo")
    );
}

#[test]
fn operation_restore_confirm_refreshes_non_empty_repo_stack() {
    OPERATION_RESTORE_REFRESH_CALLS.store(0, Ordering::SeqCst);
    let operation_id = "e".repeat(128);
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        )]);
    let mut app = test_app(ViewState::OperationLog(operation_log));
    app.services.refresh_view = mock_operation_restore_counting_refresh_ok;
    app.stack
        .push(ViewState::Status(crate::status::StatusView::test_new(&[
            "Working copy changes:",
        ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    assert_eq!(OPERATION_RESTORE_REFRESH_CALLS.load(Ordering::SeqCst), 2);
    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation restore result"),
    };
    assert!(output.completed());
    assert!(
        output
            .body_lines()
            .join("\n")
            .contains("new operation recorded | jj undo")
    );
    assert_eq!(
        app.status.message(),
        format!("operation restore {operation_id}\nnew operation recorded | jj undo")
    );
}

#[test]
fn operation_revert_confirm_keeps_stacked_refresh_failure_inspectable() {
    OPERATION_REVERT_REFRESH_CALLS.store(0, Ordering::SeqCst);
    let operation_id = "f".repeat(128);
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        )]);
    let mut app = test_app(ViewState::OperationLog(operation_log));
    app.services.refresh_view = mock_operation_revert_second_refresh_failure;
    app.stack
        .push(ViewState::Status(crate::status::StatusView::test_new(&[
            "Working copy changes:",
        ])));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Down, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    assert_eq!(OPERATION_REVERT_REFRESH_CALLS.load(Ordering::SeqCst), 2);
    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation revert result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains(&format!("operation revert {operation_id}")));
    assert!(body.contains("stacked view refresh failed: view refresh failed | jj undo"));
    assert_eq!(app.status.message(), "view refresh failed");
}

#[test]
fn operation_revert_preview_confirm_failure_keeps_output_readable() {
    let operation_id = "f".repeat(128);
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  selected")],
            Some(operation_id.clone()),
        )]);
    let mut app = test_app(ViewState::OperationLog(operation_log));

    app.handle_normal_key(key(KeyCode::Char('a'), KeyModifiers::NONE), 12)
        .unwrap();
    app.handle_mode_key(KeyCode::Down, 12).unwrap();
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { target, output } => {
            assert_eq!(target.kind(), crate::jj::JjOperationTargetKind::Revert);
            assert_eq!(target.operation_id(), operation_id.as_str());
            output
        }
        _ => panic!("expected operation target preview"),
    };
    let body = output.body_lines().join("\n");
    assert_eq!(
        output.command_label(),
        format!("jj operation revert {operation_id}")
    );
    assert!(body.contains(&format!("operation id: {operation_id}")));
    assert!(body.contains("revert exactly the selected operation by applying its inverse"));

    app.services.operation_target_run = mock_operation_target_failure;
    app.handle_mode_key(KeyCode::Enter, 12).unwrap();

    let output = match &app.mode {
        InteractionMode::OperationTargetPreview { output, .. } => output,
        _ => panic!("expected operation revert result"),
    };
    let body = output.body_lines().join("\n");
    assert!(output.completed());
    assert!(body.contains(&format!(
        "jj operation revert {operation_id} failed: first line"
    )));
    assert!(body.contains("second line"));
    assert_eq!(
        app.status.message(),
        format!("jj operation revert {operation_id} failed: first line\nsecond line")
    );
}

#[test]
fn back_from_operation_detail_returns_to_operation_log() {
    let operation_id = "abcdef".to_owned();
    let operation_log =
        crate::operation_log::OperationLogView::test_new(vec![crate::jj::OperationLogItem::new(
            vec![ratatui::text::Line::from("@  current")],
            Some(operation_id.clone()),
        )]);
    let detail = crate::operation_detail::OperationDetailView::test_new(
        ViewSpec::operation_show(operation_id),
        crate::rendered_jj::DocumentLines::new(vec![ratatui::text::Line::from(
            "operation details",
        )]),
    );
    let mut app = test_app(ViewState::OperationDetail(detail));
    app.stack.push(ViewState::OperationLog(operation_log));

    app.pop_view();

    assert!(matches!(app.view, ViewState::OperationLog(_)));
    assert_eq!(app.status.title(), "jk operation log");
    assert_eq!(app.status.message(), "1 operations");
}
