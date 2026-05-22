use ratatui::text::Line;

use super::*;
use crate::command::{CommandContext, ViewEffect};
use crate::jj::JjCommand;
use crate::menus::CopyOption;
use crate::search::SearchQuery;

fn file_show_view(path: &str, lines: &[&str]) -> FileShowView {
    FileShowView::new(
        ViewSpec::new(JjCommand::FileShow, Vec::new()),
        path,
        DocumentLines::new(
            lines
                .iter()
                .map(|line| Line::from((*line).to_owned()))
                .collect::<Vec<_>>(),
        ),
    )
}

#[test]
fn file_show_projection_is_plain_document() {
    let view = file_show_view("src/lib.rs", &["alpha", "beta"]);

    let projection = view.projection();

    assert!(projection.fixed_lines().is_empty());
    assert_eq!(projection.body_lines().len(), 2);
    assert_eq!(projection.body_scroll_offset(), 0);
}

#[test]
fn file_show_search_wraps_without_reselecting_current_line() {
    let mut view = file_show_view("src/lib.rs", &["alpha", "target one", "beta", "target two"]);
    view.set_scroll_offset(3, 1);
    let query = SearchQuery::new("target".to_owned()).unwrap();

    assert!(view.next_match(3, &query));
    assert_eq!(view.scroll_offset(), 3);

    assert!(view.previous_match(3, &query));
    assert_eq!(view.scroll_offset(), 1);
}

#[test]
fn file_show_copy_uses_exact_path() {
    let view = file_show_view("src/space file.txt", &["alpha"]);

    let options = view.copy_options();

    assert_eq!(
        options,
        vec![CopyOption::new("file path", "src/space file.txt")]
    );
}

#[test]
fn file_show_refresh_clamps_scroll_after_content_shrinks() {
    let mut view = file_show_view("src/lib.rs", &["alpha", "beta", "gamma"]);
    view.set_scroll_offset(3, 2);

    view.refresh_with_loader(|_| Ok(DocumentLines::new(vec![Line::from("alpha")])))
        .unwrap();

    assert_eq!(view.scroll_offset(), 0);
}

#[test]
fn file_show_clamps_horizontal_offset_after_refresh_shrinks_content() {
    let mut view = file_show_view("README.md", &["0123456789ABCDEFGHIJ"]);
    let _ = view.execute(ViewCommand::ToggleWrap, context(10, None));
    for _ in 0..20 {
        let _ = view.execute(ViewCommand::ScrollRight, context(10, None));
    }
    assert_eq!(view.horizontal_offset(), 10);

    view.refresh_with_loader(|_| Ok(DocumentLines::new(vec![Line::from("short")])))
        .unwrap();
    view.clamp(3, 10);

    assert_eq!(view.horizontal_offset(), 0);
}

#[test]
fn file_show_toggle_wrap_and_horizontal_scroll_clamps() {
    let mut view = file_show_view("README.md", &["0123456789ABCDEFGHIJ"]);

    assert_eq!(view.display_mode(), DocumentDisplayMode::Wrap);

    assert_eq!(
        view.execute(ViewCommand::ToggleWrap, context(10, None)),
        ViewEffect::Handled
    );
    assert_eq!(view.display_mode(), DocumentDisplayMode::NoWrap);

    for _ in 0..20 {
        let _ = view.execute(ViewCommand::ScrollRight, context(10, None));
    }

    assert_eq!(view.horizontal_offset(), 10);

    for _ in 0..20 {
        let _ = view.execute(ViewCommand::ScrollLeft, context(10, None));
    }

    assert_eq!(view.horizontal_offset(), 0);
}

#[test]
fn file_show_horizontal_scroll_does_not_change_vertical_scroll() {
    let mut view = file_show_view("README.md", &["line 0", "line 1", "0123456789ABCDEFGHIJ"]);
    view.set_scroll_offset(3, 2);

    let _ = view.execute(ViewCommand::ToggleWrap, context(10, None));
    let _ = view.execute(ViewCommand::ScrollRight, context(10, None));

    assert_eq!(view.scroll_offset(), 2);
    assert_eq!(view.horizontal_offset(), 1);
}

#[test]
fn file_show_search_still_moves_by_source_line_in_no_wrap() {
    let mut view = file_show_view(
        "README.md",
        &["alpha", "0123456789 target one", "beta", "target two"],
    );
    let query = SearchQuery::new("target".to_owned()).unwrap();

    let _ = view.execute(ViewCommand::ToggleWrap, context(10, Some(&query)));
    let _ = view.execute(ViewCommand::ScrollRight, context(10, Some(&query)));

    assert!(view.next_match(3, &query));

    assert_eq!(view.scroll_offset(), 1);
    assert_eq!(view.horizontal_offset(), 1);
}

fn context(viewport_width: u16, search: Option<&SearchQuery>) -> CommandContext<'_> {
    CommandContext {
        viewport_height: 3,
        viewport_width,
        search,
    }
}
