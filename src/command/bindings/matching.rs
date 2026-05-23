use crossterm::event::KeyEvent;

use super::Binding;
use crate::command::HelpContext;
use crate::help::command_is_visible_in_help;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingMatch {
    /// A complete binding with no longer available sequence sharing the same prefix.
    Exact(Binding),
    /// A valid prefix for longer bindings, optionally with an exact binding to run on timeout.
    ///
    /// The fallback is not executed by this module. `App` owns the prefix timer
    /// and decides whether to keep collecting keys or apply the exact command.
    Prefix {
        /// Exact binding that may run if the app's prefix timeout expires.
        fallback: Option<Binding>,
    },
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

/// Match pending keys against binding groups after applying an availability filter.
///
/// This is the core prefix-matching routine used by both normal dispatch and
/// help-mode dispatch. It preserves binding-table order for exact matches while
/// still surfacing a pending prefix whenever any longer sequence remains viable.
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

            if !binding.matches_prefix(keys) {
                continue;
            }

            if binding.sequence_len() == keys.len() {
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

/// Return unique next-key labels for every binding that still matches the prefix.
///
/// This helper stays pure and ordered so app status text and help prefix hints
/// can stay aligned without sharing mutable dispatch state.
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
            if !is_available(*binding) || !binding.matches_prefix(keys) {
                continue;
            }

            let Some(pattern) = binding.next_pattern(keys.len()) else {
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
