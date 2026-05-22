use ratatui::text::Line;

use super::*;
use crate::command::CommandContext;
use crate::menus::CopyOption;
use crate::search::SearchQuery;

fn workspace_item(label: &str, name: Option<&str>) -> WorkspaceItem {
    WorkspaceItem::new(
        vec![Line::from(label.to_owned())],
        name.map(str::to_owned),
        name.map(|name| format!("{name}-change")),
        name.map(|name| format!("{name}-commit")),
    )
}

fn workspace_context(entries: Vec<WorkspaceItem>) -> WorkspaceContext {
    WorkspaceContext::new(Some("/repo".to_owned()), None, entries, None, None)
}

#[test]
fn movement_clamps_to_workspace_rows() {
    let mut view = WorkspacesView::test_new(workspace_context(vec![
        workspace_item("default: one", Some("default")),
        workspace_item("other: two", Some("other")),
    ]));

    view.execute(
        ViewCommand::MoveLast,
        CommandContext {
            viewport_height: 3,
            viewport_width: 80,
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 1);

    view.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 3,
            viewport_width: 80,
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 1);

    view.execute(
        ViewCommand::MoveFirst,
        CommandContext {
            viewport_height: 3,
            viewport_width: 80,
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 0);
}

#[test]
fn search_wraps_by_rendered_workspace_row() {
    let mut view = WorkspacesView::test_new(workspace_context(vec![
        workspace_item("default: target", Some("default")),
        workspace_item("other: plain", Some("other")),
        workspace_item("third: target", Some("third")),
    ]));
    view.selection.set(0, view.item_count());
    let query = SearchQuery::new("target".to_owned()).unwrap();

    assert_eq!(view.search_matches(&query), 2);
    assert!(view.next_match(&query));
    assert_eq!(view.selection.index(), 2);
    assert!(view.next_match(&query));
    assert_eq!(view.selection.index(), 0);
}

#[test]
fn copy_options_use_root_and_exact_metadata_when_available() {
    let mut view = WorkspacesView::test_new(workspace_context(vec![WorkspaceItem::new(
        vec![Line::from("rendered label: abc")],
        Some("exact-name".to_owned()),
        Some("change-id".to_owned()),
        Some("commit-id".to_owned()),
    )]));
    view.selection.set(0, view.item_count());

    assert_eq!(
        view.copy_options(),
        vec![
            CopyOption::new("current root", "/repo"),
            CopyOption::new("workspace name", "exact-name"),
            CopyOption::new("change id", "change-id"),
            CopyOption::new("commit id", "commit-id"),
            CopyOption::new("row text", "rendered label: abc"),
        ]
    );
}

#[test]
fn copy_options_degrade_to_root_and_row_text_without_metadata() {
    let view = WorkspacesView::test_new(workspace_context(vec![WorkspaceItem::new(
        vec![Line::from("default: rendered")],
        None,
        None,
        None,
    )]));

    assert_eq!(
        view.copy_options(),
        vec![
            CopyOption::new("current root", "/repo"),
            CopyOption::new("row text", "default: rendered"),
        ]
    );
}

#[test]
fn refresh_preserves_selected_workspace_name_when_possible() {
    let mut view = WorkspacesView::test_new(workspace_context(vec![
        workspace_item("default: one", Some("default")),
        workspace_item("other: two", Some("other")),
    ]));
    view.selection.set(1, view.item_count());

    view.refresh_with_loader(|_| {
        Ok(workspace_context(vec![
            workspace_item("other: moved", Some("other")),
            workspace_item("third: new", Some("third")),
        ]))
    })
    .unwrap();

    assert_eq!(view.selection.index(), 0);
    assert_eq!(
        view.selected_entry().and_then(WorkspaceItem::name),
        Some("other")
    );
}

#[test]
fn refresh_falls_back_to_previous_index_when_workspace_metadata_disappears() {
    let mut view = WorkspacesView::test_new(workspace_context(vec![
        workspace_item("default: one", Some("default")),
        workspace_item("other: two", Some("other")),
        workspace_item("third: three", Some("third")),
    ]));
    view.selection.set(1, view.item_count());

    view.refresh_with_loader(|_| {
        Ok(workspace_context(vec![
            WorkspaceItem::new(vec![Line::from("default: rendered")], None, None, None),
            WorkspaceItem::new(vec![Line::from("other: rendered")], None, None, None),
            WorkspaceItem::new(vec![Line::from("third: rendered")], None, None, None),
        ]))
    })
    .unwrap();

    assert_eq!(view.selection.index(), 1);
    assert_eq!(view.selected_entry().and_then(WorkspaceItem::name), None);
}

#[test]
fn refresh_clamps_by_previous_index_when_metadata_disappears_and_list_shrinks() {
    let mut view = WorkspacesView::test_new(workspace_context(vec![
        workspace_item("default: one", Some("default")),
        workspace_item("other: two", Some("other")),
        workspace_item("third: three", Some("third")),
    ]));
    view.selection.set(2, view.item_count());

    view.refresh_with_loader(|_| {
        Ok(workspace_context(vec![WorkspaceItem::new(
            vec![Line::from("default: rendered")],
            None,
            None,
            None,
        )]))
    })
    .unwrap();

    assert_eq!(view.selection.index(), 0);
    assert_eq!(view.selected_entry().and_then(WorkspaceItem::name), None);
}

#[test]
fn empty_and_degraded_output_is_readable() {
    let context = WorkspaceContext::new(
        None,
        Some("jj root failed: no workspace".to_owned()),
        Vec::new(),
        Some("jj workspace list failed: unsupported".to_owned()),
        Some("workspace metadata parse failed".to_owned()),
    );
    let view = WorkspacesView::test_new(context);

    assert_eq!(view.item_count(), 0);
    assert_eq!(view.copy_options(), Vec::<CopyOption>::new());
    assert_eq!(
        view.header_lines()
            .into_iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>(),
        vec![
            "current root: unavailable",
            "root error: jj root failed: no workspace",
            "workspace list error: jj workspace list failed: unsupported",
            "workspace metadata warning: workspace metadata parse failed",
        ]
    );
}
