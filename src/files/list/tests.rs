use ratatui::text::Line;

use super::*;
use crate::command::{CommandContext, ViewEffect};
use crate::menus::CopyOption;
use crate::search::SearchQuery;

fn file_item(path: &str) -> FileListItem {
    FileListItem::new(vec![Line::from(path.to_owned())], path.to_owned())
}

fn file_list_view(paths: &[&str]) -> FileListView {
    FileListView {
        spec: ViewSpec::file_list(None, None),
        entries: paths.iter().map(|path| file_item(path)).collect(),
        selection: Selection::default(),
    }
}

#[test]
fn file_list_moves_by_path_item() {
    let mut view = file_list_view(&["alpha", "beta", "gamma"]);

    view.execute(
        ViewCommand::MoveLast,
        CommandContext {
            viewport_height: 3,
            viewport_width: 80,
            search: None,
        },
    );
    assert_eq!(view.selection.index(), 2);

    view.execute(
        ViewCommand::MoveUp,
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
fn file_list_search_wraps_without_reselecting_current_item() {
    let mut view = file_list_view(&["alpha", "target one", "beta", "target two"]);
    view.selection.set(1, view.item_count());
    let query = SearchQuery::new("target".to_owned()).unwrap();

    assert!(view.next_match(&query));
    assert_eq!(view.selection.index(), 3);

    assert!(view.previous_match(&query));
    assert_eq!(view.selection.index(), 1);
}

#[test]
fn file_list_copy_uses_exact_path() {
    let mut view = file_list_view(&["src/space file.txt", "docs/readme.md"]);
    view.selection.set(0, view.item_count());

    let options = view.copy_options();

    assert_eq!(
        options,
        vec![CopyOption::new("file path", "src/space file.txt")]
    );
}

#[test]
fn file_list_refresh_preserves_selected_path_when_possible() {
    let mut view = file_list_view(&["alpha", "beta", "gamma"]);
    view.selection.set(1, view.item_count());

    view.refresh_with_loader(|_| {
        Ok(vec![
            file_item("gamma"),
            file_item("beta"),
            file_item("delta"),
        ])
    })
    .unwrap();

    assert_eq!(view.selection.index(), 1);
    assert_eq!(view.selected_path(), Some("beta"));
}

#[test]
fn file_list_refresh_clamps_when_selected_path_disappears() {
    let mut view = file_list_view(&["alpha", "beta", "gamma"]);
    view.selection.set(2, view.item_count());

    view.refresh_with_loader(|_| Ok(vec![file_item("alpha")]))
        .unwrap();

    assert_eq!(view.selection.index(), 0);
    assert_eq!(view.selected_path(), Some("alpha"));
}

#[test]
fn file_list_open_selected_file_uses_exact_path() {
    let mut view = file_list_view(&["src/space file.txt"]);

    let effect = view.execute(
        ViewCommand::OpenItem,
        CommandContext {
            viewport_height: 3,
            viewport_width: 80,
            search: None,
        },
    );

    assert_eq!(
        effect,
        ViewEffect::OpenDetail(JjCommand::FileShow, "src/space file.txt".to_owned())
    );
}
