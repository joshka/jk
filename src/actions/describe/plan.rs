use color_eyre::Result;

use super::JjDescribeTarget;
use crate::actions::CommandOutput;
use crate::jj::run_direct_args;

/// Preview-first plan for updating a change description without opening an editor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjDescribePlan {
    /// Target revision policy for the describe action.
    target: JjDescribeTarget,
    /// Non-interactive description text passed to `jj describe --message`.
    message: String,
}

impl JjDescribePlan {
    /// Build a non-interactive describe plan; prompt collection stays with the app lifecycle.
    pub fn new(target: JjDescribeTarget, message: impl Into<String>) -> Self {
        Self {
            target,
            message: message.into(),
        }
    }

    /// Returns the target policy owned by this describe plan.
    pub fn target(&self) -> &JjDescribeTarget {
        &self.target
    }

    /// Returns the user-facing `jj` command label for this describe plan.
    pub fn command_label(&self) -> String {
        format!(
            "jj describe {} --message {}",
            self.target.label(),
            self.message
        )
    }

    /// Returns argv for `jj describe --message`.
    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "describe".to_owned(),
            self.target.command_arg(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Execute the described argv directly; confirmation and refresh/reveal happen outside.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "described")
    }

    /// Summarize the effect without predicting jj's post-command graph state.
    pub fn preview_summary(&self) -> String {
        format!(
            "command: {}\n\ntarget: {}\nmessage: {}\n\neffect: updates only the target change description without opening an editor\nconfirmation: press Enter to run jj describe\nundo path: jj undo",
            self.command_label(),
            self.target.preview_target(),
            self.message,
        )
    }
}
