//! `jj status` command integration.

use std::path::PathBuf;

use jk_core::{InspectionSnapshot, JjCommandSpec};
use thiserror::Error;

use crate::command::{JjCommandRunner, SystemJjCommandRunner};

const STATUS_COMMAND: &str = "status";

/// Canonical query shape supported by `jk status`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StatusQuery {
    filesets: Vec<String>,
}

impl StatusQuery {
    /// Creates a `jj status` query with optional filesets.
    #[must_use]
    pub const fn new(filesets: Vec<String>) -> Self {
        Self { filesets }
    }

    /// Returns the filesets passed to `jj status`.
    #[must_use]
    pub fn filesets(&self) -> &[String] {
        &self.filesets
    }

    /// Returns a compact target label for error and empty-output states.
    #[must_use]
    pub fn target_label(&self) -> String {
        if self.filesets.is_empty() {
            "repository".to_owned()
        } else {
            self.filesets.join(" ")
        }
    }
}

/// Loads rendered `jj status` output.
#[derive(Clone, Debug, Default)]
pub struct JjStatus {
    repository: Option<PathBuf>,
}

impl JjStatus {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Loads the rendered status output for `query`.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_query(&self, query: &StatusQuery) -> Result<InspectionSnapshot, JjStatusError> {
        self.load_query_with_runner(query, &mut SystemJjCommandRunner)
    }

    /// Loads the rendered status output for `query` using the provided command runner.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_query_with_runner(
        &self,
        query: &StatusQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<InspectionSnapshot, JjStatusError> {
        let spec = self.spec_for(query);
        let rendered = Self::run(runner, &spec)?;
        Ok(InspectionSnapshot::new(query.target_label(), rendered).with_title(spec.title()))
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &StatusQuery) -> JjCommandSpec {
        let mut argv = Vec::with_capacity(query.filesets().len() + 1);
        argv.push(STATUS_COMMAND);
        argv.extend(query.filesets().iter().map(String::as_str));

        let spec = JjCommandSpec::render_read_only(argv);
        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }

    fn run(
        runner: &mut impl JjCommandRunner,
        spec: &JjCommandSpec,
    ) -> Result<String, JjStatusError> {
        let output = runner.run(spec)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Err(JjStatusError::CommandFailed(stderr))
        }
    }
}

/// Error returned while loading rendered `jj status` output.
#[derive(Debug, Error)]
pub enum JjStatusError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj status: {0}")]
    Io(#[from] std::io::Error),

    /// `jj status` exited unsuccessfully.
    #[error("jj status failed: {0}")]
    CommandFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::build_jj_command;

    #[test]
    fn spec_uses_jj_status_with_filesets() {
        let spec = JjStatus::default().spec_for(&StatusQuery::new(vec![
            "crates/jk".to_owned(),
            "docs".to_owned(),
        ]));

        assert_eq!(
            spec.argv()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["status", "crates/jk", "docs"]
        );
        assert_eq!(spec.title(), "jj status crates/jk docs");
    }

    #[test]
    fn command_renders_repository_before_status() {
        let source = JjStatus::default().with_repository("/tmp/repo");
        let spec = source.spec_for(&StatusQuery::new(vec!["src".to_owned()]));
        let command = build_jj_command(&spec);

        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            args,
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repo",
                "status",
                "src"
            ]
        );
    }
}
