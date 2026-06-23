//! Local operation recovery command specs.

use jk_core::{GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass};

const UNDO_COMMAND: &str = "undo";
const REDO_COMMAND: &str = "redo";

/// Local recovery command supported by the first undo/redo preview surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryCommand {
    /// Preview and run `jj undo`.
    Undo,
    /// Preview and run `jj redo`.
    Redo,
}

impl RecoveryCommand {
    fn argv(self) -> [&'static str; 1] {
        match self {
            Self::Undo => [UNDO_COMMAND],
            Self::Redo => [REDO_COMMAND],
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Undo => "jj undo",
            Self::Redo => "jj redo",
        }
    }
}

/// Builds previewable local recovery commands.
#[derive(Clone, Debug, Default)]
pub struct JjRecovery {
    global_options: GlobalOptions,
}

impl JjRecovery {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Returns the command spec for `command`.
    #[must_use]
    pub fn spec_for(&self, command: RecoveryCommand) -> JjCommandSpec {
        JjCommandSpec::confirm_mutation(command.argv(), SafetyClass::LocalRewrite)
            .with_global_options(self.global_options.clone())
            .with_title(command.title())
            .with_refresh_plan(RefreshPlan::None)
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
    fn undo_builds_confirmed_local_rewrite_spec() {
        let spec = JjRecovery::default().spec_for(RecoveryCommand::Undo);

        assert_eq!(strings(spec.argv()), vec!["undo"]);
        assert_eq!(spec.title(), "jj undo");
        assert_eq!(spec.safety(), SafetyClass::LocalRewrite);
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
    }

    #[test]
    fn redo_builds_confirmed_local_rewrite_spec() {
        let spec = JjRecovery::default().spec_for(RecoveryCommand::Redo);

        assert_eq!(strings(spec.argv()), vec!["redo"]);
        assert_eq!(spec.title(), "jj redo");
        assert_eq!(spec.safety(), SafetyClass::LocalRewrite);
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
    }

    #[test]
    fn repository_renders_before_recovery_command() {
        let spec = JjRecovery::default()
            .with_repository("/tmp/repo")
            .spec_for(RecoveryCommand::Undo);
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
                "undo",
            ]
        );
    }
}
