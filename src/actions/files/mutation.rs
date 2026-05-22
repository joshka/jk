use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{exact_change_id_revset, root_file_fileset, run_direct_args};

/// File mutation subcommand represented by a preview-first plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileMutationKind {
    /// Track an exact repository-root path.
    Track,
    /// Untrack an exact repository-root path.
    Untrack,
    /// Change executable mode for an exact repository-root path.
    Chmod(JjFileChmodMode),
}

/// File executable-bit mode accepted by `jj file chmod`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileChmodMode {
    /// Mark the file executable.
    Executable,
    /// Mark the file non-executable / normal.
    Normal,
}

/// Exact target scope for a file mutation.
///
/// Paths are stored as repository-root file paths and converted to exact root-file filesets during
/// argv construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjFileMutationTarget {
    /// Mutate a path in the current working-copy change.
    WorkingCopy { path: String },
    /// Mutate a path in one exact selected revision.
    ExactRevision { revision: String, path: String },
}

/// Preview-first plan for `jj file track`, `untrack`, and `chmod`.
///
/// The plan owns argv construction for exact filesets. It does not decide whether a file action is
/// available; that policy belongs to the selected view/action-menu owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjFileMutationPlan {
    /// File mutation subcommand owned by this plan.
    kind: JjFileMutationKind,
    /// Exact target scope for the mutation.
    target: JjFileMutationTarget,
}

impl JjFileChmodMode {
    /// Returns the argv token used by `jj file chmod`.
    pub fn command_arg(self) -> &'static str {
        match self {
            Self::Executable => "x",
            Self::Normal => "n",
        }
    }

    /// Returns the user-facing mode label for preview text.
    pub fn label(self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::Normal => "normal",
        }
    }
}

impl JjFileMutationKind {
    /// Returns the user-facing action label for this mutation kind.
    pub fn label(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Untrack => "untrack",
            Self::Chmod(JjFileChmodMode::Executable) => "chmod x",
            Self::Chmod(JjFileChmodMode::Normal) => "chmod n",
        }
    }

    /// Returns fallback success wording when `jj` does not provide one.
    fn success_fallback(self) -> &'static str {
        match self {
            Self::Track => "tracked file",
            Self::Untrack => "untracked file",
            Self::Chmod(JjFileChmodMode::Executable) => "set file executable",
            Self::Chmod(JjFileChmodMode::Normal) => "set file normal",
        }
    }
}

impl JjFileMutationTarget {
    /// Returns the exact repository-root path for this target.
    fn path(&self) -> &str {
        match self {
            Self::WorkingCopy { path } | Self::ExactRevision { path, .. } => path,
        }
    }

    /// Returns the exact selected revision for this target, if any.
    fn revision(&self) -> Option<&str> {
        match self {
            Self::WorkingCopy { .. } => None,
            Self::ExactRevision { revision, .. } => Some(revision),
        }
    }

    /// Returns the user-facing scope label for preview text.
    fn scope_label(&self) -> String {
        match self {
            Self::WorkingCopy { .. } => "working-copy change (@)".to_owned(),
            Self::ExactRevision { revision, .. } => {
                format!("exact selected revision {revision}")
            }
        }
    }
}

impl JjFileMutationPlan {
    /// Track an exact repository-root working-copy path.
    pub fn track(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Track,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    /// Untrack an exact repository-root working-copy path.
    pub fn untrack(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Untrack,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    /// Change executable mode for an exact repository-root path in `@`.
    pub fn chmod_working_copy(path: impl Into<String>, mode: JjFileChmodMode) -> Self {
        Self {
            kind: JjFileMutationKind::Chmod(mode),
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    /// Change executable mode for an exact repository-root path in a rendered revision.
    pub fn chmod_exact_revision(
        revision: impl Into<String>,
        path: impl Into<String>,
        mode: JjFileChmodMode,
    ) -> Self {
        Self {
            kind: JjFileMutationKind::Chmod(mode),
            target: JjFileMutationTarget::ExactRevision {
                revision: revision.into(),
                path: path.into(),
            },
        }
    }

    /// Returns the mutation kind owned by this plan.
    pub fn kind(&self) -> JjFileMutationKind {
        self.kind
    }

    /// Returns the exact repository-root path targeted by this mutation.
    pub fn path(&self) -> &str {
        self.target.path()
    }

    /// Returns the exact selected revision targeted by this mutation, if any.
    pub fn revision(&self) -> Option<&str> {
        self.target.revision()
    }

    /// Returns the user-facing `jj` command label for this mutation plan.
    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for the underlying `jj file` mutation.
    pub fn command_argv(&self) -> Vec<String> {
        let fileset = root_file_fileset(self.path());
        match self.kind {
            JjFileMutationKind::Track => {
                vec![
                    "file".to_owned(),
                    "track".to_owned(),
                    "--".to_owned(),
                    fileset,
                ]
            }
            JjFileMutationKind::Untrack => {
                vec![
                    "file".to_owned(),
                    "untrack".to_owned(),
                    "--".to_owned(),
                    fileset,
                ]
            }
            JjFileMutationKind::Chmod(mode) => {
                let mut argv = vec!["file".to_owned(), "chmod".to_owned()];
                if let Some(revision) = self.revision() {
                    argv.push("-r".to_owned());
                    argv.push(exact_change_id_revset(revision));
                }
                argv.push(mode.command_arg().to_owned());
                argv.push("--".to_owned());
                argv.push(fileset);
                argv
            }
        }
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Execute the file mutation; availability and confirmation are external policy.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            self.kind.success_fallback(),
        )
    }

    /// Summarize the exact target scope without pretending to know jj's final status output.
    pub fn preview_summary(&self) -> String {
        let mut lines = vec![
            format!("command: {}", self.command_label()),
            String::new(),
            format!("selected path: {}", self.path()),
            format!("exact fileset: {}", root_file_fileset(self.path())),
            format!("target scope: {}", self.target.scope_label()),
        ];

        if let JjFileMutationKind::Chmod(mode) = self.kind {
            lines.push(format!(
                "chmod mode: {} ({})",
                mode.command_arg(),
                mode.label()
            ));
        }

        lines.extend([
            self.effect_summary(),
            "confirmation: press Enter to run the command above".to_owned(),
            "output: jj stdout and stderr are preserved in this result pane".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj status; jj op show -p".to_owned(),
        ]);
        lines.join("\n")
    }

    fn effect_summary(&self) -> String {
        match self.kind {
            JjFileMutationKind::Track => {
                "effect: starts tracking this exact untracked working-copy path".to_owned()
            }
            JjFileMutationKind::Untrack => {
                "effect: stops tracking this exact working-copy path; jj requires the path to already be ignored".to_owned()
            }
            JjFileMutationKind::Chmod(JjFileChmodMode::Executable) => {
                "effect: marks this exact path executable in the target scope".to_owned()
            }
            JjFileMutationKind::Chmod(JjFileChmodMode::Normal) => {
                "effect: marks this exact path non-executable in the target scope".to_owned()
            }
        }
    }
}
