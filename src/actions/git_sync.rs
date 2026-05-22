//! Git fetch and push action plans.
//!
//! This module owns git sync argv construction, dry-run preview labels, and
//! remote selection details. App prompt policy, row loading, and status wording
//! stay with their existing owners.

use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::run_direct_args;
use crate::jj::{command_label_from_argv, exact_string_pattern};

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjGitPushTarget {
    /// Push one exact bookmark by name.
    Bookmark(String),
    /// Push one exact revision revset.
    Revision(String),
    /// Push the status/default target without an explicit bookmark or revision argument.
    Status,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjGitPush {
    /// Push target policy selected by the caller.
    target: JjGitPushTarget,
    /// Optional remote override; `None` uses jj's default remote resolution.
    remote: Option<String>,
}

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

#[allow(dead_code)]
impl JjGitPush {
    /// Builds a push plan for one exact bookmark name.
    pub fn for_bookmark(name: String) -> Self {
        Self {
            target: JjGitPushTarget::Bookmark(name),
            remote: None,
        }
    }

    /// Builds a push plan for one revision revset.
    pub fn for_revision(revset: String) -> Self {
        Self {
            target: JjGitPushTarget::Revision(revset),
            remote: None,
        }
    }

    /// Builds a push plan that relies on jj's default status target.
    pub fn for_status() -> Self {
        Self {
            target: JjGitPushTarget::Status,
            remote: None,
        }
    }

    /// Adds an explicit remote override to the push plan.
    pub fn with_remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = Some(remote.into());
        self
    }

    /// Returns the selected remote override, if any.
    pub fn remote(&self) -> Option<&str> {
        self.remote.as_deref()
    }

    /// Returns the user-facing `jj` command label for this push plan.
    pub fn command_label(&self, dry_run: bool) -> String {
        let label_args = self
            .command_argv(dry_run)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj git push`, optionally in dry-run mode.
    pub fn command_argv(&self, dry_run: bool) -> Vec<String> {
        let mut argv = vec!["git".to_owned(), "push".to_owned()];

        if dry_run {
            argv.push("--dry-run".to_owned());
        }
        if let Some(remote) = &self.remote {
            argv.push("--remote".to_owned());
            argv.push(remote.clone());
        }

        match &self.target {
            JjGitPushTarget::Bookmark(name) => {
                argv.push("--bookmark".to_owned());
                argv.push(name.clone());
            }
            JjGitPushTarget::Revision(revset) => {
                argv.push("--revision".to_owned());
                argv.push(revset.clone());
            }
            JjGitPushTarget::Status => {}
        }

        argv
    }

    /// Runs the jj dry-run preview and returns its preserved output.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(true),
            &self.command_label(true),
            "preview complete",
        )
    }

    /// Runs `jj git push` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "pushed",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_push_bookmark_args_include_dry_run_when_previewing() {
        let push = JjGitPush::for_bookmark("main".to_owned()).with_remote("origin".to_owned());

        assert_eq!(
            push.command_argv(true),
            vec![
                "git",
                "push",
                "--dry-run",
                "--remote",
                "origin",
                "--bookmark",
                "main"
            ]
        );
        assert_eq!(
            push.command_label(false),
            "jj git push --remote origin --bookmark main"
        );
        assert_eq!(
            push.command_label(true),
            "jj git push --dry-run --remote origin --bookmark main"
        );
        assert_eq!(
            push.command_argv(false),
            vec!["git", "push", "--remote", "origin", "--bookmark", "main"]
        );
    }

    #[test]
    fn git_push_revision_args_follow_revision_target() {
        let push = JjGitPush::for_revision("main".to_owned()).with_remote("origin".to_owned());

        assert_eq!(
            push.command_argv(true),
            vec![
                "git",
                "push",
                "--dry-run",
                "--remote",
                "origin",
                "--revision",
                "main"
            ]
        );
    }

    #[test]
    fn git_push_revision_can_use_jj_default_remote_resolution() {
        let push = JjGitPush::for_revision("main".to_owned());

        assert_eq!(
            push.command_argv(false),
            vec!["git", "push", "--revision", "main"]
        );
        assert_eq!(
            push.command_label(true),
            "jj git push --dry-run --revision main"
        );
    }

    #[test]
    fn git_push_status_default_uses_remote_only_target() {
        let push = JjGitPush::for_status().with_remote("origin".to_owned());

        assert_eq!(
            push.command_argv(false),
            vec!["git", "push", "--remote", "origin"]
        );
        assert_eq!(
            push.command_label(true),
            "jj git push --dry-run --remote origin"
        );
    }

    #[test]
    fn git_push_bookmark_can_use_jj_default_remote_resolution() {
        let push = JjGitPush::for_bookmark("main".to_owned());

        assert_eq!(
            push.command_argv(true),
            vec!["git", "push", "--dry-run", "--bookmark", "main"]
        );
    }

    #[test]
    fn git_fetch_default_uses_jj_default_remote_resolution() {
        let fetch = JjGitFetch::default_remotes();

        assert_eq!(fetch.command_argv(), vec!["git", "fetch"]);
        assert_eq!(fetch.command_label(), "jj git fetch");
        assert!(fetch.exact_remote_pattern().is_none());
    }

    #[test]
    fn git_fetch_remote_uses_exact_remote_pattern() {
        let fetch = JjGitFetch::for_remote("origin");

        assert_eq!(
            fetch.command_argv(),
            vec!["git", "fetch", "--remote", "exact:\"origin\""]
        );
        assert_eq!(
            fetch.command_label(),
            "jj git fetch --remote exact:\"origin\""
        );
        assert_eq!(
            fetch.exact_remote_pattern().as_deref(),
            Some("exact:\"origin\"")
        );
        assert!(
            fetch
                .preview_summary()
                .contains("remote pattern: exact:\"origin\"")
        );
    }

    #[test]
    fn git_push_keeps_status_target_with_no_remote_optional() {
        assert_eq!(
            JjGitPush::for_status().command_argv(true),
            vec!["git", "push", "--dry-run"]
        );
    }
}
