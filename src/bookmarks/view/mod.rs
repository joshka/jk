//! Bookmark list view state, rendering, and item-based navigation.
//!
//! The first pass keeps bookmark rows close to rendered `jj` output while carrying exact bookmark
//! names and target ids separately for copy, search, refresh, and open-show behavior.

mod commands;
mod render;
mod state;

use color_eyre::Result;

use super::actions::{JjBookmarkForgetTarget, JjBookmarkMutationKind, JjBookmarkTrackingTarget};
use super::targets::BookmarkActionTargetResolver;
use super::{BookmarkItem, load_bookmark_entries};
use crate::command::{Binding, Command, KeyPattern, ViewCommand};
#[cfg(test)]
use crate::jj::JjCommand;
use crate::jj::ViewSpec;
use crate::selection::Selection;

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
    Binding::new(KeyPattern::char('x'), Command::BookmarkDelete),
    Binding::new(
        KeyPattern::char('n'),
        Command::View(ViewCommand::NextSearchMatch),
    ),
    Binding::new(
        KeyPattern::char('N'),
        Command::View(ViewCommand::PreviousSearchMatch),
    ),
];

/// Selectable bookmark output from `jj bookmark list`.
pub struct BookmarksView {
    /// View identity used to reload the bookmark list.
    pub spec: ViewSpec,
    /// Rendered bookmark rows paired with trusted names, targets, and row state.
    pub entries: Vec<BookmarkItem>,
    /// Current selected row within the bookmark list.
    pub selection: Selection,
}

impl BookmarksView {
    #[cfg(test)]
    pub fn test_new(entries: Vec<BookmarkItem>) -> Self {
        Self {
            entries,
            spec: ViewSpec::new(JjCommand::Bookmarks, Vec::new()),
            selection: Selection::default(),
        }
    }

    #[cfg(test)]
    pub fn test_new_with_args(entries: Vec<BookmarkItem>, args: Vec<String>) -> Self {
        Self {
            entries,
            spec: ViewSpec::new(JjCommand::Bookmarks, args),
            selection: Selection::default(),
        }
    }

    /// Loads rendered bookmark rows and initializes selection at the first row.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: load_bookmark_entries(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    /// Returns the key bindings owned by the bookmarks view.
    pub fn bindings(&self) -> &'static [Binding] {
        super::BINDINGS
    }

    /// Returns the view spec that identifies this bookmarks surface.
    pub fn spec(&self) -> &ViewSpec {
        &self.spec
    }

    /// Returns the number of selectable bookmark rows.
    pub fn item_count(&self) -> usize {
        self.entries.len()
    }

    /// Returns the total rendered line count across all bookmark rows.
    pub fn line_count(&self) -> usize {
        self.entries.iter().map(BookmarkItem::line_count).sum()
    }

    /// Returns the currently selected bookmark row, if any.
    fn selected_entry(&self) -> Option<&BookmarkItem> {
        self.entries.get(self.selection.index())
    }

    /// Returns the exact bookmark name for the selected row.
    pub fn selected_bookmark_name(&self) -> Option<&str> {
        self.selected_entry().map(BookmarkItem::bookmark_name)
    }

    /// Returns the selected local bookmark name when action policy allows it.
    pub fn selected_local_bookmark_name(&self) -> Option<&str> {
        self.action_targets().selected_local_bookmark_name()
    }

    /// Resolves the exact forget target for the selected row when the row state is safe.
    pub fn selected_bookmark_forget_target(
        &self,
    ) -> Result<Option<(&str, JjBookmarkForgetTarget)>> {
        self.action_targets().selected_bookmark_forget_target()
    }

    /// Resolves the exact track or untrack target for the selected row when the row state is safe.
    pub fn selected_bookmark_tracking_target(
        &self,
        kind: JjBookmarkMutationKind,
    ) -> Result<Option<(&str, JjBookmarkTrackingTarget)>> {
        self.action_targets()
            .selected_bookmark_tracking_target(kind)
    }

    /// Builds the bookmark action-target resolver for the current selection and visible rows.
    fn action_targets(&self) -> BookmarkActionTargetResolver<'_> {
        BookmarkActionTargetResolver::new(self.selected_entry(), &self.entries, self.spec.args())
    }
}
