//! Read-only `jj root` and `jj workspace list` utility view.
//!
//! Rendered workspace rows stay opaque. Exact workspace names and target ids
//! come only from the separate workspace metadata template, so future actions
//! do not have to depend on label parsing.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::ViewSpec;
use crate::jj_rows::{WorkspaceContext, WorkspaceItem, load_workspace_context};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;
use crate::theme;

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
    spec: ViewSpec,
    context: WorkspaceContext,
    selection: Selection,
}

impl WorkspacesView {
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

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

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

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_workspace_context)
    }

    pub fn clamp(&mut self) {
        self.selection.clamp(self.context.entries().len());
    }

    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    pub fn item_count(&self) -> usize {
        self.context.entries().len()
    }

    pub fn line_count(&self) -> usize {
        self.header_lines().len()
            + self
                .context
                .entries()
                .iter()
                .map(WorkspaceItem::line_count)
                .sum::<usize>()
    }

    fn selected_entry(&self) -> Option<&WorkspaceItem> {
        self.context.entries().get(self.selection.index())
    }

    fn search_matches(&self, query: &SearchQuery) -> usize {
        self.context
            .entries()
            .iter()
            .filter(|entry| entry_matches(&entry.lines(), query))
            .count()
    }

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

    fn header_lines(&self) -> Vec<Line<'static>> {
        workspace_header_lines(&self.context)
    }
}

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
mod tests {
    use ratatui::text::Line;

    use super::*;

    fn workspace_item(label: &str, name: Option<&str>) -> WorkspaceItem {
        WorkspaceItem::new(
            vec![Line::from(label.to_owned())],
            name.map(str::to_owned),
            name.map(|name| format!("{name}-change")),
            name.map(|name| format!("{name}-commit")),
        )
    }

    fn workspace_context(entries: Vec<WorkspaceItem>) -> WorkspaceContext {
        WorkspaceContext::new(Some("/repo".to_owned()), None, entries, None, None)
    }

    #[test]
    fn movement_clamps_to_workspace_rows() {
        let mut view = WorkspacesView::test_new(workspace_context(vec![
            workspace_item("default: one", Some("default")),
            workspace_item("other: two", Some("other")),
        ]));

        view.execute(
            ViewCommand::MoveLast,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 1);

        view.execute(
            ViewCommand::MoveDown,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 1);

        view.execute(
            ViewCommand::MoveFirst,
            CommandContext {
                viewport_height: 3,
                viewport_width: 80,
                search: None,
            },
        );
        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn search_wraps_by_rendered_workspace_row() {
        let mut view = WorkspacesView::test_new(workspace_context(vec![
            workspace_item("default: target", Some("default")),
            workspace_item("other: plain", Some("other")),
            workspace_item("third: target", Some("third")),
        ]));
        view.selection.set(0, view.item_count());
        let query = SearchQuery::new("target".to_owned()).unwrap();

        assert_eq!(view.search_matches(&query), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 2);
        assert!(view.next_match(&query));
        assert_eq!(view.selection.index(), 0);
    }

    #[test]
    fn copy_options_use_root_and_exact_metadata_when_available() {
        let mut view = WorkspacesView::test_new(workspace_context(vec![WorkspaceItem::new(
            vec![Line::from("rendered label: abc")],
            Some("exact-name".to_owned()),
            Some("change-id".to_owned()),
            Some("commit-id".to_owned()),
        )]));
        view.selection.set(0, view.item_count());

        assert_eq!(
            view.copy_options(),
            vec![
                CopyOption::new("current root", "/repo"),
                CopyOption::new("workspace name", "exact-name"),
                CopyOption::new("change id", "change-id"),
                CopyOption::new("commit id", "commit-id"),
                CopyOption::new("row text", "rendered label: abc"),
            ]
        );
    }

    #[test]
    fn copy_options_degrade_to_root_and_row_text_without_metadata() {
        let view = WorkspacesView::test_new(workspace_context(vec![WorkspaceItem::new(
            vec![Line::from("default: rendered")],
            None,
            None,
            None,
        )]));

        assert_eq!(
            view.copy_options(),
            vec![
                CopyOption::new("current root", "/repo"),
                CopyOption::new("row text", "default: rendered"),
            ]
        );
    }

    #[test]
    fn refresh_preserves_selected_workspace_name_when_possible() {
        let mut view = WorkspacesView::test_new(workspace_context(vec![
            workspace_item("default: one", Some("default")),
            workspace_item("other: two", Some("other")),
        ]));
        view.selection.set(1, view.item_count());

        view.refresh_with_loader(|_| {
            Ok(workspace_context(vec![
                workspace_item("other: moved", Some("other")),
                workspace_item("third: new", Some("third")),
            ]))
        })
        .unwrap();

        assert_eq!(view.selection.index(), 0);
        assert_eq!(
            view.selected_entry().and_then(WorkspaceItem::name),
            Some("other")
        );
    }

    #[test]
    fn empty_and_degraded_output_is_readable() {
        let context = WorkspaceContext::new(
            None,
            Some("jj root failed: no workspace".to_owned()),
            Vec::new(),
            Some("jj workspace list failed: unsupported".to_owned()),
            Some("workspace metadata parse failed".to_owned()),
        );
        let view = WorkspacesView::test_new(context);

        assert_eq!(view.item_count(), 0);
        assert_eq!(view.copy_options(), Vec::<CopyOption>::new());
        assert_eq!(
            view.header_lines()
                .into_iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>(),
            vec![
                "current root: unavailable",
                "root error: jj root failed: no workspace",
                "workspace list error: jj workspace list failed: unsupported",
                "workspace metadata warning: workspace metadata parse failed",
            ]
        );
    }
}
