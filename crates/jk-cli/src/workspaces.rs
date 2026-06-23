//! `jj workspace list` command integration.

use std::path::{Path, PathBuf};

use jk_core::{
    ColorPolicy, ExecutionMode, GlobalOptions, InspectionSnapshot, JjCommandSpec, OutputPolicy,
    RefreshPlan, SafetyClass,
};
use thiserror::Error;

use crate::command::{JjCommandRunner, SystemJjCommandRunner};

const WORKSPACE_LIST_TEMPLATE: &str = r#"name ++ "\t" ++ root ++ "\t" ++ target.change_id().short() ++ "\t" ++ target.commit_id().short() ++ "\n""#;

/// A parsed snapshot of known `jj` workspaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceListSnapshot {
    /// Parsed workspace rows.
    pub workspaces: Vec<WorkspaceSummary>,
    /// Display title for the command that produced the snapshot.
    pub title: String,
}

/// One workspace row from `jj workspace list`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSummary {
    /// Workspace name.
    pub name: String,
    /// Workspace root reported by `jj`, when present.
    pub root: Option<PathBuf>,
    /// Whether this row matches the configured repository path.
    pub current: bool,
    /// Short target change id.
    pub change_id: Option<String>,
    /// Short target commit id.
    pub commit_id: Option<String>,
}

/// Output from a selected-workspace stale metadata update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceUpdateStaleOutcome {
    /// Display title for the command that ran.
    pub title: String,
    /// Trimmed stdout emitted by `jj`.
    pub stdout: String,
    /// Trimmed stderr emitted by `jj`.
    pub stderr: String,
}

/// Query for rendering an inspection command in a selected workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceInspectionQuery {
    workspace_root: PathBuf,
}

impl WorkspaceInspectionQuery {
    /// Creates a selected-workspace query.
    #[must_use]
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }

    /// Returns the workspace root passed to `jj --repository`.
    #[must_use]
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Returns a compact target label for error and empty-output states.
    #[must_use]
    pub fn target_label(&self) -> String {
        self.workspace_root.display().to_string()
    }
}

/// Selected-workspace inspection command that failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceInspectionCommand {
    /// `jj status`.
    Status,
    /// `jj diff`.
    Diff,
}

impl std::fmt::Display for WorkspaceInspectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status => f.write_str("jj status"),
            Self::Diff => f.write_str("jj diff"),
        }
    }
}

/// Loads workspace data from `jj`.
#[derive(Clone, Debug, Default)]
pub struct JjWorkspaces {
    repository: Option<PathBuf>,
}

