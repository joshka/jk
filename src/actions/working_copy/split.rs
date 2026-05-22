use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::actions::CommandOutput;
use crate::jj::{exact_change_id_revset, interactive_jj_command};
use crate::terminal_process::{InteractiveCommand, run_with_ratatui_terminal};

/// Split target policy for `jj split`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjSplitTarget {
    /// Split one exact selected revision from rendered metadata.
    ExactChange(String),
    /// Split the current working-copy change (`@`).
    CurrentWorkingCopy,
}

/// Interactive split plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjSplitPlan {
    /// Target revision policy for the split flow.
    target: JjSplitTarget,
}

impl JjSplitTarget {
    /// Builds a split target for one exact selected revision.
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    /// Builds a split target for the current working-copy change.
    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    #[cfg(test)]
    pub fn exact_change_id(&self) -> Option<&str> {
        match self {
            Self::ExactChange(change_id) => Some(change_id),
            Self::CurrentWorkingCopy => None,
        }
    }

    /// Returns argv fragments for the selected split target.
    fn command_args(&self) -> Vec<String> {
        match self {
            Self::ExactChange(change_id) => vec![
                "split".to_owned(),
                "--revision".to_owned(),
                exact_change_id_revset(change_id),
            ],
            Self::CurrentWorkingCopy => vec!["split".to_owned()],
        }
    }

    /// Returns user-facing preview wording for the split target.
    fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => {
                format!("exact selected log revision {change_id}")
            }
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }

    /// Returns status wording for app-owned split result messages.
    fn status_context(&self) -> String {
        match self {
            Self::ExactChange(change_id) => {
                format!("split exact log revision {change_id}")
            }
            Self::CurrentWorkingCopy => "split current working-copy change (@)".to_owned(),
        }
    }
}

impl JjSplitPlan {
    /// Builds a split plan for one exact selected revision.
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self {
            target: JjSplitTarget::exact_change(change_id),
        }
    }

    /// Builds a split plan for the current working-copy change.
    pub fn current_working_copy() -> Self {
        Self {
            target: JjSplitTarget::current_working_copy(),
        }
    }

    /// Returns the split target policy for this plan.
    pub fn target(&self) -> &JjSplitTarget {
        &self.target
    }

    /// Returns the user-facing `jj` command label for this split plan.
    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj split`.
    pub fn command_argv(&self) -> Vec<String> {
        self.target.command_args()
    }

    /// Returns status wording for app-owned split result messages.
    pub fn status_context(&self) -> String {
        self.target.status_context()
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs the interactive split flow while suspending the Ratatui terminal.
    pub fn run_interactive(&self, terminal: &mut DefaultTerminal) -> Result<CommandOutput> {
        let command = self.interactive_command();
        let result = run_with_ratatui_terminal(terminal, &command)?;
        Ok(CommandOutput::new(
            self.success_result_message(result.status().description()),
        ))
    }

    /// Builds the inherited-stdio interactive command used by the terminal runner.
    pub fn interactive_command(&self) -> InteractiveCommand {
        interactive_jj_command(self.command_argv(), &self.command_label())
    }

    /// Returns the app-owned success result text shown after interactive split completes.
    pub fn success_result_message(&self, status: &str) -> String {
        [
            "result: split command completed successfully".to_owned(),
            format!("child exit status: {status}"),
            "output visibility: jj's diff editor and child output were shown live while jk's terminal was suspended; jk did not capture that output".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }

    /// Returns the app-owned failure result text shown after interactive split fails.
    pub fn failure_result_message(&self, error: &str) -> String {
        [
            "result: split command failed or did not complete".to_owned(),
            format!("runner status: {error}"),
            "output visibility: jj's diff editor and child output were live terminal output while jk's terminal was suspended; jk did not capture stderr for this result".to_owned(),
            "recovery: if jj recorded an operation, use jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }

    /// Returns the preview summary shown before confirming `jj split`.
    pub fn preview_summary(&self) -> String {
        [
            format!("command: {}", self.command_label()),
            String::new(),
            format!("target: {}", self.target.preview_target()),
            "handoff: jj split opens jj's diff editor to choose patch content".to_owned(),
            "description: jj may also invoke description editing after patch selection".to_owned(),
            "honesty: jk is not an in-app patch editor and does not choose hunks or lines".to_owned(),
            "fileset: no fileset is passed; patch selection is delegated to jj".to_owned(),
            "confirmation: press Enter to suspend jk and run jj split".to_owned(),
            "cancel: press Esc to return without running jj split".to_owned(),
            "after run: jk will refresh and show an app-owned result because inherited child output may disappear when the alternate screen is restored".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }
}
