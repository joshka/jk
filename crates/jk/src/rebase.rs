//! Rebase role selection and preview preparation.
//!
//! This module freezes revision identity before opening its selector. The selector only creates a
//! confirmation preview; command execution remains owned by the shared mutation boundary.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use jk_cli::{JjRebase, RebaseDestinationRole, RebaseQuery, RebaseSourceRole};
use jk_tui::log_view::{LogView, RevisionChoice};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::{Clear, Paragraph};

use crate::mutation_preview::PendingCommandPreview;
use crate::state::{AppState, AppView, InputMode, InputModeResult};

const BACKGROUND: Color = Color::Rgb(30, 35, 47);
const HEADER: Color = Color::Rgb(46, 55, 72);
const ACCENT: Color = Color::Rgb(28, 112, 121);

/// Frozen source revisions plus transient destination-selection state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingRebase {
    command_source: JjRebase,
    sources: Vec<String>,
    source_role: RebaseSourceRole,
    destination_role: RebaseDestinationRole,
    choices: Vec<RevisionChoice>,
    query: String,
    searching: bool,
    selected: usize,
    error: Option<String>,
}

impl PendingRebase {
    fn from_log(log: &LogView, command_source: JjRebase) -> Result<Self, String> {
        let sources = resolve_sources(
            log.marked_change_ids(),
            log.selected_commit_id(),
            |change_id| {
                log.commit_id_for_change_id(change_id)
                    .map(ToOwned::to_owned)
            },
        )?;

        let selected_commit_id = log.selected_commit_id();
        let choices = log
            .revision_choices()
            .into_iter()
            .filter(|choice| !sources.contains(&choice.revision))
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err("No distinct visible destination is available.".to_owned());
        }

        let selected = selected_commit_id
            .and_then(|commit_id| {
                choices
                    .iter()
                    .position(|choice| choice.revision == commit_id)
            })
            .unwrap_or(0);

