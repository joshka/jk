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
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;

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
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenShow)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenShow),
    ),
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
        let previous_index = self.selection.index();
        let previous_change_id = self
            .entries
            .get(previous_index)
            .and_then(LogItem::action_id)
            .map(str::to_owned);

        self.entries = load_entries(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_change_id,
        );
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
            .and_then(LogItem::action_id)
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
        options.push(CopyOption::new("row text", entry.row_text()));
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

fn restore_selection(
    selection: &mut Selection,
    entries: &[LogItem],
    previous_index: usize,
    previous_change_id: Option<String>,
) {
    if let Some(change_id) = previous_change_id {
        if let Some(index) = entries
            .iter()
            .position(|entry| entry.action_id() == Some(change_id.as_str()))
        {
            selection.set(index, entries.len());
            return;
        }
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
mod tests {
    use ratatui::text::Line;

    use super::*;
    use crate::jj::JjCommand;

    fn log_item(text: &str, change_id: Option<&str>, commit_id: Option<&str>) -> LogItem {
        LogItem::new(
            vec![Line::from(text.to_owned())],
            change_id.map(str::to_owned),
            commit_id.map(str::to_owned),
        )
    }

    fn graph_view(entries: Vec<LogItem>) -> GraphView {
        GraphView {
            spec: ViewSpec::new(JjCommand::Log, Vec::new()),
            entries,
            selection: Selection::default(),
        }
    }

    #[test]
    fn copy_options_use_row_semantics() {
        let view = graph_view(vec![log_item("row text", Some("change"), Some("commit"))]);

        let options = view.copy_options();

        assert_eq!(options.len(), 3);
        assert_eq!(options[0].label(), "change id");
        assert_eq!(options[0].value(), "change");
        assert_eq!(options[1].label(), "commit id");
        assert_eq!(options[1].value(), "commit");
        assert_eq!(options[2].label(), "row text");
        assert_eq!(options[2].value(), "row text");
    }

    #[test]
    fn current_revset_is_none_for_non_selectable_log_rows() {
        let view = graph_view(vec![log_item("(elided revisions)", None, None)]);

        assert_eq!(view.current_revset(), None);
    }

    #[test]
    fn restore_selection_prefers_matching_change_id_over_index() {
        let entries = vec![
            log_item("second", Some("second"), None),
            log_item("first", Some("first"), None),
        ];
        let mut selection = Selection::default();
        selection.set(1, 2);

        restore_selection(&mut selection, &entries, 1, Some("second".to_owned()));

        assert_eq!(selection.index(), 0);
    }

    #[test]
    fn restore_selection_clamps_when_selected_change_disappears() {
        let entries = vec![log_item("only", Some("only"), None)];
        let mut selection = Selection::default();

        restore_selection(&mut selection, &entries, 3, Some("missing".to_owned()));

        assert_eq!(selection.index(), 0);
    }
}
