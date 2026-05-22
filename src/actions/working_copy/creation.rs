use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{exact_change_id_revset, run_direct_args};

/// Working-copy creation plan for `jj new`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjNewPlan {
    /// Parent revsets passed directly to `jj new` after local normalization.
    parents: Vec<String>,
}

/// Duplicate plan for one exact selected source change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjDuplicatePlan {
    /// Exact selected source change id for `jj duplicate`.
    source: String,
}

impl JjNewPlan {
    /// Builds a `jj new` plan from the selected parent revsets.
    pub fn new(parents: Vec<String>) -> Self {
        Self { parents }.normalize()
    }

    /// Returns the normalized parent revsets used by this plan.
    pub fn parents(&self) -> &[String] {
        &self.parents
    }

    /// Returns the user-facing `jj` command label for this new plan.
    pub fn command_label(&self) -> String {
        let label_args = self
            .command_argv()
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj new`.
    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = vec!["new".to_owned()];
        argv.extend(self.parents.iter().cloned());
        argv
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs `jj new` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            "created new change",
        )
    }

    /// Returns the preview summary shown before confirming `jj new`.
    pub fn preview_summary(&self) -> String {
        let parents = self
            .parents
            .iter()
            .map(|parent| format!("parent: {parent}"))
            .collect::<Vec<_>>()
            .join("\n");
        let graph_effect = if self.parents.len() == 1 {
            "graph effect: creates a new working-copy change from the selected parent"
        } else {
            "graph effect: creates a new working-copy merge change from the selected parents"
        };

        format!(
            "command: {}\n\n{}\n\n{}\n\nconfirmation: press Enter to run jj new\nundo path: jj undo",
            self.command_label(),
            parents,
            graph_effect,
        )
    }

    /// Drops blank parent inputs before argv construction.
    fn normalize(mut self) -> Self {
        self.parents.retain(|parent| !parent.trim().is_empty());
        self
    }
}

impl JjDuplicatePlan {
    /// Builds a duplicate plan for one exact selected source revision.
    pub fn exact_change(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    /// Returns the exact selected source change id.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the user-facing `jj` command label for this duplicate plan.
    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj duplicate`.
    pub fn command_argv(&self) -> Vec<String> {
        vec!["duplicate".to_owned(), exact_change_id_revset(&self.source)]
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs `jj duplicate` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "duplicated")
    }

    /// Returns the preview summary shown before confirming `jj duplicate`.
    pub fn preview_summary(&self) -> String {
        [
            format!("command: {}", self.command_label()),
            String::new(),
            format!("source revision: {}", self.source),
            "source count: 1 exact selected change; multi-source duplicate is intentionally not exposed".to_owned(),
            "effect: creates one new change with the same content as the selected source".to_owned(),
            "placement: jj duplicates onto the source's existing parents because no destination is passed".to_owned(),
            "description: jj controls duplicate description behavior through its own configuration".to_owned(),
            "after run: jk refreshes and selects the source in recent work as a fallback because it does not parse duplicate output for the new change id".to_owned(),
            "confirmation: press Enter to run jj duplicate".to_owned(),
            "cancel: press Esc to return without running jj duplicate".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }
}
