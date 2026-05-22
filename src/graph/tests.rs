use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;

use super::*;
use crate::command::{Binding, Command, CommandContext, ViewCommand, ViewEffect, find_binding};
use crate::jj::LogViewMode;
use crate::selection::Selection;
use crate::theme;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

fn log_item(text: &str, change_id: Option<&str>, commit_id: Option<&str>) -> LogItem {
    LogItem::new(
        vec![Line::from(text.to_owned())],
        change_id.map(str::to_owned),
        commit_id.map(str::to_owned),
    )
}

fn graph_view(entries: Vec<LogItem>) -> GraphView {
    GraphView::test_new(entries)
}

fn command_context() -> CommandContext<'static> {
    CommandContext {
        viewport_height: 0,
        viewport_width: 80,
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

    super::view::test_restore_selection(&mut selection, &entries, 1, Some("second".to_owned()));

    assert_eq!(selection.index(), 0);
}

#[test]
fn restore_selection_clamps_when_selected_change_disappears() {
    let entries = vec![log_item("only", Some("only"), None)];
    let mut selection = Selection::default();

    super::view::test_restore_selection(&mut selection, &entries, 3, Some("missing".to_owned()));

    assert_eq!(selection.index(), 0);
}

#[test]
fn switch_mode_preserves_selection_by_change_id() {
    let mut view = graph_view(vec![
        log_item("first", Some("first"), None),
        log_item("second", Some("second"), None),
    ]);
    view.select_next();

    view.test_switch_mode_with_loader(LogViewMode::Trunk, |_| {
        Ok(vec![
            log_item("second", Some("second"), None),
            log_item("third", Some("third"), None),
        ])
    })
    .unwrap();

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

    view.test_switch_mode_with_loader(LogViewMode::Trunk, |_| {
        Ok(vec![
            log_item("first", Some("first"), None),
            log_item("third", Some("third"), None),
        ])
    })
    .unwrap();

    assert_eq!(view.mode_label(), "trunk work");
    assert_eq!(view.test_selected_change_ids(), ["first", "third"]);
}

