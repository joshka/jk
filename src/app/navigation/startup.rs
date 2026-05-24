use std::ffi::OsString;
use std::str::FromStr;

use color_eyre::Result;
use color_eyre::eyre::{Report, bail, eyre};
use itertools::Itertools;
use ratatui::layout::Rect;

use crate::app::App;
use crate::app::services::AppServices;
use crate::app::status_line::StatusLine;
use crate::jj::{self, ViewSpec};
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
        // Before the first draw, treat the main viewport as effectively unbounded.
        let viewport = Rect::MAX;

        Ok(Self {
            view,
            stack: Vec::new(),
            viewport,
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
/// Startup accepts only top-level shipped views here. `Command` owns the subset of command names
/// that make sense before the app has any active surface, while `jj::Command` continues to own
/// the broader rendered-view vocabulary once startup has already chosen that top-level surface.
/// Deeper drill-down views are reached from in-app navigation once the first surface is loaded.
/// Returns an error when a startup argument is not valid UTF-8 or when the first argument is not
/// one of the shipped top-level startup views.
pub fn initial_view(args: Vec<OsString>) -> Result<ViewSpec> {
    let args_utf8: Vec<String> = args
        .into_iter()
        .map(OsString::into_string)
        .try_collect()
        .map_err(|arg| eyre!("startup argument is not valid UTF-8: {arg:?}"))?;

    let Some((command, rest)) = args_utf8.split_first() else {
        return Ok(ViewSpec::home());
    };

    let command: Command = command.parse()?;
    match command {
        Command::Resolve if rest.is_empty() => Ok(ViewSpec::resolve_current()),
        _ => Ok(ViewSpec::new(command.jj_command(), rest.to_vec())),
    }
}

/// Startup-only top-level commands accepted on the `jk` CLI.
///
/// This remains narrower than `jj::Command`: startup chooses one shipped home surface, then later
/// in-app navigation can move into detail-oriented command families that are not valid as direct
/// startup entry points. This enum owns the textual startup boundary only; it does not try to
/// represent the full in-app rendered-view command vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
    Log,
    Show,
    Diff,
    Status,
    Resolve,
    Bookmarks,
    Workspaces,
    OperationLog,
}

impl Command {
    /// Map one startup-only command to the rendered `jj` command family it opens.
    fn jj_command(self) -> jj::Command {
        match self {
            Self::Log => jj::Command::Log,
            Self::Show => jj::Command::Show,
            Self::Diff => jj::Command::Diff,
            Self::Status => jj::Command::Status,
            Self::Resolve => jj::Command::Resolve,
            Self::Bookmarks => jj::Command::Bookmarks,
            Self::Workspaces => jj::Command::Workspaces,
            Self::OperationLog => jj::Command::OperationLog,
        }
    }
}

impl FromStr for Command {
    type Err = Report;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        match command {
            "log" => Ok(Self::Log),
            "show" => Ok(Self::Show),
            "diff" => Ok(Self::Diff),
            "status" => Ok(Self::Status),
            "resolve" => Ok(Self::Resolve),
            "bookmarks" => Ok(Self::Bookmarks),
            "workspaces" => Ok(Self::Workspaces),
            "operation-log" => Ok(Self::OperationLog),
            _ => bail!(
                "unsupported jk command '{command}'. Expected one of: \
                 log, show, diff, status, resolve, bookmarks, workspaces, operation-log"
            ),
        }
    }
}
