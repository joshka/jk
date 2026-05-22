use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{exact_change_id_revset, run_direct_args, run_direct_args_stdout};

/// Preview-first plan for reverse-applying one exact revision into `@`.
///
/// Previews show the selected revision's forward diff because that is the source jj will
/// reverse-apply.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRevertPlan {
    /// Exact selected revision reverse-applied into `@`.
    revision: String,
}

impl JjRevertPlan {
    /// Build a revert plan for one exact rendered revision into `@`.
    pub fn new(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
        }
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "revert".to_owned(),
            "-r".to_owned(),
            exact_change_id_revset(&self.revision),
            "-o".to_owned(),
            "@".to_owned(),
        ]
    }

    pub fn preview_diff_label(&self) -> String {
        let label_args = self.preview_diff_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn preview_diff_argv(&self) -> Vec<String> {
        vec![
            "diff".to_owned(),
            "-r".to_owned(),
            exact_change_id_revset(&self.revision),
        ]
    }

    /// Load jj's forward diff and wrap it with the revert preview contract.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    /// Execute `jj revert`; refresh and result-screen transitions happen outside the plan.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "reverted")
    }

    /// Summarize revert using jj's forward diff rather than a simulated result.
    pub fn preview_summary(&self, forward_diff: &str) -> String {
        [
            format!("target revision: {}", self.revision),
            format!("command: {}", self.command_label()),
            "effect: revert reverse-applies the selected revision's forward diff into @".to_owned(),
            format!("preview source: {}", self.preview_diff_label()),
            "honesty: the output below is the forward diff that jj revert reverse-applies into @; jk is not simulating the final graph".to_owned(),
            "confirmation: press Enter to run jj revert".to_owned(),
            "undo path: jj undo".to_owned(),
            String::new(),
            "forward diff:".to_owned(),
            forward_diff.trim_end().to_owned(),
        ]
        .join("\n")
    }
}
