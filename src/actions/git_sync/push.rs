use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::run_direct_args;

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

    /// Returns the user-facing `jj` command label for this push plan preview.
    pub fn preview_command_label(&self) -> String {
        let label_args = self
            .preview_command_argv()
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    /// Returns the user-facing `jj` command label for this push plan.
    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for `jj git push --dry-run`.
    pub fn preview_command_argv(&self) -> Vec<String> {
        let mut argv = vec!["git".to_owned(), "push".to_owned()];
        argv.push("--dry-run".to_owned());
        argv.extend(self.target_argv());
        argv
    }

    /// Returns argv for `jj git push`.
    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = vec!["git".to_owned(), "push".to_owned()];
        argv.extend(self.target_argv());
        argv
    }

    fn target_argv(&self) -> Vec<String> {
        let mut argv = Vec::new();
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
            self.preview_command_argv(),
            &self.preview_command_label(),
            "preview complete",
        )
    }

    /// Runs `jj git push` through the direct command boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "pushed")
    }
}
