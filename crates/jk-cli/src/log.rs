//! Log-like `jj` command integration.

use std::path::PathBuf;
#[cfg(test)]
use std::process::Command;

use jk_core::{JjCommandSpec, LogSnapshot};
use thiserror::Error;

#[cfg(test)]
use crate::command::build_jj_command;
use crate::command::run_jj_spec;

mod rendered;
mod semantic;

use rendered::assign_rendered_lines;
use semantic::{LOG_TEMPLATE, parse_log_json_lines};

const LOG_COMMAND: &str = "log";

/// Loads a log-like view from the local `jj` command.
///
/// The configured-default view intentionally invokes bare `jj` so jj owns its configured default
/// command, revset, graph, and template. A second semantic pass consumes newline-delimited JSON
/// emitted by jj's own template engine. That second pass is a narrow adapter while the direct
/// `jj-cli`/`jj-lib` integration contract is still being proved.
///
/// This bridge exists because the reusable jj layers do not currently expose the exact contract
/// `jk` needs. `jj-lib` owns repository and revset machinery, but not the configured CLI log view.
/// `jj-cli` owns the log behavior, but the useful path is still command-shaped and writes through a
/// terminal-oriented `Ui` instead of returning semantic row events beside rendered bytes. Until jj
/// exposes that boundary, spawning `jj` is the least duplicative way to preserve user-visible log
/// behavior.
///
/// This loader runs `jj` as a child process and removes common color-suppression environment
/// variables so the rendered pass keeps the configured terminal colors. Configured default commands
/// must be log-like enough to accept the semantic template pass; unsupported commands return
/// [`JjLogError::UnsupportedSemanticCommand`].
#[derive(Clone, Debug)]
pub struct JjLog {
    repository: Option<PathBuf>,
    command: JjLogCommand,
    limit: Option<usize>,
}

impl Default for JjLog {
    fn default() -> Self {
        Self {
            repository: None,
            command: JjLogCommand::ConfiguredDefault,
            limit: None,
        }
    }
}

/// Which jj command should provide the log-like view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum JjLogCommand {
    /// Run bare `jj` so jj resolves its configured default command.
    ConfiguredDefault,

    /// Run jj's explicit `log` command.
    Log,
}

impl JjLog {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Sets the jj command that provides the log-like view.
    #[must_use]
    pub const fn with_command(mut self, command: JjLogCommand) -> Self {
        self.command = command;
        self
    }

    /// Sets the maximum number of log entries to load.
    #[must_use]
    pub const fn with_limit(mut self, limit: Option<usize>) -> Self {
        self.limit = limit;
        self
    }

    /// Loads a rendered log snapshot and semantic entries from `jj`.
    ///
    /// This method executes `jj` twice: once for the user's rendered log output and once with a
    /// JSON template for navigation metadata. A failed retry is useful when the repository state or
    /// `jj` configuration has changed; parse and unsupported-command errors usually need
    /// configuration or integration changes instead.
    ///
    /// # Errors
    ///
    /// Returns an error if `jj` cannot be executed, exits unsuccessfully, emits a JSON record that
    /// does not match the expected schema, or the selected command cannot provide semantic log
    /// records.
    pub fn load(&self) -> Result<LogSnapshot, JjLogError> {
        let command_args = self.command_args();
        let rendered_spec = self.command_spec(DefaultCommandMode::Rendered, &command_args);
        let rendered = self.run(DefaultCommandMode::Rendered, &rendered_spec)?;
        let semantic = self.run(
            DefaultCommandMode::Json,
            &self.command_spec(DefaultCommandMode::Json, &command_args),
        )?;
        let entries = parse_log_json_lines(&semantic)?;
        let entries = assign_rendered_lines(entries, &rendered)?;

        Ok(LogSnapshot::new(rendered, entries).with_title(rendered_spec.title()))
    }

