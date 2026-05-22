//! Rendered `jj operation show` and `jj operation diff` detail views.
//!
//! Operation detail output is treated as a plain rendered document. The view
//! preserves jj's styled lines and supports the same scroll/search/copy basics
//! as other document views without interpreting transaction semantics or
//! applying file-heading stickiness to operation output.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::documents;
use crate::documents::{DocumentLines, FileAnchor, PinnedDocument, project_with_active_file};
use crate::jj::{JjCommand, ViewSpec};
use crate::rendered_rows::document_plain_text;
use crate::search::SearchQuery;

pub const BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Down),
        Command::View(ViewCommand::MoveDown),
    ),
    Binding::new(KeyPattern::char('k'), Command::View(ViewCommand::MoveUp)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Up),
        Command::View(ViewCommand::MoveUp),
    ),
    Binding::new(KeyPattern::char(' '), Command::View(ViewCommand::PageDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageDown),
        Command::View(ViewCommand::PageDown),
    ),
    Binding::new(
        KeyPattern::modified_char('f', crossterm::event::KeyModifiers::CONTROL),
        Command::View(ViewCommand::PageDown),
    ),
    Binding::new(
        KeyPattern::modified_char(' ', crossterm::event::KeyModifiers::SHIFT),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::PageUp),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(
        KeyPattern::modified_char('b', crossterm::event::KeyModifiers::CONTROL),
        Command::View(ViewCommand::PageUp),
    ),
    Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Home),
        Command::View(ViewCommand::MoveFirst),
    ),
    Binding::new(KeyPattern::char('G'), Command::View(ViewCommand::MoveLast)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::End),
        Command::View(ViewCommand::MoveLast),
    ),
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
    Binding::new(KeyPattern::char('d'), Command::View(ViewCommand::OpenDiff)),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Rendered operation detail output plus plain document scroll state.
pub struct OperationDetailView {
    /// View identity and selected operation target for refresh and view switching.
    spec: ViewSpec,
    /// Plain rendered document state used for operation output without sticky file headings.
    document: PlainDocument,
}

impl OperationDetailView {
    /// Loads rendered operation detail output for one operation-targeted view spec.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let document = PlainDocument::load(&spec)?;
        Ok(Self { spec, document })
    }

    #[cfg(test)]
    pub(crate) fn test_new(spec: ViewSpec, lines: DocumentLines) -> Self {
        Self {
            spec,
            document: PlainDocument::new(lines),
        }
    }

    /// Renders the current plain-document projection into the active viewport.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        documents::render_document(frame, area, self.projection(), search);
    }

    /// Returns the key bindings owned by the operation detail view.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Applies document navigation, search, copy, and show/diff switching commands.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem => ViewEffect::Ignored,
            ViewCommand::MoveDown => {
                self.scroll_down(1);
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.scroll_up(1);
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.scroll_down(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.scroll_up(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.scroll_to_top();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.scroll_to_bottom();
                ViewEffect::Handled
            }
            ViewCommand::OpenShow => self
                .operation_id()
                .filter(|_| self.spec.command() != JjCommand::OperationShow)
                .map(ViewSpec::operation_show)
                .map(ViewEffect::OpenView)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::OpenDiff => self
                .operation_id()
                .filter(|_| self.spec.command() != JjCommand::OperationDiff)
                .map(ViewSpec::operation_diff)
                .map(ViewEffect::OpenView)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::ToggleSelect | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
        }
    }

    /// Reloads the rendered operation detail document for the current target.
    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)
    }

    /// Returns the plain rendered projection for the current scroll offset.
    pub fn projection(&self) -> PinnedDocument {
        self.document.projection()
    }

    /// Returns the view spec that identifies this operation-detail surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the rendered line count of the current detail document.
    pub fn line_count(&self) -> usize {
        self.document.line_count()
    }

    /// Returns the current vertical scroll offset in rendered lines.
    pub fn scroll_offset(&self) -> usize {
        self.document.scroll_offset()
    }

    /// Restores a saved vertical scroll position for the plain document.
    pub fn set_scroll_offset(&mut self, _viewport_height: u16, scroll_offset: usize) {
        self.document.set_scroll_offset(scroll_offset);
    }

    /// Moves the viewport to the first rendered line.
    pub fn scroll_to_top(&mut self) {
        self.document.scroll_to_top();
    }

    /// Moves the viewport to the last rendered line.
    pub fn scroll_to_bottom(&mut self) {
        self.document.scroll_to_bottom();
    }

    /// Scrolls down by `amount` rendered lines.
    pub fn scroll_down(&mut self, amount: usize) {
        self.document.scroll_down(amount);
    }

    /// Scrolls up by `amount` rendered lines.
    pub fn scroll_up(&mut self, amount: usize) {
        self.document.scroll_up(amount);
    }

    /// Clamps the scroll offset to the current document length.
    pub fn clamp(&mut self, _viewport_height: u16) {
        self.document.clamp();
    }

    /// Counts rendered matches for the current search query.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.document.search_matches(query)
    }

    /// Advances to the next rendered search match if one exists.
    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
        self.document.next_match(query)
    }

    /// Moves to the previous rendered search match if one exists.
    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
        self.document.previous_match(query)
    }

    /// Returns copyable identifiers and visible operation detail text.
    pub fn copy_options(&self) -> Vec<CopyOption> {
        let mut options = Vec::new();
        if let Some(operation_id) = self.operation_id() {
            options.push(CopyOption::new("operation id", operation_id));
        }
        let text = document_plain_text(self.projection().body_lines());
        if !text.is_empty() {
            options.push(CopyOption::new("operation detail text", text));
        }
        options
    }

    /// Returns the exact operation id targeted by this detail surface, if present.
    fn operation_id(&self) -> Option<String> {
        self.spec.target().map(str::to_owned)
    }
}

