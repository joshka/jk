use std::collections::HashSet;
use std::fmt;

use jk_cli::{JjSquash, SquashQuery};
use jk_tui::log_view::LogView;

use crate::mutation_preview::PendingCommandPreview;
use crate::state::{AppState, AppView, InputMode};

/// Resolves the current log selection and opens a squash confirmation preview.
pub fn open_squash_preview(state: &mut AppState, squash_source: &JjSquash) {
    let AppView::Log(log) = state.views.active_mut() else {
        return;
    };
    let selection = match SquashSelection::from_log(log) {
        Ok(selection) => selection,
        Err(error) => {
            log.show_error(error.to_string());
            return;
        }
    };
    let preview = squash_source.spec_for(&selection.query()).command_preview();
    state.modes.push(InputMode::CommandPreview {
        pending: PendingCommandPreview::squash(preview, &selection),
    });
}

/// One exact revision participating in a squash selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquashRevision {
    pub(crate) change_id: String,
    pub(crate) commit_id: String,
}

impl SquashRevision {
    fn new(change_id: impl Into<String>, commit_id: impl Into<String>) -> Self {
        Self {
            change_id: change_id.into(),
            commit_id: commit_id.into(),
        }
    }
}

/// Resolved, unambiguous source and destination roles for a whole-change squash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquashSelection {
    sources: Vec<SquashRevision>,
    destination: SquashRevision,
}

impl SquashSelection {
    /// Resolves the cursor as destination and ordered marks as sources.
    pub fn from_log(log: &LogView) -> Result<Self, SquashSelectionError> {
        let destination_change_id = log
            .selected_change_id()
            .ok_or(SquashSelectionError::NoDestination)?;
        let destination_commit_id = log
            .selected_commit_id()
            .ok_or(SquashSelectionError::NoDestination)?;
        let destination = SquashRevision::new(destination_change_id, destination_commit_id);

        if log.marked_change_ids().is_empty() {
            return Err(SquashSelectionError::NoSources);
        }

        let mut sources = Vec::with_capacity(log.marked_change_ids().len());
        for change_id in log.marked_change_ids() {
            let commit_id = log.commit_id_for_change_id(change_id).ok_or_else(|| {
                SquashSelectionError::StaleSource {
                    change_id: change_id.clone(),
                }
            })?;
            sources.push(SquashRevision::new(change_id, commit_id));
        }

        Self::resolve(sources, destination)
    }

    fn resolve(
        sources: Vec<SquashRevision>,
        destination: SquashRevision,
    ) -> Result<Self, SquashSelectionError> {
        if sources.is_empty() {
            return Err(SquashSelectionError::NoSources);
        }

        let mut seen = HashSet::with_capacity(sources.len());
        for source in &sources {
            if source.commit_id == destination.commit_id
                || source.change_id == destination.change_id
            {
                return Err(SquashSelectionError::DestinationIsSource);
            }
            if !seen.insert(source.commit_id.as_str()) {
                return Err(SquashSelectionError::DuplicateSource);
            }
        }

        Ok(Self {
            sources,
            destination,
        })
    }

    /// Builds the exact whole-change command query.
    pub fn query(&self) -> SquashQuery {
        SquashQuery::new(
            self.sources.iter().map(|source| source.commit_id.clone()),
            self.destination.commit_id.clone(),
        )
    }

    /// Returns stable context to restore after the source changes disappear.
    pub fn destination_change_id(&self) -> &str {
        &self.destination.change_id
    }

    /// Returns explicit role and scope lines for the confirmation preview.
    pub fn preview_details(&self) -> Vec<String> {
        let mut details = self
            .sources
            .iter()
            .enumerate()
            .map(|(index, source)| format!("Source {}: {}", index + 1, source.commit_id))
            .collect::<Vec<_>>();
        details.push(format!("Destination: {}", self.destination.commit_id));
        details.push("Scope: whole changes (file and hunk selection deferred)".to_owned());
        details.push("Message: keep destination description".to_owned());
        details
    }
}

