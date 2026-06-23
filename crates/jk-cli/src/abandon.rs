//! `jj abandon` mutation command integration.

use jk_core::{GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass};

const ABANDON_COMMAND: &str = "abandon";

/// Abandon one selected revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbandonQuery {
    rev: String,
}

impl AbandonQuery {
    /// Creates a `jj abandon REV` query.
    #[must_use]
    pub fn new(rev: impl Into<String>) -> Self {
        Self { rev: rev.into() }
    }

    /// Returns the revision passed to `jj abandon`.
    #[must_use]
    pub fn rev(&self) -> &str {
        &self.rev
    }
}

/// Builds typed `jj abandon` mutation specs.
#[derive(Clone, Debug, Default)]
pub struct JjAbandon {
    global_options: GlobalOptions,
}

impl JjAbandon {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &AbandonQuery) -> JjCommandSpec {
        JjCommandSpec::confirm_mutation(
            [ABANDON_COMMAND, query.rev.as_str()],
            SafetyClass::DestructiveLocal,
        )
        .with_global_options(self.global_options.clone())
        .with_title(format!("jj abandon {}", query.rev()))
        .with_refresh_plan(RefreshPlan::None)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use jk_core::{ExecutionMode, RefreshPlan};

    use super::*;

    fn strings(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn abandon_builds_confirmed_destructive_spec() {
        let query = AbandonQuery::new("abc123");
        let spec = JjAbandon::default().spec_for(&query);

        assert_eq!(strings(spec.argv()), vec!["abandon", "abc123"]);
        assert_eq!(spec.title(), "jj abandon abc123");
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::DestructiveLocal);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
    }

    #[test]
    fn repository_renders_before_abandon() {
        let spec = JjAbandon::default()
            .with_repository("/tmp/repo")
            .spec_for(&AbandonQuery::new("abc123"));
        let argv = spec
            .process_argv()
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            argv,
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repo",
                "abandon",
                "abc123",
            ]
        );
    }

    #[test]
    fn command_preview_warns_about_destructive_operation() {
        let preview = JjAbandon::default()
            .spec_for(&AbandonQuery::new("abc123"))
            .command_preview();

        assert_eq!(
            preview.command_line,
            "jj --no-pager --color always abandon abc123"
        );
        assert_eq!(
            preview.warnings,
            vec![jk_core::CommandPreviewWarning::DestructiveLocal]
        );
    }
}
