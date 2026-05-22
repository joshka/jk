use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::run_direct_args;

/// Preview-first plan for committing the current working-copy change.
///
/// This plan never consumes log selection; `jj commit` always acts on `@` and
/// creates the next working-copy change according to jj's normal behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjCommitPlan {
    /// Non-interactive description text passed to `jj commit --message`.
    message: String,
}

impl JjCommitPlan {
    /// Build a commit plan for `@`; selected log rows are intentionally not accepted.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the user-facing `jj` command label for this commit plan.
    pub fn command_label(&self) -> String {
        format!("jj commit --message {}", self.message)
    }

    /// Returns argv for `jj commit --message`.
    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "commit".to_owned(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Execute `jj commit`; follow-up navigation remains an app lifecycle concern.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            "committed current working-copy change",
        )
    }

    /// Summarize jj's working-copy commit effect without inspecting future state.
    pub fn preview_summary(&self) -> String {
        format!(
            "command: {}\n\ntarget: current working-copy change (@)\nmessage: {}\n\neffect: updates @ with the message and creates a new working-copy change on top\nselection: selected log rows are not arguments to jj commit\nconfirmation: press Enter to run jj commit\nundo path: jj undo",
            self.command_label(),
            self.message,
        )
    }
}
