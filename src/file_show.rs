//! `jj file show` document view state, rendering, and scroll navigation.
//!
//! This is a single-file document surface. It keeps the selected exact path
//! alongside the rendered text so copy and refresh behavior can stay tied to
//! the same file without relying on displayed labels.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::ViewSpec;
use crate::rendered_jj::{DocumentLines, project_with_active_file};
use crate::search::{SearchQuery, line_matches};
use crate::sticky_file_view;
#[cfg(test)]
use crate::sticky_file_view::DocumentDisplayMode;
use crate::sticky_file_view::DocumentViewport;
use crate::sticky_file_view::{load_document, next_matching_line, previous_matching_line};

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

/// Rendered `jj file show` output plus scroll state for one exact path.
pub struct FileShowView {
    spec: ViewSpec,
    path: String,
    document: DocumentLines,
    scroll_offset: usize,
    viewport: DocumentViewport,
}

impl FileShowView {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let path = file_show_path(&spec);
        Ok(Self {
            path,
            document: load_document(&spec)?,
            spec,
            scroll_offset: 0,
            viewport: DocumentViewport::default(),
        })
    }

    #[cfg(test)]
    pub fn new(spec: ViewSpec, path: impl Into<String>, document: DocumentLines) -> Self {
        Self {
            spec,
            path: path.into(),
            document,
            scroll_offset: 0,
            viewport: DocumentViewport::default(),
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        sticky_file_view::render_document_with_viewport(
            frame,
            area,
            self.projection(),
            self.viewport,
            search,
        );
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff => ViewEffect::Ignored,
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
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_document)
    }

    pub fn projection(&self) -> crate::rendered_jj::PinnedDocument {
        project_with_active_file(&self.document, &[], self.scroll_offset, std::iter::empty())
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn line_count(&self) -> usize {
        self.document.line_count()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    #[cfg(test)]
    pub fn horizontal_offset(&self) -> usize {
        self.viewport.horizontal_offset()
    }

    #[cfg(test)]
    pub fn display_mode(&self) -> DocumentDisplayMode {
        self.viewport.display_mode()
    }

    pub fn set_scroll_offset(&mut self, _viewport_height: u16, scroll_offset: usize) {
        self.scroll_offset = scroll_offset.min(self.max_scroll_offset());
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self, _viewport_height: u16) {
        self.scroll_offset = self.max_scroll_offset();
    }

    pub fn scroll_down(&mut self, _viewport_height: u16, amount: usize) {
        self.scroll_offset = self
            .scroll_offset
            .saturating_add(amount)
            .min(self.max_scroll_offset());
    }

    pub fn scroll_up(&mut self, _viewport_height: u16, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn clamp(&mut self, _viewport_height: u16, viewport_width: u16) {
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
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
        self.document
            .lines()
            .iter()
            .filter(|line| line_matches(line, query))
            .count()
    }

    pub fn next_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = next_matching_line(&self.document, self.scroll_offset, query) else {
            return false;
        };
        self.scroll_offset = offset;
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
        true
    }

    pub fn previous_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(offset) = previous_matching_line(&self.document, self.scroll_offset, query) else {
            return false;
        };
        self.scroll_offset = offset;
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
        true
    }

    pub fn copy_options(&self) -> Vec<CopyOption> {
        vec![CopyOption::new("file path", self.path.as_str())]
    }

    fn max_scroll_offset(&self) -> usize {
        self.line_count().saturating_sub(1)
    }

    fn max_line_width(&self) -> usize {
        self.document
            .lines()
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or_default()
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<DocumentLines>,
    ) -> Result<()> {
        self.document = load(&self.spec)?;
        self.scroll_offset = self.scroll_offset.min(self.max_scroll_offset());
        Ok(())
    }
}

