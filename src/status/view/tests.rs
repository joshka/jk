use insta::assert_snapshot;
use ratatui::text::Line;

use super::*;
use crate::command::{CommandContext, ViewEffect};
use crate::search::SearchQuery;
use crate::status::StatusFileAction;
use crate::status::rows::parse_status_row;

fn status_view(lines: &[&str]) -> StatusView {
    StatusView::test_new(lines)
}

#[test]
fn copy_options_use_full_status_text() {
    let view = status_view(&["Working copy changes:", "M src/app.rs"]);

    let options = view.copy_options();

    assert_eq!(options.len(), 1);
    assert_eq!(options[0].label(), "status text");
    assert_eq!(options[0].value(), "Working copy changes:\nM src/app.rs");
}

#[test]
fn status_navigation_moves_through_rows() {
    let mut view = status_view(&["one", "two", "three"]);

    view.execute(
        ViewCommand::MoveDown,
        CommandContext {
            size: ratatui::layout::Size {
                height: 3,
                width: 80,
            },
            search: None,
        },
    );
    assert_eq!(view.scroll_offset(), 1);

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
    assert_eq!(view.scroll_offset(), 2);
}

#[test]
fn status_search_moves_to_next_match() {
    let mut view = status_view(&["first", "beta", "third", "beta again"]);
    let query = SearchQuery::new("beta".to_owned()).unwrap();

    let effect = view.execute(
        ViewCommand::StartSearch,
        CommandContext {
            size: ratatui::layout::Size {
                height: 3,
                width: 80,
            },
            search: Some(&query),
        },
    );

    assert_eq!(effect, ViewEffect::SearchStarted { matches: 2 });
    assert_eq!(view.scroll_offset(), 1);
}

#[test]
fn clamp_preserves_readable_selection_after_document_shrinks() {
    let mut view = status_view(&["one", "two", "three"]);
    view.scroll_to_bottom(3);
    assert_eq!(view.scroll_offset(), 2);

    view.rows = vec![parse_status_row(Line::from("one".to_owned()))];
    view.clamp(3);

    assert_eq!(view.scroll_offset(), 0);
}

