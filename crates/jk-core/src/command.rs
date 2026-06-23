//! Command descriptions shared across `jk` crates.
//!
//! [`JjCommandSpec`] stores argv as data first, then renders a display-only preview string for
//! titles, help, and future command previews. Callers must execute the argv directly instead of
//! sending the preview string through a shell.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// A typed description of one `jj` command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjCommandSpec {
    argv: Vec<OsString>,
    cwd: Option<PathBuf>,
    repository: Option<PathBuf>,
    stdin: Option<String>,
    title: String,
    mode: ExecutionMode,
    safety: SafetyClass,
    refresh_plan: RefreshPlan,
}

impl JjCommandSpec {
    /// Creates a read-only `jj` command spec from command arguments after the `jj` binary.
    #[must_use]
    pub fn render_read_only(argv: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        let argv = argv.into_iter().map(Into::into).collect::<Vec<_>>();
        let title = preview_argv(&argv);
        Self {
            argv,
            cwd: None,
            repository: None,
            stdin: None,
            title,
            mode: ExecutionMode::RenderReadOnly,
            safety: SafetyClass::ReadOnly,
            refresh_plan: RefreshPlan::ReRunSpec,
        }
    }

    /// Sets the process working directory metadata.
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Sets the repository path metadata.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Sets stdin text for a future command runner.
    #[must_use]
    pub fn with_stdin(mut self, stdin: impl Into<String>) -> Self {
        self.stdin = Some(stdin.into());
        self
    }

    /// Sets the display title independently from the executable argv.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the execution mode.
    #[must_use]
    pub const fn with_mode(mut self, mode: ExecutionMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the safety class.
    #[must_use]
    pub const fn with_safety(mut self, safety: SafetyClass) -> Self {
        self.safety = safety;
        self
    }

    /// Sets the refresh plan.
    #[must_use]
    pub const fn with_refresh_plan(mut self, refresh_plan: RefreshPlan) -> Self {
        self.refresh_plan = refresh_plan;
        self
    }

    /// Returns command arguments after the `jj` binary.
    #[must_use]
    pub fn argv(&self) -> &[OsString] {
        &self.argv
    }

    /// Returns the process working directory metadata.
    #[must_use]
    pub fn cwd(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    /// Returns the repository path metadata.
    #[must_use]
    pub fn repository(&self) -> Option<&Path> {
        self.repository.as_deref()
    }

    /// Returns stdin text for a future command runner.
    #[must_use]
    pub fn stdin(&self) -> Option<&str> {
        self.stdin.as_deref()
    }

    /// Returns the display title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns a display-only command preview.
    #[must_use]
    pub fn preview(&self) -> String {
        preview_argv(&self.argv)
    }

    /// Returns the execution mode.
    #[must_use]
    pub const fn mode(&self) -> ExecutionMode {
        self.mode
    }

    /// Returns the safety class.
    #[must_use]
    pub const fn safety(&self) -> SafetyClass {
        self.safety
    }

    /// Returns the refresh plan.
    #[must_use]
    pub const fn refresh_plan(&self) -> RefreshPlan {
        self.refresh_plan
    }
}

/// How a command should be executed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExecutionMode {
    /// Run immediately and render output without confirmation.
    RenderReadOnly,
    /// Show a mutation confirmation before execution.
    ConfirmMutation,
    /// Restore the terminal and run a foreground external tool.
    ConfirmExternalTool,
    /// Run a dry-run first, then ask before the real command.
    DryRunThenConfirm,
    /// User-entered command mode.
    CommandMode,
}

/// The safety class for command preview and confirmation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SafetyClass {
    /// Read-only command.
    ReadOnly,
    /// Local metadata update.
    LocalMetadata,
    /// Local history rewrite.
    LocalRewrite,
    /// Destructive local operation.
    DestructiveLocal,
    /// Network read.
    NetworkRead,
    /// Network write.
    NetworkWrite,
    /// External command.
    ExternalCommand,
}

/// What should refresh after a command succeeds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RefreshPlan {
    /// Do not refresh automatically.
    None,
    /// Re-run the command spec that produced the active view.
    ReRunSpec,
}

fn preview_argv(argv: &[OsString]) -> String {
    let mut preview = String::from("jj");
    for arg in argv {
        preview.push(' ');
        preview.push_str(&quote_arg(arg));
    }
    preview
}

fn quote_arg(arg: &OsStr) -> String {
    let arg = arg.to_string_lossy();
    if arg.is_empty() {
        return "''".to_owned();
    }

    if arg
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "-_./:@".contains(character))
    {
        return arg.into_owned();
    }

    format!("'{}'", arg.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_argv_previews_as_jj() {
        let spec = JjCommandSpec::render_read_only(Vec::<OsString>::new());

        assert_eq!(spec.preview(), "jj");
        assert_eq!(spec.title(), "jj");
    }

    #[test]
    fn render_read_only_sets_default_policy() {
        let spec = JjCommandSpec::render_read_only(["diff", "-r", "abc123"]);

        assert_eq!(spec.mode(), ExecutionMode::RenderReadOnly);
        assert_eq!(spec.safety(), SafetyClass::ReadOnly);
        assert_eq!(spec.refresh_plan(), RefreshPlan::ReRunSpec);
    }

    #[test]
    fn preview_quotes_whitespace_and_shell_metacharacters() {
        let spec = JjCommandSpec::render_read_only(["log", "-r", "description('a b')"]);

        assert_eq!(spec.preview(), "jj log -r 'description('\"'\"'a b'\"'\"')'");
    }

    #[test]
    fn preview_quotes_backticks() {
        let spec = JjCommandSpec::render_read_only(["log", "-r", "`echo nope`"]);

        assert_eq!(spec.preview(), "jj log -r '`echo nope`'");
    }

    #[test]
    fn metadata_builders_preserve_argv() {
        let spec = JjCommandSpec::render_read_only(["diff"])
            .with_cwd("/tmp/work")
            .with_repository("/tmp/repo")
            .with_stdin("input")
            .with_title("custom");

        assert_eq!(spec.argv(), &[OsString::from("diff")]);
        assert_eq!(spec.cwd(), Some(Path::new("/tmp/work")));
        assert_eq!(spec.repository(), Some(Path::new("/tmp/repo")));
        assert_eq!(spec.stdin(), Some("input"));
        assert_eq!(spec.title(), "custom");
        assert_eq!(spec.preview(), "jj diff");
    }
}
