//! `jj abandon` mutation command integration.

use jk_core::{
    ColorPolicy, GlobalOptions, JjCommandSpec, OutputPolicy, RefreshPlan, SafetyClass,
    WorkingCopyPolicy,
};
use thiserror::Error;

use crate::command::JjCommandRunner;

const ABANDON_COMMAND: &str = "abandon";
const EMPTY_PROBE_TEMPLATE: &str = "self.empty()";
const DETAILS_TEMPLATE: &str = concat!(
    "json(change_id) ++ \"\\t\" ++ json(description) ++ \"\\t\" ++ ",
    "json(self.contained_in(\"working_copies()\")) ++ \"\\n\""
);
const FILES_TEMPLATE: &str = concat!(
    "self.diff().stat().files().map(|file| json(file.path()) ++ \"\\t\" ++ ",
    "json(file.status_char()) ++ \"\\t\" ++ json(file.lines_added()) ++ \"\\t\" ++ ",
    "json(file.lines_removed()) ++ \"\\n\").join(\"\")"
);

/// One changed path shown before abandoning a revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbandonFile {
    /// Repository-relative destination path. Copy and rename sources are visible in the patch.
    pub path: String,
    /// Change kind reported by jj: `A`, `M`, `D`, `C`, or `R`.
    pub status: String,
    /// Lines added in this path.
    pub added: usize,
    /// Lines removed from this path.
    pub removed: usize,
}

