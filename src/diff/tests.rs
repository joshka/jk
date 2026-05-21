use ratatui::text::Line;
use ratatui_macros::line;

use super::*;
use crate::jj::{JjCommand, ViewSpec};
use crate::rendered_jj::DocumentLines;

#[test]
fn diff_view_pins_first_file_immediately() {
    let view = diff_view(
        vec![
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        0,
    );

    let projection = view.projection();

    assert_eq!(projection.fixed_lines().len(), 1);
    assert_eq!(
        line_text(projection.fixed_lines()[0].clone()),
        "Added regular file Cargo.toml:"
    );
    assert_eq!(projection.body_scroll_offset(), 0);
    assert_eq!(projection.body_lines().len(), 1);
}

#[test]
fn diff_view_updates_current_file() {
    let view = diff_view(
        vec![
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
            line!("Modified regular file src/main.rs:"),
            line!("        1: fn main() {}"),
        ],
        2,
    );

    let projection = view.projection();

    assert_eq!(
        line_text(projection.fixed_lines()[0].clone()),
        "Modified regular file src/main.rs:"
    );
}

#[test]
fn diff_scroll_clamp_accounts_for_sticky_file_line() {
    let mut view = diff_view(
        vec![
            line!("Added regular file Cargo.toml:"),
            line!("        1"),
            line!("        2"),
            line!("        3"),
            line!("        4"),
            line!("        5"),
        ],
        0,
    );

    view.scroll_to_bottom(3);

    assert_eq!(view.scroll_offset(), 4);
}

#[test]
fn diff_scroll_down_skips_offsets_with_identical_projection() {
    let mut view = diff_view(
        vec![
            line!("Added regular file .gitignore:"),
            line!("        1: /target"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
            line!("        2: name = \"jk\""),
        ],
        0,
    );

    view.scroll_down(4, 1);

    assert_eq!(view.scroll_offset(), 2);
}

#[test]
fn diff_file_navigation_moves_between_headings() {
    let mut view = diff_view(
        vec![
            line!("Added regular file .gitignore:"),
            line!("        1: /target"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        0,
    );

    view.next_file();

    assert_eq!(view.scroll_offset(), 2);

    view.previous_file();

    assert_eq!(view.scroll_offset(), 0);
}

#[test]
fn document_search_wraps_without_reselecting_current_line() {
    let mut view = diff_view(
        vec![
            line!("alpha"),
            line!("target one"),
            line!("beta"),
            line!("target two"),
        ],
        1,
    );
    let query = SearchQuery::new("target".to_owned()).unwrap();

    assert!(view.next_match(4, &query));
    assert_eq!(view.scroll_offset(), 3);

    assert!(view.next_match(4, &query));
    assert_eq!(view.scroll_offset(), 1);

    assert!(view.previous_match(4, &query));
    assert_eq!(view.scroll_offset(), 3);
}

#[test]
fn document_search_does_not_move_for_only_current_match() {
    let mut view = diff_view(vec![line!("alpha"), line!("target"), line!("beta")], 1);
    let query = SearchQuery::new("target".to_owned()).unwrap();

    assert!(!view.next_match(4, &query));
    assert_eq!(view.scroll_offset(), 1);

    assert!(!view.previous_match(4, &query));
    assert_eq!(view.scroll_offset(), 1);
}

#[test]
fn command_execution_moves_between_files() {
    let mut view = diff_view(
        vec![
            line!("Added regular file .gitignore:"),
            line!("        1: /target"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        0,
    );

    assert_eq!(
        view.execute(ViewCommand::NextFile, context(None)),
        ViewEffect::Handled
    );
    assert_eq!(view.scroll_offset(), 2);
}

#[test]
fn command_execution_opens_show_for_same_revset() {
    let mut view = diff_view(vec![line!("Added regular file Cargo.toml:")], 0);
    view.spec = ViewSpec::new(JjCommand::Diff, vec!["-r".to_owned(), "main".to_owned()]);

    assert_eq!(
        view.execute(ViewCommand::OpenShow, context(None)),
        ViewEffect::OpenDetail(JjCommand::Show, "main".to_owned())
    );
}

#[test]
fn command_execution_opens_file_list_with_exact_target_provenance() {
    let mut view = diff_view(vec![line!("Added regular file Cargo.toml:")], 0);
    view.spec = ViewSpec::diff("change-a".to_owned(), crate::jj::DiffFormat::Default);

    assert_eq!(
        view.execute(ViewCommand::OpenFiles, context(None)),
        ViewEffect::OpenView(
            ViewSpec::file_list(Some("change-a".to_owned()), Some("Cargo.toml".to_owned()))
                .with_exact_change_target()
        )
    );
}

#[test]
fn command_execution_opens_file_list_with_inexact_direct_revset() {
    let mut view = diff_view(vec![line!("Added regular file Cargo.toml:")], 0);
    view.spec = ViewSpec::new(JjCommand::Diff, vec!["-r".to_owned(), "main".to_owned()]);

    assert_eq!(
        view.execute(ViewCommand::OpenFiles, context(None)),
        ViewEffect::OpenView(ViewSpec::file_list(
            Some("main".to_owned()),
            Some("Cargo.toml".to_owned())
        ))
    );
}

#[test]
fn copy_options_use_plain_file_label() {
    let view = diff_view(
        vec![
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        0,
    );

    let file = view
        .copy_options()
        .into_iter()
        .find(|option| option.label() == "file path")
        .unwrap();

    assert_eq!(file.value(), "Cargo.toml");
}

#[test]
fn horizontal_scroll_keeps_file_navigation_on_source_anchors() {
    let mut view = diff_view(
        vec![
            line!("Added regular file .gitignore:"),
            line!("        1: /target"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: 0123456789ABCDEFGHIJ"),
        ],
        0,
    );

    let _ = view.execute(ViewCommand::ToggleWrap, context_width(12, None));
    let _ = view.execute(ViewCommand::ScrollRight, context_width(12, None));

    view.next_file();

    let file = view
        .copy_options()
        .into_iter()
        .find(|option| option.label() == "file path")
        .unwrap();

    assert_eq!(view.scroll_offset(), 2);
    assert_eq!(view.horizontal_offset(), 1);
    assert_eq!(file.value(), "Cargo.toml");
}

#[test]
fn diff_clamp_revalidates_horizontal_offset_for_current_width() {
    let mut view = diff_view(
        vec![
            line!("Added regular file Cargo.toml:"),
            line!("        1: 0123456789ABCDEFGHIJ"),
        ],
        0,
    );

    let _ = view.execute(ViewCommand::ToggleWrap, context_width(12, None));
    for _ in 0..20 {
        let _ = view.execute(ViewCommand::ScrollRight, context_width(12, None));
    }
    assert!(view.horizontal_offset() > 0);

    view.clamp(4, 80);

    assert_eq!(view.horizontal_offset(), 0);
}

fn diff_view(lines: Vec<Line<'static>>, scroll_offset: usize) -> DiffView {
    let document = DocumentLines::new(lines);
    let mut document = StickyFileDocument::new(document);
    document.set_scroll_offset(u16::MAX, scroll_offset);
    DiffView {
        spec: ViewSpec::new(JjCommand::Diff, Vec::new()),
        document,
    }
}

fn context(search: Option<&SearchQuery>) -> CommandContext<'_> {
    context_width(80, search)
}

fn context_width(viewport_width: u16, search: Option<&SearchQuery>) -> CommandContext<'_> {
    CommandContext {
        viewport_height: 4,
        viewport_width,
        search,
    }
}

fn line_text(line: Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}
