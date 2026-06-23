//! `jj workspace list` command integration.

use std::path::{Path, PathBuf};

use jk_core::JjCommandSpec;
use thiserror::Error;

use crate::command::run_jj_spec;

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
        self.repository = Some(repository.into());
        self
    }

    /// Loads and parses the workspace list.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed, exits unsuccessfully, or returns malformed
    /// machine output.
    pub fn load_list(&self) -> Result<WorkspaceListSnapshot, JjWorkspacesError> {
        let spec = self.list_spec();
        let output = run_jj_spec(&spec)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            return Err(JjWorkspacesError::CommandFailed(stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let workspaces = self.parse_list(&stdout)?;
        Ok(WorkspaceListSnapshot {
            workspaces,
            title: spec.title().to_owned(),
        })
    }

    /// Returns the `jj workspace list` command spec.
    #[must_use]
    pub fn list_spec(&self) -> JjCommandSpec {
        let spec = JjCommandSpec::render_read_only([
            "workspace",
            "list",
            "--template",
            WORKSPACE_LIST_TEMPLATE,
        ]);
        self.with_repository_if_configured(spec)
    }

    /// Returns the `jj status` command spec scoped to `query`.
    #[must_use]
    pub fn status_spec(&self, query: &WorkspaceInspectionQuery) -> JjCommandSpec {
        JjCommandSpec::render_read_only(["status"]).with_repository(query.workspace_root())
    }

    /// Returns the `jj diff` command spec scoped to `query`.
    #[must_use]
    pub fn diff_spec(&self, query: &WorkspaceInspectionQuery) -> JjCommandSpec {
        JjCommandSpec::render_read_only(["diff"]).with_repository(query.workspace_root())
    }

    fn parse_list(&self, stdout: &str) -> Result<Vec<WorkspaceSummary>, JjWorkspacesError> {
        Ok(parse_workspace_list(stdout, self.repository.as_deref())?)
    }

    fn with_repository_if_configured(&self, spec: JjCommandSpec) -> JjCommandSpec {
        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }
}

/// Error returned while loading workspace data from `jj`.
#[derive(Debug, Error)]
pub enum JjWorkspacesError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj workspace list: {0}")]
    Io(#[from] std::io::Error),

    /// `jj workspace list` exited unsuccessfully.
    #[error("jj workspace list failed: {0}")]
    CommandFailed(String),

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
                "always",
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
        let source = JjWorkspaces::default()
            .with_repository(std::env::current_dir().expect("current directory should exist"));

        let workspaces = source.parse_list(&stdout).expect("rows should parse");

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
