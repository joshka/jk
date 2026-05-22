//! `jj status` view state, rendering, and scroll navigation.
//!
//! Status stays close to rendered `jj status` output, but it also carries a
//! narrow row model for exact file-path actions. Rows that do not confidently
//! name one repo-relative tracked path remain visible and selectable, but file
//! mutation actions report why they are disabled instead of guessing.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
#[cfg(test)]
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
#[cfg(test)]
use crate::jj::JjCommand;
use crate::jj::ViewSpec;
use crate::rendered_rows::document_plain_text;
use crate::search::{SearchQuery, highlight_line, line_matches};
use crate::selection::Selection;
use crate::theme;

use super::actions::StatusFileAction;
#[cfg(test)]
use super::rows::parse_status_row;
use super::rows::{StatusRow, load_status_rows};

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
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenFiles),
    ),
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

/// Rendered `jj status` output plus exact row action contracts.
pub struct StatusView {
    /// View specification used to reload the current status screen.
    spec: ViewSpec,
    /// Rendered status rows plus exact-path action contracts.
    rows: Vec<StatusRow>,
    /// Current selected row in the rendered status output.
    selection: Selection,
}

impl StatusView {
    /// Load the status view and derive exact-path action contracts from rendered output.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            rows: load_status_rows(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    /// Render the current status rows with search highlighting and active-row styling.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(row_list(&self.rows, search), area, &mut state);
    }

    /// Return the status-specific binding table.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Execute one view-local command against the status screen.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff => ViewEffect::Ignored,
            ViewCommand::MoveDown => {
                self.move_down(1);
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.move_up(1);
                ViewEffect::Handled
            }
            ViewCommand::PageDown => {
                self.move_down(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::PageUp => {
                self.move_up(context.page_size());
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.selection.first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.selection.last(self.rows.len());
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
            ViewCommand::OpenActionMenu => self
                .selected_file_action()
                .map_or_else(ViewEffect::StatusError, |_| ViewEffect::Ignored),
            ViewCommand::ToggleSelect => ViewEffect::Ignored,
            ViewCommand::OpenItem => ViewEffect::Ignored,
        }
    }

    /// Reload rendered status rows while preserving selection when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_status_rows)?;
        Ok(())
    }

    /// Return the `ViewSpec` that owns this status screen.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Return the total rendered row count.
    pub fn line_count(&self) -> usize {
        self.rows.len()
    }

    /// Return the selected row index used as the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.selection.index()
    }

    /// Set the selected row index, clamped to the current row count.
    pub fn set_scroll_offset(&mut self, _viewport_height: u16, scroll_offset: usize) {
        self.selection.set(scroll_offset, self.rows.len());
    }

    #[cfg(test)]
    pub fn scroll_to_bottom(&mut self, _viewport_height: u16) {
        self.selection.last(self.rows.len());
    }

    #[cfg(test)]
    pub fn scroll_down(&mut self, _viewport_height: u16, amount: usize) {
        self.move_down(amount);
    }

    pub fn clamp(&mut self, _viewport_height: u16) {
        self.selection.clamp(self.rows.len());
    }

    /// Count search matches across rendered status rows.
    pub fn search_matches(&self, query: &SearchQuery) -> usize {
        self.rows
            .iter()
            .filter(|row| line_matches(row.line(), query))
            .count()
    }

    /// Move to the next matching status row, wrapping once without reselecting the current row.
    pub fn next_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.rows.len())
            .chain(0..self.selection.index().min(self.rows.len()))
            .find(|index| line_matches(self.rows[*index].line(), query))
        else {
            return false;
        };
        self.selection.set(index, self.rows.len());
        true
    }

    /// Move to the previous matching status row, wrapping once without reselecting the current row.
    pub fn previous_match(&mut self, _viewport_height: u16, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.rows.len()).rev())
            .find(|index| line_matches(self.rows[*index].line(), query))
        else {
            return false;
        };
        self.selection.set(index, self.rows.len());
        true
    }

    #[cfg(test)]
    pub fn selected_exact_path(&self) -> std::result::Result<&str, String> {
        let Some(row) = self.rows.get(self.selection.index()) else {
            return Err("status file action unavailable: status output is empty".to_owned());
        };
        row.exact_path()
    }

    pub fn selected_file_action(&self) -> std::result::Result<StatusFileAction, String> {
        let Some(row) = self.rows.get(self.selection.index()) else {
            return Err("status file action unavailable: status output is empty".to_owned());
        };
        row.file_action()
    }

    /// Return copy options for the whole rendered status document.
    fn copy_options(&self) -> Vec<CopyOption> {
        let lines = self
            .rows
            .iter()
            .map(|row| row.line().clone())
            .collect::<Vec<_>>();
        let text = document_plain_text(&lines);
        if text.is_empty() {
            Vec::new()
        } else {
            vec![CopyOption::new("status text", text)]
        }
    }

    /// Move selection down by a fixed number of rows.
    fn move_down(&mut self, amount: usize) {
        for _ in 0..amount {
            self.selection.next(self.rows.len());
        }
    }

    /// Move selection up by a fixed number of rows.
    fn move_up(&mut self, amount: usize) {
        for _ in 0..amount {
            self.selection.previous();
        }
    }

    /// Reload rows with a caller-supplied loader while restoring the best previous selection.
    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<StatusRow>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_path = self
            .rows
            .get(previous_index)
            .and_then(StatusRow::exact_path_option)
            .map(str::to_owned);
        let previous_text = self.rows.get(previous_index).map(StatusRow::row_text);

        self.rows = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.rows,
            previous_index,
            previous_path,
            previous_text,
        );
        Ok(())
    }
}

/// Build the rendered row list with active-row styling and optional search highlighting.
fn row_list(rows: &[StatusRow], search: Option<&SearchQuery>) -> List<'static> {
    let items = rows
        .iter()
        .map(|row| ListItem::new(highlight_line(row.line().clone(), search)))
        .collect::<Vec<_>>();

    List::new(items).highlight_style(theme::active_row_style())
}

/// Restore selection after refresh by exact path first, then row text, then prior index.
fn restore_selection(
    selection: &mut Selection,
    rows: &[StatusRow],
    previous_index: usize,
    previous_path: Option<String>,
    previous_text: Option<String>,
) {
    if let Some(path) = previous_path
        && let Some(index) = rows
            .iter()
            .position(|row| row.exact_path_option() == Some(path.as_str()))
    {
        selection.set(index, rows.len());
        return;
    }

    if let Some(text) = previous_text
        && let Some(index) = rows.iter().position(|row| row.row_text() == text)
    {
        selection.set(index, rows.len());
        return;
    }

    selection.set(previous_index, rows.len());
}

#[cfg(test)]
impl StatusView {
    pub(crate) fn test_new(lines: &[&str]) -> Self {
        Self {
            spec: ViewSpec::new(JjCommand::Status, Vec::new()),
            rows: lines
                .iter()
                .map(|line| parse_status_row(Line::from((*line).to_owned())))
                .collect(),
            selection: Selection::default(),
        }
    }
}

#[cfg(test)]
mod tests;
