use ratatui::text::Line;

use super::LogItem;
use super::metadata::{RevisionMetadata, parse_metadata_line, parse_revision_metadata_lines};
use super::pairing::{group_lines, starts_log_item_text};
use crate::rendered_rows::RowMetadata;

fn revision_metadata(change_id: &str, commit_id: &str) -> RevisionMetadata {
    RevisionMetadata {
        change_id: change_id.to_owned(),
        commit_id: Some(commit_id.to_owned()),
    }
}

fn log_item_text(item: &LogItem) -> String {
    item.lines()
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn metadata_line_parses_full_change_and_commit_ids() {
    let metadata = parse_metadata_line(
        "@  qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq 0123456789abcdef0123456789abcdef01234567",
    )
    .unwrap();

    assert_eq!(metadata.change_id, "qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq");
    assert_eq!(
        metadata.commit_id.as_deref(),
        Some("0123456789abcdef0123456789abcdef01234567")
    );
}

#[test]
fn metadata_line_rejects_short_ids() {
    assert!(parse_metadata_line("@  short 0123456789abcdef0123456789abcdef01234567").is_none());
    assert!(parse_metadata_line("@  qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq short").is_none());
}

#[test]
fn metadata_lines_skip_graph_only_rows() {
    let metadata = parse_revision_metadata_lines(vec![
        "│".to_owned(),
        "~".to_owned(),
        "@  qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq 0123456789abcdef0123456789abcdef01234567".to_owned(),
    ]);

    assert_eq!(
        metadata,
        RowMetadata::Valid(vec![revision_metadata(
            "qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq",
            "0123456789abcdef0123456789abcdef01234567",
        )])
    );
}

#[test]
fn starts_log_item_text_matches_graph_revision_rows() {
    assert!(starts_log_item_text("@ working copy"));
    assert!(starts_log_item_text("○ change"));
    assert!(starts_log_item_text("◆ hidden"));
    assert!(!starts_log_item_text("│ continuation"));
    assert!(!starts_log_item_text("~"));
}

#[test]
fn group_lines_pairs_revision_metadata_by_start_rows() {
    let items = group_lines(
        vec![
            Line::from("@ first".to_owned()),
            Line::from("│ details".to_owned()),
            Line::from("○ second".to_owned()),
        ],
        RowMetadata::Valid(vec![
            revision_metadata(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "0123456789abcdef0123456789abcdef01234567",
            ),
            revision_metadata(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "fedcba9876543210fedcba9876543210fedcba98",
            ),
        ]),
    );

    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0].change_id(),
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert_eq!(
        items[0].commit_id(),
        Some("0123456789abcdef0123456789abcdef01234567")
    );
    assert_eq!(log_item_text(&items[0]), "@ first\n│ details");
    assert_eq!(
        items[1].change_id(),
        Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
}

#[test]
fn group_lines_drops_drifted_metadata() {
    let items = group_lines(
        vec![
            Line::from("@ first".to_owned()),
            Line::from("○ second".to_owned()),
        ],
        RowMetadata::Drifted,
    );

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].change_id(), None);
    assert_eq!(items[1].commit_id(), None);
}
