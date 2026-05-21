//! Shared rendering and navigation for rendered file documents.
//!
//! Show, diff, status, and operation-detail surfaces use the same sticky
//! file-heading projection, search navigation, and file-to-file movement. This
//! module is limited to that shared document behavior.

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::jj::{ColorMode, ViewSpec, run_jj};
use crate::rendered_jj::{DocumentLines, FileAnchor, PinnedDocument, project_with_active_file};
use crate::search::{SearchQuery, highlight_line, line_matches};

/// Rendered file text plus file anchors and scroll state.
///
/// This type owns document navigation for file-oriented detail views. It
/// reloads from `jj` through `load_document`, then keeps scroll offsets clamped
/// to the rendered lines rather than to a reconstructed repository model.
pub struct StickyFileDocument {
    lines: DocumentLines,
    anchors: Vec<FileAnchor>,
    scroll: StickyScroll,
    viewport: DocumentViewport,
}

impl StickyFileDocument {
    pub fn load(spec: &ViewSpec) -> Result<Self> {
        let lines = load_document(spec)?;
        Ok(Self::new(lines))
    }

    pub fn new(lines: DocumentLines) -> Self {
        let anchors = lines.file_anchors();
        Self {
            lines,
            anchors,
            scroll: StickyScroll::default(),
            viewport: DocumentViewport::default(),
        }
    }

    pub fn refresh(&mut self, spec: &ViewSpec) -> Result<()> {
        self.replace_lines(load_document(spec)?);
        Ok(())
    }

    pub fn projection(&self, prefix: impl IntoIterator<Item = Line<'static>>) -> PinnedDocument {
        self.projection_at(self.scroll.offset(), prefix)
    }

    pub fn line_count(&self) -> usize {
        self.lines.line_count()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset()
    }

    #[cfg(test)]
    pub fn horizontal_offset(&self) -> usize {
        self.viewport.horizontal_offset()
    }

    pub fn viewport(&self) -> DocumentViewport {
        self.viewport
    }

    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        self.scroll.set(scroll_offset, self.max_scroll_offset());
        self.clamp(viewport_height, u16::MAX);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll.move_to_top();
    }

    pub fn scroll_to_bottom(
        &mut self,
        viewport_height: u16,
        prefix: impl Fn() -> Vec<Line<'static>>,
    ) {
        let max_offset = self.max_scroll_offset();
        let lines = &self.lines;
        let anchors = &self.anchors;
        self.scroll
            .move_to_bottom(max_offset, viewport_height, |offset| {
                project_with_active_file(lines, anchors, offset, prefix())
            });
    }

    pub fn scroll_down(
        &mut self,
        viewport_height: u16,
        amount: usize,
        prefix: impl Fn() -> Vec<Line<'static>>,
    ) {
        let max_offset = self.max_scroll_offset();
        let lines = &self.lines;
        let anchors = &self.anchors;
        self.scroll
            .down(amount, max_offset, viewport_height, |offset| {
                project_with_active_file(lines, anchors, offset, prefix())
            });
    }

    pub fn scroll_up(
        &mut self,
        viewport_height: u16,
        amount: usize,
        prefix: impl Fn() -> Vec<Line<'static>>,
    ) {
        let lines = &self.lines;
        let anchors = &self.anchors;
        self.scroll.up(amount, viewport_height, |offset| {
            project_with_active_file(lines, anchors, offset, prefix())
        });
    }

    pub fn clamp(&mut self, _viewport_height: u16, viewport_width: u16) {
        self.scroll.clamp(self.max_scroll_offset());
        self.viewport.clamp(viewport_width, self.max_line_width());
    }

    pub fn toggle_wrap(&mut self, viewport_width: u16) {
        self.viewport.toggle_wrap();
        self.viewport.clamp(viewport_width, self.max_line_width());
    }

    pub fn scroll_left(&mut self, amount: usize) {
        self.viewport.scroll_left(amount);
    }

    pub fn scroll_right(&mut self, viewport_width: u16, amount: usize) {
        self.viewport
            .scroll_right(viewport_width, amount, self.max_line_width());
    }

    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        search_matches(&self.lines, query)
    }

    pub fn next_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = next_matching_line(&self.lines, self.scroll.offset(), query) else {
            return false;
        };
        self.scroll.set(offset, self.max_scroll_offset());
        self.scroll.clamp(self.max_scroll_offset());
        true
    }

    pub fn previous_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = previous_matching_line(&self.lines, self.scroll.offset(), query) else {
            return false;
        };
        self.scroll.set(offset, self.max_scroll_offset());
        self.scroll.clamp(self.max_scroll_offset());
        true
    }

    pub fn next_file(&mut self) {
        if let Some(line_index) = next_file_offset(&self.lines, &self.anchors, self.scroll.offset())
        {
            self.scroll.set(line_index, self.max_scroll_offset());
        }
    }

    pub fn previous_file(&mut self) {
        if let Some(line_index) =
            previous_file_offset(&self.lines, &self.anchors, self.scroll.offset())
        {
            self.scroll.set(line_index, self.max_scroll_offset());
        }
    }

    pub fn current_file_label(&self) -> Option<&str> {
        current_file_label(&self.lines, &self.anchors, self.scroll.offset())
    }

    fn max_scroll_offset(&self) -> usize {
        self.line_count().saturating_sub(1)
    }

    fn max_line_width(&self) -> usize {
        max_line_width(self.lines.lines())
    }

    fn replace_lines(&mut self, lines: DocumentLines) {
        self.lines = lines;
        self.anchors = self.lines.file_anchors();
    }

    fn projection_at(
        &self,
        scroll_offset: usize,
        prefix: impl IntoIterator<Item = Line<'static>>,
    ) -> PinnedDocument {
        project_with_active_file(&self.lines, &self.anchors, scroll_offset, prefix)
    }
}