/// Why the current log selection cannot produce a safe squash command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SquashSelectionError {
    NoDestination,
    NoSources,
    StaleSource { change_id: String },
    DestinationIsSource,
    DuplicateSource,
}

impl fmt::Display for SquashSelectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDestination => formatter.write_str("Select a destination revision"),
            Self::NoSources => formatter.write_str(
                "Mark at least one source revision; the cursor is the squash destination",
            ),
            Self::StaleSource { change_id } => {
                write!(
                    formatter,
                    "Marked source {change_id} is no longer visible; refresh and reselect"
                )
            }
            Self::DestinationIsSource => formatter.write_str(
                "The destination is also marked as a source; move the cursor or clear that mark",
            ),
            Self::DuplicateSource => {
                formatter.write_str("A squash source is selected more than once")
            }
        }
    }
}

impl std::error::Error for SquashSelectionError {}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use jk_core::{LogEntry, LogSnapshot, SourceAction};
    use jk_tui::log_view::LogAction;

    use super::*;
    use crate::test_support::{SequencedRunner, output};

    fn revision(change: &str, commit: &str) -> SquashRevision {
        SquashRevision::new(change, commit)
    }

    #[test]
    fn ordered_sources_and_cursor_destination_resolve_to_full_commit_ids() {
        let selection = SquashSelection::resolve(
            vec![
                revision("source-a", "aaaaaaaaaaaaaaaa"),
                revision("source-b", "bbbbbbbbbbbbbbbb"),
            ],
            revision("destination", "dddddddddddddddd"),
        )
        .expect("distinct sources and destination should resolve");

        assert_eq!(
            selection.query().sources(),
            ["aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb"]
        );
        assert_eq!(selection.query().destination(), "dddddddddddddddd");
        assert_eq!(selection.destination_change_id(), "destination");
    }

    #[test]
    fn missing_sources_are_rejected() {
        assert_eq!(
            SquashSelection::resolve(vec![], revision("destination", "dddd")),
            Err(SquashSelectionError::NoSources)
        );
    }

    #[test]
    fn marked_destination_is_rejected() {
        assert_eq!(
            SquashSelection::resolve(
                vec![revision("destination", "dddd")],
                revision("destination", "dddd"),
            ),
            Err(SquashSelectionError::DestinationIsSource)
        );
    }

    #[test]
    fn duplicate_source_commits_are_rejected() {
        assert_eq!(
            SquashSelection::resolve(
                vec![revision("a", "same"), revision("b", "same")],
                revision("destination", "dest"),
            ),
            Err(SquashSelectionError::DuplicateSource)
        );
    }

    #[test]
    fn preview_details_name_roles_and_deferred_scope() {
        let selection = SquashSelection::resolve(
            vec![revision("source", "aaaaaaaa")],
            revision("destination", "dddddddd"),
        )
        .expect("distinct source and destination should resolve");

        assert_eq!(
            selection.preview_details(),
            [
                "Source 1: aaaaaaaa",
                "Destination: dddddddd",
                "Scope: whole changes (file and hunk selection deferred)",
                "Message: keep destination description",
            ]
        );
    }

    #[test]
    fn log_selection_opens_role_labeled_confirmation_with_exact_command() {
        let mut state = marked_source_and_destination_state();

        open_squash_preview(&mut state, &JjSquash::default());

        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("expected squash confirmation");
        };
        assert_eq!(pending.source_action, SourceAction::SquashRevision);
        assert_eq!(pending.source_key, "a s");
        assert_eq!(
            pending.preview.command_line,
            "jj --no-pager --color always squash --from aaaaaaaaaaaaaaaa --into dddddddddddddddd --use-destination-message"
        );
        assert_eq!(
            pending.details,
            [
                "Source 1: aaaaaaaaaaaaaaaa",
                "Destination: dddddddddddddddd",
                "Scope: whole changes (file and hunk selection deferred)",
                "Message: keep destination description",
            ]
        );
        assert_eq!(state.command_history().records().count(), 0);
    }

    #[test]
    fn confirmation_cancellation_preserves_log_selection_and_marks() {
        let mut state = marked_source_and_destination_state();
        open_squash_preview(&mut state, &JjSquash::default());
        let mut source = jk_cli::JjLog::default();

        crate::handle_command_preview_mode(
            &mut state,
            &mut source,
            &jk_cli::JjBookmarks::default(),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        );

        assert_eq!(state.modes.active(), None);
        let AppView::Log(log) = state.views.active() else {
            panic!("expected log");
        };
        assert_eq!(log.selected_change_id(), Some("destination"));
        assert_eq!(log.marked_change_ids(), ["source"]);
        assert_eq!(state.command_history().records().count(), 0);
    }

    #[test]
    fn confirmed_squash_records_operation_refreshes_and_reselects_destination() {
        let mut state = marked_source_and_destination_state();
        open_squash_preview(&mut state, &JjSquash::default());
        let Some(InputMode::CommandPreview { pending }) = state.modes.pop() else {
            panic!("expected squash confirmation");
        };
        let mut source = jk_cli::JjLog::default();
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "Squashed 1 commits.\n", ""),
            output(0, "222222222222\n", ""),
            output(0, "@  destination destination summary\n", ""),
            output(
                0,
                "{\"change_id\":\"destination\",\"commit_id\":\"eeeeeeeeeeeeeeee\",\"description\":\"destination summary\"}\t\"\"\n",
                "",
            ),
        ]);

        crate::execute_pending_command_with_runner(&mut state, &mut source, pending, runner);

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 3);
        assert_eq!(records[0].source.action, SourceAction::SquashRevision);
        assert_eq!(records[0].source.key.as_deref(), Some("a s"));
        assert_eq!(records[0].operation_id.as_deref(), Some("222222222222"));
        let AppView::Log(log) = state.views.active() else {
            panic!("expected log");
        };
        assert_eq!(log.selected_change_id(), Some("destination"));
        assert!(log.marked_change_ids().is_empty());
    }

    #[test]
    fn failed_squash_stays_in_log_and_preserves_useful_selection() {
        let mut state = marked_source_and_destination_state();
        open_squash_preview(&mut state, &JjSquash::default());
        let Some(InputMode::CommandPreview { pending }) = state.modes.pop() else {
            panic!("expected squash confirmation");
        };
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(1, "", "source and destination are unrelated\n"),
        ]);

        crate::execute_pending_command_with_runner(
            &mut state,
            &mut jk_cli::JjLog::default(),
            pending,
            runner,
        );

        let records = state.command_history().records().collect::<Vec<_>>();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].source.action, SourceAction::SquashRevision);
        assert_eq!(
            records[0]
                .result
                .exit_status
                .as_ref()
                .map(|status| status.success),
            Some(false)
        );
        let AppView::Log(log) = state.views.active() else {
            panic!("expected log");
        };
        assert_eq!(log.selected_change_id(), Some("destination"));
        assert_eq!(log.marked_change_ids(), ["source"]);
    }

    fn marked_source_and_destination_state() -> AppState {
        let snapshot = LogSnapshot::new(
            "@  source source summary\n○  destination destination summary\n",
            vec![
                LogEntry::new("source", "aaaaaaaaaaaaaaaa", "source summary").with_rendered_line(0),
                LogEntry::new("destination", "dddddddddddddddd", "destination summary")
                    .with_rendered_line(1),
            ],
        );
        let mut log = LogView::new(snapshot);
        let _ = log.apply(LogAction::ToggleMark);
        let _ = log.apply(LogAction::Next);
        AppState::new(AppView::Log(log))
    }
}
