//! `jj new` mutation command integration.

use jk_core::{GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass};

const NEW_COMMAND: &str = "new";

/// Create a new change from parent revisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewQuery {
    parents: Vec<String>,
}

impl NewQuery {
    /// Creates a `jj new PARENTS...` query.
    #[must_use]
    pub fn new(parents: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            parents: parents.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns whether no explicit parent revisions were provided.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.parents.is_empty()
    }

    /// Returns the parent revisions passed to `jj new`.
    #[must_use]
    pub fn parents(&self) -> &[String] {
        &self.parents
    }
}

/// Builds typed `jj new` mutation specs.
#[derive(Clone, Debug, Default)]
pub struct JjNew {
    global_options: GlobalOptions,
}

impl JjNew {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &NewQuery) -> JjCommandSpec {
        let mut argv = Vec::with_capacity(query.parents.len() + 1);
        argv.push(NEW_COMMAND.to_owned());
        argv.extend(query.parents.iter().cloned());

        JjCommandSpec::confirm_mutation(argv, SafetyClass::LocalRewrite)
            .with_global_options(self.global_options.clone())
            .with_title(new_title(query.parents()))
            .with_refresh_plan(RefreshPlan::None)
    }
}

fn new_title(parents: &[String]) -> String {
    if parents.is_empty() {
        return "jj new".to_owned();
    }

    format!("jj new {}", parents.join(" "))
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
    fn new_builds_confirmed_local_rewrite_spec() {
        let query = NewQuery::new(["abc123"]);
        let spec = JjNew::default().spec_for(&query);

        assert_eq!(strings(spec.argv()), vec!["new", "abc123"]);
        assert_eq!(spec.title(), "jj new abc123");
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::LocalRewrite);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
    }

    #[test]
    fn new_preserves_multiple_parent_revisions() {
        let query = NewQuery::new(["abc123", "def456"]);
        let spec = JjNew::default().spec_for(&query);

        assert_eq!(strings(spec.argv()), vec!["new", "abc123", "def456"]);
        assert_eq!(spec.title(), "jj new abc123 def456");
    }

    #[test]
    fn repository_renders_before_new() {
        let spec = JjNew::default()
            .with_repository("/tmp/repo")
            .spec_for(&NewQuery::new(["abc123"]));
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
                "new",
                "abc123",
            ]
        );
    }

    #[test]
    fn command_preview_warns_about_local_rewrite() {
        let preview = JjNew::default()
            .spec_for(&NewQuery::new(["abc123"]))
            .command_preview();

        assert_eq!(
            preview.command_line,
            "jj --no-pager --color always new abc123"
        );
        assert_eq!(
            preview.warnings,
            vec![jk_core::CommandPreviewWarning::LocalRewrite]
        );
    }
}
