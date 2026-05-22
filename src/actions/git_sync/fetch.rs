use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{command_label_from_argv, exact_string_pattern, run_direct_args};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjGitFetch {
    /// Optional remote override; `None` uses jj's default remote resolution.
    remote: Option<String>,
}

impl JjGitFetch {
    /// Builds a fetch plan that relies on jj's default remote resolution.
    pub fn default_remotes() -> Self {
        Self { remote: None }
    }

    /// Builds a fetch plan for one explicitly selected remote.
    pub fn for_remote(remote: impl Into<String>) -> Self {
        Self {
            remote: Some(remote.into()),
        }
    }

    /// Returns the selected remote override, if any.
    pub fn remote(&self) -> Option<&str> {
        self.remote.as_deref()
    }

    /// Returns the exact remote pattern passed to jj when a remote override is present.
    pub fn exact_remote_pattern(&self) -> Option<String> {
        self.remote.as_deref().map(exact_string_pattern)
    }

    /// Returns the user-facing `jj` command label for this fetch plan.
    pub fn command_label(&self) -> String {
        let command_argv = self.command_argv();
        command_label_from_argv(&command_argv)
    }

    /// Returns argv for `jj git fetch`.
    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = vec!["git".to_owned(), "fetch".to_owned()];
        if let Some(pattern) = self.exact_remote_pattern() {
            argv.push("--remote".to_owned());
            argv.push(pattern);
        }
        argv
    }

    /// Returns the preview summary shown before confirming `jj git fetch`.
    pub fn preview_summary(&self) -> String {
        match self.remote() {
            Some(remote) => {
                let pattern = self
                    .exact_remote_pattern()
                    .expect("remote-specific fetch has a remote pattern");
                [
                    format!("remote: {remote}"),
                    format!("remote pattern: {pattern}"),
                    "effect: fetch only the selected named remote".to_owned(),
                    format!("confirmation: press Enter to run {}", self.command_label()),
                ]
                .join("\n")
            }
            None => [
                "remote: jj default fetch resolution".to_owned(),
                "effect: run jj git fetch without a remote override".to_owned(),
                format!("confirmation: press Enter to run {}", self.command_label()),
            ]
            .join("\n"),
        }
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs `jj git fetch` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "fetched")
    }
}
