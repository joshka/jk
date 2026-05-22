use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{
    exact_change_id_revset, root_file_fileset, run_direct_args, run_direct_args_stdout,
};

/// Preview-first plan for `jj restore`.
///
/// Previews show the forward diff that jj will remove. `jk` does not simulate the resulting graph
/// or file content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRestorePlan {
    /// Restore target revision policy for this plan.
    target: JjRestoreTarget,
    /// Optional exact repository-root path restricted by this restore plan.
    path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum JjRestoreTarget {
    /// Restore content from one exact selected revision.
    ExactChange(String),
    /// Restore content from the current working-copy change.
    CurrentWorkingCopy,
}

impl JjRestorePlan {
    /// Restore the entire forward diff from one exact rendered revision.
    pub fn for_revision(revision: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: None,
        }
    }

    /// Restore one repository-root path from one exact rendered revision.
    pub fn for_path(revision: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: Some(path.into()),
        }
    }

    /// Restore one repository-root path from the current working-copy diff.
    pub fn for_working_copy_path(path: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::CurrentWorkingCopy,
            path: Some(path.into()),
        }
    }

    /// Returns the revision label targeted by this restore plan.
    pub fn revision(&self) -> &str {
        self.target.label()
    }

    /// Returns the exact repository-root path restriction, if present.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Returns the user-facing `jj` command label for this restore plan.
    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj restore`.
    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = match &self.target {
            JjRestoreTarget::ExactChange(revision) => vec![
                "restore".to_owned(),
                "--changes-in".to_owned(),
                exact_change_id_revset(revision),
            ],
            JjRestoreTarget::CurrentWorkingCopy => vec!["restore".to_owned()],
        };
        if let Some(path) = &self.path {
            argv.push(root_file_fileset(path));
        }
        argv
    }

    /// Returns the user-facing label for the forward-diff preview probe.
    pub fn preview_diff_label(&self) -> String {
        let label_args = self.preview_diff_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for the forward-diff preview probe.
    pub fn preview_diff_argv(&self) -> Vec<String> {
        let mut argv = match &self.target {
            JjRestoreTarget::ExactChange(revision) => vec![
                "diff".to_owned(),
                "-r".to_owned(),
                exact_change_id_revset(revision),
            ],
            JjRestoreTarget::CurrentWorkingCopy => vec!["diff".to_owned()],
        };
        if let Some(path) = &self.path {
            argv.push(root_file_fileset(path));
        }
        argv
    }

    /// Load jj's forward diff and wrap it with the restore preview contract.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    /// Execute `jj restore`; confirmation and result transitions happen outside the plan.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "restored")
    }

    /// Summarize the restore using jj's forward diff rather than a simulated result.
    pub fn preview_summary(&self, forward_diff: &str) -> String {
        let mut lines = vec![
            format!("target revision: {}", self.revision()),
            format!("command: {}", self.command_label()),
        ];

        match &self.path {
            Some(path) => {
                lines.push(format!("selected path: {path}"));
                lines.push(format!("exact fileset: {}", root_file_fileset(path)));
                lines.push(self.target.path_restore_effect());
            }
            None => lines.push(
                "effect: restore removes the selected revision's forward diff from that exact revision"
                    .to_owned(),
            ),
        }

        lines.extend([
            format!("preview source: {}", self.preview_diff_label()),
            "honesty: the output below is the forward diff that jj restore removes; jk is not simulating the final graph".to_owned(),
            "confirmation: press Enter to run jj restore".to_owned(),
            "undo path: jj undo".to_owned(),
            String::new(),
            "forward diff:".to_owned(),
            forward_diff.trim_end().to_owned(),
        ]);

        lines.join("\n")
    }
}

impl JjRestoreTarget {
    /// Returns the user-facing revision label for this restore target.
    fn label(&self) -> &str {
        match self {
            Self::ExactChange(revision) => revision,
            Self::CurrentWorkingCopy => "@",
        }
    }

    /// Returns user-facing effect wording for path-scoped restore previews.
    fn path_restore_effect(&self) -> String {
        match self {
            Self::ExactChange(_) => {
                "effect: restore removes the selected path's forward diff from that exact revision"
                    .to_owned()
            }
            Self::CurrentWorkingCopy => {
                "effect: restore removes the selected path's working-copy diff from @".to_owned()
            }
        }
    }
}
