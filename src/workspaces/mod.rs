//! Read-only `jj root` and `jj workspace list` utility view.
//!
//! Rendered workspace rows stay opaque. Exact workspace names and target ids
//! come only from the separate workspace metadata template, so future actions
//! do not have to depend on label parsing.

mod rows;

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::ViewSpec;
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;
use crate::theme;

#[cfg(test)]
pub(crate) use rows::WORKSPACE_METADATA_TEMPLATE;
pub(crate) use rows::{WorkspaceContext, WorkspaceItem, load_workspace_context};

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
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Read-only workspace/root context from `jj root` and `jj workspace list`.
pub struct WorkspacesView {
    /// View identity used to reload the workspace surface.
    spec: ViewSpec,
    /// Loaded root context, workspace rows, and any degraded-load diagnostics.
    context: WorkspaceContext,
    /// Current selected row within the workspace list.
    selection: Selection,
}

impl WorkspacesView {
    /// Loads root context and rendered workspace rows for the current view spec.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            context: load_workspace_context(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    #[cfg(test)]
    pub(crate) fn test_new(context: WorkspaceContext) -> Self {
        Self {
            spec: ViewSpec::workspaces(Vec::new()),
            context,
            selection: Selection::default(),
        }
    }

    /// Renders the root header plus the selectable workspace list.
    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let header_lines = self.header_lines();
        let header_height = u16::try_from(header_lines.len())
            .unwrap_or(u16::MAX)
            .min(area.height);
        let [header, rows] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .areas(area);

        frame.render_widget(Paragraph::new(header_lines), header);

        let selected = (!self.context.entries().is_empty()).then_some(self.selection.index());
        let mut state = ListState::default().with_selected(selected);
        frame.render_stateful_widget(workspace_list(&self.context, search), rows, &mut state);
    }

    /// Returns the key bindings owned by the workspaces view.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Applies selection, search, and copy commands to the read-only workspace view.
    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::MoveDown => {
                self.selection.next(self.context.entries().len());
                ViewEffect::Handled
            }
            ViewCommand::MoveUp => {
                self.selection.previous();
                ViewEffect::Handled
            }
            ViewCommand::MoveFirst => {
                self.selection.first();
                ViewEffect::Handled
            }
            ViewCommand::MoveLast => {
                self.selection.last(self.context.entries().len());
                ViewEffect::Handled
            }
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
            ViewCommand::CycleMode
            | ViewCommand::NewTrunk
            | ViewCommand::PageDown
            | ViewCommand::PageUp
            | ViewCommand::ToggleWrap
            | ViewCommand::ScrollLeft
            | ViewCommand::ScrollRight
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem
            | ViewCommand::OpenShow
            | ViewCommand::OpenDiff
            | ViewCommand::ToggleSelect
            | ViewCommand::OpenActionMenu => ViewEffect::Ignored,
        }
    }

    /// Reloads the workspace context while preserving the selected workspace name when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_workspace_context)
    }

    /// Clamps the current selection to the available rows.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.context.entries().len());
    }

    /// Returns the view spec that identifies this workspaces surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the number of selectable workspace rows.
    pub fn item_count(&self) -> usize {
        self.context.entries().len()
    }

    /// Returns the rendered line count of the header plus all workspace rows.
    pub fn line_count(&self) -> usize {
        self.header_lines().len()
            + self
                .context
                .entries()
                .iter()
                .map(WorkspaceItem::line_count)
                .sum::<usize>()
    }

    /// Returns the currently selected workspace row, if any.
    fn selected_entry(&self) -> Option<&WorkspaceItem> {
        self.context.entries().get(self.selection.index())
    }

    /// Counts rows whose rendered text matches the current search query.
    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.context
            .entries()
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

    /// Advances selection to the next matching row if one exists.
    fn next_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = ((self.selection.index() + 1)..self.context.entries().len())
            .chain(0..self.selection.index().min(self.context.entries().len()))
            .find(|index| entry_matches(&self.context.entries()[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.context.entries().len());
        true
    }

    /// Moves selection to the previous matching row if one exists.
    fn previous_match(&mut self, query: &SearchQuery) -> bool {
        let Some(index) = (0..self.selection.index())
            .rev()
            .chain(((self.selection.index() + 1)..self.context.entries().len()).rev())
            .find(|index| entry_matches(&self.context.entries()[*index].lines(), query))
        else {
            return false;
        };
        self.selection.set(index, self.context.entries().len());
        true
    }

    /// Returns copyable root context and selected-row identifiers for the workspace surface.
    fn copy_options(&self) -> Vec<CopyOption> {
        let mut options = Vec::new();
        if let Some(root) = self.context.root() {
            options.push(CopyOption::new("current root", root));
        }

        if let Some(entry) = self.selected_entry() {
            if let Some(name) = entry.name() {
                options.push(CopyOption::new("workspace name", name));
            }
            if let Some(change_id) = entry.target_change_id() {
                options.push(CopyOption::new("change id", change_id));
            }
            if let Some(commit_id) = entry.target_commit_id() {
                options.push(CopyOption::new("commit id", commit_id));
            }
            options.push(CopyOption::new("row text", entry.row_text()));
        }

        options
    }

    /// Reloads the context and restores selection by workspace name before falling back to index.
    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<WorkspaceContext>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_name = self
            .selected_entry()
            .and_then(WorkspaceItem::name)
            .map(str::to_owned);

        self.context = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            self.context.entries(),
            previous_index,
            previous_name,
        );
        Ok(())
    }

    /// Returns the rendered root and degraded-load diagnostics shown above the list.
    fn header_lines(&self) -> Vec<Line<'static>> {
        workspace_header_lines(&self.context)
    }
}

/// Builds the rendered header for the workspace root surface and any degraded-load diagnostics.
fn workspace_header_lines(context: &WorkspaceContext) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    match context.root() {
        Some(root) => lines.push(Line::from(format!("current root: {root}"))),
        None => lines.push(Line::from("current root: unavailable")),
    }
    if let Some(error) = context.root_error() {
        lines.push(Line::from(format!("root error: {error}")));
    }
    if let Some(error) = context.list_error() {
        lines.push(Line::from(format!("workspace list error: {error}")));
    }
    if let Some(error) = context.metadata_error() {
        lines.push(Line::from(format!("workspace metadata warning: {error}")));
    }
    lines
}

/// Projects workspace rows into the selectable list widget.
fn workspace_list(context: &WorkspaceContext, search: Option<&SearchQuery>) -> List<'static> {
    let items = if context.entries().is_empty() {
        vec![ListItem::new("no workspaces listed")]
    } else {
        context
            .entries()
            .iter()
            .map(|entry| {
                let lines = entry
                    .lines()
                    .into_iter()
                    .map(|line| highlight_line(line, search))
                    .collect::<Vec<_>>();
                ListItem::new(lines)
            })
            .collect::<Vec<_>>()
    };

    List::new(items).highlight_style(theme::active_row_style())
}

/// Restores selection by exact workspace name before falling back to the previous index.
fn restore_selection(
    selection: &mut Selection,
    entries: &[WorkspaceItem],
    previous_index: usize,
    previous_name: Option<String>,
) {
    if let Some(name) = previous_name
        && let Some(index) = entries
            .iter()
            .position(|entry| entry.name() == Some(name.as_str()))
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
mod tests;
