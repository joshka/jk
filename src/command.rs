//! Command vocabulary, binding metadata, and view dispatch effects.
//!
//! This module owns the app/view command vocabulary, key patterns and labels,
//! key-sequence matching, and the effects views may return to app dispatch.
//! Keep help/menu presentation policy in `help.rs`; this module only exposes the
//! command metadata and filtered match helpers those projections need.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action_menu::ActionMenu;
use crate::copy::CopyOption;
use crate::help::command_is_visible_in_help;
use crate::jj::JjCommand;
use crate::jj_actions::JjOperationRecoveryKind;
use crate::search::SearchQuery;

#[cfg(test)]
pub use crate::help::HelpSectionKind;
/// Help projection types re-exported from the command vocabulary surface.
///
/// Command tables are the source of key identity. `help.rs` decides which rows
/// are visible for a context and how they are grouped for display.
pub use crate::help::{HelpContext, HelpRow, HelpSection, project_help};

/// App-level dispatch vocabulary for global bindings and view-facing effects.
///
/// `App` matches these variants through top-level binding groups first. A
/// `Command::View` value is routed to the active view as a [`ViewCommand`].
/// Add a variant here only when app dispatch or a shared binding table needs a
/// stable command identity; feature-local policy still belongs in the owning
/// view or app submodule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Quit,
    Help,
    SearchPrompt,
    PromptLogRevset,
    OpenStatus,
    OpenResolve,
    OpenBookmarks,
    OpenWorkspaces,
    OpenOperationLog,
    OperationUndo,
    OperationRedo,
    Edit,
    NextEdit,
    PrevEdit,
    Describe,
    Commit,
    BookmarkCreate,
    BookmarkSet,
    BookmarkMove,
    BookmarkRename,
    BookmarkDelete,
    BookmarkForget,
    BookmarkTrack,
    BookmarkUntrack,
    Fetch,
    FetchRemote,
    Push,
    Copy,
    ViewFormat,
    Refresh,
    Back,
    SwitchLog,
    SwitchDefault,
    View(ViewCommand),
}

/// View-local commands that may inspect the current viewport and search state.
///
/// These commands stay on the presentation side of the boundary: they can
/// return a `ViewEffect`, but `App` owns the actual state transition that
/// follows. The active view may ignore commands it does not support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewCommand {
    CycleMode,
    NewTrunk,
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    MoveFirst,
    MoveLast,
    ToggleWrap,
    ScrollLeft,
    ScrollRight,
    NextFile,
    PreviousFile,
    OpenFiles,
    OpenItem,
    OpenShow,
    OpenDiff,
    StartSearch,
    NextSearchMatch,
    PreviousSearchMatch,
    ToggleSelect,
    OpenActionMenu,
    Copy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Binding {
    key: KeySequence,
    command: Command,
}

impl Binding {
    /// Bind one key pattern to a command identity.
    ///
    /// Bindings are metadata only. They do not execute commands, mutate pending
    /// prefix state, or decide whether a command is currently visible in help.
    pub const fn new(key: KeyPattern, command: Command) -> Self {
        Self {
            key: KeySequence::Single(key),
            command,
        }
    }

    /// Bind a fixed multi-key sequence to a command identity.
    ///
    /// The sequence slice must be static because binding tables are static.
    /// Prefix timeout and fallback behavior are owned by `App`, not by binding
    /// metadata.
    pub const fn sequence(keys: &'static [KeyPattern], command: Command) -> Self {
        Self {
            key: KeySequence::Multi(keys),
            command,
        }
    }

    #[cfg(test)]
    pub fn matches(self, key: KeyEvent) -> bool {
        self.key.matches(key)
    }

    pub fn command(self) -> Command {
        self.command
    }

    /// Return the display label for this binding's full key pattern.
    ///
    /// Labels are reused by help rows, status hints, and pending-prefix
    /// messages, so changing this output is user-visible.
    pub fn key_label(self) -> String {
        self.key.label()
    }
}