#[test]
fn switch_mode_error_keeps_prior_view_state() {
    let mut view = graph_view(vec![log_item("first", Some("first"), None)]);
    let previous_spec = view.spec().clone();
    let previous_mode = view.mode_label().to_owned();

    let error = view
        .test_switch_mode_with_loader(
            LogViewMode::CustomRevset("not-a-revset(".to_owned()),
            |_| Err(color_eyre::eyre::eyre!("invalid revset")),
        )
        .unwrap_err();

    assert_eq!(error.to_string(), "invalid revset");
    assert_eq!(view.spec(), &previous_spec);
    assert_eq!(view.mode_label(), previous_mode);
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
fn page_keys_move_selection_by_visible_page_with_saturating_bounds() {
    let mut view = graph_view(
        (0..5)
            .map(|index| {
                log_item(
                    &format!("row {index}"),
                    Some(&format!("change-{index}")),
                    None,
                )
            })
            .collect(),
    );
    let context = || CommandContext {
        viewport_height: 3,
        viewport_width: 80,
        search: None,
    };

    assert_eq!(
        view.execute(ViewCommand::PageDown, context()),
        ViewEffect::Handled
    );
    assert_eq!(view.selected_revision(), Some("change-2"));

    assert_eq!(
        view.execute(ViewCommand::PageDown, context()),
        ViewEffect::Handled
    );
    assert_eq!(view.selected_revision(), Some("change-4"));

    assert_eq!(
        view.execute(ViewCommand::PageUp, context()),
        ViewEffect::Handled
    );
    assert_eq!(view.selected_revision(), Some("change-2"));

    assert_eq!(
        view.execute(ViewCommand::PageUp, context()),
        ViewEffect::Handled
    );
    assert_eq!(view.selected_revision(), Some("change-0"));
}

#[test]
fn reveal_change_id_keeps_current_mode_when_change_is_visible() {
    let mut view = graph_view(vec![
        log_item("first", Some("first"), None),
        log_item("second", Some("second"), None),
    ]);

    let switched = view
        .test_reveal_change_id_with_loader("second", LogViewMode::Recent, |_| {
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
        .test_reveal_change_id_with_loader("new", LogViewMode::Recent, |_| {
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
        .test_reveal_change_id_with_loader("new", LogViewMode::Recent, |_| {
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
        ViewEffect::StatusMessage("selection only works on rows with exact change ids".to_owned())
    );
    assert!(view.test_selected_change_ids().is_empty());
}

#[test]
fn toggle_select_tracks_exact_change_ids() {
    let mut view = graph_view(vec![log_item("first", Some("change"), None)]);
    assert!(view.test_selected_change_ids().is_empty());

    assert_eq!(
        view.execute(ViewCommand::ToggleSelect, command_context()),
        ViewEffect::StatusMessage("selected change".to_owned())
    );
    assert_eq!(view.test_selected_change_ids(), ["change"]);

    assert_eq!(
        view.execute(ViewCommand::ToggleSelect, command_context()),
        ViewEffect::StatusMessage("unselected change".to_owned())
    );
    assert!(view.test_selected_change_ids().is_empty());
}

#[test]
fn refresh_preserves_exact_selection_ids() {
    let mut view = graph_view(vec![
        log_item("first", Some("change"), None),
        log_item("second", Some("another"), None),
    ]);

    view.execute(ViewCommand::ToggleSelect, command_context());
    view.select_next();
    view.execute(ViewCommand::ToggleSelect, command_context());

    assert_eq!(view.test_selected_change_ids(), ["change", "another"]);

    view.test_refresh_with_loader(|_| {
        Ok(vec![
            log_item("another", Some("another"), None),
            log_item("first", Some("change"), None),
        ])
    })
    .unwrap();

    assert_eq!(view.test_selected_change_ids(), ["change", "another"]);
}

#[test]
fn refresh_drops_disappeared_selection_ids() {
    let mut view = graph_view(vec![
        log_item("first", Some("change"), None),
        log_item("second", Some("another"), None),
        log_item("third", Some("third"), None),
    ]);

    view.execute(ViewCommand::ToggleSelect, command_context());
    view.select_next();
    view.execute(ViewCommand::ToggleSelect, command_context());
    view.select_next();
    view.execute(ViewCommand::ToggleSelect, command_context());

    assert_eq!(
        view.test_selected_change_ids(),
        ["change", "another", "third"]
    );

    view.test_refresh_with_loader(|_| {
        Ok(vec![
            log_item("second", Some("another"), None),
            log_item("other", None, None),
            log_item("change", Some("change"), None),
        ])
    })
    .unwrap();

    assert_eq!(view.test_selected_change_ids(), ["change", "another"]);
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
            "duplicate selected revision aaaaaaaa",
            "restore selected revision aaaaaaaa",
            "revert selected revision aaaaaaaa into @"
        ]
    );
}

#[test]
fn open_action_menu_uses_bare_split_for_visible_working_copy() {
    let mut view = graph_view(vec![
        log_item("@  current", Some("aaaaaaaa"), None),
        log_item("○  parent", Some("bbbbbbbb"), None),
    ]);

    let effect = view.execute(ViewCommand::OpenActionMenu, command_context());
    let action_menu = if let ViewEffect::OpenActionMenu(action_menu) = effect {
        action_menu
    } else {
        panic!("expected action menu");
    };

    assert_eq!(
        action_menu.items()[2].label(),
        "split current working-copy change @"
    );
    assert!(matches!(
        action_menu.items()[2].follow_up(),
        crate::action_menu::FollowUp::SplitCurrentWorkingCopy
    ));
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
            if parents.as_slice() == ["aaaaaaaa".to_owned(), "cccccccc".to_owned()]
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
            "duplicate selected revision aaaaaaaa",
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
    let selected =
        super::view::test_entry_lines(&log_item("first", Some("change"), None), None, true);
    let unselected =
        super::view::test_entry_lines(&log_item("first", Some("change"), None), None, false);

    assert_eq!(
        selected[0].style,
        super::view::test_explicit_selection_style()
    );
    assert_eq!(unselected[0].style, Style::default());
    assert_ne!(selected[0].style, unselected[0].style);
}

#[test]
fn current_row_highlight_preserves_rendered_foreground() {
    let view = graph_view(vec![LogItem::new(
        vec![Line::styled(
            "colored row",
            Style::default().fg(Color::LightRed),
        )],
        Some("change".to_owned()),
        None,
    )]);
    let mut terminal = Terminal::new(TestBackend::new(16, 1)).unwrap();

    terminal
        .draw(|frame| {
            view.render(frame, frame.area(), None);
        })
        .unwrap();

    let selected_cell = &terminal.backend().buffer()[(0, 0)];
    let highlight = theme::active_row_style();
    assert_eq!(highlight.fg, None);
    assert_eq!(selected_cell.fg, Color::LightRed);
    assert_eq!(selected_cell.bg, highlight.bg.unwrap());
    assert!(!selected_cell.modifier.contains(Modifier::REVERSED));
    assert!(selected_cell.modifier.contains(Modifier::BOLD));
}

#[test]
fn explicit_selection_preserves_rendered_foreground() {
    let selected = super::view::test_entry_lines(
        &LogItem::new(
            vec![Line::styled(
                "colored row",
                Style::default().fg(Color::LightBlue),
            )],
            Some("change".to_owned()),
            None,
        ),
        None,
        true,
    );

    assert_eq!(selected[0].style.fg, Some(Color::LightBlue));
    assert!(selected[0].style.bg.is_some());
    assert!(selected[0].style.add_modifier.contains(Modifier::BOLD));
}

#[test]
fn selected_revision_uses_exact_row_revision() {
    let view = GraphView::test_new(vec![
        log_item("@  has id", Some("abcd"), None),
        log_item("○  no id", None, None),
    ]);

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
