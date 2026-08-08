//! `jj bookmark` command integration.

use std::path::PathBuf;

use jk_core::{ColorPolicy, GlobalOptions, JjCommandSpec, OutputPolicy, RefreshPlan, SafetyClass};
use serde::Deserialize;
use thiserror::Error;

use crate::command::{JjCommandRunner, SystemJjCommandRunner};

const BOOKMARK_LIST_TEMPLATE: &str = r#"json(self) ++ "\n""#;

/// A parsed bookmark-list snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookmarkSnapshot {
    /// Bookmark references in jj's display order.
    pub bookmarks: Vec<BookmarkRef>,
    /// Display title for the command that produced the snapshot.
    pub title: String,
}

/// One local or remote bookmark reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BookmarkRef {
    /// Bookmark name.
    pub name: String,
    /// Remote name for a remote reference.
    pub remote: Option<String>,
    /// Target commit IDs. Multiple IDs represent a conflicted bookmark.
    pub targets: Vec<String>,
    /// Target commit IDs from the local tracking bookmark, when reported by jj.
    pub tracking_targets: Vec<String>,
    /// Whether the reference is tracked by a local bookmark.
    pub tracked: bool,
}

impl BookmarkRef {
    /// Returns whether this row is a local bookmark.
    #[must_use]
    pub const fn is_local(&self) -> bool {
        self.remote.is_none()
    }

    /// Returns whether the bookmark has exactly one normal target.
    #[must_use]
    pub fn normal_target(&self) -> Option<&str> {
        (self.targets.len() == 1).then(|| self.targets[0].as_str())
    }

    /// Returns whether this row can be changed by the first bookmark mutation slice.
    #[must_use]
    pub fn supports_local_mutation(&self) -> bool {
        self.is_local() && self.normal_target().is_some()
    }
}

#[derive(Deserialize)]
struct BookmarkRecord {
    name: String,
    #[serde(default)]
    remote: Option<String>,
    #[serde(default)]
    target: Vec<String>,
    #[serde(default)]
    tracking_target: Vec<String>,
    #[serde(default)]
    tracked: bool,
}

impl TryFrom<BookmarkRecord> for BookmarkRef {
    type Error = BookmarkParseError;

    fn try_from(record: BookmarkRecord) -> Result<Self, Self::Error> {
        if record.name.is_empty() {
            return Err(BookmarkParseError::MissingName);
        }
        let tracked = record.tracked || !record.tracking_target.is_empty();
        Ok(Self {
            name: record.name,
            remote: record.remote,
            targets: record.target,
            tracking_targets: record.tracking_target,
            tracked,
        })
    }
}

/// Supported local bookmark mutations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BookmarkMutation {
    /// Create a bookmark at a revision.
    Create {
        /// Bookmark name.
        name: String,
        /// Revision or revset to target.
        revision: String,
    },
    /// Move an existing bookmark to a revision.
    Move {
        /// Bookmark name.
        name: String,
        /// Revision or revset to target.
        revision: String,
    },
    /// Delete a local bookmark.
    Delete {
        /// Bookmark name.
        name: String,
    },
}

/// Loads bookmark references from a repository.
#[derive(Clone, Debug, Default)]
pub struct JjBookmarks {
    repository: Option<PathBuf>,
}

impl JjBookmarks {
    /// Sets the repository passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Loads all local and remote bookmark references.
    ///
    /// # Errors
    ///
    /// Returns an error when `jj` cannot run, exits unsuccessfully, or emits malformed JSON.
    pub fn load(&self) -> Result<BookmarkSnapshot, JjBookmarksError> {
        self.load_with_runner(&mut SystemJjCommandRunner)
    }