impl Command {
    /// Return the jj operation recovery kind represented by this global command, if any.
    ///
    /// Recovery target availability belongs to the operation-log feature. This
    /// conversion only keeps the shared command identity connected to the app
    /// action flow once a recovery command has already been accepted.
    pub fn operation_recovery(self) -> Option<JjOperationRecoveryKind> {
        match self {
            Self::OperationUndo => Some(JjOperationRecoveryKind::Undo),
            Self::OperationRedo => Some(JjOperationRecoveryKind::Redo),
            Self::Quit
            | Self::Help
            | Self::SearchPrompt
            | Self::PromptLogRevset
            | Self::OpenStatus
            | Self::OpenResolve
            | Self::OpenBookmarks
            | Self::OpenWorkspaces
            | Self::OpenOperationLog
            | Self::Edit
            | Self::NextEdit
            | Self::PrevEdit
            | Self::Describe
            | Self::Commit
            | Self::BookmarkCreate
            | Self::BookmarkSet
            | Self::BookmarkMove
            | Self::BookmarkRename
            | Self::BookmarkDelete
            | Self::BookmarkForget
            | Self::BookmarkTrack
            | Self::BookmarkUntrack
            | Self::Fetch
            | Self::FetchRemote
            | Self::Push
            | Self::Copy
            | Self::ViewFormat
            | Self::Refresh
            | Self::Back
            | Self::SwitchLog
            | Self::SwitchDefault
            | Self::View(_) => None,
        }
    }
}

