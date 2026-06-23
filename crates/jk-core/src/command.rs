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
    global_options: GlobalOptions,
    cwd: Option<PathBuf>,
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
            global_options: GlobalOptions::default(),
            cwd: None,
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
        self.global_options.repository = Some(repository.into());
        self
    }

    /// Sets the global `jj` options.
    #[must_use]
    pub fn with_global_options(mut self, global_options: GlobalOptions) -> Self {
        self.global_options = global_options;
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

    /// Returns global `jj` options owned by the spec.
    #[must_use]
    pub const fn global_options(&self) -> &GlobalOptions {
        &self.global_options
    }

    /// Returns global `jj` arguments rendered before the command family.
    #[must_use]
    pub fn global_argv(&self) -> Vec<OsString> {
        self.global_options.argv()
    }

    /// Returns process arguments after the `jj` binary.
    #[must_use]
    pub fn process_argv(&self) -> Vec<OsString> {
        let mut argv = self.global_argv();
        argv.extend(self.argv.iter().cloned());
        argv
    }

    /// Returns the process working directory metadata.
    #[must_use]
    pub fn cwd(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    /// Returns the repository path metadata.
    #[must_use]
    pub fn repository(&self) -> Option<&Path> {
        self.global_options.repository()
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

/// Global `jj` options rendered before the command family.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalOptions {
    repository: Option<PathBuf>,
    working_copy: WorkingCopyPolicy,
    operation: OperationLoadPolicy,
    operation_integration: OperationIntegrationPolicy,
    immutability: ImmutabilityPolicy,
    output: OutputPolicy,
    debug: bool,
    config_overlays: Vec<ConfigOverlay>,
}

impl GlobalOptions {
    /// Sets the repository passed with `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<PathBuf>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Sets whether `jj` should snapshot and update the working copy.
    #[must_use]
    pub const fn with_working_copy(mut self, working_copy: WorkingCopyPolicy) -> Self {
        self.working_copy = working_copy;
        self
    }

    /// Sets which operation `jj` should load.
    #[must_use]
    pub fn with_operation(mut self, operation: OperationLoadPolicy) -> Self {
        self.operation = operation;
        self
    }

    /// Sets whether `jj` should integrate the loaded operation.
    #[must_use]
    pub const fn with_operation_integration(
        mut self,
        operation_integration: OperationIntegrationPolicy,
    ) -> Self {
        self.operation_integration = operation_integration;
        self
    }

    /// Sets whether immutable commits may be rewritten.
    #[must_use]
    pub const fn with_immutability(mut self, immutability: ImmutabilityPolicy) -> Self {
        self.immutability = immutability;
        self
    }

    /// Sets output-related `jj` options.
    #[must_use]
    pub const fn with_output(mut self, output: OutputPolicy) -> Self {
        self.output = output;
        self
    }

    /// Sets whether `jj --debug` should be rendered.
    #[must_use]
    pub const fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Adds a config overlay in render order.
    #[must_use]
    pub fn with_config_overlay(mut self, overlay: ConfigOverlay) -> Self {
        self.config_overlays.push(overlay);
        self
    }

    /// Returns the configured repository path.
    #[must_use]
    pub fn repository(&self) -> Option<&Path> {
        self.repository.as_deref()
    }

    /// Returns global `jj` arguments in canonical render order.
    #[must_use]
    pub fn argv(&self) -> Vec<OsString> {
        let mut argv = Vec::new();

        self.output.push_argv(&mut argv);

        if let Some(repository) = &self.repository {
            argv.push("--repository".into());
            argv.push(repository.as_os_str().to_owned());
        }

        match (&self.working_copy, &self.operation) {
            (WorkingCopyPolicy::SnapshotAndUpdate, _) => {}
            (WorkingCopyPolicy::Ignore, OperationLoadPolicy::Latest) => {
                argv.push("--ignore-working-copy".into());
            }
            (WorkingCopyPolicy::Ignore, OperationLoadPolicy::AtOperation(_)) => {}
        }

        match &self.operation {
            OperationLoadPolicy::Latest => {}
            OperationLoadPolicy::AtOperation(operation) => {
                argv.push("--at-operation".into());
                argv.push(operation.as_str().into());
            }
        }

        match self.operation_integration {
            OperationIntegrationPolicy::Integrate => {}
            OperationIntegrationPolicy::DoNotIntegrate => {
                argv.push("--no-integrate-operation".into());
            }
        }

        match self.immutability {
            ImmutabilityPolicy::Enforce => {}
            ImmutabilityPolicy::Ignore => argv.push("--ignore-immutable".into()),
        }

        if self.debug {
            argv.push("--debug".into());
        }

        for overlay in &self.config_overlays {
            overlay.push_argv(&mut argv);
        }

        argv
    }
}

impl Default for GlobalOptions {
    fn default() -> Self {
        Self {
            repository: None,
            working_copy: WorkingCopyPolicy::SnapshotAndUpdate,
            operation: OperationLoadPolicy::Latest,
            operation_integration: OperationIntegrationPolicy::Integrate,
            immutability: ImmutabilityPolicy::Enforce,
            output: OutputPolicy::default(),
            debug: false,
            config_overlays: Vec::new(),
        }
    }
}

/// How `jj` should handle working-copy state before running a command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorkingCopyPolicy {
    /// Snapshot and update the working copy.
    SnapshotAndUpdate,
    /// Ignore the working copy.
    Ignore,
}

/// Which operation `jj` should load before running a command.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationLoadPolicy {
    /// Load the latest operation.
    Latest,
    /// Load the repository at a specific operation.
    AtOperation(String),
}

