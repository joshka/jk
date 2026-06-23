//! Read-only `jj op ...` command integration.

use jk_core::{GlobalOptions, InspectionSnapshot, JjCommandSpec};
use thiserror::Error;

use crate::command::{JjCommandRunner, SystemJjCommandRunner};

const OP_COMMAND: &str = "op";
const LOG_COMMAND: &str = "log";
const SHOW_COMMAND: &str = "show";
const DIFF_COMMAND: &str = "diff";

/// Canonical query shapes supported by operation inspection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationQuery {
    /// Render `jj op log`.
    Log,
    /// Render `jj op show OPERATION`.
    Show {
        /// Operation to inspect.
        operation: String,
    },
    /// Render `jj op diff OPERATION`.
    Diff {
        /// Operation to diff.
        operation: String,
    },
}

impl OperationQuery {
    /// Creates a `jj op log` query.
    #[must_use]
    pub const fn log() -> Self {
        Self::Log
    }

    /// Creates a `jj op show OPERATION` query.
    #[must_use]
    pub fn show(operation: impl Into<String>) -> Self {
        Self::Show {
            operation: operation.into(),
        }
    }

    /// Creates a `jj op diff OPERATION` query.
    #[must_use]
    pub fn diff(operation: impl Into<String>) -> Self {
        Self::Diff {
            operation: operation.into(),
        }
    }

    /// Returns a compact target label for error and empty-output states.
    #[must_use]
    pub fn target_label(&self) -> String {
        match self {
            Self::Log => "operations".to_owned(),
            Self::Show { operation } | Self::Diff { operation } => operation.clone(),
        }
    }
}

/// Loads rendered read-only `jj op ...` output.
#[derive(Clone, Debug, Default)]
pub struct JjOperation {
    global_options: GlobalOptions,
}

impl JjOperation {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Sets the global `jj` options used by generated operation specs.
    #[must_use]
    pub fn with_global_options(mut self, global_options: GlobalOptions) -> Self {
        self.global_options = global_options;
        self
    }

    /// Loads rendered operation output for `query`.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_query(
        &self,
        query: &OperationQuery,
    ) -> Result<InspectionSnapshot, JjOperationError> {
        self.load_query_with_runner(query, &mut SystemJjCommandRunner)
    }

    /// Loads rendered operation output for `query` using the provided command runner.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed or exits unsuccessfully.
    pub fn load_query_with_runner(
        &self,
        query: &OperationQuery,
        runner: &mut impl JjCommandRunner,
    ) -> Result<InspectionSnapshot, JjOperationError> {
        let spec = self.spec_for(query);
        let rendered = Self::run(runner, &spec)?;
        Ok(InspectionSnapshot::new(query.target_label(), rendered).with_title(spec.title()))
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &OperationQuery) -> JjCommandSpec {
        let argv = match query {
            OperationQuery::Log => vec![OP_COMMAND, LOG_COMMAND],
            OperationQuery::Show { operation } => vec![OP_COMMAND, SHOW_COMMAND, operation],
            OperationQuery::Diff { operation } => vec![OP_COMMAND, DIFF_COMMAND, operation],
        };

        JjCommandSpec::render_read_only(argv).with_global_options(self.global_options.clone())
    }

    fn run(
        runner: &mut impl JjCommandRunner,
        spec: &JjCommandSpec,
    ) -> Result<String, JjOperationError> {
        let output = runner.run(spec)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            Err(JjOperationError::CommandFailed(stderr))
        }
    }
}

/// Error returned while loading rendered `jj op ...` output.
#[derive(Debug, Error)]
pub enum JjOperationError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj op command: {0}")]
    Io(#[from] std::io::Error),

    /// `jj op ...` exited unsuccessfully.
    #[error("jj op command failed: {0}")]
    CommandFailed(String),
}

#[cfg(test)]
mod tests {
    use std::io;
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    use std::process::Output;

    use jk_core::{
        ConfigOverlay, ExecutionMode, OperationLoadPolicy, OutputPolicy, PagerPolicy, RefreshPlan,
        SafetyClass,
    };

    use super::*;

