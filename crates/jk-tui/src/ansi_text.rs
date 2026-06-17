//! ANSI text helpers used when rendered terminal output must be inspected.
//!
//! The TUI normally preserves `jj` output as terminal-styled text. These helpers are for narrow
//! cases where state logic needs to reason about visible characters without carrying terminal
//! escape sequences into graph checks.

/// Removes CSI-style ANSI escape sequences from terminal text.
pub fn strip_ansi(text: &str) -> String {
    let mut stripped = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\u{1b}' {
            stripped.push(ch);
            continue;
        }

        if chars.next_if_eq(&'[').is_none() {
            continue;
        }

        for code in chars.by_ref() {
            if ('@'..='~').contains(&code) {
                break;
            }
        }
    }

    stripped
}
