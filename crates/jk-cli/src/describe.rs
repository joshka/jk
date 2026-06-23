//! `jj describe` mutation command integration.

use std::path::PathBuf;

use jk_core::{JjCommandSpec, RefreshPlan, SafetyClass};

const DESCRIBE_COMMAND: &str = "describe";

/// Inline description update for one revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DescribeQuery {
    rev: String,
    message: String,
}

impl DescribeQuery {
    /// Creates an inline `jj describe -m MESSAGE REV` query.
    #[must_use]
    pub fn new(rev: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            rev: rev.into(),
            message: message.into(),
        }
    }

    /// Returns the revision passed to `jj describe`.
    #[must_use]
    pub fn rev(&self) -> &str {
        &self.rev
    }

    /// Returns the inline description message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Builds typed `jj describe` mutation specs.
#[derive(Clone, Debug, Default)]
pub struct JjDescribe {
    repository: Option<PathBuf>,
}

impl JjDescribe {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &DescribeQuery) -> JjCommandSpec {
        let spec = JjCommandSpec::confirm_mutation(
            [
                DESCRIBE_COMMAND,
                "-m",
                query.message.as_str(),
                query.rev.as_str(),
            ],
            SafetyClass::LocalRewrite,
        )
        .with_title(format!("jj describe {}", query.rev()))
        .with_refresh_plan(RefreshPlan::None);

        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }
}

#[cfg(test)]
mod tests {
    use jk_core::{ExecutionMode, RefreshPlan};

    use super::*;
    use crate::command::build_jj_command;

    #[test]
    fn spec_uses_inline_describe_with_message_and_revision() {
        let query = DescribeQuery::new("abc123", "Update the description");
        let spec = JjDescribe::default().spec_for(&query);

        assert_eq!(
            spec.argv()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            vec!["describe", "-m", "Update the description", "abc123"]
        );
        assert_eq!(spec.title(), "jj describe abc123");
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::LocalRewrite);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
    }

    #[test]
    fn command_renders_repository_before_describe() {
        let source = JjDescribe::default().with_repository("/tmp/repo");
        let spec = source.spec_for(&DescribeQuery::new("abc123", "message"));
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
                "describe",
                "-m",
                "message",
                "abc123"
            ]
        );
    }

    #[test]
    fn process_preview_quotes_message_and_preserves_global_order() {
        let source = JjDescribe::default().with_repository("/tmp/repo");
        let spec = source.spec_for(&DescribeQuery::new("abc123", "message with spaces"));

        assert_eq!(
            spec.process_preview(),
            "jj --no-pager --color always --repository /tmp/repo describe -m 'message with spaces' abc123"
        );
    }

    #[test]
    fn command_preview_redacts_secret_looking_message() {
        let spec = JjDescribe::default().spec_for(&DescribeQuery::new(
            "abc123",
            "token=abc123 should not render raw",
        ));

        assert_eq!(
            spec.command_preview().command_line,
            "jj --no-pager --color always describe -m 'token=<redacted> should not render raw' abc123"
        );
    }
}
