//! Provenance and context recovery for one `ViewSpec`.
//!
//! This child module owns the question "what exact target or revset does this
//! surface represent after startup or in-app navigation?" That keeps exact
//! change-id provenance, startup argv recovery, and app-level diff-format
//! rewrites together instead of mixing them with spec constructors.

use crate::jj::Command;
use crate::jj::view_spec::args::{diff_revset_arg, revision_arg, show_revset_arg};
use crate::jj::view_spec::{DiffFormat, ViewSpec, diff_format_args};

impl ViewSpec {
    pub fn exact_change_target(&self) -> Option<&str> {
        if self.target_is_exact_change {
            self.target.as_deref()
        } else {
            None
        }
    }

    pub fn has_exact_change_target(&self) -> bool {
        self.exact_change_target().is_some()
    }

    pub fn with_exact_change_target(mut self) -> Self {
        self.target_is_exact_change = self.target.is_some();
        self
    }

    pub fn without_exact_change_target(mut self) -> Self {
        self.target_is_exact_change = false;
        self
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Returns the revset to use when opening another detail view from this one.
    ///
    /// Navigated views already know their change id target. Direct startup views
    /// such as `jk show main` do not, so this falls back to command-specific
    /// jj argument parsing. Diff views intentionally ignore filesets here; when
    /// `jj diff` receives only paths, the revision still defaults to `@`.
    pub fn navigation_revset(&self) -> Option<String> {
        self.target.clone().or_else(|| match self.command {
            Command::Show => Some(show_revset_arg(&self.args).unwrap_or("@").to_owned()),
            Command::Diff => Some(diff_revset_arg(&self.args).unwrap_or("@").to_owned()),
            Command::Resolve => Some(revision_arg(&self.args).unwrap_or("@").to_owned()),
            Command::FileList => Some(revision_arg(&self.args).unwrap_or("@").to_owned()),
            Command::FileShow => Some(
                revision_arg(self.file_show_context_args())
                    .unwrap_or("@")
                    .to_owned(),
            ),
            Command::Default
            | Command::Log
            | Command::Status
            | Command::Bookmarks
            | Command::Workspaces
            | Command::OperationLog
            | Command::OperationShow
            | Command::OperationDiff => None,
        })
    }

    pub fn diff_format(&self) -> DiffFormat {
        self.diff_format
    }

    /// Replace the app-level diff format without changing the rest of the view provenance.
    pub fn with_diff_format(&self, diff_format: DiffFormat) -> Self {
        if !matches!(self.command, Command::Show | Command::Diff) {
            return self.clone();
        }

        let mut spec = self.clone();
        spec.diff_format = diff_format;
        spec.args = diff_format_args(
            diff_format,
            spec.args
                .into_iter()
                .filter(|arg| arg != "--git")
                .collect::<Vec<_>>(),
        );
        spec
    }

    /// Recover the revset to use when opening a show-style detail from this surface.
    pub fn show_context_revset(&self) -> String {
        self.target
            .clone()
            .or_else(|| match self.command {
                Command::Resolve => revision_arg(&self.args).map(str::to_owned),
                Command::FileList => revision_arg(&self.args).map(str::to_owned),
                Command::FileShow => revision_arg(self.file_show_context_args()).map(str::to_owned),
                Command::OperationShow | Command::OperationDiff => None,
                _ => show_revset_arg(&self.args).map(str::to_owned),
            })
            .unwrap_or_else(|| "@".to_owned())
    }

    fn file_show_context_args(&self) -> &[String] {
        if matches!(self.command, Command::FileShow) && self.path.is_some() && !self.args.is_empty()
        {
            &self.args[..self.args.len() - 1]
        } else {
            &self.args
        }
    }
}
