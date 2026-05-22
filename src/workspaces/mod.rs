//! Read-only `jj root` and `jj workspace list` utility view.
//!
//! Rendered workspace rows stay opaque. Exact workspace names and target ids come only from the
//! separate workspace metadata template, so future actions do not have to depend on label parsing.

mod commands;
mod render;
mod rows;
mod state;

use color_eyre::Result;

use crate::command::{Binding, Command, KeyPattern, ViewCommand};
use crate::jj::ViewSpec;
use crate::selection::Selection;

#[cfg(test)]
pub(crate) use rows::WORKSPACE_METADATA_TEMPLATE;
pub(crate) use rows::{WorkspaceContext, WorkspaceItem, load_workspace_context};

pub const BINDINGS: &[Binding] = &[
    Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Down),
        Command::View(ViewCommand::MoveDown),
    ),
    Binding::new(KeyPattern::char('k'), Command::View(ViewCommand::MoveUp)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Up),
        Command::View(ViewCommand::MoveUp),
    ),
    Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Home),
        Command::View(ViewCommand::MoveFirst),
    ),
    Binding::new(KeyPattern::char('G'), Command::View(ViewCommand::MoveLast)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::End),
        Command::View(ViewCommand::MoveLast),
    ),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Read-only workspace/root context from `jj root` and `jj workspace list`.
pub struct WorkspacesView {
    /// View identity used to reload the workspace surface.
    spec: ViewSpec,
    /// Loaded root context, workspace rows, and any degraded-load diagnostics.
    context: WorkspaceContext,
    /// Current selected row within the workspace list.
    selection: Selection,
}

impl WorkspacesView {
    /// Loads root context and rendered workspace rows for the current view spec.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            context: load_workspace_context(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    #[cfg(test)]
    pub(crate) fn test_new(context: WorkspaceContext) -> Self {
        Self {
            spec: ViewSpec::workspaces(Vec::new()),
            context,
            selection: Selection::default(),
        }
    }

    /// Returns the key bindings owned by the workspaces view.
    pub fn bindings(&self) -> &'static [Binding] {
        BINDINGS
    }

    /// Returns the view spec that identifies this workspaces surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the number of selectable workspace rows.
    pub fn item_count(&self) -> usize {
        self.context.entries().len()
    }

    /// Returns the rendered line count of the header plus all workspace rows.
    pub fn line_count(&self) -> usize {
        self.header_lines().len()
            + self
                .context
                .entries()
                .iter()
                .map(WorkspaceItem::line_count)
                .sum::<usize>()
    }

    /// Returns the currently selected workspace row, if any.
    fn selected_entry(&self) -> Option<&WorkspaceItem> {
        self.context.entries().get(self.selection.index())
    }
}

#[cfg(test)]
mod tests;