impl JjWorkspaces {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(normalized_path(&repository.into()));
        self
    }

    /// Loads and parses the workspace list.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed, exits unsuccessfully, or returns malformed
    /// machine output.
    pub fn load_list(&self) -> Result<WorkspaceListSnapshot, JjWorkspacesError> {
        self.load_list_with_runner(&mut SystemJjCommandRunner)
    }

    /// Loads and parses the workspace list using the provided command runner.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed, exits unsuccessfully, or returns malformed
    /// machine output.
    pub fn load_list_with_runner(
        &self,
        runner: &mut impl JjCommandRunner,
    ) -> Result<WorkspaceListSnapshot, JjWorkspacesError> {
        let spec = self.list_spec();
        let output = runner.run(&spec)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(JjWorkspacesError::CommandFailed(stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let current_root = self
            .load_current_root_with_runner(runner)
            .ok()
            .or_else(|| self.repository.clone());
        let workspaces = parse_workspace_list(&stdout, current_root.as_deref())?;
        Ok(WorkspaceListSnapshot {
            workspaces,
            title: spec.title().to_owned(),
        })
    }

    /// Loads rendered `jj status` output scoped to a selected workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_status(
        &self,
        query: &WorkspaceInspectionQuery,
    ) -> Result<InspectionSnapshot, JjWorkspacesError> {
        self.load_status_with_runner(query, &mut SystemJjCommandRunner)
    }

    /// Loads rendered `jj status` output scoped to a selected workspace using the provided runner.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_status_with_runner(
        &self,
        query: &WorkspaceInspectionQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<InspectionSnapshot, JjWorkspacesError> {
        let spec = self.status_spec(query);
        let rendered = Self::run_inspection(runner, &spec, WorkspaceInspectionCommand::Status)?;
        Ok(InspectionSnapshot::new(query.target_label(), rendered).with_title(spec.title()))
    }

    /// Loads rendered `jj diff` output scoped to a selected workspace.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_diff(
        &self,
        query: &WorkspaceInspectionQuery,
    ) -> Result<InspectionSnapshot, JjWorkspacesError> {
        self.load_diff_with_runner(query, &mut SystemJjCommandRunner)
    }

    /// Loads rendered `jj diff` output scoped to a selected workspace using the provided runner.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_diff_with_runner(
        &self,
        query: &WorkspaceInspectionQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<InspectionSnapshot, JjWorkspacesError> {
        let spec = self.diff_spec(query);
        let rendered = Self::run_inspection(runner, &spec, WorkspaceInspectionCommand::Diff)?;
        Ok(InspectionSnapshot::new(query.target_label(), rendered).with_title(spec.title()))
    }

    /// Runs `jj workspace update-stale` scoped to a selected workspace root.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn update_stale(
        &self,
        query: &WorkspaceInspectionQuery,
    ) -> Result<WorkspaceUpdateStaleOutcome, JjWorkspacesError> {
        self.update_stale_with_runner(query, &mut SystemJjCommandRunner)
    }

    /// Runs `jj workspace update-stale` scoped to a selected workspace root using the runner.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn update_stale_with_runner(
        &self,
        query: &WorkspaceInspectionQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<WorkspaceUpdateStaleOutcome, JjWorkspacesError> {
        let spec = self.update_stale_spec(query);
        let output = runner.run(&spec)?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        if output.status.success() {
            Ok(WorkspaceUpdateStaleOutcome {
                title: spec.title().to_owned(),
                stdout,
                stderr,
            })
        } else {
            Err(JjWorkspacesError::UpdateStaleCommandFailed {
                command: spec.title().to_owned(),
                stderr,
                stdout,
                status: output.status.to_string(),
            })
        }
    }

    /// Returns the `jj workspace list` command spec.
    #[must_use]
    pub fn list_spec(&self) -> JjCommandSpec {
        let output = OutputPolicy {
            color: ColorPolicy::Never,
            ..OutputPolicy::default()
        };
        let spec = JjCommandSpec::render_read_only([
            "workspace",
            "list",
            "--template",
            WORKSPACE_LIST_TEMPLATE,
        ])
        .with_global_options(GlobalOptions::default().with_output(output))
        .with_title("jj workspace list");
        self.with_repository_if_configured(spec)
    }

    /// Returns the `jj status` command spec scoped to `query`.
    #[must_use]
    pub fn status_spec(&self, query: &WorkspaceInspectionQuery) -> JjCommandSpec {
        let title = format!("jj -R {} status", query.workspace_root().display());
        JjCommandSpec::render_read_only(["status"])
            .with_repository(query.workspace_root())
            .with_title(title)
    }

    /// Returns the `jj diff` command spec scoped to `query`.
    #[must_use]
    pub fn diff_spec(&self, query: &WorkspaceInspectionQuery) -> JjCommandSpec {
        let title = format!("jj -R {} diff", query.workspace_root().display());
        JjCommandSpec::render_read_only(["diff"])
            .with_repository(query.workspace_root())
            .with_title(title)
    }

    /// Returns the `jj workspace update-stale` command spec scoped to `query`.
    #[must_use]
    pub fn update_stale_spec(&self, query: &WorkspaceInspectionQuery) -> JjCommandSpec {
        let output = OutputPolicy {
            color: ColorPolicy::Never,
            ..OutputPolicy::default()
        };
        let title = format!(
            "jj -R {} workspace update-stale",
            query.workspace_root().display()
        );
        JjCommandSpec::render_read_only(["workspace", "update-stale"])
            .with_global_options(GlobalOptions::default().with_output(output))
            .with_repository(query.workspace_root())
            .with_title(title)
            .with_mode(ExecutionMode::ConfirmMutation)
            .with_safety(SafetyClass::LocalMetadata)
            .with_refresh_plan(RefreshPlan::None)
    }

    /// Returns the `jj root` command spec used for current-workspace selection.
    #[must_use]
    pub fn root_spec(&self) -> JjCommandSpec {
        let output = OutputPolicy {
            color: ColorPolicy::Never,
            ..OutputPolicy::default()
        };
        let spec = JjCommandSpec::render_read_only(["root"])
            .with_global_options(GlobalOptions::default().with_output(output))
            .with_title("jj root");
        self.with_repository_if_configured(spec)
    }

    fn with_repository_if_configured(&self, spec: JjCommandSpec) -> JjCommandSpec {
        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }

    fn run_inspection(
        runner: &mut impl JjCommandRunner,
        spec: &JjCommandSpec,
        command: WorkspaceInspectionCommand,
    ) -> Result<String, JjWorkspacesError> {
        let output = runner.run(spec)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Err(JjWorkspacesError::InspectionCommandFailed { command, stderr })
        }
    }

    fn load_current_root_with_runner(
        &self,
        runner: &mut impl JjCommandRunner,
    ) -> Result<PathBuf, JjWorkspacesError> {
        let output = runner.run(&self.root_spec())?;
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            Ok(normalized_path(Path::new(&root)))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Err(JjWorkspacesError::CommandFailed(stderr))
        }
    }
}

