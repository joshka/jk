use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{exact_change_id_revset, run_direct_args};

/// Rewrite plans share explicit source/destination roles and avoid parsing or
/// predicting jj's final graph shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbsorbPlan {
    /// Source revision whose changes may be absorbed.
    source: String,
    /// Candidate destination revisions selected by the caller.
    destinations: Vec<String>,
}

impl JjAbsorbPlan {
    /// Builds an absorb plan from one source revision and candidate destinations.
    pub fn new(source: impl Into<String>, destinations: Vec<String>) -> Self {
        Self {
            source: source.into(),
            destinations,
        }
        .normalize()
    }

    /// Returns the source revision for this absorb plan.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the candidate destination revisions for this absorb plan.
    pub fn destinations(&self) -> &[String] {
        &self.destinations
    }

    /// Returns the user-facing `jj` command label for this absorb plan.
    pub fn command_label(&self) -> String {
        let label_args = self
            .command_argv()
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj absorb`.
    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = vec![
            "absorb".to_owned(),
            "--from".to_owned(),
            exact_change_id_revset(&self.source),
        ];
        for destination in &self.destinations {
            argv.push("--into".to_owned());
            argv.push(exact_change_id_revset(destination));
        }
        argv
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs `jj absorb` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "absorbed")
    }

    /// Returns the preview summary shown before confirming `jj absorb`.
    pub fn preview_summary(&self) -> String {
        let destinations = self
            .destinations
            .iter()
            .map(|destination| format!("candidate destination: {destination}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            concat!(
                "command: {}\n\n",
                "source: {}\n",
                "{}\n\n",
                "selection: selected revisions are candidate destinations; jj absorb only ",
                "considers selected revisions that are ancestors of the source\n\n",
                "effect: jj splits source changes and moves each change to the closest ",
                "selected mutable ancestor where the corresponding lines were last modified\n\n",
                "opacity: jk does not simulate line-level placement or final graph shape\n\n",
                "ambiguity: changes remain in the source when jj cannot choose unambiguously\n\n",
                "source result: the source may become empty or abandoned depending on jj ",
                "semantics\n\n",
                "confirmation: press Enter to run jj absorb\n",
                "recovery: jj undo\n",
                "review: jj op show -p"
            ),
            self.command_label(),
            self.source,
            destinations,
        )
    }

    /// Drops blank or self-target destinations before argv construction.
    fn normalize(mut self) -> Self {
        self.destinations
            .retain(|destination| !destination.trim().is_empty() && destination != &self.source);
        self
    }
}
