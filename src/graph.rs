//! The default/log graph view.
//!
//! Rows are grouped from `jj`'s rendered graph output. Detail navigation uses
//! the change id for the selected row because jj workflows and revsets are
//! change-centric; commit ids remain available through copy actions.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{List, ListItem, ListState};

use crate::action_menu::{ExactActionContext, build_action_menu};
use crate::command::{Binding, Command, CommandContext, KeyPattern, ViewCommand, ViewEffect};
use crate::copy::CopyOption;
use crate::jj::{JjCommand, LogItem, LogViewMode, ViewSpec, load_entries};
use crate::search::{SearchQuery, entry_matches, highlight_line};
use crate::selection::Selection;

pub const BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('w'), Command::View(ViewCommand::CycleMode)),
    Binding::new(KeyPattern::char('c'), Command::View(ViewCommand::NewTrunk)),
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
    Binding::new(KeyPattern::char('e'), Command::Edit),
    Binding::new(KeyPattern::char(']'), Command::NextEdit),
    Binding::new(KeyPattern::char('['), Command::PrevEdit),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
    Binding::new(
        KeyPattern::char(' '),
        Command::View(ViewCommand::ToggleSelect),
    ),
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
];

fn explicit_selection_style() -> Style {
    Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD)
}

/// Selectable graph output from `jj` or `jj log`.
pub struct GraphView {
    home_command: JjCommand,
    mode: LogViewMode,
    spec: ViewSpec,
    entries: Vec<LogItem>,
    selection: Selection,
    selected_change_ids: Vec<String>,
}

impl GraphView {
    #[cfg(test)]
    pub(crate) fn test_new(entries: Vec<LogItem>) -> Self {
        Self {
            home_command: JjCommand::Default,
            mode: LogViewMode::Default,
            spec: ViewSpec::new(JjCommand::Default, Vec::new()),
            entries,
            selection: Selection::default(),
            selected_change_ids: Vec::new(),
        }
    }

    pub fn load(spec: ViewSpec) -> Result<Self> {
        let home_command = spec.command();
        let mode = LogViewMode::from_spec(&spec);

        Ok(Self {
            home_command,
            mode,
            entries: load_entries(&spec)?,
            spec,
            selection: Selection::default(),
            selected_change_ids: Vec::new(),
        })
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        let mut state = ListState::default().with_selected(Some(self.selection.index()));
        frame.render_stateful_widget(
            entry_list(&self.entries, search, &self.selected_change_ids),
            area,
            &mut state,
        );
    }

    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match command {
            ViewCommand::CycleMode => match self.cycle_mode() {
                Ok(mode) => ViewEffect::StatusMessage(format!("mode: {}", mode.label())),
                Err(error) => ViewEffect::StatusError(error.to_string()),
            },
            ViewCommand::NewTrunk => ViewEffect::RunNewTrunk,
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
            ViewCommand::ToggleSelect => self.toggle_selection(),
            ViewCommand::OpenActionMenu => self.open_action_menu(),
            ViewCommand::Copy => ViewEffect::CopyOptions(self.copy_options()),
            ViewCommand::PageDown
            | ViewCommand::PageUp
            | ViewCommand::NextFile
            | ViewCommand::PreviousFile
            | ViewCommand::OpenFiles
            | ViewCommand::OpenItem => ViewEffect::Ignored,
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_entries)
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_change_id = self
            .entries
            .get(previous_index)
            .and_then(LogItem::action_id)
            .map(str::to_owned);

        self.entries = load(&self.spec)?;
        retain_selected_change_ids(&mut self.selected_change_ids, &self.entries);
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

    pub fn mode_label(&self) -> &str {
        self.mode.label()
    }

