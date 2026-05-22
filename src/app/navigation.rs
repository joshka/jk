//! Startup parsing, view-stack navigation, and global view selection.
//!
//! The event loop decides when navigation happens. This module owns how startup
//! arguments become the first view and how app-level transitions replace or
//! stack `ViewState` values. It also owns the top-level view menu, diff-format
//! selection, and custom log revset application because those policies choose or
//! reshape the active app view.

use std::ffi::OsString;

use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::jj::{DiffFormat, JjCommand, LogViewMode, ViewSpec};
use crate::modes::{InteractionMode, ViewMenuAction, view_menu_options};
use crate::status_line::StatusLine;

use super::App;
use super::services::AppServices;

impl App {
    /// Build the initial app state from process arguments.
    ///
    /// Startup chooses the first `ViewSpec`, wires the production service seam, and records any
    /// log argv that `switch_to_log` should later restore.
    pub(in crate::app) fn load(args: Vec<OsString>) -> Result<Self> {
        let initial_spec = initial_view(args)?;
        let startup_log_args =
            (initial_spec.command() == JjCommand::Log).then(|| initial_spec.args().to_vec());
        let diff_format = initial_spec.diff_format();
        let services = AppServices::default();
        let view = services.load_view(initial_spec)?;
        let status = StatusLine::ready(&view);

        Ok(Self {
            view,
            stack: Vec::new(),
            startup_log_args,
            diff_format,
            status,
            mode: InteractionMode::Normal,
            pending_command: None,
            search: None,
            should_quit: false,
            services,
        })
    }

    /// Open a detail surface only when the requested command has a valid detail `ViewSpec`.
    pub(in crate::app) fn push_detail(&mut self, command: JjCommand, revset: String) -> Result<()> {
        let Some(spec) = self.detail_spec(command, revset) else {
            return Ok(());
        };
        self.push_view(spec)
    }

