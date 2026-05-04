//! The default/log graph view.
//!
//! Rows are grouped from `jj`'s rendered graph output. Detail navigation uses
//! the change id for the selected row because jj workflows and revsets are
//! change-centric; commit ids remain available through copy actions.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, LogItem, ViewSpec, load_entries};
use crate::search::{SearchQuery, entry_matches, highlight_line, line_text};
use crate::selection::Selection;

pub const BINDINGS: &[Binding] = &[
    Binding::new(
        KeyPattern::char('j', "j"),
        Command::View(ViewCommand::MoveDown),
        "move",
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Down, "Down"),
        Command::View(ViewCommand::MoveDown),
        "move",
    ),
    Binding::new(
        KeyPattern::char('k', "k"),
        Command::View(ViewCommand::MoveUp),
        "move",
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Up, "Up"),
        Command::View(ViewCommand::MoveUp),
        "move",
    ),
    Binding::new(
        KeyPattern::char('g', "g"),
        Command::View(ViewCommand::MoveFirst),
        "ends",
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Home, "Home"),
        Command::View(ViewCommand::MoveFirst),
        "ends",
    ),
    Binding::new(
        KeyPattern::char('G', "G"),
        Command::View(ViewCommand::MoveLast),
        "ends",
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::End, "End"),
        Command::View(ViewCommand::MoveLast),
        "ends",
    ),
    Binding::new(
        KeyPattern::char('s', "s"),
        Command::View(ViewCommand::OpenShow),
        "show",
    ),
    Binding::new(
        KeyPattern::char('l', "l"),
        Command::View(ViewCommand::OpenShow),
        "show",
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right, "Right"),
        Command::View(ViewCommand::OpenShow),
        "show",
    ),
    Binding::new(
        KeyPattern::char('d', "d"),
        Command::View(ViewCommand::OpenDiff),
        "diff",
    ),
    Binding::new(
        KeyPattern::char('n', "n"),
        Command::View(ViewCommand::NextSearchMatch),
        "search",
    ),
    Binding::new(
        KeyPattern::char('N', "N"),
        Command::View(ViewCommand::PreviousSearchMatch),
        "search",
    ),
];

/// Selectable graph output from `jj` or `jj log`.
pub struct GraphView {
    spec: ViewSpec,
    entries: Vec<LogItem>,
    selection: Selection,
}

impl GraphView {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: load_entries(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(entry_list(&self.entries, search), area, &mut state);
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::MoveDown => {
                self.select_next();
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.select_previous();
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.select_first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.select_last();
                ViewEffect::Handled
            }
            ViewCommand::OpenShow => self
                .current_revset()
                .map(|revset| ViewEffect::OpenDetail(JjCommand::Show, revset.to_owned()))
                .unwrap_or(ViewEffect::Ignored),
            ViewCommand::OpenDiff => self
                .current_revset()
                .map(|revset| ViewEffect::OpenDetail(JjCommand::Diff, revset.to_owned()))
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
            ViewCommand::PageDown
            | ViewCommand::PageUp
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.entries = load_entries(&self.spec)?;
        self.clamp();
        Ok(())
    }

    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    pub fn line_count(&self) -> usize {
        self.entries.iter().map(LogItem::line_count).sum()
    }

    pub fn current_revset(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(LogItem::change_id)
            .or_else(|| self.spec.target())
    }

    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    pub fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(selected) = next_matching_entry(&self.entries, self.selection.index(), query)
        else {
            return false;
        };
        self.selection.set(selected, self.entries.len());
        true
    }

    pub fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(selected) = previous_matching_entry(&self.entries, self.selection.index(), query)
        else {
            return false;
        };
        self.selection.set(selected, self.entries.len());
        true
    }

    pub fn copy_options(&self) -> Vec<CopyOption> {
        let Some(entry) = self.entries.get(self.selection.index()) else {
            return Vec::new();
        };
        let mut options = Vec::new();
        if let Some(change_id) = entry.change_id() {
            options.push(CopyOption::new("change id", change_id));
        }
        if let Some(commit_id) = entry.commit_id() {
            options.push(CopyOption::new("commit id", commit_id));
        }
        options.push(CopyOption::new("row text", lines_text(&entry.lines())));
        options
    }

    pub fn select_first(&mut self) {
        self.selection.first();
    }

    pub fn select_next(&mut self) {
        self.selection.next(self.entries.len());
    }

    pub fn select_previous(&mut self) {
        self.selection.previous();
    }

    pub fn select_last(&mut self) {
        self.selection.last(self.entries.len());
    }
}

fn entry_list(entries: &[LogItem], search: Option<&SearchQuery>) -> List<'static> {
    let items = entries
        .iter()
        .map(|entry| {
            let lines = entry
                .lines()
                .into_iter()
                .map(|line| highlight_line(line, search))
                .collect::<Vec<_>>();
            ListItem::new(lines)
        })
        .collect::<Vec<_>>();

    List::new(items).highlight_style(
        Style::default()
            .bg(Color::Rgb(52, 54, 62))
            .add_modifier(Modifier::BOLD),
    )
}

fn next_matching_entry(entries: &[LogItem], selected: usize, query: &SearchQuery) -> Option<usize> {
    ((selected + 1)..entries.len())
        .chain(0..selected.min(entries.len()))
        .find(|index| entry_matches(&entries[*index].lines(), query))
}

fn previous_matching_entry(
    entries: &[LogItem],
    selected: usize,
    query: &SearchQuery,
) -> Option<usize> {
    (0..selected)
        .rev()
        .chain(((selected + 1)..entries.len()).rev())
        .find(|index| entry_matches(&entries[*index].lines(), query))
}

fn lines_text(lines: &[ratatui::text::Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}
