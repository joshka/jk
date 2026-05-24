use super::*;
use crate::command::{CommandContext, ViewCommand, ViewEffect};
use crate::jj;
use crate::menus::CopyOption;
use crate::search::SearchQuery;

fn parsed_entry(path: &str, file_type: &str, side_count: usize) -> ResolveEntry {
    ResolveEntry::parsed(
        Some(path.to_owned()),
        Some(file_type.to_owned()),
        Some(side_count),
    )
}

fn resolve_view(entries: Vec<ResolveEntry>) -> ResolveView {
    ResolveView::test_new(entries)
}

#[test]
fn resolve_view_moves_by_conflict_item() {
    let mut view = resolve_view(vec![
        parsed_entry("alpha", "file", 2),
        parsed_entry("beta", "symlink", 3),
        parsed_entry("gamma", "file", 4),
    ]);

    view.execute(
        ViewCommand::MoveLast,
        CommandContext {
            size: ratatui::layout::Size {
                height: 3,
                width: 80,
            },
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 2);

    view.execute(
        ViewCommand::MoveUp,
        CommandContext {
            size: ratatui::layout::Size {
                height: 3,
                width: 80,
            },
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 1);
}

#[test]
fn resolve_view_search_wraps_without_reselecting_current_item() {
    let mut view = resolve_view(vec![
        parsed_entry("alpha", "file", 2),
        parsed_entry("target one", "file", 2),
        parsed_entry("beta", "file", 2),
        parsed_entry("target two", "file", 2),
    ]);
    view.selection.set(1, view.item_count());
    let query = SearchQuery::new("target".to_owned()).unwrap();

    assert!(view.next_match(&query));
    assert_eq!(view.selection.index(), 3);

    assert!(view.previous_match(&query));
    assert_eq!(view.selection.index(), 1);
}

#[test]
fn resolve_copy_options_include_exact_path_and_row_text() {
    let mut view = resolve_view(vec![parsed_entry("src/space file.txt", "file", 3)]);
    view.selection.set(0, view.item_count());

    assert_eq!(
        view.copy_options(),
        vec![
            CopyOption::new("conflict path", "src/space file.txt"),
            CopyOption::new("row text", "src/space file.txt\ntype: file  sides: 3",),
        ]
    );
}

#[test]
fn resolve_refresh_preserves_selected_path_when_possible() {
    let mut view = resolve_view(vec![
        parsed_entry("alpha", "file", 2),
        parsed_entry("beta", "file", 3),
        parsed_entry("gamma", "file", 4),
    ]);
    view.selection.set(1, view.item_count());

    view.refresh_with_loader(|_| {
        Ok(vec![
            parsed_entry("gamma", "file", 4),
            parsed_entry("beta", "file", 3),
            parsed_entry("delta", "file", 2),
        ])
    })
    .unwrap();

    assert_eq!(view.selection.index(), 1);
    assert_eq!(view.selected_path(), Some("beta"));
}

#[test]
fn resolve_refresh_clamps_when_selected_path_disappears() {
    let mut view = resolve_view(vec![
        parsed_entry("alpha", "file", 2),
        parsed_entry("beta", "file", 3),
        parsed_entry("gamma", "file", 4),
    ]);
    view.selection.set(2, view.item_count());

    view.refresh_with_loader(|_| Ok(vec![parsed_entry("alpha", "file", 2)]))
        .unwrap();

    assert_eq!(view.selection.index(), 0);
    assert_eq!(view.selected_path(), Some("alpha"));
}

#[test]
fn resolve_open_selected_conflict_uses_exact_path() {
    let mut view = resolve_view(vec![parsed_entry("src/space file.txt", "file", 3)]);

    let effect = view.execute(
        ViewCommand::OpenItem,
        CommandContext {
            size: ratatui::layout::Size {
                height: 3,
                width: 80,
            },
            search: None,
        },
    );

    assert_eq!(
        effect,
        ViewEffect::OpenDetail(jj::Command::FileShow, "src/space file.txt".to_owned())
    );
}

#[test]
fn resolve_open_item_requires_exact_path() {
    let mut view = resolve_view(vec![ResolveEntry::parsed(
        None,
        Some("symlink".to_owned()),
        Some(2),
    )]);

    let effect = view.execute(
        ViewCommand::OpenItem,
        CommandContext {
            size: ratatui::layout::Size {
                height: 3,
                width: 80,
            },
            search: None,
        },
    );

    assert_eq!(
        effect,
        ViewEffect::StatusMessage(
            "resolve inspect unavailable: selected conflict has no exact path".to_owned(),
        )
    );
}
