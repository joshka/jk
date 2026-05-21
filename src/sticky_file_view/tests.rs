use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::text::Line;

use super::*;

#[test]
fn wrapped_rendering_preserves_existing_long_line_reflow() {
    let rendered = render_document_rows(
        12,
        4,
        pinned_document(&["- alpha beta gamma", "    indented"]),
        DocumentViewport::default(),
    );

    assert_snapshot!(rendered, @r"
- alpha beta
gamma
    indented
");
}

#[test]
fn no_wrap_rendering_clips_long_lines() {
    let mut viewport = DocumentViewport::default();
    viewport.toggle_wrap();

    let rendered = render_document_rows(
        12,
        3,
        pinned_document(&["- alpha beta gamma", "    indented"]),
        viewport,
    );

    assert_snapshot!(rendered, @r"
- alpha beta
    indented
");
}

#[test]
fn no_wrap_horizontal_offset_reveals_later_columns() {
    let mut viewport = DocumentViewport::default();
    viewport.toggle_wrap();
    viewport.scroll_right(10, 8, 20);

    let rendered =
        render_document_rows(10, 2, pinned_document(&["0123456789ABCDEFGHIJ"]), viewport);

    assert_snapshot!(rendered, @"89ABCDEFGH");
}

#[test]
fn no_wrap_keeps_body_scroll_and_sticky_heading_separate() {
    let document = DocumentLines::new(vec![
        Line::from("Modified regular file src/long_file_name.rs:"),
        Line::from("        1: 0123456789ABCDEFGHIJ"),
        Line::from("        2: abcdefghijklmnop"),
    ]);
    let anchors = document.file_anchors();
    let projection = project_with_active_file(&document, &anchors, 0, []);
    let mut viewport = DocumentViewport::default();
    viewport.toggle_wrap();
    viewport.scroll_right(12, 8, 34);

    let rendered = render_document_rows(12, 3, projection, viewport);

    assert_snapshot!(rendered, @r"
 regular fil
1: 012345678
2: abcdefghi
        ");
}

#[test]
fn sticky_document_clamps_horizontal_offset_after_content_shrinks() {
    let mut document = StickyFileDocument::new(document_lines(&["0123456789ABCDEFGHIJ", "short"]));
    document.toggle_wrap(10);
    for _ in 0..20 {
        document.scroll_right(10, 1);
    }
    assert_eq!(document.horizontal_offset(), 10);

    document.replace_lines(document_lines(&["short"]));
    document.clamp(3, 10);

    assert_eq!(document.horizontal_offset(), 0);
}

fn pinned_document(lines: &[&str]) -> PinnedDocument {
    let document = document_lines(lines);
    project_with_active_file(&document, &[], 0, [])
}

fn document_lines(lines: &[&str]) -> DocumentLines {
    DocumentLines::new(
        lines
            .iter()
            .map(|line| Line::from((*line).to_owned()))
            .collect(),
    )
}

fn render_document_rows(
    width: u16,
    height: u16,
    document: PinnedDocument,
    viewport: DocumentViewport,
) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            render_document_with_viewport(frame, frame.area(), document, viewport, None);
        })
        .unwrap();

    (0..height)
        .map(|row| row_text(terminal.backend().buffer(), row, width))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned()
}

fn row_text(buffer: &ratatui::buffer::Buffer, row: u16, width: u16) -> String {
    (0..width)
        .map(|column| buffer[(column, row)].symbol())
        .collect::<String>()
        .trim_end()
        .to_owned()
}