/// Information shown before abandoning a non-empty revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbandonDetails {
    /// The resolved change identifier.
    pub change_id: String,
    /// The full revision description.
    pub description: String,
    /// Whether this revision is a working copy in any workspace.
    pub is_working_copy: bool,
    /// Per-path status and line counts from jj's structured diff-stat template.
    pub files: Vec<AbandonFile>,
    /// Number of descendant revisions that will be rebased by `jj abandon`.
    pub descendant_count: usize,
    /// The full patch shown by a View diff action.
    pub diff: String,
}

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

    /// Loads the information needed to confirm abandoning a non-empty revision.
    ///
    /// # Errors
    ///
    /// Returns an error when any required `jj` query fails or returns malformed metadata. The
    /// result never substitutes missing information with an empty summary or zero descendants.
    pub fn details(&self, query: &AbandonQuery) -> Result<AbandonDetails, AbandonDetailsError> {
        self.details_with_runner(query, &mut crate::command::SystemJjCommandRunner)
    }

    /// Loads confirmation information using the provided command runner.
    ///
    /// # Errors
    ///
    /// Returns an error when any required `jj` query fails or returns malformed metadata.
    pub fn details_with_runner(
        &self,
        query: &AbandonQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<AbandonDetails, AbandonDetailsError> {
        let metadata = Self::run_details_command(runner, &self.details_spec_for(query))?;
        let (change_id, description, is_working_copy) = parse_details_metadata(&metadata)?;
        let files = parse_files(&Self::run_details_command(
            runner,
            &self.files_spec_for(query),
        )?)?;
        let descendants =
            Self::run_details_command(runner, &self.descendant_count_spec_for(query))?;
        let descendant_count = parse_descendant_count(&descendants)?;
        let diff = Self::run_details_command(runner, &self.diff_spec_for(query))?;

        Ok(AbandonDetails {
            change_id,
            description,
            is_working_copy,
            files,
            descendant_count,
            diff,
        })
    }

    /// Returns the metadata query for the confirmation dialog.
    #[must_use]
    pub fn details_spec_for(&self, query: &AbandonQuery) -> JjCommandSpec {
        self.details_spec([
            "log",
            "-r",
            query.rev(),
            "--no-graph",
            "-T",
            DETAILS_TEMPLATE,
        ])
        .with_title(format!("jj log -r {} (abandon details)", query.rev()))
    }

    /// Returns the structured changed-file query for the confirmation dialog.
    #[must_use]
    pub fn files_spec_for(&self, query: &AbandonQuery) -> JjCommandSpec {
        self.details_spec(["log", "-r", query.rev(), "--no-graph", "-T", FILES_TEMPLATE])
            .with_title(format!("jj log -r {} (abandon file stats)", query.rev()))
    }

    /// Returns the descendant-count query for the confirmation dialog.
    #[must_use]
    pub fn descendant_count_spec_for(&self, query: &AbandonQuery) -> JjCommandSpec {
        let descendants = format!("({}):: ~ ({})", query.rev(), query.rev());
        self.details_spec(["log", "-r", descendants.as_str(), "--count"])
            .with_title(format!(
                "jj log -r ({}):: ~ ({}) --count",
                query.rev(),
                query.rev()
            ))
    }

    /// Returns the full patch query for the confirmation dialog's View diff action.
    #[must_use]
    pub fn diff_spec_for(&self, query: &AbandonQuery) -> JjCommandSpec {
        self.details_spec(["diff", "--git", "-r", query.rev()])
            .with_title(format!("jj diff --git -r {}", query.rev()))
    }

    fn details_spec<'a>(&self, argv: impl IntoIterator<Item = &'a str>) -> JjCommandSpec {
        let global_options = self
            .global_options
            .clone()
            .with_working_copy(WorkingCopyPolicy::SnapshotAndUpdate)
            .with_output(OutputPolicy {
                color: ColorPolicy::Never,
                ..OutputPolicy::default()
            });
        JjCommandSpec::render_read_only(argv).with_global_options(global_options)
    }

    fn run_details_command(
        runner: &mut impl JjCommandRunner,
        spec: &JjCommandSpec,
    ) -> Result<String, AbandonDetailsError> {
        let output = runner.run(spec)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            Err(AbandonDetailsError::CommandFailed {
                title: spec.title().to_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
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

/// Error returned while loading abandon-confirmation information.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AbandonDetailsError {
    /// A required `jj` command could not be started or read.
    #[error("failed to load abandon confirmation details: {0}")]
    Io(#[from] std::io::Error),

    /// A required `jj` command exited unsuccessfully.
    #[error("{title} failed: {stderr}")]
    CommandFailed {
        /// The command's user-visible title.
        title: String,
        /// The command's error output.
        stderr: String,
    },

    /// The metadata query did not emit one complete record.
    #[error("jj abandon details returned unexpected metadata: {0}")]
    InvalidMetadata(String),

    /// The descendant-count query did not emit one non-negative integer.
    #[error("jj abandon details returned an invalid descendant count: {0}")]
    InvalidDescendantCount(String),

    /// The file-stat query did not emit complete, typed file records.
    #[error("jj abandon details returned invalid file stats: {0}")]
    InvalidFiles(String),
}

fn parse_empty_probe_output(stdout: &[u8]) -> Result<bool, AbandonProbeError> {
    match String::from_utf8_lossy(stdout).trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        value => Err(AbandonProbeError::InvalidOutput(value.to_owned())),
    }
}

fn parse_details_metadata(output: &str) -> Result<(String, String, bool), AbandonDetailsError> {
    let output = output.trim_end_matches(['\r', '\n']);
    let Some((change_id, remainder)) = output.split_once('\t') else {
        return Err(AbandonDetailsError::InvalidMetadata(output.to_owned()));
    };
    let Some((description, is_working_copy)) = remainder.split_once('\t') else {
        return Err(AbandonDetailsError::InvalidMetadata(output.to_owned()));
    };
    if is_working_copy.contains('\t') {
        return Err(AbandonDetailsError::InvalidMetadata(output.to_owned()));
    }

    let change_id = serde_json::from_str(change_id)
        .map_err(|_| AbandonDetailsError::InvalidMetadata(output.to_owned()))?;
    let description = serde_json::from_str(description)
        .map_err(|_| AbandonDetailsError::InvalidMetadata(output.to_owned()))?;
    let is_working_copy = serde_json::from_str(is_working_copy)
        .map_err(|_| AbandonDetailsError::InvalidMetadata(output.to_owned()))?;
    Ok((change_id, description, is_working_copy))
}

fn parse_descendant_count(output: &str) -> Result<usize, AbandonDetailsError> {
    let value = output.trim();
    value
        .parse()
        .map_err(|_| AbandonDetailsError::InvalidDescendantCount(value.to_owned()))
}

fn parse_files(output: &str) -> Result<Vec<AbandonFile>, AbandonDetailsError> {
    output
        .lines()
        .map(|line| {
            let mut fields = line.split('\t');
            let (Some(path), Some(status), Some(added), Some(removed), None) = (
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
                fields.next(),
            ) else {
                return Err(AbandonDetailsError::InvalidFiles(line.to_owned()));
            };
            let path = serde_json::from_str(path)
                .map_err(|_| AbandonDetailsError::InvalidFiles(line.to_owned()))?;
            let status: String = serde_json::from_str(status)
                .map_err(|_| AbandonDetailsError::InvalidFiles(line.to_owned()))?;
            if !matches!(status.as_str(), "A" | "M" | "D" | "C" | "R") {
                return Err(AbandonDetailsError::InvalidFiles(line.to_owned()));
            }
            let added = serde_json::from_str(added)
                .map_err(|_| AbandonDetailsError::InvalidFiles(line.to_owned()))?;
            let removed = serde_json::from_str(removed)
                .map_err(|_| AbandonDetailsError::InvalidFiles(line.to_owned()))?;
            Ok(AbandonFile {
                path,
                status,
                added,
                removed,
            })
        })
        .collect()
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
    fn details_specs_snapshot_and_request_each_confirmation_field() {
        let source = JjAbandon::default().with_repository("/tmp/repo");
        let query = AbandonQuery::new("abc123");

        assert_eq!(
            strings(source.details_spec_for(&query).process_argv().as_slice()),
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
                DETAILS_TEMPLATE,
            ]
        );
        assert_eq!(
            strings(source.files_spec_for(&query).argv()),
            vec!["log", "-r", "abc123", "--no-graph", "-T", FILES_TEMPLATE]
        );
        assert_eq!(
            strings(source.descendant_count_spec_for(&query).argv()),
            vec!["log", "-r", "(abc123):: ~ (abc123)", "--count"]
        );
        assert_eq!(
            strings(source.diff_spec_for(&query).argv()),
            vec!["diff", "--git", "-r", "abc123"]
        );
    }

    #[test]
    fn details_metadata_parser_preserves_description_and_working_copy_marker()
    -> Result<(), AbandonDetailsError> {
        let parsed =
            parse_details_metadata("\"zxywvu\"\t\"Explain the change\\nwith context\\n\"\ttrue\n");

        let details = parsed?;
        assert_eq!(details.0, "zxywvu");
        assert_eq!(details.1, "Explain the change\nwith context\n");
        assert!(details.2);
        Ok(())
    }

    #[test]
    fn details_parsers_reject_incomplete_metadata_and_invalid_counts() {
        assert!(matches!(
            parse_details_metadata("\"zxywvu\"\t\"description\"\n"),
            Err(AbandonDetailsError::InvalidMetadata(_))
        ));
        assert!(matches!(
            parse_files("\"file\"\t\"X\"\t1\t0\n"),
            Err(AbandonDetailsError::InvalidFiles(_))
        ));
        assert!(matches!(
            parse_descendant_count("several\n"),
            Err(AbandonDetailsError::InvalidDescendantCount(value)) if value == "several"
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
        .with_global_options(
            GlobalOptions::default()
                .with_repository(&repository)
                .with_output(OutputPolicy {
                    color: ColorPolicy::Never,
                    ..OutputPolicy::default()
                }),
        );
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

        let id_output = runner.run(&source.details_spec_for(&query))?;
        let (change_id, _, _) = parse_details_metadata(&String::from_utf8(id_output.stdout)?)?;
        let inspected_query = AbandonQuery::new(change_id);
        let details = source.details_with_runner(&inspected_query, &mut runner)?;
        assert_eq!(details.files.len(), 1);
        assert_eq!(details.files[0].path, "file.txt");
        assert_eq!(details.files[0].status, "D");
        assert_eq!(details.files[0].added, 0);
        assert_eq!(details.files[0].removed, 1);
        assert_eq!(details.descendant_count, 0);
        assert!(details.diff.contains("file.txt"));
        for _ in 0..2 {
            assert!(runner.run(&new_spec)?.status.success());
        }
        let details = source.details_with_runner(&inspected_query, &mut runner)?;
        assert_eq!(details.descendant_count, 2);
        assert!(!details.is_working_copy);
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
