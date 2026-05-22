use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

use super::*;

#[test]
fn binding_matches_key_code_and_modifiers() {
    let binding = Binding::new(
        KeyPattern::modified_char('f', KeyModifiers::CONTROL),
        Command::View(ViewCommand::PageDown),
    );

    assert!(binding.matches(key(KeyCode::Char('f'), KeyModifiers::CONTROL)));
    assert!(!binding.matches(key(KeyCode::Char('f'), KeyModifiers::NONE)));
}

#[test]
fn uppercase_bindings_accept_shifted_character_events() {
    let uppercase = Binding::new(KeyPattern::char('S'), Command::OpenStatus);
    let lowercase = Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow));

    assert!(uppercase.matches(key(KeyCode::Char('S'), KeyModifiers::SHIFT)));
    assert!(!lowercase.matches(key(KeyCode::Char('S'), KeyModifiers::SHIFT)));
    assert!(!uppercase.matches(key(KeyCode::Char('s'), KeyModifiers::NONE)));
}

#[test]
fn find_binding_returns_first_matching_command() {
    let bindings = [
        Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
        Binding::new(KeyPattern::char('q'), Command::Quit),
    ];

    assert_eq!(
        find_binding(&bindings, key(KeyCode::Char('q'), KeyModifiers::NONE)).map(Binding::command),
        Some(Command::Quit)
    );
}

#[test]
fn match_binding_sequence_reports_prefix_and_exact_fallback() {
    const BOOKMARK_CREATE: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('c')];
    let bindings = [
        Binding::new(KeyPattern::char('b'), Command::BookmarkCreate),
        Binding::sequence(BOOKMARK_CREATE, Command::BookmarkCreate),
    ];

    let pending =
        match_binding_sequence(&[&bindings], &[key(KeyCode::Char('b'), KeyModifiers::NONE)]);
    let complete = match_binding_sequence(
        &[&bindings],
        &[
            key(KeyCode::Char('b'), KeyModifiers::NONE),
            key(KeyCode::Char('c'), KeyModifiers::NONE),
        ],
    );

    assert_eq!(
        pending,
        Some(BindingMatch::Prefix {
            fallback: Some(bindings[0])
        })
    );
    assert_eq!(complete, Some(BindingMatch::Exact(bindings[1])));
    assert_eq!(bindings[1].key_label(), "bc");
}

#[test]
fn match_binding_sequence_allows_global_prefix_over_view_fallback() {
    const GIT_FETCH: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('f')];
    let global = [Binding::sequence(GIT_FETCH, Command::Fetch)];
    let view = [Binding::new(
        KeyPattern::char('g'),
        Command::View(ViewCommand::MoveFirst),
    )];

    let pending = match_binding_sequence(
        &[&global, &view],
        &[key(KeyCode::Char('g'), KeyModifiers::NONE)],
    );

    assert_eq!(
        pending,
        Some(BindingMatch::Prefix {
            fallback: Some(view[0])
        })
    );
}

#[test]
fn binding_prefix_next_labels_list_available_suffixes() {
    const GIT_FETCH: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('f')];
    const GIT_PUSH: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('p')];
    let bindings = [
        Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
        Binding::sequence(GIT_FETCH, Command::Fetch),
        Binding::sequence(GIT_PUSH, Command::Push),
    ];

    assert_eq!(
        binding_prefix_next_labels(&[&bindings], &[key(KeyCode::Char('g'), KeyModifiers::NONE)]),
        vec!["f", "p"]
    );
}

#[test]
fn key_pattern_labels_special_keys() {
    assert_eq!(KeyPattern::char(' ').label(), "Space");
    assert_eq!(
        KeyPattern::modified_char('f', KeyModifiers::CONTROL).label(),
        "C-f"
    );
    assert_eq!(KeyPattern::code(KeyCode::Down).label(), "Down");
}

#[test]
fn help_binding_match_uses_visible_help_metadata() {
    let bindings = [
        Binding::new(KeyPattern::char('q'), Command::Quit),
        Binding::new(KeyPattern::char('D'), Command::Describe),
        Binding::new(KeyPattern::char('S'), Command::OpenStatus),
    ];

    assert_eq!(
        match_help_binding_sequence(
            &[&bindings],
            &[key(KeyCode::Char('S'), KeyModifiers::NONE)],
            HelpContext::Show,
        ),
        Some(BindingMatch::Exact(bindings[2]))
    );
    assert_eq!(
        match_help_binding_sequence(
            &[&bindings],
            &[key(KeyCode::Char('D'), KeyModifiers::NONE)],
            HelpContext::Show,
        ),
        None
    );
    assert_eq!(
        match_help_binding_sequence(
            &[&bindings],
            &[key(KeyCode::Char('q'), KeyModifiers::NONE)],
            HelpContext::Log,
        ),
        None
    );
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}
