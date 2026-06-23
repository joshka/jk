//! `jj edit` mutation command integration.

use jk_core::{GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass};

const EDIT_COMMAND: &str = "edit";

/// Move the working copy to a revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditQuery {
    rev: String,
}

impl EditQuery {
    /// Creates a `jj edit REV` query.
    #[must_use]
    pub fn new(rev: impl Into<String>) -> Self {
        Self { rev: rev.into() }
    }

    /// Returns the revision passed to `jj edit`.
    #[must_use]
    pub fn rev(&self) -> &str {
        &self.rev
    }
}

/// Builds typed `jj edit` mutation specs.
#[derive(Clone, Debug, Default)]
pub struct JjEdit {
    global_options: GlobalOptions,
}

impl JjEdit {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &EditQuery) -> JjCommandSpec {
        JjCommandSpec::confirm_mutation([EDIT_COMMAND, query.rev()], SafetyClass::LocalRewrite)
            .with_global_options(self.global_options.clone())
            .with_title(format!("jj edit {}", query.rev()))
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
    fn edit_builds_confirmed_local_rewrite_spec() {
        let query = EditQuery::new("abc123");
        let spec = JjEdit::default().spec_for(&query);

        assert_eq!(strings(spec.argv()), vec!["edit", "abc123"]);
        assert_eq!(spec.title(), "jj edit abc123");
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::LocalRewrite);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
    }

    #[test]
    fn repository_renders_before_edit() {
        let spec = JjEdit::default()
            .with_repository("/tmp/repo")
            .spec_for(&EditQuery::new("abc123"));
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
                "edit",
                "abc123",
            ]
        );
    }

    #[test]
    fn command_preview_warns_about_local_rewrite() {
        let preview = JjEdit::default()
            .spec_for(&EditQuery::new("abc123"))
            .command_preview();

        assert_eq!(
            preview.command_line,
            "jj --no-pager --color always edit abc123"
        );
        assert_eq!(
            preview.warnings,
            vec![jk_core::CommandPreviewWarning::LocalRewrite]
        );
    }
}