        Ok(Self {
            command_source,
            sources,
            // Revision-only is the smallest, least surprising default. Branch and source modes
            // deliberately require an explicit role choice because they can move descendants.
            source_role: RebaseSourceRole::Revision,
            destination_role: RebaseDestinationRole::Onto,
            choices,
            query: String::new(),
            searching: false,
            selected,
            error: None,
        })
    }

    /// Renders the selector as a borderless, content-sized prompt.
    pub fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let width = area.width.saturating_sub(4).min(86);
        let desired_height = (self.filtered_choices().len().min(14) as u16 + 8).clamp(10, 22);
        let height = area.height.saturating_sub(2).min(desired_height);
        if width < 36 || height < 10 {
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new("Enlarge terminal to choose a rebase destination.\nEsc cancels")
                    .style(Style::new().fg(Color::White).bg(BACKGROUND)),
                area,
            );
            return;
        }

        let panel = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, panel);
        frame.render_widget(Paragraph::new("").style(Style::new().bg(BACKGROUND)), panel);
        frame.render_widget(
            Paragraph::new("  Choose rebase destination")
                .style(Style::new().fg(Color::White).bg(HEADER).bold()),
            Rect::new(panel.x, panel.y, panel.width, 1),
        );

        let controls = if width < 74 {
            vec![
                Line::from(format!("Source: {}", short_ids(&self.sources))),
                Line::from(format!("Scope: {} [r/b/s]", self.source_role.label())),
                Line::from(format!("Place: {} [o/A/B]", self.destination_role.label())),
                Line::from(format!("/ Search: {}", self.query)),
            ]
        } else {
            self.control_lines()
        };
        let content = Rect::new(
            panel.x + 2,
            panel.y + 2,
            panel.width.saturating_sub(4),
            panel.height.saturating_sub(4),
        );
        let controls_height = controls.len() as u16;
        frame.render_widget(
            Paragraph::new(controls).style(Style::new().fg(Color::White).bg(BACKGROUND)),
            Rect::new(content.x, content.y, content.width, controls_height),
        );
        let candidates = Rect::new(
            content.x,
            content.y + controls_height,
            content.width,
            content.height.saturating_sub(controls_height),
        );
        frame.render_widget(
            Paragraph::new(self.candidate_lines(usize::from(candidates.height)))
                .style(Style::new().fg(Color::White).bg(BACKGROUND)),
            candidates,
        );
        frame.render_widget(
            Paragraph::new(if width < 60 {
                "/ find  ↑↓ move  Enter next  Esc"
            } else {
                "  / search   ↑/↓ choose   Enter preview   Esc cancel  "
            })
            .style(Style::new().fg(Color::White).bg(ACCENT)),
            Rect::new(
                panel.x + 2,
                panel.y + panel.height.saturating_sub(2),
                panel.width.saturating_sub(4),
                1,
            ),
        );
    }

    fn control_lines(&self) -> Vec<Line<'static>> {
        vec![
            Line::from(format!("Source: {}", short_ids(&self.sources))),
            Line::from(format!(
                "Scope: {} ({})   r revision  b branch  s descendants",
                self.source_role.label(),
                self.source_role.flag()
            )),
            Line::from(format!(
                "Place: {} ({})   o onto  A after  B before",
                self.destination_role.label(),
                self.destination_role.flag()
            )),
            Line::from(format!(
                "Search: {}{}",
                if self.searching { "/" } else { "" },
                self.query
            )),
        ]
    }

    fn candidate_lines(&self, visible_rows: usize) -> Vec<Line<'static>> {
        let filtered = self.filtered_choices();
        if filtered.is_empty() {
            return vec![Line::from("  No visible destinations match.")];
        }
        if visible_rows == 0 {
            return Vec::new();
        }

        let selected = self.selected.min(filtered.len().saturating_sub(1));
        let start = selected
            .saturating_sub(visible_rows / 2)
            .min(filtered.len().saturating_sub(visible_rows));
        let mut lines = filtered
            .iter()
            .enumerate()
            .skip(start)
            .take(visible_rows)
            .map(|(index, choice)| {
                let marker = if index == selected { ">" } else { " " };
                let style = if index == selected {
                    Style::new().fg(Color::White).bg(ACCENT)
                } else {
                    Style::new().fg(Color::White).bg(BACKGROUND)
                };
                Line::from(Span::styled(
                    format!(
                        "{marker} {}  {}",
                        short_id(&choice.revision),
                        choice.summary
                    ),
                    style,
                ))
            })
            .collect::<Vec<_>>();

        if let Some(error) = &self.error {
            lines.push(Line::from(Span::styled(
                format!("Error: {error}"),
                Style::new().fg(Color::LightRed),
            )));
        }
        lines
    }

    fn preview(&mut self) -> Option<PendingCommandPreview> {
        let Some(destination) = self.selected_destination() else {
            self.error = Some("Choose a destination before previewing.".to_owned());
            return None;
        };
        let query = match RebaseQuery::new(
            self.source_role,
            self.sources.clone(),
            self.destination_role,
            destination.clone(),
        ) {
            Ok(query) => query,
            Err(error) => {
                self.error = Some(error.to_string());
                return None;
            }
        };
        let details = vec![
            format!("Source: {}", self.sources.join(", ")),
            format!("Source role: {}", self.source_role.label()),
            source_role_explanation(self.source_role).to_owned(),
            format!("Destination: {destination}"),
            format!("Placement: {}", self.destination_role.label()),
            destination_role_explanation(self.destination_role).to_owned(),
        ];
        Some(PendingCommandPreview::rebase(
            self.command_source.spec_for(&query).command_preview(),
            details,
        ))
    }

    fn selected_destination(&self) -> Option<String> {
        self.filtered_choices()
            .get(self.selected)
            .map(|choice| choice.revision.clone())
    }

    fn filtered_choices(&self) -> Vec<&RevisionChoice> {
        let query = self.query.to_lowercase();
        self.choices
            .iter()
            .filter(|choice| {
                query.is_empty()
                    || choice.revision.to_lowercase().contains(&query)
                    || choice.summary.to_lowercase().contains(&query)
            })
            .collect()
    }

    fn move_selection(&mut self, forward: bool) {
        let count = self.filtered_choices().len();
        if count == 0 {
            return;
        }
        self.selected = if forward {
            self.selected.saturating_add(1).min(count.saturating_sub(1))
        } else {
            self.selected.saturating_sub(1)
        };
        self.error = None;
    }
}

