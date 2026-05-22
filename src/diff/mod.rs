//! `jj diff` view state, rendering, and navigation.
//!
//! Diff only pins the active file heading. Unlike show, it has no commit
//! context prefix because the command output is already focused on the patch.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::documents;
use crate::documents::PinnedDocument;
use crate::documents::StickyFileDocument;
use crate::jj::{JjCommand, ViewSpec};
use crate::search::SearchQuery;

const TOGGLE_WRAP_KEYS: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('w')];
const SCROLL_LEFT_KEYS: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('h')];
const SCROLL_RIGHT_KEYS: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('l')];
const HORIZONTAL_SCROLL_AMOUNT: usize = 1;

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
    Binding::sequence(TOGGLE_WRAP_KEYS, Command::View(ViewCommand::ToggleWrap)),
    Binding::sequence(SCROLL_LEFT_KEYS, Command::View(ViewCommand::ScrollLeft)),
    Binding::sequence(SCROLL_RIGHT_KEYS, Command::View(ViewCommand::ScrollRight)),
    Binding::new(KeyPattern::char(']'), Command::View(ViewCommand::NextFile)),
    Binding::new(
        KeyPattern::char('['),
        Command::View(ViewCommand::PreviousFile),
    ),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenFiles)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenFiles),
    ),
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
];

/// Rendered `jj diff` output plus sticky file context and scroll state.
pub struct DiffView {
    /// View identity and revset navigation target for refresh and drill-down commands.
    spec: ViewSpec,
    /// Shared sticky document state for rendered lines, viewport, and file navigation.
    document: StickyFileDocument,
}

impl DiffView {
    #[cfg(test)]
    pub(crate) fn test_new(spec: ViewSpec) -> Self {
        Self {
            spec,
            document: StickyFileDocument::new(crate::documents::DocumentLines::new(Vec::new())),
        }
    }