    fn strings(argv: impl IntoIterator<Item = std::ffi::OsString>) -> Vec<String> {
        argv.into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn op_log_builds_read_only_render_spec() {
        let spec = JjOperation::default().spec_for(&OperationQuery::log());

        assert_eq!(strings(spec.argv().to_vec()), vec!["op", "log"]);
        assert_eq!(spec.title(), "jj op log");
        assert_eq!(spec.safety(), SafetyClass::ReadOnly);
        assert_eq!(spec.mode(), ExecutionMode::RenderReadOnly);
        assert_eq!(spec.refresh_plan(), RefreshPlan::ReRunSpec);
    }

    #[test]
    fn op_show_builds_read_only_render_spec() {
        let spec = JjOperation::default().spec_for(&OperationQuery::show("abc123"));

        assert_eq!(strings(spec.argv().to_vec()), vec!["op", "show", "abc123"]);
        assert_eq!(spec.title(), "jj op show abc123");
        assert_eq!(spec.safety(), SafetyClass::ReadOnly);
        assert_eq!(spec.mode(), ExecutionMode::RenderReadOnly);
        assert_eq!(spec.refresh_plan(), RefreshPlan::ReRunSpec);
    }

    #[test]
    fn op_diff_builds_read_only_render_spec() {
        let spec = JjOperation::default().spec_for(&OperationQuery::diff("abc123"));

        assert_eq!(strings(spec.argv().to_vec()), vec!["op", "diff", "abc123"]);
        assert_eq!(spec.title(), "jj op diff abc123");
        assert_eq!(spec.safety(), SafetyClass::ReadOnly);
        assert_eq!(spec.mode(), ExecutionMode::RenderReadOnly);
        assert_eq!(spec.refresh_plan(), RefreshPlan::ReRunSpec);
    }

    #[test]
    fn global_options_render_before_operation_command_family() {
        let global_options = GlobalOptions::default()
            .with_output(OutputPolicy {
                pager: PagerPolicy::Disable,
                ..OutputPolicy::default()
            })
            .with_repository("/tmp/repo")
            .with_operation(OperationLoadPolicy::AtOperation("root-op".to_owned()))
            .with_config_overlay(ConfigOverlay::Inline {
                name_value: "ui.color=always".to_owned(),
            });
        let spec = JjOperation::default()
            .with_global_options(global_options)
            .spec_for(&OperationQuery::diff("abc123"));

        assert_eq!(
            strings(spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repo",
                "--at-operation",
                "root-op",
                "--config",
                "ui.color=always",
                "op",
                "diff",
                "abc123",
            ]
        );
    }

    #[test]
    fn load_query_runs_operation_spec_and_uses_rendered_output() {
        let mut runner = FakeRunner::success("operation output\n", "");
        let snapshot = JjOperation::default()
            .load_query_with_runner(&OperationQuery::show("abc123"), &mut runner)
            .expect("fake runner succeeds");

        assert_eq!(snapshot.title(), "jj op show abc123");
        assert_eq!(snapshot.target(), "abc123");
        assert_eq!(snapshot.rendered(), "operation output\n");
        assert_eq!(runner.argv, vec![vec!["op", "show", "abc123"]]);
    }

    #[test]
    fn load_query_reports_failed_operation_command() {
        let mut runner = FakeRunner::failure("not found\n");
        let error = JjOperation::default()
            .load_query_with_runner(&OperationQuery::diff("missing"), &mut runner)
            .expect_err("fake runner fails");

        assert!(matches!(
            error,
            JjOperationError::CommandFailed(message) if message == "not found"
        ));
    }

    struct FakeRunner {
        result: io::Result<Output>,
        argv: Vec<Vec<String>>,
    }

    impl FakeRunner {
        fn success(stdout: &str, stderr: &str) -> Self {
            Self {
                result: Ok(Output {
                    status: exit_status(0),
                    stdout: stdout.as_bytes().to_vec(),
                    stderr: stderr.as_bytes().to_vec(),
                }),
                argv: Vec::new(),
            }
        }

        fn failure(stderr: &str) -> Self {
            Self {
                result: Ok(Output {
                    status: exit_status(1),
                    stdout: Vec::new(),
                    stderr: stderr.as_bytes().to_vec(),
                }),
                argv: Vec::new(),
            }
        }
    }

    impl JjCommandRunner for FakeRunner {
        fn run(&mut self, spec: &JjCommandSpec) -> io::Result<Output> {
            self.argv.push(strings(spec.argv().to_vec()));
            std::mem::replace(
                &mut self.result,
                Err(io::Error::other("fake runner result already consumed")),
            )
        }
    }

    #[cfg(unix)]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(not(unix))]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
            .args(if cfg!(windows) {
                vec!["/C".into(), format!("exit {code}").into()]
            } else {
                vec!["-c".into(), format!("exit {code}").into()]
            })
            .status()
            .unwrap()
    }
}
