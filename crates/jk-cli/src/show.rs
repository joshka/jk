//! `jj show` command integration.

use std::path::PathBuf;

use jk_core::{InspectionSnapshot, JjCommandSpec};
use thiserror::Error;

use crate::command::run_jj_spec;

const SHOW_COMMAND: &str = "show";

/// Canonical query shape supported by `jk show`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShowQuery {
    revs: Vec<String>,
}

impl ShowQuery {
    /// Creates a `jj show` query for one or more revisions.
    #[must_use]
    pub fn new(revs: Vec<String>) -> Self {
        Self { revs }
    }

    /// Returns the revisions passed to `jj show`.
    #[must_use]
    pub fn revs(&self) -> &[String] {
        &self.revs
    }

    /// Returns a compact target label for error and empty-output states.
    #[must_use]
    pub fn target_label(&self) -> String {
        if self.revs.is_empty() {
            "@".to_owned()
        } else {
            self.revs.join(" ")
        }
    }
}

impl From<String> for ShowQuery {
    fn from(rev: String) -> Self {
        Self::new(vec![rev])
    }
}

/// Loads rendered `jj show` output.
#[derive(Clone, Debug, Default)]
pub struct JjShow {
    repository: Option<PathBuf>,
}

impl JjShow {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Loads the rendered show output for `query`.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_query(&self, query: &ShowQuery) -> Result<InspectionSnapshot, JjShowError> {
        let spec = self.spec_for(query);
        let rendered = Self::run(&spec)?;
        Ok(InspectionSnapshot::new(query.target_label(), rendered).with_title(spec.title()))
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &ShowQuery) -> JjCommandSpec {
        let mut argv = Vec::with_capacity(query.revs().len() + 1);
        argv.push(SHOW_COMMAND);
        argv.extend(query.revs().iter().map(String::as_str));

        let spec = JjCommandSpec::render_read_only(argv);
        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }

    fn run(spec: &JjCommandSpec) -> Result<String, JjShowError> {
        let output = run_jj_spec(spec, "always")?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Err(JjShowError::CommandFailed(stderr))
        }
    }
}

/// Error returned while loading rendered `jj show` output.
#[derive(Debug, Error)]
pub enum JjShowError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj show: {0}")]
    Io(#[from] std::io::Error),

    /// `jj show` exited unsuccessfully.
    #[error("jj show failed: {0}")]
    CommandFailed(String),
}

#[cfg(test)]
mod tests {
    use crate::command::build_jj_command;

    use super::*;

    #[test]
    fn spec_uses_jj_show_with_revisions() {
        let spec = JjShow::default().spec_for(&ShowQuery::new(vec![
            "abc123".to_owned(),
            "def456".to_owned(),
        ]));

        assert_eq!(
            spec.argv()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["show", "abc123", "def456"]
        );
        assert_eq!(spec.title(), "jj show abc123 def456");
    }

    #[test]
    fn command_adds_repository_and_color_flags_outside_spec() {
        let source = JjShow::default().with_repository("/tmp/repo");
        let spec = source.spec_for(&ShowQuery::new(vec!["abc123".to_owned()]));
        let command = build_jj_command(&spec, "always");

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
                "show",
                "abc123"
            ]
        );
    }
}
