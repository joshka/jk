//! Named Git remote, fetch, and push command integration.

use std::path::PathBuf;

use jk_core::{
    ColorPolicy, ExecutionMode, GlobalOptions, JjCommandSpec, OutputPolicy, RefreshPlan,
    SafetyClass,
};
use thiserror::Error;

use crate::command::{JjCommandRunner, SystemJjCommandRunner};

/// One named Git remote configured in a jj repository.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitRemote {
    /// Remote name passed to `--remote`.
    pub name: String,
    /// Configured URL, retained for diagnostics and fixture assertions.
    pub url: String,
}

/// Loads named Git remotes and builds explicit fetch/push specs.
#[derive(Clone, Debug, Default)]
pub struct JjGitRemote {
    repository: Option<PathBuf>,
}

impl JjGitRemote {
    /// Sets the repository passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Loads configured Git remotes.
    ///
    /// # Errors
    ///
    /// Returns an error when `jj` cannot run, exits unsuccessfully, or emits malformed output.
    pub fn load(&self) -> Result<Vec<GitRemote>, JjGitRemoteError> {
        self.load_with_runner(&mut SystemJjCommandRunner)
    }

    /// Loads configured Git remotes with an injected command runner.
    ///
    /// # Errors
    ///
    /// Returns an error when the runner fails, `jj` exits unsuccessfully, or output is malformed.
    pub fn load_with_runner(
        &self,
        runner: &mut impl JjCommandRunner,
    ) -> Result<Vec<GitRemote>, JjGitRemoteError> {
        let spec = self.remote_list_spec();
        let output = runner.run(&spec)?;
        if !output.status.success() {
            return Err(JjGitRemoteError::CommandFailed {
                command: spec.title().to_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }
        parse_remote_list(&String::from_utf8_lossy(&output.stdout)).map_err(Into::into)
    }

    /// Returns the explicit remote-list spec.
    #[must_use]
    pub fn remote_list_spec(&self) -> JjCommandSpec {
        self.with_repository_if_configured(machine_spec(
            ["git", "remote", "list"],
            "jj git remote list",
            SafetyClass::ReadOnly,
            ExecutionMode::RenderReadOnly,
        ))
    }

    /// Returns an explicit network-read fetch spec.
    #[must_use]
    pub fn fetch_spec(&self, remote: &str) -> JjCommandSpec {
        self.with_repository_if_configured(machine_spec(
            ["git", "fetch", "--remote", &exact_string_pattern(remote)],
            "jj git fetch",
            SafetyClass::NetworkRead,
            ExecutionMode::ConfirmNetworkRead,
        ))
    }

    /// Returns a dry-run push spec for exactly one bookmark and remote.
    #[must_use]
    pub fn push_dry_run_spec(&self, remote: &str, bookmark: &str) -> JjCommandSpec {
        self.with_repository_if_configured(machine_spec(
            [
                "git",
                "push",
                "--remote",
                remote,
                "--bookmark",
                &exact_string_pattern(bookmark),
                "--dry-run",
            ],
            "jj git push --dry-run",
            SafetyClass::NetworkRead,
            ExecutionMode::ConfirmNetworkRead,
        ))
    }

    fn with_repository_if_configured(&self, spec: JjCommandSpec) -> JjCommandSpec {
        if let Some(repository) = self.repository.as_deref() {
            spec.with_repository(repository)
        } else {
            spec
        }
    }
}

/// Converts one jj string into exact string-pattern syntax.
///
/// `jj git fetch --remote` and `push --bookmark` accept glob patterns by default. Quoting keeps an
/// action scoped to the selected remote or bookmark.
fn exact_string_pattern(value: &str) -> String {
    let escaped_value = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("exact:\"{escaped_value}\"")
}

fn machine_spec<const N: usize>(
    argv: [&str; N],
    title: &str,
    safety: SafetyClass,
    mode: ExecutionMode,
) -> JjCommandSpec {
    let output = OutputPolicy {
        color: ColorPolicy::Never,
        ..OutputPolicy::default()
    };
    JjCommandSpec::render_read_only(argv)
        .with_global_options(GlobalOptions::default().with_output(output))
        .with_title(title)
        .with_safety(safety)
        .with_mode(mode)
        .with_refresh_plan(RefreshPlan::None)
}

/// Parses `jj git remote list` output, retaining the first whitespace-delimited field as name.
///
/// # Errors
///
/// Returns an error when a non-empty row does not contain both a name and URL.
pub fn parse_remote_list(stdout: &str) -> Result<Vec<GitRemote>, GitRemoteParseError> {
    let mut remotes = Vec::new();
    for (index, line) in stdout.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((name, url)) = line.split_once(char::is_whitespace) else {
            return Err(GitRemoteParseError {
                line: index + 1,
                record: line.to_owned(),
            });
        };
        let name = name.trim();
        let url = url.trim();
        if name.is_empty() || url.is_empty() {
            return Err(GitRemoteParseError {
                line: index + 1,
                record: line.to_owned(),
            });
        }
        remotes.push(GitRemote {
            name: name.to_owned(),
            url: url.to_owned(),
        });
    }
    Ok(remotes)
}

/// Error returned when a remote-list row is malformed.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
#[error("line {line}: expected remote name and URL in {record:?}")]
pub struct GitRemoteParseError {
    /// One-based output line number.
    pub line: usize,
    /// Original malformed output line.
    pub record: String,
}

/// Error returned while loading or building remote commands.
#[derive(Debug, Error)]
pub enum JjGitRemoteError {
    /// The jj process could not be started or read.
    #[error("failed to run jj git remote command: {0}")]
    Io(#[from] std::io::Error),
    /// jj exited unsuccessfully.
    #[error("{command} failed: {stderr}")]
    CommandFailed {
        /// Display command title.
        command: String,
        /// Captured stderr.
        stderr: String,
    },
    /// jj returned malformed remote-list output.
    #[error("failed to parse jj git remote list output: {0}")]
    Parse(#[from] GitRemoteParseError),
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use std::path::{Path, PathBuf};
    use std::process::{Command, Output};
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::bookmarks::JjBookmarks;

    fn strings(args: &[std::ffi::OsString]) -> Vec<String> {
        args.iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    fn fixture_directory() -> PathBuf {
        static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);
        let fixture_id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "jk-refs-remotes-fixture-{}-{fixture_id}",
            std::process::id()
        ))
    }

