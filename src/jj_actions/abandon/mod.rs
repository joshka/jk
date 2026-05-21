//! Abandon action plan and preflight preview.

use crate::jj::{
    ColorMode, base_command, run_direct_args, run_direct_args_stdout, summarize_output,
};
use crate::jj_actions::CommandOutput;
use crate::jj_syntax::exact_change_id_revset;
use color_eyre::Result;
use color_eyre::eyre::eyre;

/// Template used to read the first description line for abandon preflight text.
///
/// The abandon flow uses `jj log` to surface a title without inventing its own
/// parser or changing the selected change.
const DESCRIPTION_FIRST_LINE_TEMPLATE: &str = "description.first_line() ++ \"\\n\"";

/// Preview-first plan for abandoning one exact revision.
///
/// Abandon safety owns the preflight probes and strong-confirm preview; keep it separate from
/// rewrite plans because it classifies destructive risk before command execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPlan {
    revision: String,
}

impl JjAbandonPlan {
    /// Build an abandon plan for one exact rendered revision.
    pub fn new(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
        }
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn command_label(&self) -> String {
        format!("jj abandon {}", self.revision)
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec!["abandon".to_owned(), self.exact_change_id_revset()]
    }

    pub fn diff_summary_label(&self) -> String {
        format!("jj diff -r {} --summary", self.revision)
    }

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

    fn exact_change_id_revset(&self) -> String {
        exact_change_id_revset(&self.revision)
    }
}

/// Preflight result for an abandon confirmation screen.
///
/// The preview keeps jj's diff summary text and only classifies empty versus non-empty changes for
/// confirmation strength. It does not decide refresh or reveal behavior after abandon completes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPreview {
    revision: String,
    title: Option<String>,
    summary: String,
    change_state: AbandonChangeState,
}

impl JjAbandonPreview {
    /// Classify preflight output only by whether jj reported a non-empty diff summary.
    pub fn new(revision: String, title: Option<String>, summary: String) -> Self {
        let change_state = if summary.trim().is_empty() {
            AbandonChangeState::Empty
        } else {
            AbandonChangeState::NonEmpty
        };

        Self {
            revision,
            title,
            summary,
            change_state,
        }
    }

    #[cfg(test)]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Return whether abandon can use the weaker empty-change confirmation flow.
    pub fn is_empty_change(&self) -> bool {
        self.change_state == AbandonChangeState::Empty
    }

    /// Build the confirmation text from jj preflight output without simulating abandon results.
    pub fn preview_text(&self) -> String {
        let title = self.title.as_deref().unwrap_or("<no description>");
        let summary = if self.summary.trim().is_empty() {
            "empty change".to_owned()
        } else {
            self.summary.trim().to_owned()
        };
        let confirmation = if self.is_empty_change() {
            "press Enter to abandon this empty change".to_owned()
        } else {
            format!(
                "type exact revision '{}' before abandon runs",
                self.revision
            )
        };

        format!(
            "change: {}\ntitle: {}\ndiff summary:\n{}\n\neffect: abandon removes the selected change from the visible history; recovery stays available through jj undo\nconfirmation: {}\nundo path: jj undo",
            self.revision, title, summary, confirmation
        )
    }
}

/// Preflight only cares whether the diff summary is empty or not.
///
/// More detailed abandon policy belongs in the preview builder and app
/// confirmation flow, not in this local classifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AbandonChangeState {
    Empty,
    NonEmpty,
}

#[cfg(test)]
mod tests;