    fn run(&self, mode: DefaultCommandMode, spec: &JjCommandSpec) -> Result<String, JjLogError> {
        let output = run_jj_spec(spec, "always")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
            if mode == DefaultCommandMode::Json {
                Err(JjLogError::UnsupportedSemanticCommand {
                    command: spec.title().to_owned(),
                    stderr,
                })
            } else {
                Err(JjLogError::CommandFailed(stderr))
            }
        }
    }

    #[cfg(test)]
    fn command(&self, mode: DefaultCommandMode, command_args: &[String]) -> Command {
        build_jj_command(&self.command_spec(mode, command_args), "always")
    }

    fn command_spec(&self, mode: DefaultCommandMode, command_args: &[String]) -> JjCommandSpec {
        let mut argv = command_args.to_vec();
        if let Some(limit) = self.limit {
            argv.push("-n".to_owned());
            argv.push(limit.to_string());
        }
        if mode == DefaultCommandMode::Json {
            argv.push("-T".to_owned());
            argv.push(LOG_TEMPLATE.to_owned());
        }

        let spec = JjCommandSpec::render_read_only(argv).with_title(command_title(command_args));
        if let Some(repository) = &self.repository {
            spec.with_repository(repository)
        } else {
            spec
        }
    }

    fn command_args(&self) -> Vec<String> {
        match self.command {
            JjLogCommand::ConfiguredDefault => Vec::new(),
            JjLogCommand::Log => vec![LOG_COMMAND.to_owned()],
        }
    }
}

/// Builds the title-bar command label from the jj command arguments.
fn command_title(command_args: &[String]) -> String {
    let mut title = String::from("jj");
    for arg in command_args {
        title.push(' ');
        title.push_str(arg);
    }
    title
}

/// Which pass is being run against `jj`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DefaultCommandMode {
    Rendered,
    Json,
}

/// Error returned while loading log entries from `jj`.
///
/// The set of variants can grow as `jk` learns more precise `jj` integration failure modes. Callers
/// should include a fallback arm when matching.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum JjLogError {
    /// The `jj` process could not be started or read.
    #[error("failed to run jj log")]
    Io(#[from] std::io::Error),

    /// The `jj` command exited unsuccessfully.
    #[error("jj failed: {0}")]
    CommandFailed(String),

    /// The configured default command cannot provide semantic log records.
    #[error("unsupported jj command for jk log view: {command}: {stderr}")]
    UnsupportedSemanticCommand {
        /// The jj command that failed with the semantic log template.
        command: String,

        /// The stderr emitted by jj.
        stderr: String,
    },

    /// A JSON log record could not be decoded.
    #[error("failed to parse jj log JSON record on line {line}: {source}")]
    Parse {
        /// One-based output line number.
        line: usize,

        /// JSON parser error.
        source: serde_json::Error,
    },

    /// A semantic log record was missing the template-derived details field.
    #[error("missing jj log details field on line {line}")]
    MissingDetails {
        /// One-based output line number.
        line: usize,
    },

    /// The rendered jj log rows could not be aligned with semantic records.
    #[error(
        "jj rendered {rendered_rows} commit rows, but the semantic log template emitted {entries} entries"
    )]
    RenderedEntryMismatch {
        /// Number of commit rows detected in the rendered jj output.
        rendered_rows: usize,

        /// Number of semantic entries emitted by the template pass.
        entries: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendered_command_forces_color_and_ignores_color_suppression_env() {
        let command_args = vec!["log".to_owned()];
        let command = JjLog::default().command(DefaultCommandMode::Rendered, &command_args);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let envs = command
            .get_envs()
            .map(|(key, value)| (key.to_string_lossy().into_owned(), value.is_none()))
            .collect::<Vec<_>>();

        assert!(args.windows(2).any(|args| args == ["--color", "always"]));
        assert!(!args.iter().any(|arg| arg == "-n"));
        assert!(envs.contains(&("NO_COLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR_FORCE".to_owned(), true)));
    }

    #[test]
    fn explicit_log_command_uses_jj_log_with_user_supplied_limit() {
        let source = JjLog::default()
            .with_command(JjLogCommand::Log)
            .with_limit(Some(3));
        let command_args = source.command_args();
        let command = source.command(DefaultCommandMode::Rendered, &command_args);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(args.iter().any(|arg| arg == "log"));
        assert!(args.windows(2).any(|args| args == ["-n", "3"]));
    }

    #[test]
    fn configured_default_command_uses_bare_jj_with_user_supplied_limit() {
        let source = JjLog::default().with_limit(Some(3));
        let command_args = source.command_args();
        let command = source.command(DefaultCommandMode::Json, &command_args);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(command_args.is_empty());
        assert!(!args.iter().any(|arg| arg == "log"));
        assert!(args.windows(2).any(|args| args == ["-n", "3"]));
        assert!(args.windows(2).any(|args| args == ["-T", LOG_TEMPLATE]));
        assert_eq!(command_title(&command_args), "jj");
    }

    #[test]
    fn command_title_names_jj_command_context() {
        let command_args = vec!["log".to_owned(), "-r".to_owned(), "@".to_owned()];

        assert_eq!(command_title(&command_args), "jj log -r @");
    }
}
