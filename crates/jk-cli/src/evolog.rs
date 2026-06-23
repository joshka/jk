//! Selected-change `jj evolog` command integration.

use std::path::PathBuf;

use jk_core::{InspectionSnapshot, JjCommandSpec};
use thiserror::Error;

use crate::command::{JjCommandRunner, SystemJjCommandRunner};

const EVOLOG_COMMAND: &str = "evolog";

/// Canonical query shape supported by selected-change evolog inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvologQuery {
    rev: String,
}

impl EvologQuery {
    /// Creates a `jj evolog` query for one revision.
    #[must_use]
    pub fn new(rev: impl Into<String>) -> Self {
        Self { rev: rev.into() }
    }

    /// Returns the revision passed to `jj evolog -r`.
    #[must_use]
    pub fn rev(&self) -> &str {
        &self.rev
    }

    /// Returns a compact target label for error and empty-output states.
    #[must_use]
    pub fn target_label(&self) -> String {
        self.rev.clone()
    }
}

impl From<String> for EvologQuery {
    fn from(rev: String) -> Self {
        Self::new(rev)
    }
}

/// Loads rendered `jj evolog` output.
#[derive(Clone, Debug, Default)]
pub struct JjEvolog {
    repository: Option<PathBuf>,
}

impl JjEvolog {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Loads the rendered evolog output for `query`.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_query(&self, query: &EvologQuery) -> Result<InspectionSnapshot, JjEvologError> {
        self.load_query_with_runner(query, &mut SystemJjCommandRunner)
    }

    /// Loads the rendered evolog output for `query` using the provided command runner.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_query_with_runner(
        &self,
        query: &EvologQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<InspectionSnapshot, JjEvologError> {
        let spec = self.spec_for(query);
        let rendered = Self::run(runner, &spec)?;
        Ok(InspectionSnapshot::new(query.target_label(), rendered).with_title(spec.title()))
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &EvologQuery) -> JjCommandSpec {
        let spec = JjCommandSpec::render_read_only([EVOLOG_COMMAND, "-r", query.rev()]);
        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }

    fn run(
        runner: &mut impl JjCommandRunner,
        spec: &JjCommandSpec,
    ) -> Result<String, JjEvologError> {
        let output = runner.run(spec)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Err(JjEvologError::CommandFailed(stderr))
        }
    }
}

/// Error returned while loading rendered `jj evolog` output.
#[derive(Debug, Error)]
pub enum JjEvologError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj evolog: {0}")]
    Io(#[from] std::io::Error),

    /// `jj evolog` exited unsuccessfully.
    #[error("jj evolog failed: {0}")]
    CommandFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::build_jj_command;

    #[test]
    fn spec_uses_jj_evolog_with_revision() {
        let spec = JjEvolog::default().spec_for(&EvologQuery::new("abc123"));

        assert_eq!(
            spec.argv()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["evolog", "-r", "abc123"]
        );
        assert_eq!(spec.title(), "jj evolog -r abc123");
    }

    #[test]
    fn command_renders_repository_before_evolog() {
        let source = JjEvolog::default().with_repository("/tmp/repo");
        let spec = source.spec_for(&EvologQuery::new("abc123"));
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
                "evolog",
                "-r",
                "abc123"
            ]
        );
    }
}
