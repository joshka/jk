use ratatui::text::Line;
use ratatui_macros::line;

use super::*;
use crate::command::{CommandContext, ViewEffect};
use crate::documents::DocumentLines;
use crate::jj::{JjCommand, ViewSpec};
use crate::search::SearchQuery;

#[test]
fn show_view_is_plain_before_first_file() {
    let view = show_view(
        vec![
            line!("Commit ID: abc"),
            line!("    A long message"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        1,
    );

    let projection = view.projection();

    assert!(projection.fixed_lines().is_empty());
    assert_eq!(projection.body_lines().len(), 4);
    assert_eq!(projection.body_scroll_offset(), 1);
}

#[test]
fn show_view_pins_commit_context_and_current_file() {
    let view = show_view(
        vec![
            line!("Commit ID: abc"),
            line!("    A long message"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        2,
    );

    let projection = view.projection();

    assert_eq!(projection.fixed_lines().len(), 4);
    assert_eq!(line_text(projection.fixed_lines()[0].clone()), "@  abc");
    assert_eq!(
        line_text(projection.fixed_lines()[3].clone()),
        "Added regular file Cargo.toml:"
    );
    assert!(
        projection
            .body_lines()
            .iter()
            .all(|line| line_text(line.clone()) != "Added regular file Cargo.toml:")
    );
}

#[test]
fn show_view_long_message_stays_scrollable() {
    let view = show_view(
        vec![
            line!("Commit ID: abc"),
            line!("    line 1"),
            line!("    line 2"),
            line!("    line 3"),
            line!("    line 4"),
            line!("Added regular file Cargo.toml:"),
        ],
        4,
    );

    assert!(view.projection().fixed_lines().is_empty());
}

#[test]
fn show_scroll_down_skips_separator_heading_and_first_body_duplicates() {
    let mut view = show_view(
        vec![
            line!("Commit ID: abc"),
            line!("    subject"),
            line!(""),
            line!("Added regular file .gitignore:"),
            line!("        1: /target"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        1,
    );

    view.scroll_down(6, 1);

    assert_eq!(view.scroll_offset(), 2);

    view.scroll_down(6, 1);

    assert_eq!(view.scroll_offset(), 5);
}

#[test]
fn show_file_navigation_uses_sticky_activation_offsets() {
    let mut view = show_view(
        vec![
            line!("Commit ID: abc"),
            line!("    subject"),
            line!(""),
            line!("Added regular file .gitignore:"),
            line!("        1: /target"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ],
        0,
    );

    view.next_file();

    assert_eq!(view.scroll_offset(), 2);

    view.next_file();

    assert_eq!(view.scroll_offset(), 5);

    view.previous_file();

    assert_eq!(view.scroll_offset(), 2);
}

#[test]
fn command_execution_opens_diff_for_same_revset() {
    let mut view = show_view(vec![line!("Added regular file Cargo.toml:")], 0);
    view.spec = ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]);

    assert_eq!(
        view.execute(ViewCommand::OpenDiff, context(None)),
        ViewEffect::OpenDetail(JjCommand::Diff, "main".to_owned())
    );
}

#[test]
fn command_execution_opens_file_list_with_exact_target_provenance() {
    let mut view = show_view(vec![line!("Added regular file Cargo.toml:")], 0);
    view.spec = ViewSpec::show("change-a".to_owned(), crate::jj::DiffFormat::Default);

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
    let mut view = show_view(vec![line!("Added regular file Cargo.toml:")], 0);
    view.spec = ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]);

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
    let view = show_view(
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
fn horizontal_scroll_preserves_current_file_copy_label() {
    let mut view = show_view(
        vec![
            line!("Added regular file Cargo.toml:"),
            line!("        1: 0123456789ABCDEFGHIJ"),
        ],
        0,
    );

    let _ = view.execute(ViewCommand::ToggleWrap, context_width(12, None));
    let _ = view.execute(ViewCommand::ScrollRight, context_width(12, None));

    let file = view
        .copy_options()
        .into_iter()
        .find(|option| option.label() == "file path")
        .unwrap();

    assert_eq!(view.scroll_offset(), 0);
    assert_eq!(view.horizontal_offset(), 1);
    assert_eq!(file.value(), "Cargo.toml");
}

#[test]
fn show_clamp_revalidates_horizontal_offset_for_current_width() {
    let mut view = show_view(
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

    view.clamp(6, 80);

    assert_eq!(view.horizontal_offset(), 0);
}

fn show_view(lines: Vec<Line<'static>>, scroll_offset: usize) -> ShowView {
    let document = DocumentLines::new(lines);
    let mut document = StickyFileDocument::new(document);
    document.set_scroll_offset(u16::MAX, scroll_offset);
    ShowView {
        spec: ViewSpec::new(JjCommand::Show, Vec::new()),
        document,
        compact_context: vec![line!("@  abc"), line!("│  subject")],
    }
}

fn context(search: Option<&SearchQuery>) -> CommandContext<'_> {
    context_width(80, search)
}

fn context_width(viewport_width: u16, search: Option<&SearchQuery>) -> CommandContext<'_> {
    CommandContext {
        viewport_height: 6,
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