    /// Loads all local and remote bookmark references with an injected runner.
    ///
    /// # Errors
    ///
    /// Returns an error when the runner fails, `jj` exits unsuccessfully, or output is malformed.
    pub fn load_with_runner(
        &self,
        runner: &mut impl JjCommandRunner,
    ) -> Result<BookmarkSnapshot, JjBookmarksError> {
        let spec = self.list_spec();
        let output = runner.run(&spec)?;
        if !output.status.success() {
            return Err(JjBookmarksError::CommandFailed {
                command: spec.title().to_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        let bookmarks = parse_bookmark_list(&String::from_utf8_lossy(&output.stdout))?;
        Ok(BookmarkSnapshot {
            bookmarks,
            title: spec.title().to_owned(),
        })
    }

    /// Returns the machine-readable bookmark-list spec.
    #[must_use]
    pub fn list_spec(&self) -> JjCommandSpec {
        let output = OutputPolicy {
            color: ColorPolicy::Never,
            ..OutputPolicy::default()
        };
        self.with_repository_if_configured(
            JjCommandSpec::render_read_only([
                "bookmark",
                "list",
                "--all-remotes",
                "--template",
                BOOKMARK_LIST_TEMPLATE,
            ])
            .with_global_options(GlobalOptions::default().with_output(output))
            .with_title("jj bookmark list"),
        )
    }

    /// Returns a typed local bookmark mutation spec.
    #[must_use]
    pub fn mutation_spec(&self, mutation: &BookmarkMutation) -> JjCommandSpec {
        let (argv, safety) = match mutation {
            BookmarkMutation::Create { name, revision } => (
                vec![
                    "bookmark".to_owned(),
                    "create".to_owned(),
                    "--revision".to_owned(),
                    revision.clone(),
                    "--".to_owned(),
                    name.clone(),
                ],
                SafetyClass::LocalMetadata,
            ),
            BookmarkMutation::Move { name, revision } => (
                vec![
                    "bookmark".to_owned(),
                    "move".to_owned(),
                    "--to".to_owned(),
                    revision.clone(),
                    "--".to_owned(),
                    exact_bookmark_pattern(name),
                ],
                SafetyClass::LocalMetadata,
            ),
            BookmarkMutation::Delete { name } => (
                vec![
                    "bookmark".to_owned(),
                    "delete".to_owned(),
                    "--".to_owned(),
                    exact_bookmark_pattern(name),
                ],
                SafetyClass::DestructiveLocal,
            ),
        };
        self.with_repository_if_configured(
            JjCommandSpec::confirm_mutation(argv, safety)
                .with_refresh_plan(RefreshPlan::None)
                .with_title(match mutation {
                    BookmarkMutation::Create { .. } => "jj bookmark create",
                    BookmarkMutation::Move { .. } => "jj bookmark move",
                    BookmarkMutation::Delete { .. } => "jj bookmark delete",
                }),
        )
    }

    fn with_repository_if_configured(&self, spec: JjCommandSpec) -> JjCommandSpec {
        if let Some(repository) = self.repository.as_deref() {
            spec.with_repository(repository)
        } else {
            spec
        }
    }
}

/// Converts one bookmark name into jj's exact string-pattern syntax.
///
/// `jj bookmark move` and `delete` accept glob patterns by default. Quoting prevents a literal
/// bookmark name containing wildcard characters from selecting other bookmarks.
fn exact_bookmark_pattern(name: &str) -> String {
    let escaped_name = name.replace('\\', "\\\\").replace('"', "\\\"");
    format!("exact:\"{escaped_name}\"")
}

/// Parses JSON-lines output from `jj bookmark list`.
///
/// # Errors
///
/// Returns an error when a non-empty line is not valid JSON or lacks a bookmark name.
pub fn parse_bookmark_list(stdout: &str) -> Result<Vec<BookmarkRef>, BookmarkParseError> {
    let mut bookmarks = Vec::new();
    for (index, line) in stdout.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record = serde_json::from_str::<BookmarkRecord>(line).map_err(|source| {
            BookmarkParseError::Json {
                line: index + 1,
                detail: source.to_string(),
            }
        })?;
        bookmarks.push(BookmarkRef::try_from(record).map_err(|error| {
            BookmarkParseError::Record {
                line: index + 1,
                detail: error.to_string(),
            }
        })?);
    }
    Ok(bookmarks)
}

/// Error returned while parsing bookmark JSON-lines output.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum BookmarkParseError {
    /// A JSON record could not be decoded.
    #[error("line {line}: invalid bookmark JSON: {detail}")]
    Json {
        /// One-based output line number.
        line: usize,
        /// Parser detail.
        detail: String,
    },
    /// A decoded JSON record did not contain required bookmark data.
    #[error("line {line}: invalid bookmark record: {detail}")]
    Record {
        /// One-based output line number.
        line: usize,
        /// Validation detail.
        detail: String,
    },
    /// A bookmark record omitted its name.
    #[error("bookmark name is empty")]
    MissingName,
}

/// Error returned while loading bookmark data.
#[derive(Debug, Error)]
pub enum JjBookmarksError {
    /// The jj process could not be started or read.
    #[error("failed to run jj bookmark command: {0}")]
    Io(#[from] std::io::Error),
    /// jj exited unsuccessfully.
    #[error("{command} failed: {stderr}")]
    CommandFailed {
        /// Display command title.
        command: String,
        /// Captured stderr.
        stderr: String,
    },
    /// jj returned malformed machine output.
    #[error("failed to parse jj bookmark list output: {0}")]
    Parse(#[from] BookmarkParseError),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use std::path::Path;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use jk_core::ExecutionMode;

    use super::*;
    use crate::command::build_jj_command;

    fn strings(args: &[std::ffi::OsString]) -> Vec<String> {
        args.iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    fn fixture_directory() -> std::path::PathBuf {
        static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);
        let fixture_id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "jk-bookmark-fixture-{}-{fixture_id}",
            std::process::id()
        ))
    }

