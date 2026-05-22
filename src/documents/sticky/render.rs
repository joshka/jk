use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::documents::{DocumentLines, PinnedDocument};
use crate::jj::{ColorMode, ViewSpec, run_jj};
use crate::search::{SearchQuery, highlight_line, line_matches};

use super::viewport::{DocumentDisplayMode, DocumentViewport, max_projected_line_width};

pub fn load_document(spec: &ViewSpec) -> Result<DocumentLines> {
    let output = run_jj(spec, ColorMode::Always)?;

    Ok(DocumentLines::new(output.stdout.into_text()?.lines))
}

pub fn render_document(
    frame: &mut Frame<'_>,
    area: Rect,
    document: PinnedDocument,
    search: Option<&SearchQuery>,
) {
    render_document_with_viewport(frame, area, document, DocumentViewport::default(), search);
}

pub fn render_document_with_viewport(
    frame: &mut Frame<'_>,
    area: Rect,
    document: PinnedDocument,
    mut viewport: DocumentViewport,
    search: Option<&SearchQuery>,
) {
    let fixed_lines = highlight_lines(document.fixed_lines().to_vec(), search);
    let body_lines = highlight_lines(document.body_lines().to_vec(), search);
    viewport.clamp(
        area.width,
        max_projected_line_width(&fixed_lines, &body_lines),
    );
    let [fixed_area, body_area] =
        document_areas(area, fixed_lines.len().min(u16::MAX as usize) as u16);
    if !fixed_lines.is_empty() {
        frame.render_widget(document_widget(fixed_lines, 0, viewport), fixed_area);
    }
    frame.render_widget(
        document_widget(body_lines, document.body_scroll_offset(), viewport),
        body_area,
    );
}

pub fn search_matches(document: &DocumentLines, query: &SearchQuery) -> usize {
    document
        .lines()
        .iter()
        .filter(|line| line_matches(line, query))
        .count()
}

pub fn next_matching_line(
    document: &DocumentLines,
    scroll_offset: usize,
    query: &SearchQuery,
) -> Option<usize> {
    ((scroll_offset + 1)..document.line_count())
        .chain(0..scroll_offset.min(document.line_count()))
        .find(|index| line_matches(&document.lines()[*index], query))
}

pub fn previous_matching_line(
    document: &DocumentLines,
    scroll_offset: usize,
    query: &SearchQuery,
) -> Option<usize> {
    (0..scroll_offset)
        .rev()
        .chain(((scroll_offset + 1)..document.line_count()).rev())
        .find(|index| line_matches(&document.lines()[*index], query))
}

pub fn lines_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

pub(super) fn max_line_width(lines: &[Line<'_>]) -> usize {
    lines
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or_default()
}

pub(super) fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn highlight_lines(lines: Vec<Line<'static>>, search: Option<&SearchQuery>) -> Vec<Line<'static>> {
    lines
        .into_iter()
        .map(|line| highlight_line(line, search))
        .collect()
}

fn document_widget(
    lines: Vec<Line<'static>>,
    scroll_offset: usize,
    viewport: DocumentViewport,
) -> Paragraph<'static> {
    let paragraph = Paragraph::new(lines).scroll((
        scroll_offset.min(u16::MAX as usize) as u16,
        viewport.horizontal_offset().min(u16::MAX as usize) as u16,
    ));

    match viewport.display_mode() {
        DocumentDisplayMode::Wrap => paragraph.wrap(Wrap { trim: false }),
        DocumentDisplayMode::NoWrap => paragraph,
    }
}

fn document_areas(area: Rect, fixed_height: u16) -> [Rect; 2] {
    let fixed_height = fixed_height.min(area.height);
    let fixed_area = Rect {
        height: fixed_height,
        ..area
    };
    let body_area = Rect {
        y: area.y + fixed_height,
        height: area.height - fixed_height,
        ..area
    };

    [fixed_area, body_area]
}
