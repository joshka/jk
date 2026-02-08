//! Revision selection helpers for log-like views.
//!
//! This module builds row-to-revision mappings from rendered output so selection-driven commands
//! keep working across wrapped sections and graph lines.

use std::collections::HashMap;

use crate::flow::{FlowAction, plan_command};
use crate::jj;
use crate::keys::KeyBinding;
use crossterm::event::KeyEvent;

/// Build initial startup action from CLI tokens.
pub(crate) fn startup_action(startup_tokens: &[String]) -> FlowAction {
    if startup_tokens.is_empty() {
        FlowAction::Execute(vec!["log".to_string()])
    } else {
        let startup_command = startup_tokens.join(" ");
        plan_command(&startup_command, None)
    }
}

/// Build a line-indexed revision map for rendered command output.
///
/// Mapping is only populated for `log` commands; other commands return an all-`None` map.
pub(crate) fn derive_row_revision_map(tokens: &[String], lines: &[String]) -> Vec<Option<String>> {
    if !matches!(tokens.first().map(String::as_str), Some("log")) {
        return vec![None; lines.len()];
    }

    let Some(metadata_tokens) = metadata_log_tokens(tokens) else {
        return vec![None; lines.len()];
    };

    let revisions = match jj::run_plain(&metadata_tokens) {
        Ok(result) if result.success => parse_log_revisions(&result.output),
        _ => Vec::new(),
    };

    build_row_revision_map(lines, &revisions)
}

/// Derive metadata-safe `log` command tokens for revision extraction.
pub(crate) fn metadata_log_tokens(tokens: &[String]) -> Option<Vec<String>> {
    if !matches!(tokens.first().map(String::as_str), Some("log")) {
        return None;
    }

    let mut metadata_tokens = vec![
        "log".to_string(),
        "--no-graph".to_string(),
        "-T".to_string(),
        "change_id.short() ++ \" \" ++ commit_id.short()".to_string(),
    ];

    let mut skip_next_value = false;
    for token in tokens.iter().skip(1) {
        if skip_next_value {
            skip_next_value = false;
            continue;
        }

        match token.as_str() {
            "-T" | "--template" => {
                skip_next_value = true;
            }
            "--graph" | "--no-graph" | "-p" | "--patch" => {}
            value => metadata_tokens.push(value.to_string()),
        }
    }

    Some(metadata_tokens)
}

/// Parse change/commit identifiers from plain log output lines.
pub(crate) fn parse_log_revisions(lines: &[String]) -> Vec<String> {
    let mut revisions = Vec::new();
    for line in lines {
        let stripped = strip_ansi(line);
        let Some(token) = stripped.split_whitespace().next().map(trim_revision_token) else {
            continue;
        };
        if is_change_id(token) || is_commit_id(token) {
            revisions.push(token.to_string());
        }
    }
    revisions
}

/// Associate each rendered line with the nearest visible revision.
///
/// Explicit revision tokens take precedence. For graph-only rows, mapping advances through ordered
/// metadata revisions to keep multiline graph output selectable.
pub(crate) fn build_row_revision_map(
    lines: &[String],
    ordered_revisions: &[String],
) -> Vec<Option<String>> {
    let mut revision_positions = HashMap::new();
    for (index, revision) in ordered_revisions.iter().enumerate() {
        revision_positions.insert(revision.clone(), index);
    }

    let mut map = Vec::with_capacity(lines.len());
    let mut current: Option<String> = None;
    let mut next_ordinal = 0usize;

    for line in lines {
        if let Some(explicit) = extract_revision(line) {
            if let Some(position) = revision_positions.get(&explicit) {
                current = Some(explicit);
                next_ordinal = (*position + 1).max(next_ordinal);
            } else if ordered_revisions.is_empty() {
                current = Some(explicit);
            }
        } else if looks_like_graph_commit_row(line) && next_ordinal < ordered_revisions.len() {
            current = ordered_revisions.get(next_ordinal).cloned();
            next_ordinal += 1;
        }

        map.push(current.clone());
    }

    map
}

