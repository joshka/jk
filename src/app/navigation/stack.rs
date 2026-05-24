use color_eyre::Result;

use super::super::App;
use crate::app::status_line::StatusLine;
use crate::jj::{JjCommand, LogViewMode, ViewSpec};
use crate::modes::InteractionMode;

impl App {
    /// Open a detail surface only when the requested command has a valid detail
    /// `ViewSpec`.
    pub fn push_detail(&mut self, command: JjCommand, revset: String) -> Result<()> {
        let Some(spec) = self.detail_spec(command, revset) else {
            return Ok(());
        };
        self.push_view(spec)
    }

    /// Build the detail `ViewSpec` implied by the current surface and
    /// requested detail command.
    ///
    /// Exact-change provenance is preserved only when the source surface
    /// actually knows an exact change target. Direct startup revsets such as
    /// `jk show main` intentionally stay inexact.
    pub fn detail_spec(&self, command: JjCommand, revset: String) -> Option<ViewSpec> {
        let source_has_exact_target = self.view_has_exact_detail_target();
        let spec = match command {
            JjCommand::Show => {
                let spec = ViewSpec::show(revset, self.diff_format);
                if source_has_exact_target {
                    spec
                } else {
                    spec.without_exact_change_target()
                }
            }
            JjCommand::Diff => {
                let spec = ViewSpec::diff(revset, self.diff_format);
                if source_has_exact_target {
                    spec
                } else {
                    spec.without_exact_change_target()
                }
            }
            JjCommand::FileShow => {
                let spec = ViewSpec::file_show(self.view.spec().navigation_revset(), revset);
                if self.view.spec().has_exact_change_target() {
                    spec.with_exact_change_target()
                } else {
                    spec
                }
            }
            JjCommand::Default
            | JjCommand::Log
            | JjCommand::Status
            | JjCommand::Resolve
            | JjCommand::FileList
            | JjCommand::Bookmarks
            | JjCommand::Workspaces
            | JjCommand::OperationLog
            | JjCommand::OperationShow
            | JjCommand::OperationDiff => return None,
        };
        Some(spec)
    }

    /// Report whether the current surface can vouch for an exact change-id
    /// detail target.
    fn view_has_exact_detail_target(&self) -> bool {
        matches!(self.view.command(), JjCommand::Default | JjCommand::Log)
            || self.view.spec().has_exact_change_target()
    }

    /// Push the shipped status surface unless it is already active.
    pub fn open_status(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Status) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Status, Vec::new()))
    }

    /// Push the shipped resolve surface unless it is already active.
    pub fn open_resolve(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Resolve) {
            return Ok(());
        }

        self.push_view(ViewSpec::resolve(None))
    }

    /// Push the shipped operation-log surface unless it is already active.
    pub fn open_operation_log(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::OperationLog) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::OperationLog, Vec::new()))
    }

    /// Push the shipped bookmarks surface unless it is already active.
    pub fn open_bookmarks(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Bookmarks) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Bookmarks, Vec::new()))
    }

    /// Push the shipped workspaces surface unless it is already active.
    pub fn open_workspaces(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Workspaces) {
            return Ok(());
        }

        self.push_view(ViewSpec::workspaces(Vec::new()))
    }

    /// Push a newly loaded view and keep the previous view on the app-owned
    /// back stack.
    pub fn push_view(&mut self, spec: ViewSpec) -> Result<()> {
        let next = self.services.load_view(spec)?;
        let previous = std::mem::replace(&mut self.view, next);
        self.stack.push(previous);
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    pub fn pop_view(&mut self) {
        if let Some(previous) = self.stack.pop() {
            self.view = previous;
            self.status = StatusLine::ready(&self.view);
        }
    }

    /// Replace the current stack with the explicit top-level log view.
    pub fn switch_to_log(&mut self) -> Result<()> {
        self.stack.clear();
        self.view = self
            .services
            .load_view(ViewSpec::new(JjCommand::Log, Vec::new()))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    /// Replace the current stack with the default view.
    pub fn switch_to_default(&mut self) -> Result<()> {
        self.stack.clear();
        self.view = self
            .services
            .load_view(ViewSpec::new(JjCommand::Default, Vec::new()))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    /// Enter the custom-revset prompt only from the log-oriented home surfaces.
    pub fn open_log_revset_prompt(&mut self) {
        if matches!(self.view.command(), JjCommand::Default | JjCommand::Log) {
            self.mode = InteractionMode::LogRevsetPrompt(String::new());
        }
    }

    /// Apply a user-provided log revset to the current log surface.
    pub fn apply_custom_log_revset(&mut self, revset: String) {
        if revset.trim().is_empty() {
            self.status = StatusLine::ready(&self.view);
            return;
        }

        match self.view.set_log_mode(LogViewMode::CustomRevset(revset)) {
            Ok(()) => self.status = StatusLine::with_message(&self.view, "mode: custom revset"),
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }
}
