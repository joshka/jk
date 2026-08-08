use jk_cli::{
    BookmarkMutation, BookmarkSnapshot, JjBookmarks, JjCommandRunner, JjGitRemote, JjShow,
    RecordingJjCommandRunner, ShowQuery, SystemJjCommandRunner,
};
use jk_core::{CommandSource, SourceAction, SourceView};
use jk_tui::bookmark_view::{
    BookmarkAction, BookmarkActionResult, BookmarkView, BookmarkViewSnapshot,
};
use jk_tui::rendered_view::RenderedView;

use crate::mutation_preview::PendingCommandPreview;
use crate::recording_runner;
use crate::state::{AppState, AppView, BookmarkMutationField, BookmarkMutationKind, InputMode};

/// Opens the bookmark list as a child view.
pub fn open_bookmarks(state: &mut AppState, source: &JjBookmarks) {
    open_bookmarks_with_runner(state, source, SystemJjCommandRunner);
}

pub fn open_bookmarks_with_runner<R: JjCommandRunner>(
    state: &mut AppState,
    source: &JjBookmarks,
    runner: R,
) {
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        &mut state.history,
        CommandSource::new(SourceView::Bookmarks, SourceAction::BookmarkList),
    );
    let view = match source.load_with_runner(&mut runner) {
        Ok(snapshot) => BookmarkView::new(bookmark_view_snapshot(snapshot)),
        Err(error) => {
            let mut view = BookmarkView::new(BookmarkViewSnapshot::new(Vec::new()));
            view.show_error(error.to_string());
            view
        }
    };
    state.views.push(AppView::Bookmarks { view });
}

/// Applies a bookmark-list action and performs requested local inspection.
pub fn apply_bookmark_action(
    state: &mut AppState,
    source: &JjBookmarks,
    show_source: &JjShow,
    action: BookmarkAction,
) {
    let result = {
        let AppView::Bookmarks { view } = state.views.active_mut() else {
            return;
        };
        view.apply(action)
    };
    match result {
        BookmarkActionResult::Refresh => refresh_bookmarks(state, source),
        BookmarkActionResult::OpenTarget { commit_id } => {
            let query = ShowQuery::from(commit_id);
            let mut runner = recording_runner(
                &mut state.history,
                CommandSource::new(SourceView::Bookmarks, SourceAction::BookmarkTarget),
            );
            match show_source.load_query_with_runner(&query, &mut runner) {
                Ok(snapshot) => state.views.push(AppView::Show {
                    view: RenderedView::new(snapshot),
                    query,
                }),
                Err(error) => {
                    if let AppView::Bookmarks { view } = state.views.active_mut() {
                        view.show_error(error.to_string());
                    }
                }
            }
        }
        BookmarkActionResult::ReturnBack => {
            state.views.pop();
        }
        BookmarkActionResult::Quit => {}
        BookmarkActionResult::Continue => {}
        BookmarkActionResult::Create => open_bookmark_create_prompt(state),
        BookmarkActionResult::Move { name } => open_bookmark_move_prompt(state, name),
        BookmarkActionResult::Delete { name } => {
            open_bookmark_preview(state, source, BookmarkMutation::Delete { name });
        }
        _ => {}
    }
}

/// Starts the bookmark creation prompt.
pub fn open_bookmark_create_prompt(state: &mut AppState) {
    state.modes.push(InputMode::BookmarkMutation {
        kind: BookmarkMutationKind::Create,
        name: String::new(),
        revision: "@".to_owned(),
        field: BookmarkMutationField::Name,
    });
}

/// Starts the bookmark move prompt for a selected bookmark.
pub fn open_bookmark_move_prompt(state: &mut AppState, name: String) {
    state.modes.push(InputMode::BookmarkMutation {
        kind: BookmarkMutationKind::Move,
        name,
        revision: "@".to_owned(),
        field: BookmarkMutationField::Revision,
    });
}

/// Opens a typed bookmark mutation preview without executing it.
pub fn open_bookmark_preview(
    state: &mut AppState,
    source: &JjBookmarks,
    mutation: BookmarkMutation,
) {
    let preview = source.mutation_spec(&mutation).command_preview();
    let pending = match mutation {
        BookmarkMutation::Create { .. } => PendingCommandPreview::bookmark_create(preview),
        BookmarkMutation::Move { .. } => PendingCommandPreview::bookmark_move(preview),
        BookmarkMutation::Delete { .. } => PendingCommandPreview::bookmark_delete(preview),
    };
    state.modes.push(InputMode::CommandPreview { pending });
}

/// Builds a bookmark mutation from prompt fields.
pub fn bookmark_mutation_from_prompt(
    kind: BookmarkMutationKind,
    name: String,
    revision: String,
) -> Option<BookmarkMutation> {
    if name.trim().is_empty() || revision.trim().is_empty() {
        return None;
    }
    Some(match kind {
        BookmarkMutationKind::Create => BookmarkMutation::Create { name, revision },
        BookmarkMutationKind::Move => BookmarkMutation::Move { name, revision },
    })
}

