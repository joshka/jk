//! Alignment between rendered `jj` output and semantic entries.
//!
//! The rendered pass keeps user-facing `jj` colors, templates, and graph layout. This module
//! detects commit rows in that rendered text so semantic entries can navigate and highlight the
//! matching rendered line.

use jk_core::LogEntry;

use super::JjLogError;

/// Assigns rendered line numbers to semantic entries.
pub(super) fn assign_rendered_lines(
    entries: Vec<LogEntry>,
    rendered: &str,
) -> Result<Vec<LogEntry>, JjLogError> {
    let rendered_lines = commit_row_lines(rendered).collect::<Vec<_>>();
    let entry_count = entries.len();
    if rendered_lines.len() != entry_count {
        return Err(JjLogError::RenderedEntryMismatch {
            rendered_rows: rendered_lines.len(),
            entries: entry_count,
        });
    }

    let entries = entries
        .into_iter()
        .zip(rendered_lines)
        .map(|(entry, rendered_line)| entry.with_rendered_line(rendered_line))
        .collect();

    Ok(entries)
}

/// Returns rendered line indexes that look like commit rows.
fn commit_row_lines(rendered: &str) -> impl Iterator<Item = usize> + '_ {
    rendered
        .lines()
        .enumerate()
        .filter_map(|(index, line)| is_commit_row(line).then_some(index))
}

/// Returns whether a visible rendered line starts at a jj commit marker.
fn is_commit_row(line: &str) -> bool {
    // This deliberately duplicates the small part of jj's rendered graph shape jk needs until this
    // boundary can use jj internals directly. If jj adds or changes commit row symbols, fail the
    // rendered/semantic alignment instead of silently assigning selection to the wrong row.
    let stripped = strip_ansi(line);
    let graph_item = stripped
        .trim_start()
        .chars()
        .find(|character| !is_graph_prefix(*character));

    graph_item.is_some_and(is_commit_marker)
}

/// Returns whether a graph item character represents a commit row.
const fn is_commit_marker(character: char) -> bool {
    matches!(character, '@' | '○' | '◆' | '×' | '+')
}

/// Returns whether a character can appear before a commit marker in the graph.
const fn is_graph_prefix(character: char) -> bool {
    matches!(
        character,
        ' ' | '│' | '─' | '├' | '╭' | '╮' | '╯' | '╰' | '╲' | '╱'
    )
}

/// Removes ANSI control sequences before inspecting visible graph characters.
fn strip_ansi(text: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_rendered_lines_from_commit_rows() -> Result<(), JjLogError> {
        let entries = vec![
            LogEntry::new("aaa", "111", "first"),
            LogEntry::new("bbb", "222", "second"),
            LogEntry::new("ccc", "333", "third"),
        ];
        let rendered = concat!(
            "\u{1b}[1m@  aaa first\u{1b}[0m\n",
            "│  first body\n",
            "○  bbb second\n",
            "│  second body\n",
            "│ ◆  ccc third\n",
            "~\n",
        );

        let entries = assign_rendered_lines(entries, rendered)?;

        assert_eq!(entries[0].rendered_line(), 0);
        assert_eq!(entries[1].rendered_line(), 2);
        assert_eq!(entries[2].rendered_line(), 4);
        Ok(())
    }

    #[test]
    fn commit_rows_can_start_after_graph_prefixes() {
        assert!(is_commit_row("│ ○  side branch"));
        assert!(is_commit_row("│ │ ×  hidden branch"));
        assert!(is_commit_row("├─╮ @  merge graph"));
        assert!(!is_commit_row("│ │  description body"));
        assert!(!is_commit_row("~  (elided revisions)"));
    }

    #[test]
    fn rejects_rendered_output_with_missing_commit_rows() {
        let entries = vec![
            LogEntry::new("aaa", "111", "first"),
            LogEntry::new("bbb", "222", "second"),
        ];
        let rendered = "@  aaa first\n│  missing second row\n";

        let error = assign_rendered_lines(entries, rendered).err();

        assert!(matches!(
            error,
            Some(JjLogError::RenderedEntryMismatch {
                rendered_rows: 1,
                entries: 2,
            })
        ));
    }

    #[test]
    fn rejects_rendered_output_with_extra_commit_rows() {
        let entries = vec![LogEntry::new("aaa", "111", "first")];
        let rendered = "@  aaa first\n○  bbb second\n";

        let error = assign_rendered_lines(entries, rendered).err();

        assert!(matches!(
            error,
            Some(JjLogError::RenderedEntryMismatch {
                rendered_rows: 2,
                entries: 1,
            })
        ));
    }
}
