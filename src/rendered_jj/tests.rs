use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui_macros::line;

use super::*;

#[test]
fn extracts_file_headings() {
    let document = DocumentLines::new(vec![
        line!("Commit ID: abc"),
        line!("Added regular file Cargo.toml:"),
        line!("        1: [package]"),
        line!("Modified regular file src/main.rs:"),
    ]);

    let anchors = document.file_anchors();

    assert_eq!(anchors.len(), 2);
    assert_eq!(anchors[0].line_index(), 1);
    assert_eq!(
        line_text(&anchors[0].heading()),
        "Added regular file Cargo.toml:"
    );
    assert_eq!(anchors[0].label(), "Cargo.toml");
    assert_eq!(anchors[1].line_index(), 3);
    assert_eq!(anchors[1].label(), "src/main.rs");
}

#[test]
fn extracts_git_file_headings_as_clean_paths() {
    let document = DocumentLines::new(vec![
        line!("diff --git a/src/main.rs b/src/main.rs"),
        line!("index 0000000..1111111"),
        line!("--- a/src/main.rs"),
        line!("+++ b/src/main.rs"),
        line!("@@ -1 +1 @@"),
    ]);

    let anchors = document.file_anchors();

    assert_eq!(anchors.len(), 1);
    assert_eq!(anchors[0].line_index(), 0);
    assert_eq!(anchors[0].label(), "src/main.rs");
    assert_eq!(line_text(&anchors[0].heading()), "src/main.rs");
}

#[test]
fn default_file_heading_labels_remove_status_and_file_kind() {
    assert_eq!(
        default_file_label("Added regular file Cargo.toml:").as_deref(),
        Some("Cargo.toml")
    );
    assert_eq!(
        default_file_label("Modified executable file scripts/run:").as_deref(),
        Some("scripts/run")
    );
    assert_eq!(
        default_file_label("Removed symlink docs/current:").as_deref(),
        Some("docs/current")
    );
    assert_eq!(
        default_file_label("Renamed regular file src/old.rs => src/new.rs:").as_deref(),
        Some("src/new.rs")
    );
}

#[test]
fn default_file_heading_retains_source_style_when_pinned() {
    let style = Style::default().fg(Color::Green);
    let document = DocumentLines::new(vec![
        Line::from(Span::styled("Added regular file Cargo.toml:", style)),
        line!("        1: [package]"),
    ]);
    let anchors = document.file_anchors();

    assert_eq!(
        line_text(&anchors[0].heading()),
        "Added regular file Cargo.toml:"
    );
    assert_eq!(anchors[0].heading().spans[0].style, style);
}

#[test]
fn git_file_heading_retains_source_path_style_when_pinned() {
    let style = Style::default().fg(Color::Yellow);
    let document = DocumentLines::new(vec![Line::from(vec![
        Span::raw("diff --git a/src/main.rs "),
        Span::styled("b/src/main.rs", style),
    ])]);
    let anchors = document.file_anchors();

    assert_eq!(line_text(&anchors[0].heading()), "src/main.rs");
    assert_eq!(anchors[0].heading().spans[0].style, style);
}

#[test]
fn active_file_is_nearest_heading_at_or_before_scroll() {
    let anchors = vec![
        FileAnchor {
            line_index: 2,
            heading: line!("Added regular file Cargo.toml:"),
            label: "Added regular file Cargo.toml:".to_owned(),
        },
        FileAnchor {
            line_index: 5,
            heading: line!("Modified regular file src/main.rs:"),
            label: "Modified regular file src/main.rs:".to_owned(),
        },
    ];

    assert!(active_file(&anchors, 1).is_none());
    assert_eq!(active_file(&anchors, 2).unwrap().line_index(), 2);
    assert_eq!(active_file(&anchors, 4).unwrap().line_index(), 2);
    assert_eq!(active_file(&anchors, 5).unwrap().line_index(), 5);
}

#[test]
fn projection_pins_active_file_without_duplication() {
    let document = DocumentLines::new(vec![
        line!("Commit ID: abc"),
        line!("Added regular file Cargo.toml:"),
        line!("        1: [package]"),
    ]);
    let anchors = document.file_anchors();

    let projection = project_with_active_file(&document, &anchors, 1, [line!("@  abc")]);

    assert_eq!(projection.fixed_lines().len(), 3);
    assert_eq!(projection.body_scroll_offset(), 0);
    assert_eq!(projection.body_lines().len(), 1);
    assert!(
        projection
            .body_lines()
            .iter()
            .all(|line| line_text(line) != "Added regular file Cargo.toml:")
    );
}

