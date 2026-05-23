use ratatui::text::Line;

use super::*;
use crate::command::{Command, CommandContext, ViewCommand, ViewEffect};
use crate::jj::ViewSpec;
use crate::menus::{ActionKind, FollowUp};
use crate::search::SearchQuery;

fn operation_item(text: &[&str], operation_id: Option<&str>) -> OperationLogItem {
    OperationLogItem::new(
        text.iter()
            .map(|line| Line::from((*line).to_owned()))
            .collect::<Vec<_>>(),
        operation_id.map(str::to_owned),
    )
}

fn operation_log_view(entries: Vec<OperationLogItem>) -> OperationLogView {
    OperationLogView::test_new(entries)
}

#[test]
fn copy_options_include_exact_operation_id_when_known() {
    let view = operation_log_view(vec![operation_item(
        &["@  current", "│  describe commit"],
        Some(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ),
    )]);

    let options = view.test_copy_options();

    assert_eq!(options.len(), 2);
    assert_eq!(options[0].label(), "operation id");
    assert_eq!(options[0].value().len(), 128);
    assert_eq!(options[1].value(), "@  current\n│  describe commit");
}

#[test]
fn movement_is_operation_item_based() {
    let mut view = operation_log_view(vec![
        operation_item(&["@  current", "│  args: jj describe"], Some("a")),
        operation_item(&["○  previous"], Some("b")),
    ]);

    view.execute(
        ViewCommand::MoveDown,
        CommandContext {
            size: ratatui::layout::Size {
                height: 10,
                width: 80,
            },
            search: None,
        },
    );

    assert_eq!(view.selection.index(), 1);
    view.execute(
        ViewCommand::MoveUp,
        CommandContext {
            size: ratatui::layout::Size {
                height: 10,
                width: 80,
            },
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 0);
}

#[test]
fn refresh_preserves_selected_operation_id() {
    let mut view = operation_log_view(vec![
        operation_item(&["@  current"], Some("first")),
        operation_item(&["○  previous"], Some("second")),
    ]);
    view.selection.set(1, view.entries.len());

    view.test_refresh_with_loader(|_| {
        Ok(vec![
            operation_item(&["@  second"], Some("second")),
            operation_item(&["○  third"], Some("third")),
        ])
    })
    .unwrap();

    assert_eq!(view.selection.index(), 0);
    assert_eq!(view.entries[0].operation_id(), Some("second"));
}

#[test]
fn refresh_clamps_when_selected_operation_disappears() {
    let mut view = operation_log_view(vec![
        operation_item(&["@  current"], Some("first")),
        operation_item(&["○  previous"], Some("second")),
    ]);
    view.selection.set(1, view.entries.len());

    view.test_refresh_with_loader(|_| Ok(vec![operation_item(&["@  current"], Some("first"))]))
        .unwrap();

    assert_eq!(view.selection.index(), 0);
}

#[test]
fn search_wraps_by_operation_item() {
    let mut view = operation_log_view(vec![
        operation_item(&["@  current", "│  args: jj describe"], Some("first")),
        operation_item(&["○  previous", "│  snapshot working copy"], Some("second")),
        operation_item(&["○  oldest", "│  snapshot before describe"], Some("third")),
    ]);
    view.selection.set(1, view.entries.len());
    let query = SearchQuery::new("describe".to_owned()).unwrap();

    assert_eq!(view.test_search_matches(&query), 2);
    assert!(view.test_next_match(&query));
    assert_eq!(view.selection.index(), 2);
    assert!(view.test_next_match(&query));
    assert_eq!(view.selection.index(), 0);
}

#[test]
fn operation_show_and_diff_open_selected_operation_detail() {
    let mut view = operation_log_view(vec![operation_item(&["@  current"], Some("first"))]);

    assert_eq!(
        view.execute(
            ViewCommand::OpenShow,
            CommandContext {
                size: ratatui::layout::Size {
                    height: 10,
                    width: 80
                },
                search: None,
            },
        ),
        ViewEffect::OpenView(ViewSpec::operation_show("first".to_owned()))
    );
    assert_eq!(
        view.execute(
            ViewCommand::OpenDiff,
            CommandContext {
                size: ratatui::layout::Size {
                    height: 10,
                    width: 80
                },
                search: None,
            },
        ),
        ViewEffect::OpenView(ViewSpec::operation_diff("first".to_owned()))
    );
}

#[test]
fn operation_detail_actions_are_disabled_without_operation_id() {
    let mut view = operation_log_view(vec![operation_item(&["@  current"], None)]);

    assert_eq!(
        view.execute(
            ViewCommand::OpenShow,
            CommandContext {
                size: ratatui::layout::Size {
                    height: 10,
                    width: 80
                },
                search: None,
            },
        ),
        ViewEffect::StatusMessage(
            "operation show unavailable: selected row has no operation id".to_owned()
        )
    );
    assert_eq!(
        view.execute(
            ViewCommand::OpenDiff,
            CommandContext {
                size: ratatui::layout::Size {
                    height: 10,
                    width: 80
                },
                search: None,
            },
        ),
        ViewEffect::StatusMessage(
            "operation diff unavailable: selected row has no operation id".to_owned()
        )
    );
}

#[test]
fn operation_recovery_action_menu_requires_exact_operation_id() {
    let mut view = operation_log_view(vec![operation_item(&["@  current"], None)]);

    assert_eq!(
        view.execute(
            ViewCommand::OpenActionMenu,
            CommandContext {
                size: ratatui::layout::Size {
                    height: 10,
                    width: 80
                },
                search: None,
            },
        ),
        ViewEffect::StatusMessage(
            "operation recovery actions unavailable: selected row has no operation id".to_owned()
        )
    );
}

#[test]
fn operation_recovery_action_menu_uses_selected_operation_id() {
    let operation_id = "b".repeat(128);
    let mut view = operation_log_view(vec![operation_item(&["@  current"], Some(&operation_id))]);

    let effect = view.execute(
        ViewCommand::OpenActionMenu,
        CommandContext {
            size: ratatui::layout::Size {
                height: 10,
                width: 80,
            },
            search: None,
        },
    );

    let ViewEffect::OpenActionMenu(menu) = effect else {
        panic!("expected operation action menu");
    };
    assert_eq!(menu.items().len(), 2);
    assert_eq!(menu.items()[0].action(), ActionKind::Restore);
    assert_eq!(
        menu.items()[0].label(),
        "restore repository to operation bbbbbbbb"
    );
    assert!(matches!(
        menu.items()[0].follow_up(),
        FollowUp::OperationRestoreExactTarget { operation_id: id } if id == &operation_id
    ));
    assert_eq!(menu.items()[1].action(), ActionKind::Revert);
    assert_eq!(menu.items()[1].label(), "revert operation bbbbbbbb");
    assert!(matches!(
        menu.items()[1].follow_up(),
        FollowUp::OperationRevertExactTarget { operation_id: id } if id == &operation_id
    ));
}

#[test]
fn bindings_expose_global_recovery_without_view_execution_target() {
    assert_eq!(
        BINDINGS
            .iter()
            .find(|binding| binding.command() == Command::OperationUndo)
            .map(|binding| binding.command()),
        Some(Command::OperationUndo)
    );
    assert_eq!(
        BINDINGS
            .iter()
            .find(|binding| binding.command() == Command::OperationRedo)
            .map(|binding| binding.command()),
        Some(Command::OperationRedo)
    );
}