    pub fn select_change_id(&mut self, change_id: &str) -> bool {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.action_id() == Some(change_id))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    pub fn reveal_change_id(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        self.reveal_change_id_with_loader(change_id, fallback_mode, load_entries)
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

    pub fn selected_revision(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(LogItem::action_id)
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

    fn toggle_selection(&mut self) -> ViewEffect {
        let Some(change_id) = self.current_exact_change_id() else {
            return ViewEffect::StatusMessage(
                "selection only works on rows with exact change ids".to_owned(),
            );
        };
        let change_id = change_id.to_owned();

        let Some(position) = self
            .selected_change_ids
            .iter()
            .position(|selected| selected == &change_id)
        else {
            self.selected_change_ids.push(change_id.clone());
            return ViewEffect::StatusMessage(format!("selected {change_id}"));
        };
        self.selected_change_ids.remove(position);
        ViewEffect::StatusMessage(format!("unselected {change_id}"))
    }

    fn open_action_menu(&mut self) -> ViewEffect {
        let Some(current_revision) = self.current_exact_change_id() else {
            return ViewEffect::StatusMessage(
                "action menu requires current row to have an exact revision id".to_owned(),
            );
        };
        let selected_revisions = self.selected_revisions_in_graph_order();
        let menu = build_action_menu(
            &ExactActionContext::with_current(current_revision).with_sources(selected_revisions),
        );
        if menu.is_empty() {
            ViewEffect::StatusMessage("no preview actions available for selection".to_owned())
        } else {
            ViewEffect::OpenActionMenu(menu)
        }
    }

    pub fn set_mode(&mut self, mode: LogViewMode) -> Result<()> {
        self.switch_mode_with_loader(mode, load_entries)
    }

    fn cycle_mode(&mut self) -> Result<LogViewMode> {
        let next_mode = self.mode.next();
        self.set_mode(next_mode.clone())?;
        Ok(next_mode)
    }

    fn current_exact_change_id(&self) -> Option<&str> {
        self.entries
            .get(self.selection.index())
            .and_then(LogItem::action_id)
    }

    fn selected_revisions_in_graph_order(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter_map(LogItem::action_id)
            .filter(|action_id| {
                self.selected_change_ids
                    .iter()
                    .any(|selected| selected == action_id)
            })
            .map(str::to_owned)
            .collect()
    }

    fn reveal_change_id_with_loader(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<bool> {
        if self.select_change_id(change_id) {
            return Ok(false);
        }

        self.switch_mode_with_loader(fallback_mode, load)?;
        if self.select_change_id(change_id) {
            Ok(true)
        } else {
            Err(color_eyre::eyre::eyre!(
                "refreshed graph did not include the new working-copy change"
            ))
        }
    }

    fn switch_mode_with_loader(
        &mut self,
        mode: LogViewMode,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<()> {
        let previous_spec = self.spec.clone();
        let previous_mode = self.mode.clone();
        let previous_index = self.selection.index();
        let previous_change_id = self
            .entries
            .get(previous_index)
            .and_then(LogItem::action_id)
            .map(str::to_owned);
        let spec = ViewSpec::for_log_mode(self.home_command, &mode);
        let entries = match load(&spec) {
            Ok(entries) => entries,
            Err(error) => {
                self.spec = previous_spec;
                self.mode = previous_mode;
                return Err(error);
            }
        };

        self.spec = spec;
        self.mode = mode;
        self.entries = entries;
        retain_selected_change_ids(&mut self.selected_change_ids, &self.entries);
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_change_id,
        );
        Ok(())
    }
}

fn entry_list(
    entries: &[LogItem],
    search: Option<&SearchQuery>,
    selected_change_ids: &[String],
) -> List<'static> {
    let items = entries
        .iter()
        .map(|entry| {
            let is_selected = entry.action_id().is_some_and(|action_id| {
                selected_change_ids
                    .iter()
                    .any(|selected| selected == action_id)
            });
            let lines = entry_lines(entry, search, is_selected);
            ListItem::new(lines).style(if is_selected {
                explicit_selection_style()
            } else {
                Style::default()
            })
        })
        .collect::<Vec<_>>();

    List::new(items).highlight_style(
        Style::default()
            .bg(Color::Rgb(52, 54, 62))
            .add_modifier(Modifier::BOLD),
    )
}

fn entry_lines(
    entry: &LogItem,
    search: Option<&SearchQuery>,
    is_selected: bool,
) -> Vec<Line<'static>> {
    let lines = entry
        .lines()
        .into_iter()
        .map(|line| highlight_line(line, search))
        .collect::<Vec<_>>();
    if is_selected {
        lines
            .into_iter()
            .map(|line| line.patch_style(explicit_selection_style()))
            .collect()
    } else {
        lines
    }
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

fn retain_selected_change_ids(selected_change_ids: &mut Vec<String>, entries: &[LogItem]) {
    let retained = selected_change_ids
        .iter()
        .filter(|selected| {
            entries
                .iter()
                .any(|entry| entry.action_id() == Some(selected.as_str()))
        })
        .cloned()
        .collect();
    *selected_change_ids = retained;
}

#[cfg(test)]
mod tests {
    use ratatui::text::Line;

    use super::*;
    use crate::command::find_binding;
    use crate::jj::JjCommand;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn log_item(text: &str, change_id: Option<&str>, commit_id: Option<&str>) -> LogItem {
        LogItem::new(
            vec![Line::from(text.to_owned())],
            change_id.map(str::to_owned),
            commit_id.map(str::to_owned),
        )
    }

    fn graph_view(entries: Vec<LogItem>) -> GraphView {
        GraphView {
            home_command: JjCommand::Default,
            mode: LogViewMode::Default,
            spec: ViewSpec::new(JjCommand::Log, Vec::new()),
            entries,
            selection: Selection::default(),
            selected_change_ids: Vec::new(),
        }
    }

    fn command_context() -> CommandContext<'static> {
        CommandContext {
            viewport_height: 0,
            search: None,
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

    #[test]
    fn switch_mode_preserves_selection_by_change_id() {
        let mut view = graph_view(vec![
            log_item("first", Some("first"), None),
            log_item("second", Some("second"), None),
        ]);
        view.selection.set(1, view.entries.len());

        view.switch_mode_with_loader(LogViewMode::Trunk, |_| {
            Ok(vec![
                log_item("second", Some("second"), None),
                log_item("third", Some("third"), None),
            ])
        })
        .unwrap();

        assert_eq!(view.selection.index(), 0);
        assert_eq!(view.current_revset(), Some("second"));
        assert_eq!(view.mode_label(), "trunk work");
    }

    #[test]
    fn switch_mode_prunes_invisible_selection_ids() {
        let mut view = graph_view(vec![
            log_item("first", Some("first"), None),
            log_item("second", Some("second"), None),
            log_item("third", Some("third"), None),
        ]);

        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected first".to_owned())
        );
        view.select_next();
        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected second".to_owned())
        );
        view.select_next();
        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected third".to_owned())
        );

        view.switch_mode_with_loader(LogViewMode::Trunk, |_| {
            Ok(vec![
                log_item("first", Some("first"), None),
                log_item("third", Some("third"), None),
            ])
        })
        .unwrap();

        assert_eq!(view.mode_label(), "trunk work");
        assert_eq!(view.selected_change_ids, vec!["first", "third"]);
    }