    /// Build the detail `ViewSpec` implied by the current surface and requested detail command.
    ///
    /// Exact-change provenance is preserved only when the source surface actually knows an exact
    /// change target. Direct startup revsets such as `jk show main` intentionally stay inexact.
    pub(in crate::app) fn detail_spec(
        &self,
        command: JjCommand,
        revset: String,
    ) -> Option<ViewSpec> {
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

    /// Report whether the current surface can vouch for an exact change-id detail target.
    fn view_has_exact_detail_target(&self) -> bool {
        matches!(self.view.command(), JjCommand::Default | JjCommand::Log)
            || self.view.spec().has_exact_change_target()
    }

    /// Push the shipped status surface unless it is already active.
    pub(in crate::app) fn open_status(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Status) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Status, Vec::new()))
    }

    /// Push the shipped resolve surface unless it is already active.
    pub(in crate::app) fn open_resolve(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Resolve) {
            return Ok(());
        }

        self.push_view(ViewSpec::resolve(None))
    }

    /// Push the shipped operation-log surface unless it is already active.
    pub(in crate::app) fn open_operation_log(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::OperationLog) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::OperationLog, Vec::new()))
    }

    /// Push the shipped bookmarks surface unless it is already active.
    pub(in crate::app) fn open_bookmarks(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Bookmarks) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Bookmarks, Vec::new()))
    }

    /// Push the shipped workspaces surface unless it is already active.
    pub(in crate::app) fn open_workspaces(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Workspaces) {
            return Ok(());
        }

        self.push_view(ViewSpec::workspaces(Vec::new()))
    }

    /// Push a newly loaded view and keep the previous view on the app-owned back stack.
    pub(in crate::app) fn push_view(&mut self, spec: ViewSpec) -> Result<()> {
        let next = self.services.load_view(spec)?;
        let previous = std::mem::replace(&mut self.view, next);
        self.stack.push(previous);
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    pub(in crate::app) fn pop_view(&mut self) {
        if let Some(previous) = self.stack.pop() {
            self.view = previous;
            self.status = StatusLine::ready(&self.view);
        }
    }

    /// Replace the current stack with the startup log view.
    pub(in crate::app) fn switch_to_log(&mut self) -> Result<()> {
        let args = self.startup_log_args.clone().unwrap_or_default();
        self.stack.clear();
        self.view = self
            .services
            .load_view(ViewSpec::new(JjCommand::Log, args))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    /// Replace the current stack with the default view.
    pub(in crate::app) fn switch_to_default(&mut self) -> Result<()> {
        self.stack.clear();
        self.view = self
            .services
            .load_view(ViewSpec::new(JjCommand::Default, Vec::new()))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    /// Enter the custom-revset prompt only from the log-oriented home surfaces.
    pub(in crate::app) fn open_log_revset_prompt(&mut self) {
        if matches!(self.view.command(), JjCommand::Default | JjCommand::Log) {
            self.mode = InteractionMode::LogRevsetPrompt(String::new());
        }
    }

    /// Apply a user-provided log revset to the current log surface.
    pub(in crate::app) fn apply_custom_log_revset(&mut self, revset: String) {
        if revset.trim().is_empty() {
            self.status = StatusLine::ready(&self.view);
            return;
        }

        match self.view.set_log_mode(LogViewMode::CustomRevset(revset)) {
            Ok(()) => self.status = StatusLine::with_message(&self.view, "mode: custom revset"),
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    /// Open the top-level view menu with the current surface preselected when possible.
    pub(in crate::app) fn open_view_menu(&mut self) {
        let selected = view_menu_options()
            .iter()
            .position(|option| self.view_menu_option_is_current(option.action()))
            .unwrap_or(0);
        self.mode = InteractionMode::ViewMenu { selected };
    }

    fn view_menu_option_is_current(&self, action: ViewMenuAction) -> bool {
        match action {
            ViewMenuAction::Open(command) => self.view.command() == command,
            ViewMenuAction::DiffFormat(format) => {
                matches!(self.view.command(), JjCommand::Show | JjCommand::Diff)
                    && self.diff_format == format
            }
        }
    }

    /// Apply one top-level view-menu choice.
    pub(in crate::app) fn apply_view_menu_action(
        &mut self,
        action: ViewMenuAction,
        viewport_height: u16,
    ) -> Result<()> {
        match action {
            ViewMenuAction::Open(JjCommand::Log) => self.switch_to_log(),
            ViewMenuAction::Open(JjCommand::Default) => self.switch_to_default(),
            ViewMenuAction::Open(JjCommand::Status) => self.open_status(),
            ViewMenuAction::Open(JjCommand::Resolve) => self.open_resolve(),
            ViewMenuAction::Open(JjCommand::Bookmarks) => self.open_bookmarks(),
            ViewMenuAction::Open(JjCommand::Workspaces) => self.open_workspaces(),
            ViewMenuAction::Open(JjCommand::OperationLog) => self.open_operation_log(),
            ViewMenuAction::DiffFormat(diff_format) => {
                self.apply_diff_format(diff_format, viewport_height)
            }
            ViewMenuAction::Open(
                JjCommand::Show
                | JjCommand::Diff
                | JjCommand::FileList
                | JjCommand::FileShow
                | JjCommand::OperationShow
                | JjCommand::OperationDiff,
            ) => {
                self.status = StatusLine::with_message(
                    &self.view,
                    "view menu only opens top-level shipped views",
                );
                Ok(())
            }
        }
    }

    /// Apply the app-level show/diff format toggle and reload the current detail view if needed.
    fn apply_diff_format(&mut self, diff_format: DiffFormat, viewport_height: u16) -> Result<()> {
        self.diff_format = diff_format;
        if !matches!(self.view.command(), JjCommand::Show | JjCommand::Diff) {
            self.status = StatusLine::with_message(
                &self.view,
                format!("show/diff format: {}", diff_format.label()),
            );
            return Ok(());
        }

        let scroll_offset = self.view.scroll_offset();
        let spec = self.view.spec().with_diff_format(diff_format);
        self.view = self.services.load_view(spec)?;
        self.view.set_scroll_offset(viewport_height, scroll_offset);
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }
}

/// Parse process arguments into the first `ViewSpec` the app should load.
///
/// Startup accepts only top-level shipped views here. Deeper drill-down views are reached from
/// in-app navigation once the first surface is loaded.
pub(in crate::app) fn initial_view(args: Vec<OsString>) -> Result<ViewSpec> {
    let args = args
        .into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|arg| eyre!("argument is not valid UTF-8: {arg:?}"))
        })
        .collect::<Result<Vec<_>>>()?;

    let Some((command, rest)) = args.split_first() else {
        return Ok(ViewSpec::new(JjCommand::Default, Vec::new()));
    };

    match command.as_str() {
        "log" => Ok(ViewSpec::new(JjCommand::Log, rest.to_vec())),
        "show" => Ok(ViewSpec::new(JjCommand::Show, rest.to_vec())),
        "diff" => Ok(ViewSpec::new(JjCommand::Diff, rest.to_vec())),
        "status" => Ok(ViewSpec::new(JjCommand::Status, rest.to_vec())),
        "resolve" => {
            if rest.is_empty() {
                Ok(ViewSpec::resolve(None))
            } else {
                Ok(ViewSpec::new(JjCommand::Resolve, rest.to_vec()))
            }
        }
        "bookmarks" => Ok(ViewSpec::bookmarks(rest.to_vec())),
        "workspaces" => Ok(ViewSpec::workspaces(rest.to_vec())),
        "operation-log" => Ok(ViewSpec::new(JjCommand::OperationLog, rest.to_vec())),
        unknown => Err(eyre!(
            "unsupported jk command '{unknown}'. Expected one of: log, show, diff, status, resolve, bookmarks, workspaces, operation-log"
        )),
    }
}
