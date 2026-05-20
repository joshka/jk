//! `jj status` view state, rendering, and scroll navigation.
//!
//! The first pass stays close to rendered `jj status` output. It is a
//! scroll-oriented triage surface with refresh, search, fetch, and copy, but
//! no mutation-capable file actions that would require exact path contracts.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
#[cfg(test)]
use crate::jj::JjCommand;
use crate::jj::{ViewSpec, document_plain_text};
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
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenFiles)),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Rendered `jj status` output plus plain document scroll state.
pub struct StatusView {
    spec: ViewSpec,
    document: StickyFileDocument,
}

impl StatusView {
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
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff => ViewEffect::Ignored,
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
            ViewCommand::OpenFiles => ViewEffect::OpenView(ViewSpec::file_list(None, None)),
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
            ViewCommand::OpenItem => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.document.refresh(&self.spec)?;
        Ok(())
    }

    pub fn projection(&self) -> crate::rendered_jj::PinnedDocument {
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

    pub fn clamp(&mut self, viewport_height: u16) {
        self.document.clamp(viewport_height);
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

    fn copy_options(&self) -> Vec<CopyOption> {
        let text = document_plain_text(self.projection().body_lines());
        if text.is_empty() {
            Vec::new()
        } else {
            vec![CopyOption::new("status text", text)]
        }
    }
}

#[cfg(test)]
impl StatusView {
    pub(crate) fn test_new(lines: &[&str]) -> Self {
        Self {
            spec: ViewSpec::new(JjCommand::Status, Vec::new()),
            document: StickyFileDocument::new(crate::rendered_jj::DocumentLines::new(
                lines
                    .iter()
                    .map(|line| ratatui::text::Line::from((*line).to_owned()))
                    .collect::<Vec<_>>(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::text::Line;

    use super::*;
    use crate::jj::JjCommand;
    use crate::sticky_file_view::lines_text;

    fn status_view(lines: &[&str]) -> StatusView {
        StatusView {
            spec: ViewSpec::new(JjCommand::Status, Vec::new()),
            document: StickyFileDocument::new(crate::rendered_jj::DocumentLines::new(
                lines
                    .iter()
                    .map(|line| Line::from((*line).to_owned()))
                    .collect::<Vec<_>>(),
            )),
        }
    }

    #[test]
    fn copy_options_use_full_status_text() {
        let view = status_view(&["Working copy changes:", "M src/app.rs"]);

        let options = view.copy_options();

        assert_eq!(options.len(), 1);
        assert_eq!(options[0].label(), "status text");
        assert_eq!(options[0].value(), "Working copy changes:\nM src/app.rs");
    }

    #[test]
    fn status_navigation_scrolls_through_document() {
        let mut view = status_view(&["one", "two", "three"]);

        view.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 3,
                search: None,
            },
        );
        assert_eq!(view.scroll_offset(), 1);

        view.execute(
            ViewCommand::MoveLast,
            CommandContext {
                viewport_height: 3,
                search: None,
            },
        );
        assert_eq!(view.scroll_offset(), 1);
    }

    #[test]
    fn status_search_moves_to_next_match() {
        let mut view = status_view(&["first", "beta", "third", "beta again"]);
        let query = SearchQuery::new("beta".to_owned()).unwrap();

        let effect = view.execute(
            ViewCommand::StartSearch,
            CommandContext {
                viewport_height: 3,
                search: Some(&query),
            },
        );

        assert_eq!(effect, ViewEffect::SearchStarted { matches: 2 });
        assert_eq!(view.scroll_offset(), 1);
    }

    #[test]
    fn clamp_preserves_readable_scroll_after_document_shrinks() {
        let mut view = status_view(&["one", "two", "three"]);
        view.scroll_to_bottom(3);
        assert_eq!(view.scroll_offset(), 1);

        view.document =
            StickyFileDocument::new(crate::rendered_jj::DocumentLines::new(vec![Line::from(
                "one".to_owned(),
            )]));
        view.clamp(3);

        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn status_projection_keeps_rendered_sections_readable() {
        let view = status_view(&[
            "The working copy has conflicts:",
            "UU src/app.rs",
            "",
            "Working copy changes:",
            "M src/status.rs",
            "A docs/plan/progress.md",
            "",
            "Working copy  (@) : yostqsxw 12345678 Slice 6 work",
            "Parent commit (@-): mzvwutkl 87654321 Prior change",
        ]);

        let projection = view.projection();
        let rendered = format!(
            "[fixed]\n{}\n[body @{}]\n{}",
            if projection.fixed_lines().is_empty() {
                "<none>".to_owned()
            } else {
                lines_text(projection.fixed_lines())
            },
            projection.body_scroll_offset(),
            lines_text(projection.body_lines())
        );

        assert_snapshot!(rendered, @r"
        [fixed]
        <none>
        [body @0]
        The working copy has conflicts:
        UU src/app.rs

        Working copy changes:
        M src/status.rs
        A docs/plan/progress.md

        Working copy  (@) : yostqsxw 12345678 Slice 6 work
        Parent commit (@-): mzvwutkl 87654321 Prior change
        ");
    }
}