    #[test]
    fn switch_mode_error_keeps_prior_view_state() {
        let mut view = graph_view(vec![log_item("first", Some("first"), None)]);
        let previous_spec = view.spec().clone();
        let previous_mode = view.mode.clone();

        let error = view
            .switch_mode_with_loader(
                LogViewMode::CustomRevset("not-a-revset(".to_owned()),
                |_| Err(color_eyre::eyre::eyre!("invalid revset")),
            )
            .unwrap_err();

        assert_eq!(error.to_string(), "invalid revset");
        assert_eq!(view.spec(), &previous_spec);
        assert_eq!(view.mode, previous_mode);
        assert_eq!(view.current_revset(), Some("first"));
    }

    #[test]
    fn select_change_id_moves_selection_to_matching_row() {
        let mut view = graph_view(vec![
            log_item("first", Some("first"), None),
            log_item("second", Some("second"), None),
        ]);

        assert!(view.select_change_id("second"));
        assert_eq!(view.current_revset(), Some("second"));
        assert!(!view.select_change_id("missing"));
        assert_eq!(view.current_revset(), Some("second"));
    }

    #[test]
    fn reveal_change_id_keeps_current_mode_when_change_is_visible() {
        let mut view = graph_view(vec![
            log_item("first", Some("first"), None),
            log_item("second", Some("second"), None),
        ]);

        let switched = view
            .reveal_change_id_with_loader("second", LogViewMode::Recent, |_| {
                panic!("fallback mode should not load when the change is already visible");
            })
            .unwrap();

        assert!(!switched);
        assert_eq!(view.current_revset(), Some("second"));
        assert_eq!(view.mode_label(), "default work");
    }

