//! Shared execution adapter for typed `jj` command specs.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use jk_core::JjCommandSpec;

/// Runs a typed `jj` command spec with the color policy required by the caller.
pub(crate) fn run_jj_spec(spec: &JjCommandSpec, color: &str) -> std::io::Result<Output> {
    let mut command = build_jj_command(spec, color);
    if spec.stdin().is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command.spawn()?;
    if let Some(stdin) = spec.stdin() {
        let child_stdin = child.stdin.as_mut().expect("stdin was configured as piped");
        child_stdin.write_all(stdin.as_bytes())?;
    }

    child.wait_with_output()
}

/// Builds the process command for a typed `jj` command spec.
pub(crate) fn build_jj_command(spec: &JjCommandSpec, color: &str) -> Command {
    let mut command = Command::new("jj");
    command.args(["--no-pager", "--color", color]);
    command.env_remove("NO_COLOR");
    command.env_remove("CLICOLOR");
    command.env_remove("CLICOLOR_FORCE");

    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }

    if let Some(repository) = spec.repository() {
        command.arg("--repository").arg(repository);
    }

    command.args(spec.argv());
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_adapter_forces_color_and_cleans_color_env() {
        let command = build_jj_command(&JjCommandSpec::render_read_only(["log"]), "always");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let envs = command
            .get_envs()
            .map(|(key, value)| (key.to_string_lossy().into_owned(), value.is_none()))
            .collect::<Vec<_>>();

        assert!(args.windows(2).any(|args| args == ["--color", "always"]));
        assert!(args.iter().any(|arg| arg == "log"));
        assert!(envs.contains(&("NO_COLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR".to_owned(), true)));
        assert!(envs.contains(&("CLICOLOR_FORCE".to_owned(), true)));
    }

    #[test]
    fn command_adapter_includes_repository_before_spec_argv() {
        let spec =
            JjCommandSpec::render_read_only(["diff", "-r", "@"]).with_repository("/tmp/repository");
        let command = build_jj_command(&spec, "always");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|args| args == ["--repository", "/tmp/repository"])
        );
        assert!(args.windows(3).any(|args| args == ["diff", "-r", "@"]));
    }
}
