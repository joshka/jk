//! `jj diff` view state, rendering, and navigation.
//!
//! Diff only pins the active file heading. Unlike show, it has no commit
//! context prefix because the command output is already focused on the patch.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, ViewSpec};
use crate::rendered_jj::PinnedDocument;
use crate::search::SearchQuery;
use crate::sticky_file_view::{self, StickyFileDocument};

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
    Binding::new(KeyPattern::char(']'), Command::View(ViewCommand::NextFile)),
    Binding::new(
        KeyPattern::char('['),
        Command::View(ViewCommand::PreviousFile),
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
];

/// Rendered `jj diff` output plus sticky file context and scroll state.
pub struct DiffView {
    spec: ViewSpec,
    document: StickyFileDocument,
}

impl DiffView {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let document = StickyFileDocument::load(&spec)?;
        Ok(Self { spec, document })
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        sticky_file_view::render_document(frame, area, self.projection(), search);
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode => ViewEffect::Ignored,
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
            ViewCommand::OpenDiff => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)?;
        Ok(())
    }

    pub fn projection(&self) -> PinnedDocument {
        self.document.projection([])
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

    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        self.document
            .set_scroll_offset(viewport_height, scroll_offset);
    }

    pub fn scroll_to_top(&mut self) {
        self.document.scroll_to_top();
    }

    pub fn scroll_to_bottom(&mut self, viewport_height: u16) {
        self.document.scroll_to_bottom(viewport_height, Vec::new);
    }

    pub fn scroll_down(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document.scroll_down(viewport_height, 1, Vec::new);
        }
    }

    pub fn scroll_up(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document.scroll_up(viewport_height, 1, Vec::new);
        }
    }

    pub fn clamp(&mut self, _viewport_height: u16) {
        self.document.clamp(_viewport_height);
    }

    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.document.search_matches(query)
    }

    pub fn next_match(&mut self, viewport_height: u16, query: &SearchQuery) -> bool {
        self.document.next_match(viewport_height, query)
    }

    pub fn previous_match(&mut self, viewport_height: u16, query: &SearchQuery) -> bool {
        self.document.previous_match(viewport_height, query)
    }

    pub fn next_file(&mut self) {
        self.document.next_file();
    }

    pub fn previous_file(&mut self) {
        self.document.previous_file();
    }

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
mod tests {
    use ratatui::text::Line;
    use ratatui_macros::line;

    use super::*;
    use crate::jj::{JjCommand, ViewSpec};
    use crate::rendered_jj::DocumentLines;

    #[test]
    fn diff_view_pins_first_file_immediately() {
        let view = diff_view(
            vec![
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            0,
        );

        let projection = view.projection();

        assert_eq!(projection.fixed_lines().len(), 1);
        assert_eq!(
            line_text(projection.fixed_lines()[0].clone()),
            "Added regular file Cargo.toml:"
        );
        assert_eq!(projection.body_scroll_offset(), 0);
        assert_eq!(projection.body_lines().len(), 1);
    }

    #[test]
    fn diff_view_updates_current_file() {
        let view = diff_view(
            vec![
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
                line!("Modified regular file src/main.rs:"),
                line!("        1: fn main() {}"),
            ],
            2,
        );

        let projection = view.projection();

        assert_eq!(
            line_text(projection.fixed_lines()[0].clone()),
            "Modified regular file src/main.rs:"
        );
    }

    #[test]
    fn diff_scroll_clamp_accounts_for_sticky_file_line() {
        let mut view = diff_view(
            vec![
                line!("Added regular file Cargo.toml:"),
                line!("        1"),
                line!("        2"),
                line!("        3"),
                line!("        4"),
                line!("        5"),
            ],
            0,
        );

        view.scroll_to_bottom(3);

        assert_eq!(view.scroll_offset(), 4);
    }

    #[test]
    fn diff_scroll_down_skips_offsets_with_identical_projection() {
        let mut view = diff_view(
            vec![
                line!("Added regular file .gitignore:"),
                line!("        1: /target"),
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
                line!("        2: name = \"jk\""),
            ],
            0,
        );

        view.scroll_down(4, 1);

        assert_eq!(view.scroll_offset(), 2);
    }

    #[test]
    fn diff_file_navigation_moves_between_headings() {
        let mut view = diff_view(
            vec![
                line!("Added regular file .gitignore:"),
                line!("        1: /target"),
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            0,
        );

        view.next_file();

        assert_eq!(view.scroll_offset(), 2);

        view.previous_file();

        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn document_search_wraps_without_reselecting_current_line() {
        let mut view = diff_view(
            vec![
                line!("alpha"),
                line!("target one"),
                line!("beta"),
                line!("target two"),
            ],
            1,
        );
        let query = SearchQuery::new("target".to_owned()).unwrap();

        assert!(view.next_match(4, &query));
        assert_eq!(view.scroll_offset(), 3);

        assert!(view.next_match(4, &query));
        assert_eq!(view.scroll_offset(), 1);

        assert!(view.previous_match(4, &query));
        assert_eq!(view.scroll_offset(), 3);
    }

    #[test]
    fn document_search_does_not_move_for_only_current_match() {
        let mut view = diff_view(vec![line!("alpha"), line!("target"), line!("beta")], 1);
        let query = SearchQuery::new("target".to_owned()).unwrap();

        assert!(!view.next_match(4, &query));
        assert_eq!(view.scroll_offset(), 1);

        assert!(!view.previous_match(4, &query));
        assert_eq!(view.scroll_offset(), 1);
    }

    #[test]
    fn command_execution_moves_between_files() {
        let mut view = diff_view(
            vec![
                line!("Added regular file .gitignore:"),
                line!("        1: /target"),
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            0,
        );

        assert_eq!(
            view.execute(ViewCommand::NextFile, context(None)),
            ViewEffect::Handled
        );
        assert_eq!(view.scroll_offset(), 2);
    }

    #[test]
    fn command_execution_opens_show_for_same_revset() {
        let mut view = diff_view(vec![line!("Added regular file Cargo.toml:")], 0);
        view.spec = ViewSpec::new(JjCommand::Diff, vec!["-r".to_owned(), "main".to_owned()]);

        assert_eq!(
            view.execute(ViewCommand::OpenShow, context(None)),
            ViewEffect::OpenDetail(JjCommand::Show, "main".to_owned())
        );
    }

    #[test]
    fn copy_options_use_plain_file_label() {
        let view = diff_view(
            vec![
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            0,
        );

        let file = view
            .copy_options()
            .into_iter()
            .find(|option| option.label() == "file path")
            .unwrap();

        assert_eq!(file.value(), "Cargo.toml");
    }

    fn diff_view(lines: Vec<Line<'static>>, scroll_offset: usize) -> DiffView {
        let document = DocumentLines::new(lines);
        let mut document = StickyFileDocument::new(document);
        document.set_scroll_offset(u16::MAX, scroll_offset);
        DiffView {
            spec: ViewSpec::new(JjCommand::Diff, Vec::new()),
            document,
        }
    }

    fn context(search: Option<&SearchQuery>) -> CommandContext<'_> {
        CommandContext {
            viewport_height: 4,
            search,
        }
    }

    fn line_text(line: Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }
}