struct PlainDocument {
    /// Preserved rendered operation detail lines from `jj operation show` or `diff`.
    lines: DocumentLines,
    /// Current top-of-viewport offset within the preserved rendered lines.
    scroll_offset: usize,
}

impl PlainDocument {
    /// Loads one plain rendered document from the `jj` boundary.
    fn load(spec: &ViewSpec) -> Result<Self> {
        let lines = documents::load_document(spec)?;
        Ok(Self::new(lines))
    }

    /// Builds scroll state around already-rendered document lines.
    fn new(lines: DocumentLines) -> Self {
        Self {
            lines,
            scroll_offset: 0,
        }
    }

    /// Reloads the rendered lines and clamps the old scroll position to the new size.
    fn refresh(&mut self, spec: &ViewSpec) -> Result<()> {
        self.lines = documents::load_document(spec)?;
        self.clamp();
        Ok(())
    }

    /// Projects the plain document without any sticky-file anchors.
    fn projection(&self) -> PinnedDocument {
        let anchors: &[FileAnchor] = &[];
        project_with_active_file(&self.lines, anchors, self.scroll_offset, [])
    }

    /// Returns the number of rendered lines in the current document.
    fn line_count(&self) -> usize {
        self.lines.line_count()
    }

    /// Returns the current vertical scroll offset.
    fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Sets the scroll offset, clamped to the last reachable line.
    fn set_scroll_offset(&mut self, scroll_offset: usize) {
        self.scroll_offset = scroll_offset.min(self.max_scroll_offset());
    }

    /// Moves the viewport to the first rendered line.
    fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Moves the viewport to the last rendered line.
    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    /// Scrolls down by `amount` lines.
    fn scroll_down(&mut self, amount: usize) {
        self.set_scroll_offset(self.scroll_offset.saturating_add(amount));
    }

    /// Scrolls up by `amount` lines.
    fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// Reapplies the current offset through clamping after document changes.
    fn clamp(&mut self) {
        self.set_scroll_offset(self.scroll_offset);
    }

    /// Counts rendered matches for the current query.
    fn search_matches(&self, query: &SearchQuery) -> usize {
        documents::search_matches(&self.lines, query)
    }

    /// Advances to the next rendered search match if one exists.
    fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(offset) = documents::next_matching_line(&self.lines, self.scroll_offset, query)
        else {
            return false;
        };
        self.set_scroll_offset(offset);
        true
    }

    /// Moves to the previous rendered search match if one exists.
    fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(offset) =
            documents::previous_matching_line(&self.lines, self.scroll_offset, query)
        else {
            return false;
        };
        self.set_scroll_offset(offset);
        true
    }

    /// Returns the last reachable vertical offset for the current document size.
    fn max_scroll_offset(&self) -> usize {
        self.line_count().saturating_sub(1)
    }
}

#[cfg(test)]
mod tests;
