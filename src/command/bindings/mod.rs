use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::command::Command;

mod matching;

#[cfg(test)]
pub use matching::find_binding;
pub use matching::{
    BindingMatch, binding_prefix_next_labels, help_binding_prefix_next_labels,
    match_binding_sequence, match_help_binding_sequence,
};

/// One physical key pattern used by a binding table entry.
///
/// Matching is exact on key code and modifiers except for crossterm's shifted
/// printable-character events, where the shifted character may appear both in
/// the key code and as a `SHIFT` modifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPattern {
    /// Terminal key code that must match.
    code: KeyCode,
    /// Terminal modifiers that must match, subject to shifted-character normalization.
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

    pub fn matches(self, key: KeyEvent) -> bool {
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

    pub fn is_plain_char(self) -> bool {
        matches!(self.code, KeyCode::Char(_)) && self.modifiers.is_empty()
    }

    pub fn plain_char(self) -> Option<char> {
        match self.code {
            KeyCode::Char(character) if self.modifiers.is_empty() => Some(character),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Binding {
    /// Physical key pattern or sequence accepted by this binding.
    key: KeySequence,
    /// Stable command identity routed by the app or active view.
    command: Command,
}

impl Binding {
    /// Bind one printable character to a command identity.
    pub const fn char(character: char, command: Command) -> Self {
        Self::new(KeyPattern::char(character), command)
    }

    /// Bind one non-character key code to a command identity.
    pub const fn code(code: KeyCode, command: Command) -> Self {
        Self::new(KeyPattern::code(code), command)
    }

    /// Bind a two-key printable-character chord to a command identity.
    pub const fn chord(prefix: char, suffix: char, command: Command) -> Self {
        Self {
            key: KeySequence::Chord(KeyPattern::char(prefix), KeyPattern::char(suffix)),
            command,
        }
    }

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

    pub fn matches_prefix(self, keys: &[KeyEvent]) -> bool {
        self.key.matches_prefix(keys)
    }

    pub fn sequence_len(self) -> usize {
        self.key.len()
    }

    pub fn next_pattern(self, key_count: usize) -> Option<KeyPattern> {
        self.key.next_pattern(key_count)
    }
}

/// Physical key shape used by a binding table entry.
///
/// Binding metadata stays separate from dispatch state. `App` owns prefix
/// timing and fallback behavior; this enum only records the exact key pattern.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeySequence {
    /// A one-key binding that can complete immediately.
    Single(KeyPattern),
    /// A fixed two-key chord stored directly in binding metadata.
    Chord(KeyPattern, KeyPattern),
    /// A fixed sequence whose full label and prefix behavior are user-visible.
    Multi(&'static [KeyPattern]),
}

impl KeySequence {
    #[cfg(test)]
    fn matches(self, key: KeyEvent) -> bool {
        match self {
            Self::Single(pattern) => pattern.matches(key),
            Self::Chord(pattern, _) => pattern.matches(key),
            Self::Multi([pattern]) => pattern.matches(key),
            Self::Multi(_) => false,
        }
    }

    fn label(self) -> String {
        match self {
            Self::Single(pattern) => pattern.label(),
            Self::Chord(first, second) if first.is_plain_char() && second.is_plain_char() => {
                [first, second]
                    .into_iter()
                    .filter_map(KeyPattern::plain_char)
                    .collect()
            }
            Self::Chord(first, second) => format!("{} {}", first.label(), second.label()),
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
            Self::Chord(_, _) => 2,
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
            Self::Chord(first, second) => {
                let patterns = [first, second];
                keys.iter()
                    .zip(patterns)
                    .all(|(key, pattern)| pattern.matches(*key))
            }
            Self::Multi(patterns) => keys
                .iter()
                .zip(patterns)
                .all(|(key, pattern)| pattern.matches(*key)),
        }
    }

    fn next_pattern(self, key_count: usize) -> Option<KeyPattern> {
        match self {
            Self::Single(_) => None,
            Self::Chord(first, second) => [first, second].get(key_count).copied(),
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
