use std::panic::{AssertUnwindSafe, catch_unwind};

use super::*;

#[test]
fn command_records_inherited_stdio_intent() {
    let mut command = InteractiveCommand::new("jj", "jj split");
    command.args(["--no-pager", "split"]);

    assert_eq!(command.program(), OsStr::new("jj"));
    assert_eq!(
        command.argv(),
        vec![OsStr::new("--no-pager"), OsStr::new("split")]
    );
    assert_eq!(command.stdio_intent(), StdioIntent::Inherit);
}

#[test]
fn command_records_current_dir_intent() {
    let mut command = InteractiveCommand::new("jj", "jj split");
    command.current_dir("/tmp/jk-runner-proof");

    assert_eq!(
        command.current_dir_path(),
        Some(Path::new("/tmp/jk-runner-proof"))
    );
}

#[test]
fn runner_restores_terminal_after_spawn_error() {
    let mut lifecycle = FakeLifecycle::default();
    let mut spawner = FakeSpawner::error("spawn error");
    let command = InteractiveCommand::new("jj", "jj split");

    let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
        .expect_err("spawn errors should be reported");

    assert_eq!(lifecycle.events, ["suspend", "restore"]);
    assert_eq!(spawner.labels, ["jj split"]);
    assert!(
        error
            .to_string()
            .contains("jj split failed while running inherited-stdio command: spawn error")
    );
}

#[test]
fn runner_reports_nonzero_status_after_restoring_terminal() {
    let mut lifecycle = FakeLifecycle::default();
    let mut spawner = FakeSpawner::status(InteractiveExitStatus::failure("exit status: 1"));
    let command = InteractiveCommand::new("jj", "jj split");

    let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
        .expect_err("nonzero exits should be reported");

    assert_eq!(lifecycle.events, ["suspend", "restore"]);
    assert_eq!(
        error.to_string(),
        "jj split failed with status exit status: 1"
    );
}

#[test]
fn runner_reports_restore_failure_after_success() {
    let mut lifecycle = FakeLifecycle {
        restore_error: Some("restore error"),
        ..FakeLifecycle::default()
    };
    let mut spawner = FakeSpawner::status(InteractiveExitStatus::success("exit status: 0"));
    let command = InteractiveCommand::new("jj", "jj split");

    let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
        .expect_err("restore failures should be reported");

    assert_eq!(lifecycle.events, ["suspend", "restore"]);
    assert_eq!(
        error.to_string(),
        "jj split completed with status exit status: 0 but failed to restore terminal: \
         restore error"
    );
}

#[test]
fn runner_attempts_restore_after_suspend_failure() {
    let mut lifecycle = FakeLifecycle {
        suspend_error: Some("suspend error"),
        ..FakeLifecycle::default()
    };
    let mut spawner = FakeSpawner::status(InteractiveExitStatus::success("exit status: 0"));
    let command = InteractiveCommand::new("jj", "jj split");

    let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
        .expect_err("suspend failures should be reported");

    assert_eq!(lifecycle.events, ["suspend", "restore"]);
    assert!(spawner.labels.is_empty());
    assert_eq!(
        error.to_string(),
        "jj split was not started because terminal suspension failed: suspend error"
    );
}

#[test]
fn restore_guard_restores_if_spawner_panics() {
    let mut lifecycle = FakeLifecycle::default();
    let mut spawner = FakeSpawner::panic();
    let command = InteractiveCommand::new("jj", "jj split");

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = run_interactive_command(&mut lifecycle, &mut spawner, &command);
    }));

    assert!(panic.is_err());
    assert_eq!(lifecycle.events, ["suspend", "restore"]);
}

#[test]
#[ignore = "manual proof uses JK_INTERACTIVE_PROOF_REPO to force cwd into a /tmp jj repo"]
fn real_runner_reports_jj_failure_from_tmp_repo() {
    let repo = canonical_tmp_proof_repo();

    let mut lifecycle = FakeLifecycle::default();
    let mut spawner = ProcessSpawner;
    let mut command = InteractiveCommand::new("jj", "jj split --tool false");
    command
        .current_dir(repo)
        .args(["--no-pager", "split", "--tool", "false"]);

    let error = run_interactive_command(&mut lifecycle, &mut spawner, &command)
        .expect_err("jj split --tool false should report a clean runner failure");

    assert_eq!(lifecycle.events, ["suspend", "restore"]);
    assert!(error.to_string().contains("jj split --tool false failed"));
}

