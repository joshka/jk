//! Operation-log view state, rendering, and item-based navigation.
//!
//! The first pass keeps the operation log close to rendered `jj` output while
//! carrying exact operation ids separately for copy, search, and refresh
//! stability. Recovery includes global `jj undo`/`jj redo` on the repository
//! cursor and selected-row `jj operation restore`/`jj operation revert` flows
//! using exact operation ids.

use crate::command::{Binding, Command, KeyPattern, ViewCommand};
use crate::jj::ViewSpec;
use crate::operation_log::OperationLogItem;
use crate::selection::Selection;

mod commands;
mod render;
mod state;

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
    Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
    Binding::new(KeyPattern::char('l'), Command::View(ViewCommand::OpenShow)),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Right),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(
        KeyPattern::code(crossterm::event::KeyCode::Enter),
        Command::View(ViewCommand::OpenShow),
    ),
    Binding::new(KeyPattern::char('d'), Command::View(ViewCommand::OpenDiff)),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('a'),
        Command::View(ViewCommand::OpenActionMenu),
    ),
    Binding::new(KeyPattern::char('u'), Command::OperationUndo),
    Binding::new(
        KeyPattern::modified_char('r', crossterm::event::KeyModifiers::CONTROL),
        Command::OperationRedo,
    ),
];

/// Selectable rendered operation-log output.
pub struct OperationLogView {
    /// View identity used to reload the operation log.
    pub spec: ViewSpec,
    /// Rendered operation-log items paired with exact operation ids when metadata matches.
    pub entries: Vec<OperationLogItem>,
    /// Current selected row within the operation-log item list.
    pub selection: Selection,
}

impl OperationLogView {
    /// Returns the key bindings owned by the operation-log view.
    pub fn bindings(&self) -> &'static [Binding] {
        super::BINDINGS
    }

    /// Returns the view spec that identifies this operation-log surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the number of selectable operation-log rows.
    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the total rendered line count across all operation-log rows.
    pub fn line_count(&self) -> usize {
        self.entries.iter().map(OperationLogItem::line_count).sum()
    }

    /// Clamps the current selection to the available rows.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }
}
#[cfg(test)]
impl OperationLogView {
    pub fn test_copy_options(&self) -> Vec<crate::menus::CopyOption> {
        self.copy_options()
    }

    pub fn test_refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> color_eyre::Result<Vec<OperationLogItem>>,
    ) -> color_eyre::Result<()> {
        self.refresh_with_loader(load)
    }

    pub fn test_search_matches(&self, query: &crate::search::SearchQuery) -> usize {
        self.search_matches(query)
    }

    pub fn test_next_match(&mut self, query: &crate::search::SearchQuery) -> bool {
        self.next_match(query)
    }
}