    #[test]
    fn reveal_change_id_switches_mode_when_current_mode_hides_change() {
        let mut view = graph_view(vec![log_item("trunk", Some("trunk"), None)]);

        let switched = view
            .reveal_change_id_with_loader("new", LogViewMode::Recent, |_| {
                Ok(vec![
                    log_item("new", Some("new"), None),
                    log_item("trunk", Some("trunk"), None),
                ])
            })
            .unwrap();

        assert!(switched);
        assert_eq!(view.current_revset(), Some("new"));
        assert_eq!(view.mode_label(), "recent work");
    }

    #[test]
    fn reveal_change_id_errors_when_fallback_mode_still_hides_change() {
        let mut view = graph_view(vec![log_item("trunk", Some("trunk"), None)]);

        let error = view
            .reveal_change_id_with_loader("new", LogViewMode::Recent, |_| {
                Ok(vec![log_item("trunk", Some("trunk"), None)])
            })
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "refreshed graph did not include the new working-copy change"
        );
        assert_eq!(view.mode_label(), "recent work");
    }

    #[test]
    fn toggle_select_requires_exact_change_id() {
        let mut view = graph_view(vec![log_item("(elided revisions)", None, None)]);

        let effect = view.execute(ViewCommand::ToggleSelect, command_context());

        assert_eq!(
            effect,
            ViewEffect::StatusMessage(
                "selection only works on rows with exact change ids".to_owned()
            )
        );
        assert!(view.selected_change_ids.is_empty());
    }

    #[test]
    fn toggle_select_tracks_exact_change_ids() {
        let mut view = graph_view(vec![log_item("first", Some("change"), None)]);
        assert!(view.selected_change_ids.is_empty());

        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected change".to_owned())
        );
        assert_eq!(view.selected_change_ids, vec!["change"]);

        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("unselected change".to_owned())
        );
        assert!(view.selected_change_ids.is_empty());
    }

    #[test]
    fn refresh_preserves_exact_selection_ids() {
        let mut view = graph_view(vec![
            log_item("first", Some("change"), None),
            log_item("second", Some("another"), None),
        ]);

        view.execute(ViewCommand::ToggleSelect, command_context());
        view.selection.set(1, view.entries.len());
        view.execute(ViewCommand::ToggleSelect, command_context());

        assert_eq!(view.selected_change_ids, vec!["change", "another"]);

        view.refresh_with_loader(|_| {
            Ok(vec![
                log_item("another", Some("another"), None),
                log_item("first", Some("change"), None),
            ])
        })
        .unwrap();

        assert_eq!(view.selected_change_ids, vec!["change", "another"]);
    }

    #[test]
    fn refresh_drops_disappeared_selection_ids() {
        let mut view = graph_view(vec![
            log_item("first", Some("change"), None),
            log_item("second", Some("another"), None),
            log_item("third", Some("third"), None),
        ]);

        view.execute(ViewCommand::ToggleSelect, command_context());
        view.selection.set(1, view.entries.len());
        view.execute(ViewCommand::ToggleSelect, command_context());
        view.selection.set(2, view.entries.len());
        view.execute(ViewCommand::ToggleSelect, command_context());

        assert_eq!(view.selected_change_ids, vec!["change", "another", "third"]);

        view.refresh_with_loader(|_| {
            Ok(vec![
                log_item("second", Some("another"), None),
                log_item("other", None, None),
                log_item("change", Some("change"), None),
            ])
        })
        .unwrap();

        assert_eq!(view.selected_change_ids, vec!["change", "another"]);
    }

    #[test]
    fn open_action_menu_prefers_single_row_context() {
        let mut view = graph_view(vec![
            log_item("first", Some("aaaaaaaa"), None),
            log_item("second", Some("bbbbbbbb"), None),
        ]);

        let effect = view.execute(ViewCommand::OpenActionMenu, command_context());
        let action_menu = if let ViewEffect::OpenActionMenu(action_menu) = effect {
            action_menu
        } else {
            panic!("expected action menu");
        };
        let labels = action_menu
            .items()
            .iter()
            .map(|item| item.label().to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                "edit selected revision aaaaaaaa",
                "new child of aaaaaaaa",
                "split selected revision aaaaaaaa",
                "abandon selected revision aaaaaaaa",
                "restore selected revision aaaaaaaa",
                "revert selected revision aaaaaaaa into @"
            ]
        );
    }

    #[test]
    fn open_action_menu_uses_explicit_selections_as_sources() {
        let mut view = graph_view(vec![
            log_item("first", Some("aaaaaaaa"), None),
            log_item("second", Some("bbbbbbbb"), None),
        ]);

        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected aaaaaaaa".to_owned())
        );
        view.select_next();
        let effect = view.execute(ViewCommand::OpenActionMenu, command_context());
        let action_menu = if let ViewEffect::OpenActionMenu(action_menu) = effect {
            action_menu
        } else {
            panic!("expected action menu");
        };
        let labels = action_menu
            .items()
            .iter()
            .map(|item| item.label().to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                "new child of aaaaaaaa",
                "rebase 1 source revision into destination bbbbbbbb",
                "squash 1 source revision into destination bbbbbbbb",
                "absorb current revision bbbbbbbb into 1 candidate destination",
                "restore selected revision bbbbbbbb",
                "revert selected revision bbbbbbbb into @"
            ]
        );
    }

    #[test]
    fn open_action_menu_orders_new_parents_by_graph_rows() {
        let mut view = graph_view(vec![
            log_item("first", Some("aaaaaaaa"), None),
            log_item("second", Some("bbbbbbbb"), None),
            log_item("third", Some("cccccccc"), None),
        ]);

        view.select_last();
        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected cccccccc".to_owned())
        );
        view.select_first();
        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected aaaaaaaa".to_owned())
        );
        view.select_next();
        let effect = view.execute(ViewCommand::OpenActionMenu, command_context());
        let action_menu = if let ViewEffect::OpenActionMenu(action_menu) = effect {
            action_menu
        } else {
            panic!("expected action menu");
        };

        assert!(matches!(
            action_menu.items()[0].follow_up(),
            crate::action_menu::FollowUp::NewParents { parents }
                if parents == &vec!["aaaaaaaa".to_owned(), "cccccccc".to_owned()]
        ));
    }

    #[test]
    fn open_action_menu_prefers_single_row_actions_for_self_selection() {
        let mut view = graph_view(vec![
            log_item("first", Some("aaaaaaaa"), None),
            log_item("second", Some("bbbbbbbb"), None),
        ]);

        assert_eq!(
            view.execute(ViewCommand::ToggleSelect, command_context()),
            ViewEffect::StatusMessage("selected aaaaaaaa".to_owned())
        );
        let effect = view.execute(ViewCommand::OpenActionMenu, command_context());
        let action_menu = if let ViewEffect::OpenActionMenu(action_menu) = effect {
            action_menu
        } else {
            panic!("expected action menu");
        };
        let labels = action_menu
            .items()
            .iter()
            .map(|item| item.label().to_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec![
                "edit selected revision aaaaaaaa",
                "new child of aaaaaaaa",
                "split selected revision aaaaaaaa",
                "abandon selected revision aaaaaaaa",
                "restore selected revision aaaaaaaa",
                "revert selected revision aaaaaaaa into @"
            ]
        );
    }

    #[test]
    fn graph_bindings_expose_edit_next_and_prev_keys() {
        assert_eq!(
            find_binding(BINDINGS, key(KeyCode::Char('e'))).map(Binding::command),
            Some(Command::Edit)
        );
        assert_eq!(
            find_binding(BINDINGS, key(KeyCode::Char(']'))).map(Binding::command),
            Some(Command::NextEdit)
        );
        assert_eq!(
            find_binding(BINDINGS, key(KeyCode::Char('['))).map(Binding::command),
            Some(Command::PrevEdit)
        );
    }

    #[test]
    fn entry_lines_apply_explicit_selection_style() {
        let selected = entry_lines(&log_item("first", Some("change"), None), None, true);
        let unselected = entry_lines(&log_item("first", Some("change"), None), None, false);

        assert_eq!(selected[0].style, explicit_selection_style());
        assert_eq!(unselected[0].style, Style::default());
        assert_ne!(selected[0].style, unselected[0].style);
    }

    #[test]
    fn selected_revision_uses_exact_row_revision() {
        let view = GraphView {
            home_command: JjCommand::Default,
            mode: LogViewMode::Default,
            spec: ViewSpec::new(JjCommand::Default, Vec::new()),
            entries: vec![
                log_item("@  has id", Some("abcd"), None),
                log_item("○  no id", None, None),
            ],
            selection: Selection::default(),
            selected_change_ids: Vec::new(),
        };

        assert_eq!(view.selected_revision(), Some("abcd"));
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}