    fn init_repository(repository: &Path, colocate: bool) {
        let mut command = Command::new("jj");
        command.args(["git", "init"]);
        if colocate {
            command.arg("--colocate");
        } else {
            command.arg("--no-colocate");
        }
        let status = command.arg(repository).status().expect("jj is installed");
        assert!(status.success(), "initialize fixture repository");
    }

    fn run_jj(repository: &Path, args: &[&str]) {
        let status = Command::new("jj")
            .arg("--repository")
            .arg(repository)
            .args(args)
            .status()
            .expect("jj is installed");
        assert!(status.success(), "jj command should succeed: {args:?}");
    }

    fn jj_output(repository: &Path, args: &[&str]) -> Output {
        let output = Command::new("jj")
            .arg("--repository")
            .arg(repository)
            .args(args)
            .output()
            .expect("jj is installed");
        assert!(
            output.status.success(),
            "jj command should succeed: {args:?}"
        );
        output
    }

    fn run_spec(spec: &JjCommandSpec) -> Output {
        let output = Command::new("jj")
            .args(spec.process_argv())
            .output()
            .expect("jj is installed");
        assert!(
            output.status.success(),
            "generated jj command should succeed"
        );
        output
    }

    #[test]
    fn remote_parser_keeps_url_remainder() {
        let remotes =
            parse_remote_list("origin /tmp/repo with spaces\nupstream https://example.test/repo\n")
                .expect("valid remotes");
        assert_eq!(remotes[0].name, "origin");
        assert_eq!(remotes[0].url, "/tmp/repo with spaces");
        assert_eq!(remotes[1].name, "upstream");
    }

    #[test]
    fn remote_parser_reports_malformed_line() {
        let error = parse_remote_list("origin\n").expect_err("missing URL should fail");
        assert_eq!(error.line, 1);
    }