/// Loads configured remotes and asks for a destination before building a preview.
pub fn open_remote_preview(state: &mut AppState, remotes: &JjGitRemote, push: bool) {
    open_remote_picker_with_runner(state, remotes, push, SystemJjCommandRunner);
}

fn open_remote_picker_with_runner(
    state: &mut AppState,
    remotes: &JjGitRemote,
    push: bool,
    runner: impl JjCommandRunner,
) {
    let AppView::Bookmarks { view } = state.views.active_mut() else {
        return;
    };
    let bookmark = if push {
        let Some(row) = view.selected_row().filter(|row| row.can_mutate()) else {
            view.show_error("select a local bookmark with one target for push dry-run");
            return;
        };
        Some(row.name.clone())
    } else {
        None
    };
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        &mut state.history,
        CommandSource::new(SourceView::Bookmarks, SourceAction::Refresh),
    );
    match remotes.load_with_runner(&mut runner) {
        Ok(remotes) if !remotes.is_empty() => {
            state.modes.push(InputMode::RemotePicker {
                names: remotes.into_iter().map(|remote| remote.name).collect(),
                selected: 0,
                bookmark,
            });
        }
        Ok(_) => {
            if let AppView::Bookmarks { view } = state.views.active_mut() {
                view.show_error("no Git remotes configured");
            }
        }
        Err(error) => {
            if let AppView::Bookmarks { view } = state.views.active_mut() {
                view.show_error(error.to_string());
            }
        }
    }
}

