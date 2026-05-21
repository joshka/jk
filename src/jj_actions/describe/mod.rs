//! Describe and commit action plans.

use crate::jj::run_direct_args;
use crate::jj_actions::CommandOutput;
use crate::jj_syntax::exact_change_id_revset;
use color_eyre::Result;

/// Target for non-interactive `jj describe --message` finalization.
///
/// Exact changes come from rendered row metadata and are quoted before argv construction; the
/// working-copy target stays as jj's `@` revset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjDescribeTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

/// Preview-first plan for updating a change description without opening an editor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjDescribePlan {
    target: JjDescribeTarget,
    message: String,
}

/// Preview-first plan for committing the current working-copy change.
///
/// This plan never consumes graph selection; `jj commit` always acts on `@` and creates the next
/// working-copy change according to jj's normal behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjCommitPlan {
    message: String,
}

impl JjDescribeTarget {
    /// Target an exact rendered change id, quoted during argv construction.
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    /// Target jj's current working-copy revset (`@`) without exact-change quoting.
    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    /// Expose the exact change id only when the target came from rendered row metadata.
    pub fn exact_change_id(&self) -> Option<&str> {
        match self {
            Self::ExactChange(change_id) => Some(change_id),
            Self::CurrentWorkingCopy => None,
        }
    }

    fn command_arg(&self) -> String {
        match self {
            Self::ExactChange(change_id) => exact_change_id_revset(change_id),
            Self::CurrentWorkingCopy => "@".to_owned(),
        }
    }

    fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => format!("exact selected revision {change_id}"),
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }
}

impl JjDescribePlan {
    /// Build a non-interactive describe plan; prompt collection stays with the app lifecycle.
    pub fn new(target: JjDescribeTarget, message: impl Into<String>) -> Self {
        Self {
            target,
            message: message.into(),
        }
    }

    pub fn target(&self) -> &JjDescribeTarget {
        &self.target
    }

    pub fn command_label(&self) -> String {
        format!(
            "jj describe {} --message {}",
            self.target.label(),
            self.message
        )
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "describe".to_owned(),
            self.target.command_arg(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

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

impl JjCommitPlan {
    /// Build a commit plan for `@`; selected graph rows are intentionally not accepted.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn command_label(&self) -> String {
        format!("jj commit --message {}", self.message)
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "commit".to_owned(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

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
            "command: {}\n\ntarget: current working-copy change (@)\nmessage: {}\n\neffect: updates @ with the message and creates a new working-copy change on top\nselection: selected graph rows are not arguments to jj commit\nconfirmation: press Enter to run jj commit\nundo path: jj undo",
            self.command_label(),
            self.message,
        )
    }
}

#[cfg(test)]
mod tests;
