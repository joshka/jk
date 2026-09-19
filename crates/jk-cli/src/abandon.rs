//! `jj abandon` mutation command integration.

use jk_core::{
    ColorPolicy, GlobalOptions, JjCommandSpec, OutputPolicy, RefreshPlan, SafetyClass,
    WorkingCopyPolicy,
};
use thiserror::Error;

use crate::command::JjCommandRunner;

const ABANDON_COMMAND: &str = "abandon";
const EMPTY_PROBE_TEMPLATE: &str = "self.empty()";

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

    /// Returns the probe used to decide whether confirmation is necessary.
    ///
    /// Snapshot the working copy first so newly added or edited files cannot be mistaken for an
    /// empty revision. This does not abandon anything, but can update jj's working-copy snapshot.
    #[must_use]
    pub fn empty_probe_spec_for(&self, query: &AbandonQuery) -> JjCommandSpec {
        let global_options = self
            .global_options
            .clone()
            .with_working_copy(WorkingCopyPolicy::SnapshotAndUpdate)
            .with_output(OutputPolicy {
                color: ColorPolicy::Never,
                ..OutputPolicy::default()
            });
        JjCommandSpec::render_read_only([
            "log",
            "-r",
            query.rev(),
            "--no-graph",
            "-T",
            EMPTY_PROBE_TEMPLATE,
        ])
        .with_global_options(global_options)
        .with_title(format!("jj log -r {} (empty?)", query.rev()))
    }

    /// Checks whether a revision changes no files without recording the probe in history.
    ///
    /// # Errors
    ///
    /// Returns an error when the probe cannot run, exits unsuccessfully, or emits an unexpected
    /// value.
    pub fn is_empty_with_runner(
        &self,
        query: &AbandonQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<bool, AbandonProbeError> {
        let output = runner.run(&self.empty_probe_spec_for(query))?;
        if !output.status.success() {
            return Err(AbandonProbeError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ));
        }

        parse_empty_probe_output(&output.stdout)
    }
}

/// Error returned while checking whether a revision is empty.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AbandonProbeError {
    /// The `jj log` probe could not be started or read.
    #[error("failed to run jj empty probe: {0}")]
    Io(#[from] std::io::Error),

    /// The probe command exited unsuccessfully.
    #[error("jj empty probe failed: {0}")]
    CommandFailed(String),

    /// The probe did not emit exactly `true` or `false`.
    #[error("jj empty probe returned unexpected output: {0}")]
    InvalidOutput(String),
}

fn parse_empty_probe_output(stdout: &[u8]) -> Result<bool, AbandonProbeError> {
    match String::from_utf8_lossy(stdout).trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        value => Err(AbandonProbeError::InvalidOutput(value.to_owned())),
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
    fn empty_probe_snapshots_before_evaluating_the_log_template() {
        let source = JjAbandon::default().with_repository("/tmp/repo");
        let spec = source.empty_probe_spec_for(&AbandonQuery::new("abc123"));

        assert_eq!(
            strings(spec.process_argv().as_slice()),
            vec![
                "--no-pager",
                "--color",
                "never",
                "--repository",
                "/tmp/repo",
                "log",
                "-r",
                "abc123",
                "--no-graph",
                "-T",
                "self.empty()",
            ]
        );
        assert_eq!(spec.title(), "jj log -r abc123 (empty?)");
    }

    #[test]
    fn empty_probe_parser_accepts_only_boolean_template_output() {
        assert!(matches!(parse_empty_probe_output(b"true\n"), Ok(true)));
        assert!(matches!(parse_empty_probe_output(b"false\n"), Ok(false)));
        assert!(matches!(
            parse_empty_probe_output(b"true false\n"),
            Err(AbandonProbeError::InvalidOutput(value)) if value == "true false"
        ));
    }

    #[test]
    fn empty_probe_detects_unsnapshotted_file_changes() -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        use std::process::Command;
        use std::time::SystemTime;

        use crate::command::SystemJjCommandRunner;

        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let repository =
            std::env::temp_dir().join(format!("jk-abandon-probe-{}-{nonce}", std::process::id()));
        fs::create_dir(&repository)?;
        let init = Command::new("jj")
            .current_dir(&repository)
            .env("JJ_CONFIG", "")
            .args(["git", "init", "."])
            .output()?;
        assert!(
            init.status.success(),
            "{}",
            String::from_utf8_lossy(&init.stderr)
        );
        fs::write(
            repository.join(".jj/repo/config.toml"),
            "[user]\nname = 'Test'\nemail = 'test@example.com'\n[signing]\nbehavior = 'drop'\n[snapshot]\nauto-track = 'all()'\n",
        )?;

        let source = JjAbandon::default().with_repository(&repository);
        let mut runner = SystemJjCommandRunner;
        let query = AbandonQuery::new("@");
        assert!(source.is_empty_with_runner(&query, &mut runner)?);

        // Keep the same change identifier across the snapshot, as the log selection does.
        let id_spec = JjCommandSpec::render_read_only([
            "log",
            "--color",
            "never",
            "-r",
            "@",
            "--no-graph",
            "-T",
            "change_id",
        ])
        .with_repository(&repository);
        let id_output = runner.run(&id_spec)?;
        assert!(id_output.status.success());
        let query = AbandonQuery::new(String::from_utf8(id_output.stdout)?);
        let file = repository.join("file.txt");
        fs::write(&file, "initial contents\n")?;
        assert!(!source.is_empty_with_runner(&query, &mut runner)?);

        let new_spec = JjCommandSpec::render_read_only(["new"]).with_repository(&repository);
        assert!(runner.run(&new_spec)?.status.success());
        let query = AbandonQuery::new("@");
        assert!(source.is_empty_with_runner(&query, &mut runner)?);
        fs::write(&file, "edited contents\n")?;
        assert!(!source.is_empty_with_runner(&query, &mut runner)?);

        fs::write(&file, "initial contents\n")?;
        assert!(source.is_empty_with_runner(&query, &mut runner)?);
        fs::remove_file(&file)?;
        assert!(!source.is_empty_with_runner(&query, &mut runner)?);
        fs::remove_dir_all(&repository)?;
        Ok(())
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
