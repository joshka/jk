//! Shared execution adapter for typed `jj` command specs.

use std::io::Write;
use std::process::{Command, Output, Stdio};

use jk_core::JjCommandSpec;

/// Runs a typed `jj` command spec.
pub(crate) fn run_jj_spec(spec: &JjCommandSpec) -> std::io::Result<Output> {
    let mut command = build_jj_command(spec);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
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
pub(crate) fn build_jj_command(spec: &JjCommandSpec) -> Command {
    let mut command = Command::new("jj");
    command.args(spec.global_argv());
    command.env_remove("NO_COLOR");
    command.env_remove("CLICOLOR");
    command.env_remove("CLICOLOR_FORCE");

    if let Some(cwd) = spec.cwd() {
        command.current_dir(cwd);
    }

    command.args(spec.argv());
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_adapter_forces_color_and_cleans_color_env() {
        let command = build_jj_command(&JjCommandSpec::render_read_only(["log"]));
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
                "always",
                "--repository",
                "/tmp/repository",
                "diff",
                "-r",
                "@"
            ]
        );
        assert_eq!(
            args.iter()
                .filter(|arg| arg.as_str() == "--repository")
                .count(),
            1
        );
    }

    #[test]
    fn command_adapter_uses_spec_rendered_process_argv() {
        let spec = JjCommandSpec::render_read_only(["status"]).with_repository("/tmp/repository");
        let command = build_jj_command(&spec);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        let spec_args = spec
            .process_argv()
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(args, spec_args);
    }

    #[test]
    fn command_adapter_captures_stdout() {
        let spec = JjCommandSpec::render_read_only(["--version"]);
        let output = run_jj_spec(&spec).expect("jj --version should run");

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("jj "));
        assert!(output.stderr.is_empty());
    }
}