    /// Loads rendered `jj diff` output into the shared sticky document model.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let document = StickyFileDocument::load(&spec)?;
        Ok(Self { spec, document })
    }

    /// Renders the current sticky document projection into the active viewport.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        documents::render_document_with_viewport(
            frame,
            area,
            self.projection(),
            self.document.viewport(),
            search,
        );
    }

    /// Returns the key bindings owned by the diff view.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Applies a view command to diff-specific navigation, search, and drill-down state.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode => ViewEffect::Ignored,
            ViewCommand::NewTrunk => ViewEffect::Ignored,
            ViewCommand::ToggleWrap => {
                self.toggle_wrap(context.viewport_width);
                ViewEffect::Handled
            }
            ViewCommand::ScrollLeft => {
                self.scroll_left(HORIZONTAL_SCROLL_AMOUNT);
                ViewEffect::Handled
            }
            ViewCommand::ScrollRight => {
                self.scroll_right(context.viewport_width, HORIZONTAL_SCROLL_AMOUNT);
                ViewEffect::Handled
            }
            ViewCommand::MoveDown => {
                self.scroll_down(context.viewport_height, 1);
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.scroll_up(context.viewport_height, 1);
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.scroll_down(context.viewport_height, context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.scroll_up(context.viewport_height, context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.scroll_to_top();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.scroll_to_bottom(context.viewport_height);
                ViewEffect::Handled
            }
            ViewCommand::NextFile => {
                self.next_file();
                ViewEffect::Handled
            }
            ViewCommand::PreviousFile => {
                self.previous_file();
                ViewEffect::Handled
            }
            ViewCommand::OpenFiles => {
                let spec = ViewSpec::file_list(
                    self.spec.navigation_revset(),
                    self.document.current_file_label().map(str::to_owned),
                );
                let spec = if self.spec.has_exact_change_target() {
                    spec.with_exact_change_target()
                } else {
                    spec
                };
                ViewEffect::OpenView(spec)
            }
            ViewCommand::OpenShow => self
                .spec
                .navigation_revset()
                .map(|revset| ViewEffect::OpenDetail(JjCommand::Show, revset))
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::StartSearch => {
                let Some(query) = context.search else {
                    return ViewEffect::Ignored;
                };
                let matches = self.search_matches(query);
                if matches > 0 {
                    let _ = self.next_match(context.viewport_height, query);
                }
                ViewEffect::SearchStarted { matches }
            }
            ViewCommand::NextSearchMatch => context
                .search
                .filter(|query| self.next_match(context.viewport_height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::PreviousSearchMatch => context
                .search
                .filter(|query| self.previous_match(context.viewport_height, query))
                .map(|_| ViewEffect::SearchMoved)
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::ToggleSelect | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
            ViewCommand::OpenItem | ViewCommand::OpenDiff => ViewEffect::Ignored,
        }
    }

    /// Reloads the rendered diff body while preserving the same view identity.
    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)?;
        Ok(())
    }

    /// Returns the rendered projection with sticky file context applied.
    pub fn projection(&self) -> PinnedDocument {
        self.document.projection([])
    }

    /// Returns the view spec that identifies this diff surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the rendered line count of the underlying diff body.
    pub fn line_count(&self) -> usize {
        self.document.line_count()
    }

    /// Returns the current vertical scroll offset in rendered lines.
    pub fn scroll_offset(&self) -> usize {
        self.document.scroll_offset()
    }

    #[cfg(test)]
    pub fn horizontal_offset(&self) -> usize {
        self.document.horizontal_offset()
    }

    /// Restores a saved vertical scroll position, clamped to the current viewport.
    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        self.document
            .set_scroll_offset(viewport_height, scroll_offset);
    }

    /// Moves the viewport to the first rendered line.
    pub fn scroll_to_top(&mut self) {
        self.document.scroll_to_top();
    }

    /// Moves the viewport to the last reachable rendered line.
    pub fn scroll_to_bottom(&mut self, viewport_height: u16) {
        self.document.scroll_to_bottom(viewport_height, Vec::new);
    }

    /// Scrolls down by `amount` rendered lines while preserving sticky projection rules.
    pub fn scroll_down(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document.scroll_down(viewport_height, 1, Vec::new);
        }
    }

    /// Scrolls up by `amount` rendered lines while preserving sticky projection rules.
    pub fn scroll_up(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document.scroll_up(viewport_height, 1, Vec::new);
        }
    }

    /// Clamps vertical and horizontal offsets to the current viewport dimensions.
    pub fn clamp(&mut self, viewport_height: u16, viewport_width: u16) {
        self.document.clamp(viewport_height, viewport_width);
    }

    /// Toggles wrapped rendering for the current viewport width.
    pub fn toggle_wrap(&mut self, viewport_width: u16) {
        self.document.toggle_wrap(viewport_width);
    }

    /// Moves the horizontal offset left by `amount` columns.
    pub fn scroll_left(&mut self, amount: usize) {
        self.document.scroll_left(amount);
    }

    /// Moves the horizontal offset right by `amount` columns within the viewport width.
    pub fn scroll_right(&mut self, viewport_width: u16, amount: usize) {
        self.document.scroll_right(viewport_width, amount);
    }

    /// Counts rendered matches for the current search query.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.document.search_matches(query)
    }

    /// Advances to the next rendered search match if one exists.
    pub fn next_match(&mut self, viewport_height: u16, query: &SearchQuery) -> bool {
        self.document.next_match(viewport_height, query)
    }

    /// Moves to the previous rendered search match if one exists.
    pub fn previous_match(&mut self, viewport_height: u16, query: &SearchQuery) -> bool {
        self.document.previous_match(viewport_height, query)
    }

    /// Selects the next detected file heading in the rendered document.
    pub fn next_file(&mut self) {
        self.document.next_file();
    }

    /// Selects the previous detected file heading in the rendered document.
    pub fn previous_file(&mut self) {
        self.document.previous_file();
    }

    /// Returns copyable identifiers for the current diff surface.
    pub fn copy_options(&self) -> Vec<CopyOption> {
        let mut options = Vec::new();
        if let Some(target) = self.spec.target() {
            options.push(CopyOption::new("revset", target));
        }
        if let Some(file) = self.document.current_file_label() {
            options.push(CopyOption::new("file path", file));
        }
        options
    }
}

#[cfg(test)]
mod tests;
