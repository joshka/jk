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
    spec: ViewSpec,
    document: PlainDocument,
}

impl OperationDetailView {
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

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        documents::render_document(frame, area, self.projection(), search);
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

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

    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)
    }

    pub fn projection(&self) -> PinnedDocument {
        self.document.projection()
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn line_count(&self) -> usize {
        self.document.line_count()
    }

    pub fn scroll_offset(&self) -> usize {
        self.document.scroll_offset()
    }

    pub fn set_scroll_offset(&mut self, _viewport_height: u16, scroll_offset: usize) {
        self.document.set_scroll_offset(scroll_offset);
    }

    pub fn scroll_to_top(&mut self) {
        self.document.scroll_to_top();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.document.scroll_to_bottom();
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.document.scroll_down(amount);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.document.scroll_up(amount);
    }

    pub fn clamp(&mut self, _viewport_height: u16) {
        self.document.clamp();
    }

    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.document.search_matches(query)
    }

    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
        self.document.next_match(query)
    }

    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
        self.document.previous_match(query)
    }

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

    fn operation_id(&self) -> Option<String> {
        self.spec.target().map(str::to_owned)
    }
}

struct PlainDocument {
    lines: DocumentLines,
    scroll_offset: usize,
}

impl PlainDocument {
    fn load(spec: &ViewSpec) -> Result<Self> {
        let lines = documents::load_document(spec)?;
        Ok(Self::new(lines))
    }

    fn new(lines: DocumentLines) -> Self {
        Self {
            lines,
            scroll_offset: 0,
        }
    }

    fn refresh(&mut self, spec: &ViewSpec) -> Result<()> {
        self.lines = documents::load_document(spec)?;
        self.clamp();
        Ok(())
    }

    fn projection(&self) -> PinnedDocument {
        let anchors: &[FileAnchor] = &[];
        project_with_active_file(&self.lines, anchors, self.scroll_offset, [])
    }

    fn line_count(&self) -> usize {
        self.lines.line_count()
    }

    fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    fn set_scroll_offset(&mut self, scroll_offset: usize) {
        self.scroll_offset = scroll_offset.min(self.max_scroll_offset());
    }

    fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    fn scroll_down(&mut self, amount: usize) {
        self.set_scroll_offset(self.scroll_offset.saturating_add(amount));
    }

    fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    fn clamp(&mut self) {
        self.set_scroll_offset(self.scroll_offset);
    }

    fn search_matches(&self, query: &SearchQuery) -> usize {
        documents::search_matches(&self.lines, query)
    }

    fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(offset) = documents::next_matching_line(&self.lines, self.scroll_offset, query)
        else {
            return false;
        };
        self.set_scroll_offset(offset);
        true
    }

    fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(offset) =
            documents::previous_matching_line(&self.lines, self.scroll_offset, query)
        else {
            return false;
        };
        self.set_scroll_offset(offset);
        true
    }

    fn max_scroll_offset(&self) -> usize {
        self.line_count().saturating_sub(1)
    }
}

#[cfg(test)]
mod tests;
