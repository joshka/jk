//! Startup parsing and app-owned view-stack navigation.
//!
//! The event loop decides when navigation happens. This module owns how startup
//! arguments become the first view and how app-level transitions replace or
//! stack `ViewState` values.

use std::ffi::OsString;

use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::app_screen::InteractionMode;
use crate::app_status::StatusLine;
use crate::jj::{JjCommand, ViewSpec};

use super::App;
use super::services::AppServices;

impl App {
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

    pub(in crate::app) fn push_detail(&mut self, command: JjCommand, revset: String) -> Result<()> {
        let Some(spec) = self.detail_spec(command, revset) else {
            return Ok(());
        };
        self.push_view(spec)
    }

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
            | JjCommand::OperationLog
            | JjCommand::OperationShow
            | JjCommand::OperationDiff => return None,
        };
        Some(spec)
    }

    fn view_has_exact_detail_target(&self) -> bool {
        matches!(self.view.command(), JjCommand::Default | JjCommand::Log)
            || self.view.spec().has_exact_change_target()
    }

    pub(in crate::app) fn open_status(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Status) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Status, Vec::new()))
    }

    pub(in crate::app) fn open_resolve(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Resolve) {
            return Ok(());
        }

        self.push_view(ViewSpec::resolve(None))
    }

    pub(in crate::app) fn open_operation_log(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::OperationLog) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::OperationLog, Vec::new()))
    }

    pub(in crate::app) fn open_bookmarks(&mut self) -> Result<()> {
        if matches!(self.view.command(), JjCommand::Bookmarks) {
            return Ok(());
        }

        self.push_view(ViewSpec::new(JjCommand::Bookmarks, Vec::new()))
    }

    pub(in crate::app) fn push_view(&mut self, spec: ViewSpec) -> Result<()> {
        let next = self.load_view_state(spec)?;
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

    pub(in crate::app) fn switch_to_log(&mut self) -> Result<()> {
        let args = self.startup_log_args.clone().unwrap_or_default();
        self.stack.clear();
        self.view = self.load_view_state(ViewSpec::new(JjCommand::Log, args))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }

    pub(in crate::app) fn switch_to_default(&mut self) -> Result<()> {
        self.stack.clear();
        self.view = self.load_view_state(ViewSpec::new(JjCommand::Default, Vec::new()))?;
        self.status = StatusLine::ready(&self.view);
        Ok(())
    }
}

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
        "bookmarks" => Ok(ViewSpec::new(JjCommand::Bookmarks, rest.to_vec())),
        "operation-log" => Ok(ViewSpec::new(JjCommand::OperationLog, rest.to_vec())),
        unknown => Err(eyre!(
            "unsupported jk command '{unknown}'. Expected one of: log, show, diff, status, resolve, bookmarks, operation-log"
        )),
    }
}