/// Display policy for rendered jj document text.
///
/// Wrapped mode preserves the original behavior: Ratatui wraps long lines with
/// `trim: false`, keeping indentation and blank lines visible. No-wrap mode
/// leaves the spans intact, clips at the viewport edge, and uses a horizontal
/// offset owned by the document view state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DocumentDisplayMode {
    #[default]
    Wrap,
    NoWrap,
}

/// Viewport state for rendered document text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentViewport {
    display_mode: DocumentDisplayMode,
    horizontal_offset: usize,
}

impl DocumentViewport {
    pub fn display_mode(self) -> DocumentDisplayMode {
        self.display_mode
    }

    pub fn horizontal_offset(self) -> usize {
        match self.display_mode {
            DocumentDisplayMode::Wrap => 0,
            DocumentDisplayMode::NoWrap => self.horizontal_offset,
        }
    }

    pub fn toggle_wrap(&mut self) {
        self.display_mode = match self.display_mode {
            DocumentDisplayMode::Wrap => DocumentDisplayMode::NoWrap,
            DocumentDisplayMode::NoWrap => {
                self.horizontal_offset = 0;
                DocumentDisplayMode::Wrap
            }
        };
    }

    pub fn scroll_left(&mut self, amount: usize) {
        if self.display_mode == DocumentDisplayMode::NoWrap {
            self.horizontal_offset = self.horizontal_offset.saturating_sub(amount);
        }
    }

    pub fn scroll_right(&mut self, viewport_width: u16, amount: usize, max_line_width: usize) {
        if self.display_mode == DocumentDisplayMode::NoWrap {
            self.horizontal_offset = self
                .horizontal_offset
                .saturating_add(amount)
                .min(max_horizontal_offset(viewport_width, max_line_width));
        }
    }