fn file_show_path(spec: &ViewSpec) -> String {
    spec.path()
        .map(str::to_owned)
        .or_else(|| spec.target().map(str::to_owned))
        .or_else(|| spec.args().last().cloned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use ratatui::text::Line;

    use super::*;
    use crate::jj::JjCommand;

    fn file_show_view(path: &str, lines: &[&str]) -> FileShowView {
        FileShowView::new(
            ViewSpec::new(JjCommand::FileShow, Vec::new()),
            path,
            DocumentLines::new(
                lines
                    .iter()
                    .map(|line| Line::from((*line).to_owned()))
                    .collect::<Vec<_>>(),
            ),
        )
    }

    #[test]
    fn file_show_projection_is_plain_document() {
        let view = file_show_view("src/lib.rs", &["alpha", "beta"]);

        let projection = view.projection();

        assert!(projection.fixed_lines().is_empty());
        assert_eq!(projection.body_lines().len(), 2);
        assert_eq!(projection.body_scroll_offset(), 0);
    }

    #[test]
    fn file_show_search_wraps_without_reselecting_current_line() {
        let mut view = file_show_view("src/lib.rs", &["alpha", "target one", "beta", "target two"]);
        view.set_scroll_offset(3, 1);
        let query = SearchQuery::new("target".to_owned()).unwrap();

        assert!(view.next_match(3, &query));
        assert_eq!(view.scroll_offset(), 3);

        assert!(view.previous_match(3, &query));
        assert_eq!(view.scroll_offset(), 1);
    }

    #[test]
    fn file_show_copy_uses_exact_path() {
        let view = file_show_view("src/space file.txt", &["alpha"]);

        let options = view.copy_options();

        assert_eq!(
            options,
            vec![CopyOption::new("file path", "src/space file.txt")]
        );
    }

    #[test]
    fn file_show_refresh_clamps_scroll_after_content_shrinks() {
        let mut view = file_show_view("src/lib.rs", &["alpha", "beta", "gamma"]);
        view.set_scroll_offset(3, 2);

        view.refresh_with_loader(|_| Ok(DocumentLines::new(vec![Line::from("alpha")])))
            .unwrap();

        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn file_show_clamps_horizontal_offset_after_refresh_shrinks_content() {
        let mut view = file_show_view("README.md", &["0123456789ABCDEFGHIJ"]);
        let _ = view.execute(ViewCommand::ToggleWrap, context(10, None));
        for _ in 0..20 {
            let _ = view.execute(ViewCommand::ScrollRight, context(10, None));
        }
        assert_eq!(view.horizontal_offset(), 10);

        view.refresh_with_loader(|_| Ok(DocumentLines::new(vec![Line::from("short")])))
            .unwrap();
        view.clamp(3, 10);

        assert_eq!(view.horizontal_offset(), 0);
    }

    #[test]
    fn file_show_toggle_wrap_and_horizontal_scroll_clamps() {
        let mut view = file_show_view("README.md", &["0123456789ABCDEFGHIJ"]);

        assert_eq!(view.display_mode(), DocumentDisplayMode::Wrap);

        assert_eq!(
            view.execute(ViewCommand::ToggleWrap, context(10, None)),
            ViewEffect::Handled
        );
        assert_eq!(view.display_mode(), DocumentDisplayMode::NoWrap);

        for _ in 0..20 {
            let _ = view.execute(ViewCommand::ScrollRight, context(10, None));
        }

        assert_eq!(view.horizontal_offset(), 10);

        for _ in 0..20 {
            let _ = view.execute(ViewCommand::ScrollLeft, context(10, None));
        }

        assert_eq!(view.horizontal_offset(), 0);
    }

    #[test]
    fn file_show_horizontal_scroll_does_not_change_vertical_scroll() {
        let mut view = file_show_view("README.md", &["line 0", "line 1", "0123456789ABCDEFGHIJ"]);
        view.set_scroll_offset(3, 2);

        let _ = view.execute(ViewCommand::ToggleWrap, context(10, None));
        let _ = view.execute(ViewCommand::ScrollRight, context(10, None));

        assert_eq!(view.scroll_offset(), 2);
        assert_eq!(view.horizontal_offset(), 1);
    }

    #[test]
    fn file_show_search_still_moves_by_source_line_in_no_wrap() {
        let mut view = file_show_view(
            "README.md",
            &["alpha", "0123456789 target one", "beta", "target two"],
        );
        let query = SearchQuery::new("target".to_owned()).unwrap();

        let _ = view.execute(ViewCommand::ToggleWrap, context(10, Some(&query)));
        let _ = view.execute(ViewCommand::ScrollRight, context(10, Some(&query)));

        assert!(view.next_match(3, &query));

        assert_eq!(view.scroll_offset(), 1);
        assert_eq!(view.horizontal_offset(), 1);
    }

    fn context(viewport_width: u16, search: Option<&SearchQuery>) -> CommandContext<'_> {
        CommandContext {
            viewport_height: 3,
            viewport_width,
            search,
        }
    }
}
