//! App-facing dispatcher over the concrete view slices.
//!
//! Feature modules own their state, rendering, bindings, and tests. `ViewState`
//! only routes rendering and command execution to the active slice.

use color_eyre::Result;
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::command::{Binding, CommandContext, ViewCommand, ViewEffect};
use crate::diff::DiffView;
use crate::graph::GraphView;
use crate::jj::{JjCommand, ViewSpec};
use crate::search::SearchQuery;
use crate::show::ShowView;
use crate::tui::StatusHints;

/// The currently active top-level view.
pub enum ViewState {
    Graph(GraphView),
    Show(ShowView),
    Diff(DiffView),
}

impl ViewState {
    pub fn load(spec: ViewSpec) -> Result<Self> {
        match spec.command() {
            JjCommand::Default | JjCommand::Log => Ok(Self::Graph(GraphView::load(spec)?)),
            JjCommand::Show => Ok(Self::Show(ShowView::load(spec)?)),
            JjCommand::Diff => Ok(Self::Diff(DiffView::load(spec)?)),
        }
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, search: Option<&SearchQuery>) {
        match self {
            Self::Graph(view) => view.render(frame, area, search),
            Self::Show(view) => view.render(frame, area, search),
            Self::Diff(view) => view.render(frame, area, search),
        }
    }

    pub fn bindings(&self) -> &'static [Binding] {
        match self {
            Self::Graph(view) => view.bindings(),
            Self::Show(view) => view.bindings(),
            Self::Diff(view) => view.bindings(),
        }
    }

    pub fn execute(&mut self, command: ViewCommand, context: CommandContext<'_>) -> ViewEffect {
        match self {
            Self::Graph(view) => view.execute(command, context),
            Self::Show(view) => view.execute(command, context),
            Self::Diff(view) => view.execute(command, context),
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        match self {
            Self::Graph(view) => view.refresh(),
            Self::Show(view) => view.refresh(),
            Self::Diff(view) => view.refresh(),
        }
    }

    pub fn clamp(&mut self, viewport_height: u16) {
        match self {
            Self::Graph(view) => view.clamp(),
            Self::Show(view) => view.clamp(viewport_height),
            Self::Diff(view) => view.clamp(viewport_height),
        }
    }

    pub fn spec(&self) -> &ViewSpec {
        match self {
            Self::Graph(view) => view.spec(),
            Self::Show(view) => view.spec(),
            Self::Diff(view) => view.spec(),
        }
    }

    pub fn status_hints(&self) -> StatusHints {
        match self {
            Self::Graph(_) => StatusHints::Graph,
            Self::Show(_) => StatusHints::ShowDocument,
            Self::Diff(_) => StatusHints::DiffDocument,
        }
    }

    pub fn command(&self) -> JjCommand {
        self.spec().command()
    }

    pub fn scroll_offset(&self) -> usize {
        match self {
            Self::Graph(_) => 0,
            Self::Show(view) => view.scroll_offset(),
            Self::Diff(view) => view.scroll_offset(),
        }
    }

    pub fn set_scroll_offset(&mut self, viewport_height: u16, scroll_offset: usize) {
        match self {
            Self::Graph(_) => {}
            Self::Show(view) => view.set_scroll_offset(viewport_height, scroll_offset),
            Self::Diff(view) => view.set_scroll_offset(viewport_height, scroll_offset),
        }
    }

    pub fn graph_item_count(&self) -> Option<usize> {
        match self {
            Self::Graph(view) => Some(view.item_count()),
            Self::Show(_) | Self::Diff(_) => None,
        }
    }

    pub fn document_line_count(&self) -> usize {
        match self {
            Self::Graph(view) => view.line_count(),
            Self::Show(view) => view.line_count(),
            Self::Diff(view) => view.line_count(),
        }
    }
}