fn resolve_sources(
    marked_change_ids: &[String],
    selected_commit_id: Option<&str>,
    commit_id_for_change_id: impl FnOnce(&str) -> Option<String>,
) -> Result<Vec<String>, String> {
    match marked_change_ids {
        [] => selected_commit_id
            .map(|commit_id| vec![commit_id.to_owned()])
            .ok_or_else(|| "Select a source revision before rebasing.".to_owned()),
        [change_id] => commit_id_for_change_id(change_id)
            .map(|commit_id| vec![commit_id])
            .ok_or_else(|| {
                format!("Marked source {change_id} is missing or divergent; refresh and reselect.")
            }),
        _ => Err("Rebase source is ambiguous: keep one source marked, then press R.".to_owned()),
    }
}

/// Opens the contextual rebase selector without running `jj`.
pub fn open_rebase_destination(state: &mut AppState, command_source: &JjRebase) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };

    match PendingRebase::from_log(log, command_source.clone()) {
        Ok(pending) => state.modes.push(InputMode::RebaseDestination { pending }),
        Err(error) => log.show_error(error),
    }
}

/// Handles selector input. Confirming creates a preview and never executes a command.
pub fn handle_input(state: &mut AppState, key: KeyEvent) -> InputModeResult {
    if key.kind == KeyEventKind::Release {
        return InputModeResult::Handled;
    }

    let Some(InputMode::RebaseDestination { pending }) = state.modes.active_mut() else {
        return InputModeResult::Unhandled;
    };

    if key.code == KeyCode::Esc {
        if pending.searching {
            pending.searching = false;
            pending.error = None;
        } else {
            state.modes.pop();
        }
        return InputModeResult::Handled;
    }
    if pending.searching && key.code != KeyCode::Enter {
        match key.code {
            KeyCode::Backspace => {
                pending.query.pop();
                pending.selected = 0;
                pending.error = None;
            }
            KeyCode::Up => pending.move_selection(false),
            KeyCode::Down => pending.move_selection(true),
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                pending.query.push(character);
                pending.selected = 0;
                pending.error = None;
            }
            _ => {}
        }
        return InputModeResult::Handled;
    }

    match key.code {
        KeyCode::Char('/') => {
            pending.searching = true;
            pending.error = None;
        }
        KeyCode::Up | KeyCode::Char('k') => pending.move_selection(false),
        KeyCode::Down | KeyCode::Char('j') => pending.move_selection(true),
        KeyCode::Char('r') => pending.source_role = RebaseSourceRole::Revision,
        KeyCode::Char('b') => pending.source_role = RebaseSourceRole::Branch,
        KeyCode::Char('s') => pending.source_role = RebaseSourceRole::Source,
        KeyCode::Char('o') => pending.destination_role = RebaseDestinationRole::Onto,
        KeyCode::Char('A') => pending.destination_role = RebaseDestinationRole::InsertAfter,
        KeyCode::Char('B') => pending.destination_role = RebaseDestinationRole::InsertBefore,
        KeyCode::Enter if key.kind != KeyEventKind::Repeat => {
            let preview = pending.preview();
            if let Some(pending) = preview {
                state.modes.pop();
                state.modes.push(InputMode::CommandPreview { pending });
            }
        }
        _ => {}
    }
    InputModeResult::Handled
}

