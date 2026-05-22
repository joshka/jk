use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::actions::CommandOutput;
use crate::jj::exact_change_id_revset;
use crate::jj::{
    ColorMode, base_command, run_direct_args, run_direct_args_stdout, summarize_output,
};

use super::JjAbandonPreview;

/// Template used to read the first description line for abandon preflight text.
///
/// The abandon flow uses `jj log` to surface a title without inventing its own
/// parser or changing the selected change.
#[cfg(test)]
pub const DESCRIPTION_FIRST_LINE_TEMPLATE: &str = "description.first_line() ++ \"\\n\"";
#[cfg(not(test))]
const DESCRIPTION_FIRST_LINE_TEMPLATE: &str = "description.first_line() ++ \"\\n\"";

/// Preview-first plan for abandoning one exact revision.
///
/// Abandon safety owns the preflight probes and strong-confirm preview; keep
/// it separate from rewrite plans because it classifies destructive risk before
/// command execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPlan {
    /// Exact selected revision targeted by `jj abandon`.
    revision: String,
}

impl JjAbandonPlan {
    /// Build an abandon plan for one exact rendered revision.
    pub fn new(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
        }
    }

    /// Returns the exact selected revision targeted by this plan.
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Returns the user-facing `jj` command label for this abandon plan.
    pub fn command_label(&self) -> String {
        format!("jj abandon {}", self.revision)
    }

    /// Returns argv for `jj abandon`.
    pub fn command_argv(&self) -> Vec<String> {
        vec!["abandon".to_owned(), self.exact_change_id_revset()]
    }

    /// Returns the user-facing label for the diff-summary preflight probe.
    pub fn diff_summary_label(&self) -> String {
        format!("jj diff -r {} --summary", self.revision)
    }

    /// Returns argv for the diff-summary preflight probe.
    pub fn diff_summary_argv(&self) -> Vec<String> {
        vec![
            "diff".to_owned(),
            "-r".to_owned(),
            self.exact_change_id_revset(),
            "--summary".to_owned(),
        ]
    }

    /// Load jj preflight text for the confirmation screen without deciding app transitions.
    pub fn run_preview(&self) -> Result<JjAbandonPreview> {
        let summary = run_direct_args_stdout(self.diff_summary_argv(), &self.diff_summary_label())?;
        let title = self.load_title().ok().flatten();

        Ok(JjAbandonPreview::new(self.revision.clone(), title, summary))
    }

    /// Execute `jj abandon`; refresh/reveal and result-screen routing are external policy.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "abandoned")
    }

    /// Read the selected change title for abandon preflight.
    ///
    /// Failure surfaces as preview failure instead of guessing a title or
    /// changing the target revision.
    fn load_title(&self) -> Result<Option<String>> {
        let mut jj = base_command(ColorMode::Never);
        jj.args(self.title_argv());

        let output = jj.output()?;
        if !output.status.success() {
            return Err(eyre!(
                "{} title failed: {}",
                self.revision,
                summarize_output(&output.stdout, &output.stderr, "could not read title")
            ));
        }

        let title = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok((!title.is_empty()).then_some(title))
    }

    /// Returns argv for the first-line title preflight probe.
    #[cfg(test)]
    pub fn title_argv(&self) -> Vec<String> {
        vec![
            "log".to_owned(),
            "-r".to_owned(),
            self.exact_change_id_revset(),
            "--no-graph".to_owned(),
            "-T".to_owned(),
            DESCRIPTION_FIRST_LINE_TEMPLATE.to_owned(),
        ]
    }

    /// Returns argv for the first-line title preflight probe.
    #[cfg(not(test))]
    fn title_argv(&self) -> Vec<String> {
        vec![
            "log".to_owned(),
            "-r".to_owned(),
            self.exact_change_id_revset(),
            "--no-graph".to_owned(),
            "-T".to_owned(),
            DESCRIPTION_FIRST_LINE_TEMPLATE.to_owned(),
        ]
    }

    /// Returns the quoted exact-change revset for the selected revision.
    fn exact_change_id_revset(&self) -> String {
        exact_change_id_revset(&self.revision)
    }
}
