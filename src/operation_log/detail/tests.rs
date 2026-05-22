use ratatui::text::Line;

use super::*;
use crate::documents::lines_text;

fn operation_detail_view(
    command: JjCommand,
    operation_id: &str,
    lines: &[&str],
) -> OperationDetailView {
    let spec = match command {
        JjCommand::OperationShow => ViewSpec::operation_show(operation_id.to_owned()),
        JjCommand::OperationDiff => ViewSpec::operation_diff(operation_id.to_owned()),
        _ => panic!("expected operation detail command"),
    };
    OperationDetailView::test_new(
        spec,
        DocumentLines::new(
            lines
                .iter()
                .map(|line| Line::from((*line).to_owned()))
                .collect(),
        ),
    )
}

#[test]
fn operation_detail_scrolls_searches_and_copies_plain_document() {
    let mut view = operation_detail_view(
        JjCommand::OperationShow,
        "0123456789abcdef",
        &[
            "operation abc",
            "args: jj describe",
            "snapshot working copy",
        ],
    );

    view.execute(
        ViewCommand::MoveDown,
        CommandContext {
            viewport_height: 3,
            viewport_width: 80,
            search: None,
        },
    );
    assert_eq!(view.scroll_offset(), 1);

    let query = SearchQuery::new("snapshot".to_owned()).unwrap();
    assert_eq!(view.search_matches(&query), 1);
    assert!(view.next_match(&query));
    assert_eq!(view.scroll_offset(), 2);

    let options = view.copy_options();
    assert_eq!(options[0].label(), "operation id");
    assert_eq!(options[0].value(), "0123456789abcdef");
    assert_eq!(options[1].label(), "operation detail text");
    assert_eq!(
        options[1].value(),
        "operation abc\nargs: jj describe\nsnapshot working copy"
    );
}

#[test]
fn operation_detail_does_not_pin_file_like_headings() {
    let view = operation_detail_view(
        JjCommand::OperationDiff,
        "abcdef",
        &["Modified regular file src/main.rs:", "        1: old"],
    );

    let projection = view.projection();

    assert!(projection.fixed_lines().is_empty());
    assert_eq!(
        lines_text(projection.body_lines()),
        "Modified regular file src/main.rs:\n        1: old"
    );
}

#[test]
fn operation_detail_switches_between_show_and_diff_for_same_operation() {
    let mut show = operation_detail_view(JjCommand::OperationShow, "abcdef", &["operation"]);
    let mut diff = operation_detail_view(JjCommand::OperationDiff, "abcdef", &["operation"]);

    assert_eq!(
        show.execute(
            ViewCommand::OpenDiff,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        ),
        ViewEffect::OpenView(ViewSpec::operation_diff("abcdef".to_owned()))
    );
    assert_eq!(
        diff.execute(
            ViewCommand::OpenShow,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        ),
        ViewEffect::OpenView(ViewSpec::operation_show("abcdef".to_owned()))
    );
    assert_eq!(
        show.execute(
            ViewCommand::OpenShow,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        ),
        ViewEffect::Ignored
    );
    assert_eq!(
        diff.execute(
            ViewCommand::OpenDiff,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        ),
        ViewEffect::Ignored
    );
}