fn source_role_explanation(role: RebaseSourceRole) -> &'static str {
    match role {
        RebaseSourceRole::Revision => {
            "Moves the selected revision (-r); reconnects descendants to its old parents."
        }
        RebaseSourceRole::Branch => {
            "Moves the destination-relative branch (-b), including applicable ancestors and descendants."
        }
        RebaseSourceRole::Source => "Moves the source and all descendants (-s).",
    }
}

fn destination_role_explanation(role: RebaseDestinationRole) -> &'static str {
    match role {
        RebaseDestinationRole::Onto => "Makes this revision the new parent (-o).",
        RebaseDestinationRole::InsertAfter => {
            "Inserts after (-A), rewriting existing descendants, including other workspaces."
        }
        RebaseDestinationRole::InsertBefore => {
            "Inserts before (-B), rewriting the destination and descendants, including other workspaces."
        }
    }
}

fn short_ids(ids: &[String]) -> String {
    ids.iter()
        .map(|id| short_id(id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn short_id(id: &str) -> &str {
    const DISPLAY_ID_LEN: usize = 12;

    id.get(..DISPLAY_ID_LEN).unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use jk_core::{LogEntry, LogSnapshot, SourceAction};
    use jk_tui::log_view::LogAction;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::state::AppView;
    use crate::test_support::{SequencedRunner, output};

    const SOURCE_CHANGE: &str = "source-change";
    const SOURCE_COMMIT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const TARGET_CHANGE: &str = "target-change";
    const TARGET_COMMIT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const OTHER_COMMIT: &str = "cccccccccccccccccccccccccccccccccccccccc";

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn repeated_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Repeat,
            state: KeyEventState::NONE,
        }
    }

    fn log() -> LogView {
        LogView::new(LogSnapshot::new(
            "@  source source summary\n○  target target summary\n○  other other summary\n",
            vec![
                LogEntry::new(SOURCE_CHANGE, SOURCE_COMMIT, "source summary").with_rendered_line(0),
                LogEntry::new(TARGET_CHANGE, TARGET_COMMIT, "target summary").with_rendered_line(1),
                LogEntry::new("other-change", OTHER_COMMIT, "other summary").with_rendered_line(2),
            ],
        ))
    }

    fn marked_source_state() -> AppState {
        let mut state = AppState::new(AppView::Log(log()));
        let AppView::Log(log) = state.views.active_mut() else {
            panic!("expected log");
        };
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Next);
        state
    }

    #[test]
    fn marked_sources_reject_ambiguous_or_stale_selection() {
        let marked = vec!["first".to_owned(), "second".to_owned()];
        assert_eq!(
            resolve_sources(&marked, Some(SOURCE_COMMIT), |_| Some(
                SOURCE_COMMIT.to_owned()
            )),
            Err("Rebase source is ambiguous: keep one source marked, then press R.".to_owned())
        );

        let marked = vec![SOURCE_CHANGE.to_owned()];
        assert_eq!(
            resolve_sources(&marked, Some(SOURCE_COMMIT), |_| None),
            Err(
                "Marked source source-change is missing or divergent; refresh and reselect."
                    .to_owned()
            )
        );
    }

    #[test]
    fn selector_uses_full_commit_ids_and_revision_only_by_default() {
        let pending = PendingRebase::from_log(&log(), JjRebase::default())
            .unwrap_or_else(|error| panic!("selector opens: {error}"));

        assert_eq!(pending.sources, [SOURCE_COMMIT]);
        assert_eq!(pending.source_role, RebaseSourceRole::Revision);
        assert_eq!(
            pending.selected_destination().as_deref(),
            Some(TARGET_COMMIT)
        );
    }

    #[test]
    fn search_and_roles_create_an_exact_full_id_preview() {
        let mut pending = PendingRebase::from_log(&log(), JjRebase::default())
            .unwrap_or_else(|error| panic!("selector opens: {error}"));
        pending.searching = true;
        pending.query = "other".to_owned();
        pending.searching = false;
        pending.source_role = RebaseSourceRole::Source;
        pending.destination_role = RebaseDestinationRole::InsertBefore;

        let preview = pending.preview().expect("matching destination previews");
        let argv = preview
            .preview
            .spec
            .argv()
            .iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(argv, ["rebase", "-s", SOURCE_COMMIT, "-B", OTHER_COMMIT]);
        assert!(
            preview
                .details
                .iter()
                .any(|detail| detail.contains("all descendants"))
        );
        assert!(
            preview
                .details
                .iter()
                .any(|detail| detail.contains("other workspaces"))
        );
    }

    #[test]
    fn escape_cancels_without_creating_a_history_record() {
        let mut state = AppState::new(AppView::Log(log()));
        open_rebase_destination(&mut state, &JjRebase::default());

        assert_eq!(
            handle_input(&mut state, key(KeyCode::Esc)),
            InputModeResult::Handled
        );
        assert!(state.modes.active().is_none());
        assert_eq!(state.command_history().records().count(), 0);
    }

    #[test]
    fn search_enter_opens_preview_without_executing() {
        let mut state = AppState::new(AppView::Log(log()));
        open_rebase_destination(&mut state, &JjRebase::default());
        let _ = handle_input(&mut state, key(KeyCode::Char('/')));
        for character in "other".chars() {
            let _ = handle_input(&mut state, key(KeyCode::Char(character)));
        }

        let _ = handle_input(&mut state, repeated_key(KeyCode::Enter));
        assert!(matches!(
            state.modes.active(),
            Some(InputMode::RebaseDestination { .. })
        ));
        let _ = handle_input(&mut state, key(KeyCode::Enter));
        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("search selection opens a separate preview");
        };
        assert!(
            pending
                .preview
                .spec
                .argv()
                .iter()
                .any(|arg| arg == OTHER_COMMIT)
        );
        assert!(!pending.can_confirm);
        assert_eq!(state.command_history().records().count(), 0);
    }

    #[test]
    fn selector_repeat_enter_does_not_open_or_confirm_a_preview() {
        let mut state = AppState::new(AppView::Log(log()));
        open_rebase_destination(&mut state, &JjRebase::default());

        assert_eq!(
            handle_input(&mut state, repeated_key(KeyCode::Enter)),
            InputModeResult::Handled
        );
        assert!(matches!(
            state.modes.active(),
            Some(InputMode::RebaseDestination { .. })
        ));
        assert_eq!(state.command_history().records().count(), 0);
    }

    #[test]
    fn preview_cancel_and_repeat_enter_preserve_log_and_marks() {
        let mut state = marked_source_state();
        open_rebase_destination(&mut state, &JjRebase::default());
        let _ = handle_input(&mut state, key(KeyCode::Enter));
        assert!(matches!(
            state.modes.active(),
            Some(InputMode::CommandPreview { .. })
        ));

        let mut source = jk_cli::JjLog::default();
        assert_eq!(
            crate::handle_command_preview_mode(
                &mut state,
                &mut source,
                &jk_cli::JjBookmarks::default(),
                repeated_key(KeyCode::Enter),
            ),
            InputModeResult::Handled
        );
        assert!(matches!(
            state.modes.active(),
            Some(InputMode::CommandPreview { .. })
        ));
        assert_eq!(
            crate::handle_command_preview_mode(
                &mut state,
                &mut source,
                &jk_cli::JjBookmarks::default(),
                key(KeyCode::Esc)
            ),
            InputModeResult::Handled
        );
        assert!(state.modes.active().is_none());
        assert_eq!(state.command_history().records().count(), 0);
        let AppView::Log(log) = state.views.active() else {
            panic!("expected log");
        };
        assert_eq!(log.selected_change_id(), Some(TARGET_CHANGE));
        assert_eq!(log.marked_change_ids(), [SOURCE_CHANGE]);
    }

    #[test]
    fn confirmed_rebase_records_operation_and_refreshes() {
        let mut state = marked_source_state();
        open_rebase_destination(&mut state, &JjRebase::default());
        let _ = handle_input(&mut state, key(KeyCode::Enter));
        let Some(InputMode::CommandPreview { pending }) = state.modes.pop() else {
            panic!("expected rebase preview");
        };
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "", ""),
            output(0, "222222222222\n", ""),
            output(0, "refreshed rendered log\n", ""),
            output(0, "{}\n", ""),
        ]);

        crate::mutations::execute_pending_command_with_runner(
            &mut state,
            &mut jk_cli::JjLog::default(),
            pending,
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].source.action, SourceAction::RebaseRevision);
        assert_eq!(records[0].source.key.as_deref(), Some("R"));
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        assert_eq!(records[1].source.action, SourceAction::Refresh);
        assert_eq!(records[2].source.action, SourceAction::Refresh);
    }

    #[test]
    fn immutable_or_spawn_failure_keeps_selection_without_refresh() {
        let mut immutable_state = marked_source_state();
        open_rebase_destination(&mut immutable_state, &JjRebase::default());
        let _ = handle_input(&mut immutable_state, key(KeyCode::Enter));
        let Some(InputMode::CommandPreview { pending }) = immutable_state.modes.pop() else {
            panic!("expected rebase preview");
        };
        let immutable_runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(1, "", "Commit source is immutable\n"),
        ]);
        crate::mutations::execute_pending_command_with_runner(
            &mut immutable_state,
            &mut jk_cli::JjLog::default(),
            pending,
            immutable_runner,
        );
        let records = immutable_state
            .command_history()
            .records()
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].result.stderr.snippet,
            "Commit source is immutable\n"
        );
        assert_eq!(records[0].operation_id, None);

        let mut spawn_state = marked_source_state();
        open_rebase_destination(&mut spawn_state, &JjRebase::default());
        let _ = handle_input(&mut spawn_state, key(KeyCode::Enter));
        let Some(InputMode::CommandPreview { pending }) = spawn_state.modes.pop() else {
            panic!("expected rebase preview");
        };
        let spawn_runner = SequencedRunner::results(vec![
            Ok(output(0, "111111111111\n", "")),
            Err(std::io::Error::other("jj missing")),
        ]);
        crate::mutations::execute_pending_command_with_runner(
            &mut spawn_state,
            &mut jk_cli::JjLog::default(),
            pending,
            spawn_runner,
        );
        let records = spawn_state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].result.spawn_error.as_deref(), Some("jj missing"));
        assert_eq!(records[0].operation_id, None);

        for state in [&immutable_state, &spawn_state] {
            let AppView::Log(log) = state.views.active() else {
                panic!("expected log");
            };
            assert_eq!(log.selected_change_id(), Some(TARGET_CHANGE));
            assert_eq!(log.marked_change_ids(), [SOURCE_CHANGE]);
        }
    }

    #[test]
    fn narrow_render_is_borderless_and_stays_in_bounds() {
        let pending = PendingRebase::from_log(&log(), JjRebase::default())
            .unwrap_or_else(|error| panic!("selector opens: {error}"));
        let backend = TestBackend::new(40, 16);
        let mut terminal = Terminal::new(backend).expect("test terminal");

        terminal
            .draw(|frame| pending.render(frame))
            .expect("selector renders in a narrow terminal");
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();

        assert!(rendered.contains("Choose rebase"));
        assert!(rendered.contains("Scope:"));
        assert!(rendered.contains("Place:"));
        assert!(rendered.contains("target summary"));
        assert!(rendered.contains("Enter next"));
        assert!(!rendered.contains('┌'));
        assert!(!rendered.contains('╭'));

        let mut terminal = Terminal::new(TestBackend::new(120, 50)).expect("large test terminal");
        terminal
            .draw(|frame| pending.render(frame))
            .expect("selector renders");
        let panel_rows = terminal
            .backend()
            .buffer()
            .content()
            .chunks(120)
            .filter(|row| row[60].bg != Color::Reset)
            .count();
        assert_eq!(
            panel_rows, 10,
            "short destination lists have content-sized panels"
        );
    }
}