#[test]
fn status_rows_keep_rendered_sections_readable() {
    let view = status_view(&[
        "The working copy has conflicts:",
        "UU src/app.rs",
        "",
        "Working copy changes:",
        "M src/status.rs",
        "A docs/plan/progress.md",
        "",
        "Working copy  (@) : yostqsxw 12345678 Slice 6 work",
        "Parent commit (@-): mzvwutkl 87654321 Prior change",
    ]);

    let rendered = view
        .rows
        .iter()
        .map(StatusRow::row_text)
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!(rendered, @r"
        The working copy has conflicts:
        UU src/app.rs

        Working copy changes:
        M src/status.rs
        A docs/plan/progress.md

        Working copy  (@) : yostqsxw 12345678 Slice 6 work
        Parent commit (@-): mzvwutkl 87654321 Prior change
        ");
}

#[test]
fn status_parser_accepts_modified_added_and_deleted_paths() {
    let modified = parse_status_row(Line::from("M src/status.rs".to_owned()));
    let added = parse_status_row(Line::from("A docs/plan/progress.md".to_owned()));
    let deleted = parse_status_row(Line::from("D dir/with spaces/file.txt".to_owned()));

    assert_eq!(modified.exact_path().unwrap(), "src/status.rs");
    assert_eq!(added.exact_path().unwrap(), "docs/plan/progress.md");
    assert_eq!(deleted.exact_path().unwrap(), "dir/with spaces/file.txt");
}

#[test]
fn status_parser_classifies_untracked_tracked_and_non_chmod_rows() {
    let untracked = parse_status_row(Line::from("? scratch.txt".to_owned()));
    let modified = parse_status_row(Line::from("M src/lib.rs".to_owned()));
    let added = parse_status_row(Line::from("A src/new.rs".to_owned()));
    let deleted = parse_status_row(Line::from("D src/old.rs".to_owned()));
    let missing = parse_status_row(Line::from("! src/missing.rs".to_owned()));

    assert_eq!(
        untracked.file_action().unwrap(),
        StatusFileAction::Track {
            path: "scratch.txt".to_owned()
        }
    );
    assert_eq!(
        modified.file_action().unwrap(),
        StatusFileAction::Tracked {
            path: "src/lib.rs".to_owned(),
            restore_allowed: true,
            chmod_allowed: true,
        }
    );
    assert!(added.file_action().unwrap().chmod_allowed());
    assert!(!deleted.file_action().unwrap().chmod_allowed());
    assert!(!missing.file_action().unwrap().restore_allowed());
    assert!(!missing.file_action().unwrap().chmod_allowed());
}

#[test]
fn status_parser_disables_renamed_and_conflicted_rows() {
    let renamed = parse_status_row(Line::from("R {old.rs => new.rs}".to_owned()));
    let conflict = parse_status_row(Line::from("UU src/lib.rs".to_owned()));

    assert_eq!(
        renamed.exact_path().unwrap_err(),
        "status file action unavailable: renamed status rows contain multiple paths"
    );
    assert_eq!(
        conflict.exact_path().unwrap_err(),
        "status file action unavailable: conflicted status rows are not file hygiene targets"
    );
}

#[test]
fn status_parser_disables_ambiguous_paths() {
    let absolute = parse_status_row(Line::from("M /tmp/file.txt".to_owned()));
    let parent = parse_status_row(Line::from("M ../outside.txt".to_owned()));
    let multiple = parse_status_row(Line::from("M {old => new}".to_owned()));
    let leading_space = parse_status_row(Line::from("M  leading-space.txt".to_owned()));

    assert_eq!(
        absolute.exact_path().unwrap_err(),
        "status file action unavailable: selected path is absolute"
    );
    assert_eq!(
        parent.exact_path().unwrap_err(),
        "status file action unavailable: selected path is not a clean repo-relative path"
    );
    assert_eq!(
        multiple.exact_path().unwrap_err(),
        "status file action unavailable: selected row appears to contain multiple paths"
    );
    assert_eq!(
        leading_space.exact_path().unwrap_err(),
        "status file action unavailable: selected path has ambiguous surrounding whitespace"
    );
}

#[test]
fn status_action_path_follows_selected_exact_row() {
    let mut view = status_view(&["Working copy changes:", "M src/status.rs"]);

    assert_eq!(
        view.selected_exact_path().unwrap_err(),
        "status file action unavailable: selected row has no exact file path"
    );

    view.execute(
        ViewCommand::MoveDown,
        CommandContext {
            size: ratatui::layout::Size {
                height: 4,
                width: 80,
            },
            search: None,
        },
    );

    assert_eq!(view.selected_exact_path().unwrap(), "src/status.rs");
}

#[test]
fn status_refresh_preserves_selected_exact_path_when_present() {
    let mut view = status_view(&["M alpha", "M beta"]);
    view.selection.set(1, view.rows.len());

    view.refresh_with_loader(|_| {
        Ok(vec![
            parse_status_row(Line::from("M beta".to_owned())),
            parse_status_row(Line::from("M alpha".to_owned())),
        ])
    })
    .unwrap();

    assert_eq!(view.scroll_offset(), 0);
    assert_eq!(view.selected_exact_path().unwrap(), "beta");
}

#[test]
fn status_refresh_clamps_when_selected_path_disappears() {
    let mut view = status_view(&["M alpha", "M beta"]);
    view.selection.set(1, view.rows.len());

    view.refresh_with_loader(|_| Ok(vec![parse_status_row(Line::from("M alpha".to_owned()))]))
        .unwrap();

    assert_eq!(view.scroll_offset(), 0);
    assert_eq!(view.selected_exact_path().unwrap(), "alpha");
}