/// One physical key pattern used by a binding table entry.
///
/// Matching is exact on key code and modifiers except for crossterm's shifted
/// printable-character events, where the shifted character may appear both in
/// the key code and as a `SHIFT` modifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPattern {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyPattern {
    /// Construct an exact key pattern from the terminal key code and modifiers.
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Construct an unmodified printable-character key pattern.
    pub const fn char(character: char) -> Self {
        Self::new(KeyCode::Char(character), KeyModifiers::NONE)
    }

    /// Construct a printable-character key pattern with explicit modifiers.
    pub const fn modified_char(character: char, modifiers: KeyModifiers) -> Self {
        Self::new(KeyCode::Char(character), modifiers)
    }

    /// Construct an unmodified non-character key pattern.
    pub const fn code(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    fn matches(self, key: KeyEvent) -> bool {
        key.code == self.code
            && (key.modifiers == self.modifiers
                || (self.modifiers.is_empty()
                    && key.modifiers == KeyModifiers::SHIFT
                    && shifted_character_is_encoded_in_key_code(self.code)))
    }

    /// Human-readable key label used in help, status hints, and pending-prefix messages.
    pub fn label(self) -> String {
        let code = match self.code {
            KeyCode::Backspace => "Backspace".to_owned(),
            KeyCode::Down => "Down".to_owned(),
            KeyCode::End => "End".to_owned(),
            KeyCode::Enter => "Enter".to_owned(),
            KeyCode::Esc => "Esc".to_owned(),
            KeyCode::Home => "Home".to_owned(),
            KeyCode::Left => "Left".to_owned(),
            KeyCode::PageDown => "PageDown".to_owned(),
            KeyCode::PageUp => "PageUp".to_owned(),
            KeyCode::Right => "Right".to_owned(),
            KeyCode::Up => "Up".to_owned(),
            KeyCode::Char(' ') => "Space".to_owned(),
            KeyCode::Char(character) => character.to_string(),
            _ => format!("{:?}", self.code),
        };

        if self.modifiers.is_empty() {
            code
        } else {
            format!("{}-{code}", key_modifier_label(self.modifiers))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeySequence {
    /// A one-key binding that can complete immediately.
    Single(KeyPattern),
    /// A fixed sequence whose full label and prefix behavior are user-visible.
    Multi(&'static [KeyPattern]),
}

impl KeySequence {
    #[cfg(test)]
    fn matches(self, key: KeyEvent) -> bool {
        match self {
            Self::Single(pattern) => pattern.matches(key),
            Self::Multi([pattern]) => pattern.matches(key),
            Self::Multi(_) => false,
        }
    }

    fn label(self) -> String {
        match self {
            Self::Single(pattern) => pattern.label(),
            // Plain character chords render compactly for help and prefix
            // fallback labels, while modified/non-character chords keep spaces
            // between physical key labels.
            Self::Multi(patterns) if patterns.iter().all(|pattern| pattern.is_plain_char()) => {
                patterns
                    .iter()
                    .filter_map(|pattern| pattern.plain_char())
                    .collect()
            }
            Self::Multi(patterns) => patterns
                .iter()
                .map(|pattern| pattern.label())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    fn len(self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multi(patterns) => patterns.len(),
        }
    }

    fn matches_prefix(self, keys: &[KeyEvent]) -> bool {
        if keys.len() > self.len() {
            return false;
        }

        match self {
            Self::Single(pattern) => keys
                .first()
                .is_some_and(|key| keys.len() == 1 && pattern.matches(*key)),
            Self::Multi(patterns) => keys
                .iter()
                .zip(patterns)
                .all(|(key, pattern)| pattern.matches(*key)),
        }
    }

    fn next_pattern(self, key_count: usize) -> Option<KeyPattern> {
        match self {
            Self::Single(_) => None,
            Self::Multi(patterns) => patterns.get(key_count).copied(),
        }
    }
}

fn shifted_character_is_encoded_in_key_code(code: KeyCode) -> bool {
    // Some terminals report shifted printable characters as both
    // `KeyModifiers::SHIFT` and the already-shifted `KeyCode::Char`.
    // Accept that encoding only for printable characters whose shifted form is
    // visible in the key code; control/alt/explicit shift bindings remain exact.
    matches!(
        code,
        KeyCode::Char(character)
            if character.is_ascii_uppercase()
                || (!character.is_ascii_alphanumeric() && !character.is_ascii_whitespace())
    )
}

impl KeyPattern {
    fn is_plain_char(self) -> bool {
        matches!(self.code, KeyCode::Char(_)) && self.modifiers.is_empty()
    }

    fn plain_char(self) -> Option<char> {
        match self.code {
            KeyCode::Char(character) if self.modifiers.is_empty() => Some(character),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingMatch {
    /// A complete binding with no longer available sequence sharing the same prefix.
    Exact(Binding),
    /// A valid prefix for longer bindings, optionally with an exact binding to run on timeout.
    ///
    /// The fallback is not executed by this module. `App` owns the prefix timer
    /// and decides whether to keep collecting keys or apply the exact command.
    Prefix { fallback: Option<Binding> },
}

/// Snapshot of the live viewport and search state for one view dispatch.
///
/// `App` rebuilds this for each key event, so view code must treat it as
/// read-only input for the current dispatch instead of retained state.
pub struct CommandContext<'a> {
    /// Current content viewport height in terminal rows.
    pub viewport_height: u16,
    /// Current content viewport width in terminal columns.
    pub viewport_width: u16,
    /// Active search query, if search is currently scoped to the view.
    pub search: Option<&'a SearchQuery>,
}

impl CommandContext<'_> {
    /// Return the page jump size used by view-local page movement.
    ///
    /// Page movement keeps one row of overlap and never returns zero, even for
    /// very small terminal viewports.
    pub fn page_size(&self) -> usize {
        usize::from(self.viewport_height.saturating_sub(1).max(1))
    }
}

/// One-way output from a view command back to the app dispatcher.
///
/// The app interprets these effects and performs the resulting navigation,
/// refresh, status update, search update, copy menu, or action menu transition.
/// Views do not mutate app-owned state directly or run `jj` mutation flows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewEffect {
    Ignored,
    Handled,
    StatusMessage(String),
    StatusError(String),
    RunNewTrunk,
    OpenDetail(JjCommand, String),
    OpenView(crate::jj::ViewSpec),
    SearchMoved,
    SearchStarted { matches: usize },
    CopyOptions(Vec<CopyOption>),
    OpenActionMenu(ActionMenu),
}

#[cfg(test)]
pub fn find_binding(bindings: &[Binding], key: KeyEvent) -> Option<Binding> {
    bindings
        .iter()
        .copied()
        .find(|binding| binding.matches(key))
}

/// Match pending key events against binding groups in priority order.
///
/// The matcher is pure and does not apply timeouts. If both an exact binding
/// and a longer binding share a prefix, callers receive `Prefix { fallback }`
/// so `App` can wait briefly before running the fallback. Earlier binding
/// groups win only among exact matches; any longer available sequence keeps the
/// prefix pending.
pub fn match_binding_sequence(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
) -> Option<BindingMatch> {
    match_binding_sequence_by(binding_groups, keys, |_| true)
}

/// Match pending key events against commands visible in the active help context.
///
/// This keeps help/prefix hints aligned with `help.rs` projection without
/// letting help visibility change the underlying command tables or dispatch
/// behavior.
pub fn match_help_binding_sequence(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    context: HelpContext,
) -> Option<BindingMatch> {
    match_binding_sequence_by(binding_groups, keys, |binding| {
        command_is_visible_in_help(binding.command(), context)
    })
}

/// Return unique next-key labels for bindings that continue the pending prefix.
///
/// Labels preserve binding-table order and are deduplicated by rendered label so
/// status hints do not repeat the same next key.
pub fn binding_prefix_next_labels(binding_groups: &[&[Binding]], keys: &[KeyEvent]) -> Vec<String> {
    binding_prefix_next_labels_by(binding_groups, keys, |_| true)
}

/// Return unique next-key labels after applying active help-context visibility.
///
/// Use this for help and menu-adjacent prefix hints. Dispatch should use
/// [`binding_prefix_next_labels`] so hidden help rows do not disable commands.
pub fn help_binding_prefix_next_labels(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    context: HelpContext,
) -> Vec<String> {
    binding_prefix_next_labels_by(binding_groups, keys, |binding| {
        command_is_visible_in_help(binding.command(), context)
    })
}

fn match_binding_sequence_by(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    is_available: impl Fn(Binding) -> bool,
) -> Option<BindingMatch> {
    if keys.is_empty() {
        return None;
    }

    // Keep the first exact match in binding-table priority order, but do not
    // let it hide a longer sequence that still matches the pending prefix.
    let mut exact = None;
    let mut has_prefix = false;

    for bindings in binding_groups {
        for binding in *bindings {
            if !is_available(*binding) {
                continue;
            }

            if !binding.key.matches_prefix(keys) {
                continue;
            }

            if binding.key.len() == keys.len() {
                exact.get_or_insert(*binding);
            } else {
                has_prefix = true;
            }
        }
    }

    if has_prefix {
        Some(BindingMatch::Prefix { fallback: exact })
    } else {
        exact.map(BindingMatch::Exact)
    }
}

fn binding_prefix_next_labels_by(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    is_available: impl Fn(Binding) -> bool,
) -> Vec<String> {
    if keys.is_empty() {
        return Vec::new();
    }

    let mut labels = Vec::new();
    for bindings in binding_groups {
        for binding in *bindings {
            if !is_available(*binding) || !binding.key.matches_prefix(keys) {
                continue;
            }

            let Some(pattern) = binding.key.next_pattern(keys.len()) else {
                continue;
            };
            let label = pattern.label();
            // Deduplicate by user-visible label rather than key identity so
            // hints stay stable when two commands share the same next key.
            if !labels.iter().any(|existing| existing == &label) {
                labels.push(label);
            }
        }
    }
    labels
}

fn key_modifier_label(modifiers: KeyModifiers) -> &'static str {
    if modifiers == KeyModifiers::CONTROL {
        "C"
    } else if modifiers == KeyModifiers::SHIFT {
        "S"
    } else if modifiers == KeyModifiers::ALT {
        "A"
    } else {
        "M"
    }
}

#[cfg(test)]
mod tests {
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
            find_binding(&bindings, key(KeyCode::Char('q'), KeyModifiers::NONE))
                .map(Binding::command),
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
            binding_prefix_next_labels(
                &[&bindings],
                &[key(KeyCode::Char('g'), KeyModifiers::NONE)]
            ),
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
                HelpContext::Graph,
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
}
