//! Semantic log parsing from a narrow `jj` template.
//!
//! The template emits one JSON commit object plus one JSON details string per commit row. The
//! parser accepts that stream even when it is embedded in rendered graph lines, because `jj` may
//! still print graph prefixes around the template output.

use jk_core::LogEntry;
use serde::Deserialize;

use super::JjLogError;

/// `jj` template used to emit semantic records for navigation.
pub(super) const LOG_TEMPLATE: &str = concat!(
    "json(self)",
    " ++ \"\\t\" ++ ",
    "json(description.remove_prefix(description.first_line()).trim_start())",
    " ++ \"\\n\"",
);

/// Commit fields emitted by jj's `json(self)` template expression.
#[derive(Debug, Deserialize)]
struct JjCommit {
    change_id: String,
    commit_id: String,
    description: String,
}

/// Parses semantic log records emitted by [`LOG_TEMPLATE`].
pub(super) fn parse_log_json_lines(stdout: &str) -> Result<Vec<LogEntry>, JjLogError> {
    let mut entries = Vec::new();

    for (index, line) in stdout.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let Some(json_start) = line.find('{') else {
            continue;
        };

        let (commit_json, details_json) = split_commit_record(index + 1, &line[json_start..])?;
        let commit = parse_commit(index + 1, commit_json)?;
        let details = parse_details(index + 1, details_json)?;
        entries.push(
            LogEntry::new(commit.change_id, commit.commit_id, commit.description)
                .with_details(details),
        );
    }

    Ok(entries)
}

/// Splits one template record into commit JSON and details JSON.
fn split_commit_record(line: usize, text: &str) -> Result<(&str, &str), JjLogError> {
    text.split_once('\t')
        .ok_or(JjLogError::MissingDetails { line })
}

/// Parses the commit object half of one semantic template record.
fn parse_commit(line: usize, text: &str) -> Result<JjCommit, JjLogError> {
    serde_json::from_str(text).map_err(|source| JjLogError::Parse { line, source })
}

/// Parses the details string half of one semantic template record.
fn parse_details(line: usize, text: &str) -> Result<String, JjLogError> {
    serde_json::from_str(text).map_err(|source| JjLogError::Parse { line, source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_log_entries() -> Result<(), JjLogError> {
        let output = concat!(
            "\u{1b}[1m@  {\"change_id\":\"aaa\",\"commit_id\":\"111\",\"description\":\"first\\n\\nbody\"}\t\"body\"\n",
            "│\n",
            "○  {\"change_id\":\"bbb\",\"commit_id\":\"222\",\"description\":\"second\"}\t\"\"\n",
            "~\n",
        );

        let entries = parse_log_json_lines(output)?;

        assert_eq!(
            entries,
            vec![
                LogEntry::new("aaa", "111", "first\n\nbody").with_details("body"),
                LogEntry::new("bbb", "222", "second"),
            ]
        );
        Ok(())
    }

    #[test]
    fn reports_parse_line_number() {
        let output = concat!(
            "{\"change_id\":\"aaa\",\"commit_id\":\"111\",\"description\":\"first\"}\t\"\"\n",
            "○  {not json}\t\"\"\n",
        );

        let error = parse_log_json_lines(output).err();

        assert!(matches!(error, Some(JjLogError::Parse { line: 2, .. })));
    }

    #[test]
    fn reports_missing_details_field() {
        let output = "{\"change_id\":\"aaa\",\"commit_id\":\"111\",\"description\":\"first\"}\n";

        let error = parse_log_json_lines(output).err();

        assert!(matches!(
            error,
            Some(JjLogError::MissingDetails { line: 1 })
        ));
    }
}