/// Whether `jj` should integrate the loaded operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum OperationIntegrationPolicy {
    /// Integrate the loaded operation.
    Integrate,
    /// Do not integrate the loaded operation.
    DoNotIntegrate,
}

/// Whether `jj` should enforce immutable commits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ImmutabilityPolicy {
    /// Enforce immutable commits.
    Enforce,
    /// Allow commands to rewrite immutable commits.
    Ignore,
}

/// Output-related global `jj` options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputPolicy {
    /// The color policy passed to `jj --color`.
    pub color: ColorPolicy,
    /// The pager policy for `jj` output.
    pub pager: PagerPolicy,
    /// Whether to pass `jj --quiet`.
    pub quiet: bool,
}

impl OutputPolicy {
    fn push_argv(&self, argv: &mut Vec<OsString>) {
        match self.pager {
            PagerPolicy::Disable => argv.push("--no-pager".into()),
            PagerPolicy::Inherit => {}
        }

        argv.push("--color".into());
        argv.push(self.color.as_str().into());

        if self.quiet {
            argv.push("--quiet".into());
        }
    }
}

impl Default for OutputPolicy {
    fn default() -> Self {
        Self {
            color: ColorPolicy::Always,
            pager: PagerPolicy::Disable,
            quiet: false,
        }
    }
}

/// Color rendering policy passed to `jj --color`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ColorPolicy {
    /// Always render color.
    Always,
    /// Never render color.
    Never,
    /// Render color for debugging.
    Debug,
    /// Let `jj` choose automatically.
    Auto,
}

impl ColorPolicy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Never => "never",
            Self::Debug => "debug",
            Self::Auto => "auto",
        }
    }
}

/// Pager rendering policy passed to `jj`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PagerPolicy {
    /// Disable paging with `jj --no-pager`.
    Disable,
    /// Inherit `jj`'s normal pager behavior.
    Inherit,
}

/// Config overlays passed through global `jj` options.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConfigOverlay {
    /// Inline `NAME=VALUE` config passed with `jj --config`.
    Inline {
        /// The raw `NAME=VALUE` value.
        name_value: String,
    },
    /// A config file passed with `jj --config-file`.
    File(PathBuf),
}