/// Error returned while loading workspace data from `jj`.
#[derive(Debug, Error)]
pub enum JjWorkspacesError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj workspace command: {0}")]
    Io(#[from] std::io::Error),

    /// `jj workspace list` exited unsuccessfully.
    #[error("jj workspace list failed: {0}")]
    CommandFailed(String),

    /// Selected-workspace inspection exited unsuccessfully.
    #[error("{command} failed: {stderr}")]
    InspectionCommandFailed {
        /// The failed `jj` command.
        command: WorkspaceInspectionCommand,
        /// Trimmed stderr from `jj`.
        stderr: String,
    },

    /// Selected-workspace stale metadata update exited unsuccessfully.
    #[error("{command} failed: {}", command_error_summary(stderr, stdout, status))]
    UpdateStaleCommandFailed {
        /// Display title for the failed command.
        command: String,
        /// Trimmed stderr from `jj`.
        stderr: String,
        /// Trimmed stdout from `jj`.
        stdout: String,
        /// Process exit status.
        status: String,
    },

    /// `jj workspace list` returned output that did not match the machine template.
    #[error("failed to parse jj workspace list output: {0}")]
    Parse(#[from] WorkspaceListParseError),
}

/// Error returned when machine-formatted workspace rows are malformed.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("line {line}: expected 4 tab-separated fields, got {field_count} in {record:?}")]
pub struct WorkspaceListParseError {
    line: usize,
    field_count: usize,
    record: String,
}

fn parse_workspace_list(
    stdout: &str,
    repository: Option<&Path>,
) -> Result<Vec<WorkspaceSummary>, WorkspaceListParseError> {
    let current_repository = repository.map(normalized_path);
    let mut workspaces = Vec::new();

    for (index, record) in stdout.lines().enumerate() {
        if record.is_empty() {
            continue;
        }

        let fields = record.split('\t').collect::<Vec<_>>();
        if fields.len() != 4 || fields[0].is_empty() {
            return Err(WorkspaceListParseError {
                line: index + 1,
                field_count: fields.len(),
                record: record.to_owned(),
            });
        }

        let root = optional_path(fields[1]);
        let current = root
            .as_deref()
            .zip(current_repository.as_deref())
            .is_some_and(|(root, repository)| normalized_path(root) == repository);

        workspaces.push(WorkspaceSummary {
            name: fields[0].to_owned(),
            root,
            current,
            change_id: optional_string(fields[2]),
            commit_id: optional_string(fields[3]),
        });
    }

    Ok(workspaces)
}

fn optional_path(value: &str) -> Option<PathBuf> {
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

fn optional_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn normalized_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn command_error_summary(stderr: &str, stdout: &str, status: &str) -> String {
    if !stderr.is_empty() {
        stderr.to_owned()
    } else if !stdout.is_empty() {
        stdout.to_owned()
    } else {
        status.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::build_jj_command;

    fn strings(args: &[std::ffi::OsString]) -> Vec<String> {
        args.iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn list_spec_uses_machine_template_and_repository() {
        let source = JjWorkspaces::default().with_repository("/tmp/repo");
        let spec = source.list_spec();

        assert_eq!(
            strings(spec.argv()),
            vec!["workspace", "list", "--template", WORKSPACE_LIST_TEMPLATE]
        );
        assert_eq!(spec.repository(), Some(Path::new("/tmp/repo")));
        assert_eq!(
            strings(&spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "never",
                "--repository",
                "/tmp/repo",
                "workspace",
                "list",
                "--template",
                WORKSPACE_LIST_TEMPLATE
            ]
        );
    }

    #[test]
    fn list_spec_canonicalizes_relative_repository() {
        let source = JjWorkspaces::default().with_repository(".");
        let spec = source.list_spec();
        let current_dir = std::env::current_dir().expect("current directory should exist");

        assert_eq!(spec.repository(), Some(current_dir.as_path()));
    }

    #[test]
    fn list_spec_without_repository_uses_cwd_discovery() {
        let spec = JjWorkspaces::default().list_spec();

        assert_eq!(spec.repository(), None);
        assert_eq!(
            strings(&spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "never",
                "workspace",
                "list",
                "--template",
                WORKSPACE_LIST_TEMPLATE
            ]
        );
    }

    #[test]
    fn root_spec_uses_plain_jj_root_for_current_selection() {
        let spec = JjWorkspaces::default().root_spec();

        assert_eq!(
            strings(&spec.process_argv()),
            vec!["--no-pager", "--color", "never", "root"]
        );
    }

    #[test]
    fn status_and_diff_specs_render_repository_before_command() {
        let query = WorkspaceInspectionQuery::new("/tmp/workspace");

        let status = build_jj_command(&JjWorkspaces::default().status_spec(&query));
        let status_args = status
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            status_args,
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/workspace",
                "status"
            ]
        );

        let diff = build_jj_command(&JjWorkspaces::default().diff_spec(&query));
        let diff_args = diff
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            diff_args,
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/workspace",
                "diff"
            ]
        );
    }

    #[test]
    fn update_stale_spec_scopes_command_to_selected_workspace() {
        use jk_core::{ExecutionMode, SafetyClass};

        let query = WorkspaceInspectionQuery::new("/tmp/workspace");
        let spec = JjWorkspaces::default().update_stale_spec(&query);
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
                "never",
                "--repository",
                "/tmp/workspace",
                "workspace",
                "update-stale"
            ]
        );
        assert_eq!(spec.title(), "jj -R /tmp/workspace workspace update-stale");
        assert_eq!(spec.repository(), Some(Path::new("/tmp/workspace")));
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::LocalMetadata);
    }

    #[test]
    fn parser_handles_multiple_rows_and_optional_fields() {
        let stdout = concat!(
            "default\t/tmp/repo\tabc123\tdef456\n",
            "scratch\t\tfedcba\t\n",
            "empty-target\t/tmp/other\t\t\n",
        );

        let workspaces = parse_workspace_list(stdout, None).expect("rows should parse");

        assert_eq!(
            workspaces,
            vec![
                WorkspaceSummary {
                    name: "default".to_owned(),
                    root: Some(PathBuf::from("/tmp/repo")),
                    current: false,
                    change_id: Some("abc123".to_owned()),
                    commit_id: Some("def456".to_owned()),
                },
                WorkspaceSummary {
                    name: "scratch".to_owned(),
                    root: None,
                    current: false,
                    change_id: Some("fedcba".to_owned()),
                    commit_id: None,
                },
                WorkspaceSummary {
                    name: "empty-target".to_owned(),
                    root: Some(PathBuf::from("/tmp/other")),
                    current: false,
                    change_id: None,
                    commit_id: None,
                },
            ]
        );
    }

    #[test]
    fn parser_marks_current_workspace_by_repository_root() {
        let stdout = format!(
            "default\t{}\tabc123\tdef456\nother\t/tmp/other\tfedcba\t654321\n",
            std::env::current_dir()
                .expect("current directory should exist")
                .display()
        );
        let current_dir = std::env::current_dir().expect("current directory should exist");

        let workspaces =
            parse_workspace_list(&stdout, Some(&current_dir)).expect("rows should parse");

        assert!(workspaces[0].current);
        assert!(!workspaces[1].current);
    }

    #[test]
    fn parser_reports_malformed_records() {
        let error = parse_workspace_list("default\t/tmp/repo\tabc123\n", None)
            .expect_err("missing commit id should fail");

        assert_eq!(
            error,
            WorkspaceListParseError {
                line: 1,
                field_count: 3,
                record: "default\t/tmp/repo\tabc123".to_owned(),
            }
        );
        assert!(
            error
                .to_string()
                .contains("line 1: expected 4 tab-separated fields")
        );
    }
}
