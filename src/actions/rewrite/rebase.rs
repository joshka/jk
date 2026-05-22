use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::run_direct_args;

/// Rewrite plans share explicit source/destination roles and avoid parsing or
/// predicting jj's final graph shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRebasePlan {
    /// Source revisions selected for rebase.
    sources: Vec<String>,
    /// Destination revision the sources will be rebased onto.
    destination: String,
}

impl JjRebasePlan {
    /// Builds a rebase plan from explicit source and destination roles.
    pub fn new(sources: Vec<String>, destination: impl Into<String>) -> Self {
        Self {
            sources,
            destination: destination.into(),
        }
        .normalize()
    }

    /// Returns the source revisions owned by this rebase plan.
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Returns the destination revision for this rebase plan.
    pub fn destination(&self) -> &str {
        &self.destination
    }

    /// Returns the user-facing `jj` command label for this rebase plan.
    pub fn command_label(&self, _dry_run: bool) -> String {
        let label_args = self
            .command_argv(false)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj rebase`.
    pub fn command_argv(&self, _dry_run: bool) -> Vec<String> {
        let mut argv = vec!["rebase".to_owned()];
        for source in &self.sources {
            argv.push("-r".to_owned());
            argv.push(source.clone());
        }
        argv.push("-o".to_owned());
        argv.push(self.destination.clone());
        argv
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs `jj rebase` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "rebased",
        )
    }

    /// Returns the preview summary shown before confirming `jj rebase`.
    pub fn preview_summary(&self) -> String {
        let sources = self
            .sources
            .iter()
            .map(|source| format!("source revision: {source}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "command: {}\n\nroles:\n{}\ndestination revision: {}\n\ncurrent log context:\n- source rows are selected in jk\n- destination is the current row\n\nexpected jj effect:\n- semantics: jj rebase --revision <source> --onto <destination>\n- only listed source revisions are rebased\n- dependencies among listed sources are preserved\n- descendants outside the selected set may be rebased to fill holes\n- destination descendants are not inserted or rebased by -o\n\nnot a graph preview: jk has not run jj and is not simulating the final graph\n\nreview after run: jj op show -p\nundo path: jj undo\nconfirmation: press Enter to run jj rebase",
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
