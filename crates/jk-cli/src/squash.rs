//! `jj squash` mutation command integration.

use jk_core::{GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass};

const SQUASH_COMMAND: &str = "squash";

/// Whole-change sources and destination for one squash operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquashQuery {
    sources: Vec<String>,
    destination: String,
}

impl SquashQuery {
    /// Creates a whole-change squash query with explicit revision roles.
    #[must_use]
    pub fn new(
        sources: impl IntoIterator<Item = impl Into<String>>,
        destination: impl Into<String>,
    ) -> Self {
        Self {
            sources: sources.into_iter().map(Into::into).collect(),
            destination: destination.into(),
        }
    }

    /// Returns source revisions in selection order.
    #[must_use]
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Returns the destination revision.
    #[must_use]
    pub fn destination(&self) -> &str {
        &self.destination
    }
}

/// Builds typed, noninteractive `jj squash` mutation specs.
#[derive(Clone, Debug, Default)]
pub struct JjSquash {
    global_options: GlobalOptions,
}

impl JjSquash {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Returns the command spec for `query`.
    ///
    /// No filesets are included: this workflow always moves each entire source change. Keeping the
    /// destination message also prevents `jj` from opening an editor or prompting during execution.
    #[must_use]
    pub fn spec_for(&self, query: &SquashQuery) -> JjCommandSpec {
        let mut argv = vec![SQUASH_COMMAND.to_owned()];
        for source in query.sources() {
            argv.push("--from".to_owned());
            argv.push(source.clone());
        }
        argv.push("--into".to_owned());
        argv.push(query.destination().to_owned());
        argv.push("--use-destination-message".to_owned());

        JjCommandSpec::confirm_mutation(argv, SafetyClass::LocalRewrite)
            .with_global_options(self.global_options.clone())
            .with_title(format!("jj squash into {}", query.destination()))
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
    fn squash_builds_explicit_whole_change_spec() {
        let query = SquashQuery::new(["source-a", "source-b"], "destination");
        let spec = JjSquash::default().spec_for(&query);

        assert_eq!(
            strings(spec.argv()),
            vec![
                "squash",
                "--from",
                "source-a",
                "--from",
                "source-b",
                "--into",
                "destination",
                "--use-destination-message",
            ]
        );
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::LocalRewrite);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
        assert!(
            !strings(spec.argv())
                .iter()
                .any(|arg| arg == "--interactive")
        );
    }

    #[test]
    fn repository_and_exact_roles_are_visible_in_preview() {
        let spec = JjSquash::default()
            .with_repository("/tmp/repo")
            .spec_for(&SquashQuery::new(["source"], "destination"));
        let preview = spec.command_preview();

        assert_eq!(
            preview.command_line,
            "jj --no-pager --color always --repository /tmp/repo squash --from source --into destination --use-destination-message"
        );
        assert_eq!(preview.title, "jj squash into destination");
        assert_eq!(
            preview.warnings,
            vec![jk_core::CommandPreviewWarning::LocalRewrite]
        );
    }
}