#[test]
fn projection_scroll_transitions_do_not_reattach_prior_context() {
    let document = DocumentLines::new(vec![
        line!("Commit ID: abc"),
        line!("Change ID: def"),
        line!(""),
        line!("    subject"),
        line!(""),
        line!("Added regular file .gitignore:"),
        line!("        1: /target"),
        line!("Added regular file Cargo.toml:"),
        line!("        1: [package]"),
        line!("        2: name = \"jk\""),
    ]);
    let anchors = document.file_anchors();

    insta::assert_snapshot!(scroll_transitions(&document, &anchors, 0..9), @r#"
    == scroll 0 ==
    [body @0]
    Commit ID: abc
    Change ID: def
    
        subject
    
    Added regular file .gitignore:
            1: /target
    Added regular file Cargo.toml:
            1: [package]
            2: name = "jk"

    == scroll 1 ==
    [body @1]
    Commit ID: abc
    Change ID: def
    
        subject
    
    Added regular file .gitignore:
            1: /target
    Added regular file Cargo.toml:
            1: [package]
            2: name = "jk"

    == scroll 2 ==
    [body @2]
    Commit ID: abc
    Change ID: def
    
        subject
    
    Added regular file .gitignore:
            1: /target
    Added regular file Cargo.toml:
            1: [package]
            2: name = "jk"

    == scroll 3 ==
    [body @3]
    Commit ID: abc
    Change ID: def
    
        subject
    
    Added regular file .gitignore:
            1: /target
    Added regular file Cargo.toml:
            1: [package]
            2: name = "jk"

    == scroll 4 ==
    [fixed]
    @  abc
    │  subject
    
    Added regular file .gitignore:
    [body @0]
            1: /target
    Added regular file Cargo.toml:
            1: [package]
            2: name = "jk"

    == scroll 5 ==
    [fixed]
    @  abc
    │  subject
    
    Added regular file .gitignore:
    [body @0]
            1: /target
    Added regular file Cargo.toml:
            1: [package]
            2: name = "jk"

    == scroll 6 ==
    [fixed]
    @  abc
    │  subject
    
    Added regular file .gitignore:
    [body @0]
            1: /target
    Added regular file Cargo.toml:
            1: [package]
            2: name = "jk"

    == scroll 7 ==
    [fixed]
    @  abc
    │  subject
    
    Added regular file Cargo.toml:
    [body @0]
            1: [package]
            2: name = "jk"

    == scroll 8 ==
    [fixed]
    @  abc
    │  subject
    
    Added regular file Cargo.toml:
    [body @0]
            1: [package]
            2: name = "jk"
    "#);
}

#[test]
fn projection_is_plain_document_before_first_file() {
    let document = DocumentLines::new(vec![
        line!("Commit ID: abc"),
        line!("Added regular file Cargo.toml:"),
    ]);
    let anchors = document.file_anchors();

    let projection = project_with_active_file(&document, &anchors, 0, [line!("@  abc")]);

    assert!(projection.fixed_lines().is_empty());
    assert_eq!(projection.body_lines().len(), 2);
    assert_eq!(projection.body_scroll_offset(), 0);
}

fn scroll_transitions(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    offsets: std::ops::Range<usize>,
) -> String {
    offsets
        .map(|offset| {
            let projection = project_with_active_file(
                document,
                anchors,
                offset,
                [line!("@  abc"), line!("│  subject")],
            );
            format_projection(offset, projection)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_projection(offset: usize, projection: PinnedDocument) -> String {
    let mut output = format!("== scroll {offset} ==\n");
    if !projection.fixed_lines().is_empty() {
        output.push_str("[fixed]\n");
        output.push_str(&format_lines(projection.fixed_lines()));
    }
    output.push_str(&format!("[body @{}]\n", projection.body_scroll_offset()));
    output.push_str(&format_lines(projection.body_lines()));
    output
}

fn format_lines(lines: &[Line<'_>]) -> String {
    let mut output = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
    output.push('\n');
    output
}