    pub fn clamp(&mut self, viewport_width: u16, max_line_width: usize) {
        match self.display_mode {
            DocumentDisplayMode::Wrap => self.horizontal_offset = 0,
            DocumentDisplayMode::NoWrap => {
                self.horizontal_offset = self
                    .horizontal_offset
                    .min(max_horizontal_offset(viewport_width, max_line_width));
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StickyScroll {
    offset: usize,
}

impl StickyScroll {
    fn offset(self) -> usize {
        self.offset
    }

    fn set(&mut self, offset: usize, max_offset: usize) {
        self.offset = offset.min(max_offset);
    }

    fn move_to_top(&mut self) {
        self.offset = 0;
    }

    fn move_to_bottom(
        &mut self,
        max_offset: usize,
        viewport_height: u16,
        project: impl Fn(usize) -> PinnedDocument,
    ) {
        self.offset = previous_meaningful_offset(max_offset, viewport_height, project);
    }

    fn down(
        &mut self,
        amount: usize,
        max_offset: usize,
        viewport_height: u16,
        project: impl Fn(usize) -> PinnedDocument,
    ) {
        for _ in 0..amount {
            self.offset =
                next_meaningful_offset(self.offset, max_offset, viewport_height, &project);
        }
        self.clamp(max_offset);
    }

    fn up(
        &mut self,
        amount: usize,
        viewport_height: u16,
        project: impl Fn(usize) -> PinnedDocument,
    ) {
        for _ in 0..amount {
            self.offset = previous_meaningful_offset(self.offset, viewport_height, &project);
        }
    }

    fn clamp(&mut self, max_offset: usize) {
        self.offset = self.offset.min(max_offset);
    }
}

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

pub fn next_meaningful_offset(
    current_offset: usize,
    max_offset: usize,
    viewport_height: u16,
    project: impl Fn(usize) -> PinnedDocument,
) -> usize {
    // Skip offsets that render the same visible projection. Otherwise a key can
    // mutate hidden scroll state while the terminal appears unchanged.
    let current_key = projection_key(&project(current_offset), viewport_height);
    ((current_offset + 1)..=max_offset)
        .find(|offset| projection_key(&project(*offset), viewport_height) != current_key)
        .unwrap_or(max_offset)
}

pub fn previous_meaningful_offset(
    current_offset: usize,
    viewport_height: u16,
    project: impl Fn(usize) -> PinnedDocument,
) -> usize {
    let current_key = projection_key(&project(current_offset), viewport_height);
    (0..current_offset)
        .rev()
        .find(|offset| projection_key(&project(*offset), viewport_height) != current_key)
        .unwrap_or(0)
}

pub fn next_file_offset(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
) -> Option<usize> {
    let current = current_file_index(document, anchors, scroll_offset);
    let next_index = current.map_or(0, |index| index.saturating_add(1));
    anchors
        .get(next_index)
        .map(|anchor| file_activation_offset(document, anchor))
}

pub fn previous_file_offset(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
) -> Option<usize> {
    let current = current_file_index(document, anchors, scroll_offset)?;
    current
        .checked_sub(1)
        .and_then(|index| anchors.get(index))
        .map(|anchor| file_activation_offset(document, anchor))
}

pub fn current_file_label<'a>(
    document: &DocumentLines,
    anchors: &'a [FileAnchor],
    scroll_offset: usize,
) -> Option<&'a str> {
    current_file_index(document, anchors, scroll_offset)
        .and_then(|index| anchors.get(index))
        .map(FileAnchor::label)
}

pub fn lines_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
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

fn projection_key(document: &PinnedDocument, viewport_height: u16) -> Vec<String> {
    let body_height = viewport_height.saturating_sub(document.sticky_height()) as usize;
    document
        .fixed_lines()
        .iter()
        .chain(
            document
                .body_lines()
                .iter()
                .skip(document.body_scroll_offset())
                .take(body_height),
        )
        .map(line_text)
        .collect()
}

fn max_line_width(lines: &[Line<'_>]) -> usize {
    lines
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or_default()
}

fn max_projected_line_width(fixed_lines: &[Line<'_>], body_lines: &[Line<'_>]) -> usize {
    fixed_lines
        .iter()
        .chain(body_lines)
        .map(|line| line.width())
        .max()
        .unwrap_or_default()
}

fn max_horizontal_offset(viewport_width: u16, max_line_width: usize) -> usize {
    max_line_width.saturating_sub(usize::from(viewport_width))
}

fn current_file_index(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
) -> Option<usize> {
    anchors
        .iter()
        .enumerate()
        .take_while(|(_, anchor)| file_activation_offset(document, anchor) <= scroll_offset)
        .last()
        .map(|(index, _)| index)
}

fn file_activation_offset(document: &DocumentLines, anchor: &FileAnchor) -> usize {
    let previous_line = anchor.line_index().saturating_sub(1);
    if anchor.line_index() > 0 && document.line_is_blank(previous_line) {
        previous_line
    } else {
        anchor.line_index()
    }
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[cfg(test)]
mod tests;