/// Heuristic for lines that visually represent graph commit rows.
pub(crate) fn looks_like_graph_commit_row(line: &str) -> bool {
    for ch in line.chars() {
        if ch.is_whitespace()
            || matches!(
                ch,
                '│' | '┃'
                    | '┆'
                    | '┊'
                    | '┄'
                    | '┈'
                    | '─'
                    | '┬'
                    | '┴'
                    | '┼'
                    | '╭'
                    | '╮'
                    | '╯'
                    | '╰'
                    | '|'
                    | '/'
                    | '\\'
            )
        {
            continue;
        }
        return matches!(ch, '@' | '○' | '◉' | '●' | '◆' | '◌' | 'x' | 'X' | '*');
    }

    false
}

/// Trim a string to terminal width while preserving ANSI escape integrity.
pub(crate) fn trim_to_width(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut visible = 0usize;

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}'
            && let Some('[') = chars.peek().copied()
        {
            result.push(ch);
            result.push('[');
            let _ = chars.next();
            for control in chars.by_ref() {
                result.push(control);
                if ('@'..='~').contains(&control) {
                    break;
                }
            }
            continue;
        }

        if visible >= width {
            break;
        }

        result.push(ch);
        visible += 1;
    }

    if visible >= width {
        loop {
            if !matches!(chars.peek(), Some('\u{1b}')) {
                break;
            }

            let Some(escape) = chars.next() else {
                break;
            };
            if !matches!(chars.peek(), Some('[')) {
                continue;
            }
            result.push(escape);
            result.push('[');
            let _ = chars.next();

            for control in chars.by_ref() {
                result.push(control);
                if ('@'..='~').contains(&control) {
                    break;
                }
            }
        }
    }

    result
}

/// Remove ANSI escape sequences from text.
pub(crate) fn strip_ansi(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}'
            && let Some('[') = chars.peek().copied()
        {
            let _ = chars.next();
            for control in chars.by_ref() {
                if ('@'..='~').contains(&control) {
                    break;
                }
            }
            continue;
        }

        result.push(ch);
    }

    result
}

/// Extract selected revision token from one rendered log line.
pub(crate) fn extract_revision(line: &str) -> Option<String> {
    let stripped = strip_ansi(line);
    let tokens: Vec<&str> = stripped
        .split_whitespace()
        .map(trim_revision_token)
        .filter(|token| !token.is_empty())
        .collect();

    let commit_index = tokens.iter().position(|token| is_commit_id(token))?;

    for token in &tokens[..commit_index] {
        if is_change_id(token) {
            return Some((*token).to_string());
        }
    }

    tokens.get(commit_index).map(|token| (*token).to_string())
}

/// Trim punctuation around a candidate revision token.
pub(crate) fn trim_revision_token(token: &str) -> &str {
    token.trim_matches(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '/'))
}

/// Return whether token looks like a short commit hash.
pub(crate) fn is_commit_id(value: &str) -> bool {
    value.len() >= 8 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

/// Return whether token looks like a change id with optional counter suffix.
pub(crate) fn is_change_id(value: &str) -> bool {
    let Some((head, _counter)) = value.split_once('/') else {
        return value.len() >= 8 && value.chars().all(|ch| ch.is_ascii_lowercase());
    };

    !head.is_empty()
        && head.chars().all(|ch| ch.is_ascii_lowercase())
        && value.len() >= 8
        && value
            .rsplit_once('/')
            .map(|(_, suffix)| suffix.chars().all(|ch| ch.is_ascii_digit()))
            .unwrap_or(false)
}

/// Return whether any binding in the slice matches the current key event.
pub(crate) fn matches_any(bindings: &[KeyBinding], key: KeyEvent) -> bool {
    bindings.iter().any(|binding| binding.matches(key))
}