    fn run_jj(repository: &Path, args: &[&str]) {
        let status = Command::new("jj")
            .arg("--repository")
            .arg(repository)
            .args(args)
            .status()
            .expect("jj is installed");
        assert!(status.success(), "jj command should succeed: {args:?}");
    }

    fn init_repository(repository: &Path) {
        let status = Command::new("jj")
            .args(["git", "init", "--no-colocate"])
            .arg(repository)
            .status()
            .expect("jj is installed");
        assert!(status.success(), "initialize fixture repository");
    }

    fn run_spec(spec: &JjCommandSpec) {
        let status = Command::new("jj")
            .args(spec.process_argv())
            .status()
            .expect("jj is installed");
        assert!(status.success(), "generated jj command should succeed");
    }

    #[test]
    fn list_spec_uses_json_template_and_explicit_repository() {
        let source = JjBookmarks::default().with_repository("/tmp/repo");
        let spec = source.list_spec();
        assert_eq!(
            strings(spec.argv()),
            vec![
                "bookmark",
                "list",
                "--all-remotes",
                "--template",
                BOOKMARK_LIST_TEMPLATE
            ]
        );
        assert_eq!(
            strings(&spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "never",
                "--repository",
                "/tmp/repo",
                "bookmark",
                "list",
                "--all-remotes",
                "--template",
                BOOKMARK_LIST_TEMPLATE
            ]
        );
    }

    #[test]
    fn parsed_rows_preserve_local_remote_tracking_and_conflict_shape() {
        let rows = parse_bookmark_list(
            r#"{"name":"main","target":["abc"]}
{"name":"main","remote":"origin","target":["def"],"tracking_target":["abc"],"tracked":true}
{"name":"inferred","remote":"origin","target":["ghi"],"tracking_target":["abc"]}
{"name":"conflicted","target":["old","new"]}
{"name":"deleted"}
"#,
        )
        .expect("valid bookmark JSON");
        assert!(rows[0].is_local());
        assert_eq!(rows[1].remote.as_deref(), Some("origin"));
        assert!(rows[1].tracked);
        assert!(rows[2].tracked);
        assert_eq!(rows[3].normal_target(), None);
        assert_eq!(rows[4].normal_target(), None);
    }

    #[test]
    fn malformed_json_reports_line_number() {
        let error = parse_bookmark_list("{\"name\":\"ok\"}\nnot-json\n")
            .expect_err("malformed JSON should fail");
        assert!(error.to_string().contains("line 2"));
    }

    #[test]
    fn mutation_specs_keep_values_as_argv_elements() {
        let source = JjBookmarks::default().with_repository("/tmp/repo");
        let spec = source.mutation_spec(&BookmarkMutation::Create {
            name: "feature;echo bad".to_owned(),
            revision: "description(\"quoted\")".to_owned(),
        });
        assert_eq!(
            strings(&spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repo",
                "bookmark",
                "create",
                "--revision",
                "description(\"quoted\")",
                "--",
                "feature;echo bad"
            ]
        );
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::LocalMetadata);
    }

    #[test]
    fn delete_is_marked_destructive_local() {
        let spec = JjBookmarks::default().mutation_spec(&BookmarkMutation::Delete {
            name: "feature".to_owned(),
        });
        assert_eq!(spec.safety(), SafetyClass::DestructiveLocal);
    }

    #[test]
    fn local_fixture_mutations_treat_wildcards_as_literal_bookmark_names() {
        let repository = fixture_directory();
        init_repository(&repository);

        let source = JjBookmarks::default().with_repository(&repository);
        let name = "feature*literal";
        run_spec(&source.mutation_spec(&BookmarkMutation::Create {
            name: name.to_owned(),
            revision: "@".to_owned(),
        }));

        run_jj(&repository, &["new", "-m", "second commit"]);
        run_spec(&source.mutation_spec(&BookmarkMutation::Move {
            name: name.to_owned(),
            revision: "@".to_owned(),
        }));
        let moved = source.load().expect("load moved bookmark");
        assert!(
            moved
                .bookmarks
                .iter()
                .any(|bookmark| bookmark.name == name && bookmark.is_local())
        );

        run_spec(&source.mutation_spec(&BookmarkMutation::Delete {
            name: name.to_owned(),
        }));
        let deleted = source.load().expect("load deleted bookmark");
        assert!(
            !deleted
                .bookmarks
                .iter()
                .any(|bookmark| bookmark.name == name && bookmark.is_local())
        );

        std::fs::remove_dir_all(repository).expect("remove fixture repository");
    }

    #[test]
    fn command_builder_preserves_jj_argv_order() {
        let spec = JjBookmarks::default().list_spec();
        let command = build_jj_command(&spec);
        assert_eq!(
            command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            strings(&spec.process_argv())
        );
    }
}
