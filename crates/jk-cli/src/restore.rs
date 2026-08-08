//! `jj restore` mutation command integration.

use jk_core::{GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass};

const RESTORE_COMMAND: &str = "restore";

/// Copy all paths from one revision into another revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreQuery {
    source: String,
    destination: String,
}

impl RestoreQuery {
    /// Creates an all-path restore with explicit source and destination revisions.
    #[must_use]
    pub fn all_paths(source: impl Into<String>, destination: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            destination: destination.into(),
        }
    }

    /// Returns the source revision whose content will be copied.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the destination revision that will be rewritten.
    #[must_use]
    pub fn destination(&self) -> &str {
        &self.destination
    }
}

/// Builds typed `jj restore` mutation specs.
#[derive(Clone, Debug, Default)]
pub struct JjRestore {
    global_options: GlobalOptions,
}

impl JjRestore {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &RestoreQuery) -> JjCommandSpec {
        JjCommandSpec::confirm_mutation(
            [
                RESTORE_COMMAND,
                "--from",
                query.source(),
                "--into",
                query.destination(),
            ],
            SafetyClass::DestructiveLocal,
        )
        .with_global_options(self.global_options.clone())
        .with_title("Restore all paths")
        .with_refresh_plan(RefreshPlan::None)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use jk_core::{CommandPreviewWarning, ExecutionMode, RefreshPlan};

    use super::*;

    fn strings(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn restore_builds_explicit_all_path_spec() {
        let query = RestoreQuery::all_paths("abc123", "@");
        let spec = JjRestore::default().spec_for(&query);

        assert_eq!(
            strings(spec.argv()),
            vec!["restore", "--from", "abc123", "--into", "@"]
        );
        assert_eq!(spec.title(), "Restore all paths");
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::DestructiveLocal);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
    }

    #[test]
    fn repository_renders_before_restore() {
        let spec = JjRestore::default()
            .with_repository("/tmp/repo")
            .spec_for(&RestoreQuery::all_paths("abc123", "@"));

        assert_eq!(
            strings(&spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repo",
                "restore",
                "--from",
                "abc123",
                "--into",
                "@",
            ]
        );
    }

    #[test]
    fn command_preview_warns_that_restore_is_destructive() {
        let preview = JjRestore::default()
            .spec_for(&RestoreQuery::all_paths("abc123", "@"))
            .command_preview();

        assert_eq!(
            preview.command_line,
            "jj --no-pager --color always restore --from abc123 --into @"
        );
        assert_eq!(
            preview.warnings,
            vec![CommandPreviewWarning::DestructiveLocal]
        );
    }
}
