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
    Bookmark(String),
    Revision(String),
    Status,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjGitPush {
    target: JjGitPushTarget,
    remote: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjGitFetch {
    remote: Option<String>,
}

impl JjGitFetch {
    pub fn default_remotes() -> Self {
        Self { remote: None }
    }

    pub fn for_remote(remote: impl Into<String>) -> Self {
        Self {
            remote: Some(remote.into()),
        }
    }

    pub fn remote(&self) -> Option<&str> {
        self.remote.as_deref()
    }

    pub fn exact_remote_pattern(&self) -> Option<String> {
        self.remote.as_deref().map(exact_string_pattern)
    }

    pub fn command_label(&self) -> String {
        let command_argv = self.command_argv();
        command_label_from_argv(&command_argv)
    }

    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = vec!["git".to_owned(), "fetch".to_owned()];
        if let Some(pattern) = self.exact_remote_pattern() {
            argv.push("--remote".to_owned());
            argv.push(pattern);
        }
        argv
    }

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

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "fetched")
    }
}

#[allow(dead_code)]
impl JjGitPush {
    pub fn for_bookmark(name: String) -> Self {
        Self {
            target: JjGitPushTarget::Bookmark(name),
            remote: None,
        }
    }

    pub fn for_revision(revset: String) -> Self {
        Self {
            target: JjGitPushTarget::Revision(revset),
            remote: None,
        }
    }

    pub fn for_status() -> Self {
        Self {
            target: JjGitPushTarget::Status,
            remote: None,
        }
    }

    pub fn with_remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = Some(remote.into());
        self
    }

    pub fn remote(&self) -> Option<&str> {
        self.remote.as_deref()
    }

    pub fn command_label(&self, dry_run: bool) -> String {
        let label_args = self
            .command_argv(dry_run)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

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

    pub fn run_preview(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(true),
            &self.command_label(true),
            "preview complete",
        )
    }

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
