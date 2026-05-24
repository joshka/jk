use std::ffi::OsString;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use itertools::Itertools;
use ratatui::layout::Rect;

use super::super::App;
use super::super::services::AppServices;
use crate::app::status_line::StatusLine;
use crate::jj::{JjCommand, ViewSpec};
use crate::modes::InteractionMode;

impl App {
    /// Build the initial app state from process arguments.
    ///
    /// Startup chooses the first `ViewSpec` and wires the production service seam.
    pub fn load(args: Vec<OsString>) -> Result<Self> {
        let initial_spec = initial_view(args)?;
        let diff_format = initial_spec.diff_format();
        let services = AppServices::default();
        let view = services.load_view(initial_spec)?;
        let status = StatusLine::ready(&view);

        Ok(Self {
            view,
            stack: Vec::new(),
            viewport: Rect {
                x: 0,
                y: 0,
                height: u16::MAX,
                width: u16::MAX,
            },
            diff_format,
            status,
            mode: InteractionMode::Normal,
            pending_command: None,
            pending_interactive_action: None,
            search: None,
            should_quit: false,
            services,
        })
    }
}

/// Parse process arguments into the first `ViewSpec` the app should load.
///
/// Startup accepts only top-level shipped views here. Deeper drill-down views
/// are reached from in-app navigation once the first surface is loaded.
pub fn initial_view(args: Vec<OsString>) -> Result<ViewSpec> {
    let args_utf8: Vec<String> = args
        .into_iter()
        .map(OsString::into_string)
        .try_collect()
        .map_err(|arg| eyre!("startup argument is not valid UTF-8: {arg:?}"))?;

    let Some((command, rest)) = args_utf8.split_first() else {
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