impl ConfigOverlay {
    fn push_argv(&self, argv: &mut Vec<OsString>) {
        match self {
            Self::Inline { name_value } => {
                argv.push("--config".into());
                argv.push(name_value.as_str().into());
            }
            Self::File(path) => {
                argv.push("--config-file".into());
                argv.push(path.as_os_str().to_owned());
            }
        }
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

    fn strings(argv: impl IntoIterator<Item = OsString>) -> Vec<String> {
        argv.into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn empty_argv_previews_as_jj() {
        let spec = JjCommandSpec::render_read_only(Vec::<OsString>::new());

        assert_eq!(spec.preview(), "jj");
        assert_eq!(spec.title(), "jj");
    }

    #[test]
    fn default_global_options_render_current_app_globals() {
        let options = GlobalOptions::default();

        assert_eq!(
            strings(options.argv()),
            vec!["--no-pager", "--color", "always"]
        );
    }

    #[test]
    fn repository_renders_before_command_argv() {
        let spec =
            JjCommandSpec::render_read_only(["diff", "-r", "@"]).with_repository("/tmp/repo");

        assert_eq!(
            strings(spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repo",
                "diff",
                "-r",
                "@"
            ]
        );
        assert_eq!(spec.repository(), Some(Path::new("/tmp/repo")));
        assert_eq!(
            spec.global_options().repository(),
            Some(Path::new("/tmp/repo"))
        );
    }

    #[test]
    fn advanced_global_options_render_before_command_argv() {
        let global_options = GlobalOptions::default()
            .with_operation(OperationLoadPolicy::AtOperation("abc123".to_owned()))
            .with_operation_integration(OperationIntegrationPolicy::DoNotIntegrate)
            .with_immutability(ImmutabilityPolicy::Ignore)
            .with_debug(true);
        let spec = JjCommandSpec::render_read_only(["status"]).with_global_options(global_options);

        assert_eq!(
            strings(spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--at-operation",
                "abc123",
                "--no-integrate-operation",
                "--ignore-immutable",
                "--debug",
                "status"
            ]
        );
    }

    #[test]
    fn ignore_working_copy_renders_before_command_argv() {
        let global_options = GlobalOptions::default().with_working_copy(WorkingCopyPolicy::Ignore);
        let spec = JjCommandSpec::render_read_only(["log"]).with_global_options(global_options);

        assert_eq!(
            strings(spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--ignore-working-copy",
                "log"
            ]
        );
    }

    #[test]
    fn at_operation_does_not_duplicate_implied_working_copy_ignore() {
        let global_options = GlobalOptions::default()
            .with_working_copy(WorkingCopyPolicy::Ignore)
            .with_operation(OperationLoadPolicy::AtOperation("abc123".to_owned()));
        let spec = JjCommandSpec::render_read_only(["status"]).with_global_options(global_options);

        assert_eq!(
            strings(spec.process_argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--at-operation",
                "abc123",
                "status"
            ]
        );
    }

    #[test]
    fn config_overlays_preserve_global_order() {
        let options = GlobalOptions::default()
            .with_config_overlay(ConfigOverlay::Inline {
                name_value: "ui.color=always".to_owned(),
            })
            .with_config_overlay(ConfigOverlay::File("/tmp/jj.toml".into()))
            .with_config_overlay(ConfigOverlay::Inline {
                name_value: "aliases.l=log".to_owned(),
            });

        assert_eq!(
            strings(options.argv()),
            vec![
                "--no-pager",
                "--color",
                "always",
                "--config",
                "ui.color=always",
                "--config-file",
                "/tmp/jj.toml",
                "--config",
                "aliases.l=log"
            ]
        );
    }

    #[test]
    fn process_argv_does_not_duplicate_repository() {
        let spec =
            JjCommandSpec::render_read_only(["show", "@"]).with_repository("/tmp/repository");
        let argv = strings(spec.process_argv());

        assert_eq!(
            argv.iter()
                .filter(|arg| arg.as_str() == "--repository")
                .count(),
            1
        );
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