    #[test]
    fn fetch_spec_requires_named_remote_and_confirmation() {
        let spec = JjGitRemote::default()
            .with_repository("/tmp/repo")
            .fetch_spec("origin");
        assert_eq!(
            strings(&spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "never",
                "--repository",
                "/tmp/repo",
                "git",
                "fetch",
                "--remote",
                "exact:\"origin\""
            ]
        );
        assert_eq!(spec.safety(), SafetyClass::NetworkRead);
        assert_eq!(spec.mode(), ExecutionMode::ConfirmNetworkRead);
    }

    #[test]
    fn push_dry_run_spec_has_one_remote_one_bookmark_and_dry_run() {
        let spec = JjGitRemote::default().push_dry_run_spec("origin", "feature");
        assert_eq!(
            strings(spec.argv()),
            vec![
                "git",
                "push",
                "--remote",
                "origin",
                "--bookmark",
                "exact:\"feature\"",
                "--dry-run"
            ]
        );
        assert_eq!(spec.safety(), SafetyClass::NetworkRead);
        assert_eq!(spec.mode(), ExecutionMode::ConfirmNetworkRead);
    }

    #[test]
    fn local_fixture_fetches_and_dry_run_push_preserves_remote_refs() {
        let root = fixture_directory();
        let remote = root.join("remote");
        let local = root.join("local");
        std::fs::create_dir_all(&root).expect("create fixture root");
        init_repository(&remote, true);
        run_jj(&remote, &["new", "-m", "remote commit"]);
        run_jj(
            &remote,
            &["bookmark", "create", "--revision", "@", "--", "fixture"],
        );

        init_repository(&local, false);
        run_jj(
            &local,
            &[
                "git",
                "remote",
                "add",
                "fixture",
                remote.to_str().expect("UTF-8 path"),
            ],
        );

        let remotes = JjGitRemote::default()
            .with_repository(&local)
            .load()
            .expect("fixture remote list");
        assert_eq!(
            remotes,
            [GitRemote {
                name: "fixture".to_owned(),
                url: remote
                    .canonicalize()
                    .expect("canonical fixture remote")
                    .to_string_lossy()
                    .into_owned(),
            }]
        );

        let git_remote = JjGitRemote::default().with_repository(&local);
        run_spec(&git_remote.fetch_spec("fixture"));
        let missing_remote = Command::new("jj")
            .args(git_remote.fetch_spec("missing").process_argv())
            .output()
            .expect("jj is installed");
        assert!(
            !missing_remote.status.success(),
            "missing remote should fail"
        );
        assert!(String::from_utf8_lossy(&missing_remote.stderr).contains("missing"));
        let fetched = JjBookmarks::default()
            .with_repository(&local)
            .load()
            .expect("load fetched remote bookmark");
        assert!(fetched.bookmarks.iter().any(|bookmark| {
            bookmark.name == "fixture" && bookmark.remote.as_deref() == Some("fixture")
        }));

        run_jj(&local, &["bookmark", "track", "fixture@fixture"]);
        run_jj(&local, &["new", "fixture", "-m", "local commit"]);
        run_jj(
            &local,
            &["bookmark", "move", "--to", "@", "--", "exact:\"fixture\""],
        );
        let before = jj_output(&remote, &["bookmark", "list", "--all-remotes"]);
        let wildcard_selection = run_spec(&git_remote.push_dry_run_spec("fixture", "fixture*"));
        assert!(String::from_utf8_lossy(&wildcard_selection.stderr).contains("Nothing changed"));
        let dry_run = run_spec(&git_remote.push_dry_run_spec("fixture", "fixture"));
        assert!(String::from_utf8_lossy(&dry_run.stderr).contains("Dry-run requested"));
        let after = jj_output(&remote, &["bookmark", "list", "--all-remotes"]);
        assert_eq!(
            before.stdout, after.stdout,
            "dry-run must not update remote refs"
        );

        std::fs::remove_dir_all(root).expect("remove fixture repositories");
    }
}