#[test]
#[ignore = "manual proof requires a live terminal and JK_INTERACTIVE_PROOF_REPO under /tmp"]
fn real_ratatui_runner_reports_jj_failure_from_tmp_repo() {
    let repo = canonical_tmp_proof_repo();

    let mut terminal = ratatui::try_init().expect("manual proof requires a live terminal");
    let mut command = InteractiveCommand::new("jj", "jj split --tool false");
    command
        .current_dir(repo)
        .args(["--no-pager", "split", "--tool", "false"]);

    let result = run_with_ratatui_terminal(&mut terminal, &command);
    let cleanup_result = ratatui::try_restore();
    let error = result.expect_err("jj split --tool false should fail cleanly");
    let error = error.to_string();

    assert!(
        error.contains("jj split --tool false failed with status"),
        "expected a clean child nonzero status, got {error}"
    );
    assert!(
        !error.contains("failed to restore terminal"),
        "expected child failure without terminal restore failure, got {error}"
    );
    assert!(
        !error.contains("additionally failed to restore terminal"),
        "expected child failure without additional restore failure, got {error}"
    );

    cleanup_result.expect("manual proof should restore the terminal for the shell");
}

fn canonical_tmp_proof_repo() -> PathBuf {
    let repo = std::env::var_os("JK_INTERACTIVE_PROOF_REPO")
        .expect("set JK_INTERACTIVE_PROOF_REPO to a disposable /tmp jj repo");
    let canonical_repo = PathBuf::from(repo)
        .canonicalize()
        .expect("proof repo must exist before running the manual proof");
    let canonical_tmp = Path::new("/tmp")
        .canonicalize()
        .expect("the system /tmp directory should exist");

    assert!(
        canonical_repo.starts_with(&canonical_tmp),
        "proof repo must resolve inside {}, got {}",
        canonical_tmp.display(),
        canonical_repo.display()
    );

    canonical_repo
}

#[derive(Default)]
struct FakeLifecycle {
    events: Vec<&'static str>,
    suspend_error: Option<&'static str>,
    restore_error: Option<&'static str>,
}

impl TerminalLifecycle for FakeLifecycle {
    fn suspend(&mut self) -> Result<()> {
        self.events.push("suspend");
        match self.suspend_error {
            Some(error) => Err(eyre!(error)),
            None => Ok(()),
        }
    }

    fn restore(&mut self) -> Result<()> {
        self.events.push("restore");
        match self.restore_error {
            Some(error) => Err(eyre!(error)),
            None => Ok(()),
        }
    }
}

enum FakeSpawnResult {
    Status(InteractiveExitStatus),
    Error(&'static str),
    Panic,
}

struct FakeSpawner {
    result: FakeSpawnResult,
    labels: Vec<String>,
}

impl FakeSpawner {
    fn status(status: InteractiveExitStatus) -> Self {
        Self {
            result: FakeSpawnResult::Status(status),
            labels: Vec::new(),
        }
    }

    fn error(error: &'static str) -> Self {
        Self {
            result: FakeSpawnResult::Error(error),
            labels: Vec::new(),
        }
    }

    fn panic() -> Self {
        Self {
            result: FakeSpawnResult::Panic,
            labels: Vec::new(),
        }
    }
}

impl InteractiveCommandSpawner for FakeSpawner {
    fn spawn_and_wait(&mut self, command: &InteractiveCommand) -> Result<InteractiveExitStatus> {
        self.labels.push(command.label().to_owned());
        match &self.result {
            FakeSpawnResult::Status(status) => Ok(status.clone()),
            FakeSpawnResult::Error(error) => Err(eyre!(*error)),
            FakeSpawnResult::Panic => panic!("spawner panic"),
        }
    }
}
