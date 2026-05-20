//! `jj show` view state, rendering, and navigation.
//!
//! The body remains rendered jj output. Once the scroll reaches file content,
//! the view pins a compact log line, a blank separator, and the active file
//! heading so the commit context remains visible without rebuilding jj output.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, ViewSpec, load_compact_log_context};
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
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenFiles)),
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

/// Rendered `jj show` output plus sticky context and scroll state.
pub struct ShowView {
    spec: ViewSpec,
    document: StickyFileDocument,
    compact_context: Vec<Line<'static>>,
}

impl ShowView {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let document = StickyFileDocument::load(&spec)?;
        let compact_context = load_compact_log_context(&spec.show_context_revset())?;
        Ok(Self {
            spec,
            document,
            compact_context,
        })
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
            ViewCommand::NewTrunk => ViewEffect::Ignored,
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
            ViewCommand::OpenFiles => ViewEffect::OpenView(ViewSpec::file_list(
                self.spec.navigation_revset(),
                self.document.current_file_label().map(str::to_owned),
            )),
            ViewCommand::OpenDiff => self
                .spec
                .navigation_revset()
                .map(|revset| ViewEffect::OpenDetail(JjCommand::Diff, revset))
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
            ViewCommand::OpenItem | ViewCommand::OpenShow => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)?;
        self.compact_context = load_compact_log_context(&self.spec.show_context_revset())?;
        Ok(())
    }

    pub fn projection(&self) -> PinnedDocument {
        self.document.projection(self.compact_context.clone())
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
        self.document
            .scroll_to_bottom(viewport_height, || self.compact_context.clone());
    }

    pub fn scroll_down(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document
                .scroll_down(viewport_height, 1, || self.compact_context.clone());
        }
    }

    pub fn scroll_up(&mut self, viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.document
                .scroll_up(viewport_height, 1, || self.compact_context.clone());
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
            options.push(CopyOption::new("change id", target));
        }
        if let Some(file) = self.document.current_file_label() {
            options.push(CopyOption::new("file path", file));
        }
        options.push(CopyOption::new(
            "visible context",
            sticky_file_view::lines_text(self.projection().fixed_lines()),
        ));
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
    fn show_view_is_plain_before_first_file() {
        let view = show_view(
            vec![
                line!("Commit ID: abc"),
                line!("    A long message"),
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            1,
        );

        let projection = view.projection();

        assert!(projection.fixed_lines().is_empty());
        assert_eq!(projection.body_lines().len(), 4);
        assert_eq!(projection.body_scroll_offset(), 1);
    }

    #[test]
    fn show_view_pins_commit_context_and_current_file() {
        let view = show_view(
            vec![
                line!("Commit ID: abc"),
                line!("    A long message"),
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            2,
        );

        let projection = view.projection();

        assert_eq!(projection.fixed_lines().len(), 4);
        assert_eq!(line_text(projection.fixed_lines()[0].clone()), "@  abc");
        assert_eq!(
            line_text(projection.fixed_lines()[3].clone()),
            "Added regular file Cargo.toml:"
        );
        assert!(
            projection
                .body_lines()
                .iter()
                .all(|line| line_text(line.clone()) != "Added regular file Cargo.toml:")
        );
    }

    #[test]
    fn show_view_long_message_stays_scrollable() {
        let view = show_view(
            vec![
                line!("Commit ID: abc"),
                line!("    line 1"),
                line!("    line 2"),
                line!("    line 3"),
                line!("    line 4"),
                line!("Added regular file Cargo.toml:"),
            ],
            4,
        );

        assert!(view.projection().fixed_lines().is_empty());
    }

    #[test]
    fn show_scroll_down_skips_separator_heading_and_first_body_duplicates() {
        let mut view = show_view(
            vec![
                line!("Commit ID: abc"),
                line!("    subject"),
                line!(""),
                line!("Added regular file .gitignore:"),
                line!("        1: /target"),
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            1,
        );

        view.scroll_down(6, 1);

        assert_eq!(view.scroll_offset(), 2);

        view.scroll_down(6, 1);

        assert_eq!(view.scroll_offset(), 5);
    }

    #[test]
    fn show_file_navigation_uses_sticky_activation_offsets() {
        let mut view = show_view(
            vec![
                line!("Commit ID: abc"),
                line!("    subject"),
                line!(""),
                line!("Added regular file .gitignore:"),
                line!("        1: /target"),
                line!("Added regular file Cargo.toml:"),
                line!("        1: [package]"),
            ],
            0,
        );

        view.next_file();

        assert_eq!(view.scroll_offset(), 2);

        view.next_file();

        assert_eq!(view.scroll_offset(), 5);

        view.previous_file();

        assert_eq!(view.scroll_offset(), 2);
    }

    #[test]
    fn command_execution_opens_diff_for_same_revset() {
        let mut view = show_view(vec![line!("Added regular file Cargo.toml:")], 0);
        view.spec = ViewSpec::new(JjCommand::Show, vec!["main".to_owned()]);

        assert_eq!(
            view.execute(ViewCommand::OpenDiff, context(None)),
            ViewEffect::OpenDetail(JjCommand::Diff, "main".to_owned())
        );
    }

    #[test]
    fn copy_options_use_plain_file_label() {
        let view = show_view(
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

    fn show_view(lines: Vec<Line<'static>>, scroll_offset: usize) -> ShowView {
        let document = DocumentLines::new(lines);
        let mut document = StickyFileDocument::new(document);
        document.set_scroll_offset(u16::MAX, scroll_offset);
        ShowView {
            spec: ViewSpec::new(JjCommand::Show, Vec::new()),
            document,
            compact_context: vec![line!("@  abc"), line!("│  subject")],
        }
    }

    fn context(search: Option<&SearchQuery>) -> CommandContext<'_> {
        CommandContext {
            viewport_height: 6,
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
