use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::run_direct_args;

/// Rewrite plans share explicit source/destination roles and avoid parsing or
/// predicting jj's final graph shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjSquashPlan {
    /// Source revisions selected to squash into the destination.
    sources: Vec<String>,
    /// Destination revision that receives the squashed content.
    destination: String,
}

impl JjSquashPlan {
    /// Builds a squash plan from explicit source and destination roles.
    pub fn new(sources: Vec<String>, destination: impl Into<String>) -> Self {
        Self {
            sources,
            destination: destination.into(),
        }
        .normalize()
    }

    /// Returns the source revisions owned by this squash plan.
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Returns the destination revision for this squash plan.
    pub fn destination(&self) -> &str {
        &self.destination
    }

    /// Returns the user-facing `jj` command label for this squash plan.
    pub fn command_label(&self, _dry_run: bool) -> String {
        let label_args = self
            .command_argv(false)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj squash`.
    pub fn command_argv(&self, _dry_run: bool) -> Vec<String> {
        let mut argv = vec!["squash".to_owned()];
        for source in &self.sources {
            argv.push("--from".to_owned());
            argv.push(source.clone());
        }
        argv.push("--into".to_owned());
        argv.push(self.destination.clone());
        argv.push("--use-destination-message".to_owned());
        argv
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs `jj squash` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "squashed",
        )
    }

    /// Returns the preview summary shown before confirming `jj squash`.
    pub fn preview_summary(&self) -> String {
        let sources = self
            .sources
            .iter()
            .map(|source| format!("source: {source}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "command: {}\n\n{}\n\ndestination: {}\n\ngraph effect: moves the selected source changes into the destination; jj may abandon emptied sources and rebase descendants\n\ndescription behavior: --use-destination-message keeps the destination description, discards source descriptions, and avoids an editor or prompt\n\nconfirmation: press Enter to run jj squash\nundo path: jj undo",
            self.command_label(false),
            sources,
            self.destination,
        )
    }

    /// Drops blank source inputs before argv construction.
    fn normalize(mut self) -> Self {
        self.sources.retain(|source| !source.trim().is_empty());
        self
    }
}