/// Moves through named destinations; Enter opens a preview without network effects.
pub fn handle_remote_picker(
    state: &mut AppState,
    remotes: &JjGitRemote,
    key: crossterm::event::KeyEvent,
) -> crate::state::InputModeResult {
    use crossterm::event::KeyCode;

    use crate::state::InputModeResult;
    match key.code {
        KeyCode::Esc => {
            state.modes.pop();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(InputMode::RemotePicker { selected, .. }) = state.modes.active_mut() {
                *selected = selected.saturating_sub(1);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(InputMode::RemotePicker {
                names, selected, ..
            }) = state.modes.active_mut()
            {
                *selected = (*selected + 1).min(names.len().saturating_sub(1));
            }
        }
        KeyCode::Enter => {
            if let Some(InputMode::RemotePicker {
                names,
                selected,
                bookmark,
            }) = state.modes.pop()
                && let Some(remote) = names.get(selected)
            {
                let pending = if let Some(bookmark) = bookmark {
                    PendingCommandPreview::git_push_dry_run(
                        remotes
                            .push_dry_run_spec(remote, &bookmark)
                            .command_preview(),
                    )
                } else {
                    PendingCommandPreview::git_fetch(remotes.fetch_spec(remote).command_preview())
                };
                state.modes.push(InputMode::CommandPreview { pending });
            }
        }
        _ => {}
    }
    InputModeResult::Handled
}

pub fn refresh_bookmarks(state: &mut AppState, source: &JjBookmarks) {
    refresh_bookmarks_with_runner(state, source, SystemJjCommandRunner);
}

pub fn refresh_bookmarks_with_runner(
    state: &mut AppState,
    source: &JjBookmarks,
    runner: impl JjCommandRunner,
) {
    let mut runner = RecordingJjCommandRunner::new(
        runner,
        &mut state.history,
        CommandSource::new(SourceView::Bookmarks, SourceAction::Refresh),
    );
    let result = source.load_with_runner(&mut runner);
    let AppView::Bookmarks { view } = state.views.active_mut() else {
        return;
    };
    match result {
        Ok(snapshot) => view.refresh(bookmark_view_snapshot(snapshot)),
        Err(error) => view.show_error(error.to_string()),
    }
}

fn bookmark_view_snapshot(snapshot: BookmarkSnapshot) -> BookmarkViewSnapshot {
    crate::root_views::bookmark_view_snapshot(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::InputMode;
    use crate::test_support::{SequencedRunner, log_app_view, output};

    #[test]
    fn fetch_works_without_remote_bookmarks_and_cancel_does_not_run_it() {
        let mut state = AppState::new(AppView::Bookmarks {
            view: BookmarkView::new(BookmarkViewSnapshot::new(Vec::new())),
        });
        open_remote_picker_with_runner(
            &mut state,
            &JjGitRemote::default(),
            false,
            SequencedRunner::successes(vec![output(0, "one /fixture/one\ntwo /fixture/two\n", "")]),
        );
        assert!(
            matches!(state.modes.active(), Some(InputMode::RemotePicker { names, .. }) if names.len() == 2)
        );
        handle_remote_picker(
            &mut state,
            &JjGitRemote::default(),
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Esc,
                crossterm::event::KeyModifiers::NONE,
            ),
        );
        assert!(state.modes.active().is_none());
        assert_eq!(state.history.records().count(), 1);
        assert!(
            state
                .history
                .records()
                .all(|record| record.source.action != SourceAction::GitFetch)
        );
    }

    #[test]
    fn push_requires_a_normal_local_bookmark_before_loading_remotes() {
        use jk_tui::bookmark_view::BookmarkRow;
        for row in [
            BookmarkRow::new("deleted", vec![]),
            BookmarkRow::new("conflicted", vec!["a".into(), "b".into()]),
            BookmarkRow::new("remote", vec!["a".into()]).with_remote("origin"),
        ] {
            let mut state = AppState::new(AppView::Bookmarks {
                view: BookmarkView::new(BookmarkViewSnapshot::new(vec![row])),
            });
            open_remote_picker_with_runner(
                &mut state,
                &JjGitRemote::default(),
                true,
                SequencedRunner::successes(vec![]),
            );
            assert!(state.modes.active().is_none());
            assert_eq!(state.history.records().count(), 0);
        }
    }

    #[test]
    fn push_preview_freezes_selected_local_bookmark_and_named_remote() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use jk_tui::bookmark_view::BookmarkRow;
        let mut state = AppState::new(AppView::Bookmarks {
            view: BookmarkView::new(BookmarkViewSnapshot::new(vec![BookmarkRow::new(
                "topic",
                vec!["abc".into()],
            )])),
        });
        let source = JjGitRemote::default();
        open_remote_picker_with_runner(
            &mut state,
            &source,
            true,
            SequencedRunner::successes(vec![output(0, "one /fixture/one\ntwo /fixture/two\n", "")]),
        );
        handle_remote_picker(
            &mut state,
            &source,
            KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        );
        handle_remote_picker(
            &mut state,
            &source,
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        );
        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("preview");
        };
        assert!(pending.preview.command_line.contains("--remote two"));
        assert!(pending.preview.command_line.contains("topic"));
        assert!(pending.preview.command_line.contains("--dry-run"));
        assert_eq!(state.history.records().count(), 1);
    }

    #[test]
    fn opening_bookmarks_records_machine_list_and_renders_rows() {
        let mut state = AppState::new(log_app_view("abc123"));
        open_bookmarks_with_runner(
            &mut state,
            &JjBookmarks::default(),
            SequencedRunner::successes(vec![output(
                0,
                "{\"name\":\"main\",\"target\":[\"abc123\"]}\n",
                "",
            )]),
        );

        let AppView::Bookmarks { view } = state.views.active() else {
            panic!("bookmark view expected");
        };
        assert_eq!(
            view.selected_row().map(|row| row.name.as_str()),
            Some("main")
        );
        let record = state
            .command_history()
            .records()
            .next()
            .expect("history record");
        assert_eq!(record.source.view, SourceView::Bookmarks);
        assert_eq!(record.source.action, SourceAction::BookmarkList);
    }

    #[test]
    fn delete_and_remote_actions_only_open_previews() {
        let mut state = AppState::new(log_app_view("abc123"));
        open_bookmarks_with_runner(
            &mut state,
            &JjBookmarks::default(),
            SequencedRunner::successes(vec![output(
                0,
                "{\"name\":\"main\",\"target\":[\"abc123\"]}\n{\"name\":\"main\",\"remote\":\"fixture\",\"target\":[\"abc123\"]}\n",
                "",
            )]),
        );
        apply_bookmark_action(
            &mut state,
            &JjBookmarks::default(),
            &JjShow::default(),
            BookmarkAction::Delete,
        );
        assert!(
            matches!(state.modes.active(), Some(InputMode::CommandPreview { pending })
            if pending.preview.command_line.contains("bookmark delete")
                && pending.preview.requires_confirmation())
        );
        state.modes.pop();
        apply_bookmark_action(
            &mut state,
            &JjBookmarks::default(),
            &JjShow::default(),
            BookmarkAction::Next,
        );
        open_remote_picker_with_runner(
            &mut state,
            &JjGitRemote::default(),
            false,
            SequencedRunner::successes(vec![output(0, "fixture /local/fixture\n", "")]),
        );
        handle_remote_picker(
            &mut state,
            &JjGitRemote::default(),
            crossterm::event::KeyEvent::new(
                crossterm::event::KeyCode::Enter,
                crossterm::event::KeyModifiers::NONE,
            ),
        );
        assert!(
            matches!(state.modes.active(), Some(InputMode::CommandPreview { pending })
            if pending.preview.command_line.contains("git fetch")
                && pending.preview.execution_mode == jk_core::ExecutionMode::ConfirmNetworkRead)
        );
    }
}
