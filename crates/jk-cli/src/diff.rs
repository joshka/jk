//! Selected-change `jj diff` command integration.

use std::path::PathBuf;
#[cfg(test)]
use std::process::Command;

use jk_core::{DiffFileStat, DiffSnapshot, JjCommandSpec};
use thiserror::Error;

#[cfg(test)]
use crate::command::build_jj_command;
use crate::command::run_jj_spec;

const DIFF_COMMAND: &str = "diff";

/// Loads a rendered diff for a selected `jj` change.
///
/// This bridge keeps the first diff slice command-shaped for the same reason the log bridge does:
/// `jj` should own the exact rendered output, color choices, and diff formatting while `jk` adds
/// only view state such as refresh, return navigation, and file-section collapse.
#[derive(Clone, Debug, Default)]
pub struct JjDiff {
    repository: Option<PathBuf>,
}

impl JjDiff {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Loads the rendered diff for `change_id`.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load(&self, change_id: &str) -> Result<DiffSnapshot, JjDiffError> {
        let spec = self.diff_spec(change_id);
        let rendered = Self::run(&spec)?;
        let stats = Self::run(&self.stats_spec(change_id))?;
        let file_stats = parse_stats_lines(&stats);

        Ok(DiffSnapshot::new(change_id, rendered)
            .with_file_stats(file_stats)
            .with_title(spec.title()))
    }

    /// Returns the title used for the selected-change diff command.
    #[must_use]
    pub fn title(change_id: &str) -> String {
        command_title(change_id)
    }

    fn run(spec: &JjCommandSpec) -> Result<String, JjDiffError> {
        let output = run_jj_spec(spec, "always")?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Err(JjDiffError::CommandFailed(stderr))
        }
    }

    #[cfg(test)]
    fn diff_command(&self, change_id: &str) -> Command {
        build_jj_command(&self.diff_spec(change_id), "always")
    }

    #[cfg(test)]
    fn stats_command(&self, change_id: &str) -> Command {
        build_jj_command(&self.stats_spec(change_id), "always")
    }

    fn diff_spec(&self, change_id: &str) -> JjCommandSpec {
        self.spec([DIFF_COMMAND, "-r", change_id])
            .with_title(command_title(change_id))
    }

    fn stats_spec(&self, change_id: &str) -> JjCommandSpec {
        self.spec([DIFF_COMMAND, "-r", change_id, "--stat"])
            .with_title(format!("{} --stat", command_title(change_id)))
    }

    fn spec<'a>(&self, argv: impl IntoIterator<Item = &'a str>) -> JjCommandSpec {
        let spec = JjCommandSpec::render_read_only(argv);
        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }
}

/// Builds the title-bar command label for a selected-change diff.
fn command_title(change_id: &str) -> String {
    format!("jj diff -r {change_id}")
}

/// Parses per-file diff stats from jj's rendered `--stat` rows.
fn parse_stats_lines(stdout: &str) -> Vec<DiffFileStat> {
    let mut stats = Vec::new();
    for line in stdout.lines() {
        let plain = strip_ansi(line);
        if !plain.contains('|') || plain.trim_start().starts_with(char::is_numeric) {
            continue;
        }

        let Some((plain_path, _)) = plain.split_once('|') else {
            continue;
        };
        let Some((_, rendered_suffix)) = line.split_once('|') else {
            continue;
        };

        let rendered_suffix = format!("|{rendered_suffix}");
        let (added, removed) = count_stat_marks(&plain);
        stats.push(
            DiffFileStat::new(plain_path.trim_end().to_owned(), added, removed)
                .with_rendered(rendered_suffix),
        );
    }

    stats
}

/// Counts plus and minus marks in a rendered stat row.
fn count_stat_marks(line: &str) -> (usize, usize) {
    let Some((_, suffix)) = line.split_once('|') else {
        return (0, 0);
    };
    (
        suffix.chars().filter(|character| *character == '+').count(),
        suffix.chars().filter(|character| *character == '-').count(),
    )
}

/// Removes CSI-style ANSI escape sequences from terminal text.
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

/// Error returned while loading a selected-change diff from `jj`.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JjDiffError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj diff")]
    Io(#[from] std::io::Error),

    /// The `jj` command exited unsuccessfully.
    #[error("jj failed: {0}")]
    CommandFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_command_forces_color_and_targets_change() {
        let command = JjDiff::default().diff_command("abc123");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let envs = command
            .get_envs()
            .map(|(key, value)| (key.to_string_lossy().into_owned(), value.is_none()))
            .collect::<Vec<_>>();

        assert!(args.windows(2).any(|args| args == ["--color", "always"]));
        assert!(args.windows(3).any(|args| args == ["diff", "-r", "abc123"]));
        assert!(envs.contains(&("NO_COLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR_FORCE".to_owned(), true)));
    }

    #[test]
    fn command_title_names_targeted_jj_diff() {
        assert_eq!(command_title("abc123"), "jj diff -r abc123");
    }

    #[test]
    fn stats_command_uses_jj_diff_stat_output() {
        let command = JjDiff::default().stats_command("abc123");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(args.windows(3).any(|args| args == ["diff", "-r", "abc123"]));
        assert!(args.iter().any(|arg| arg == "--stat"));
        assert!(args.windows(2).any(|args| args == ["--color", "always"]));
    }

    #[test]
    fn parses_rendered_diff_stat_lines() {
        let output = concat!(
            "src/a.rs | 3 \u{1b}[38;5;2m++\u{1b}[38;5;1m-\u{1b}[39m\n",
            "src/b.rs | 2 \u{1b}[38;5;1m--\u{1b}[39m\n",
            "2 files changed, 2 insertions(+), 3 deletions(-)\n",
        );

        let stats = parse_stats_lines(output);

        assert_eq!(
            stats,
            vec![
                DiffFileStat::new("src/a.rs", 2, 1)
                    .with_rendered("| 3 \u{1b}[38;5;2m++\u{1b}[38;5;1m-\u{1b}[39m"),
                DiffFileStat::new("src/b.rs", 0, 2).with_rendered("| 2 \u{1b}[38;5;1m--\u{1b}[39m"),
            ]
        );
    }
}
