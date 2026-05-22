use color_eyre::Result;

use crate::jj::ViewSpec;
use crate::selection::Selection;

use super::{BookmarkItem, BookmarksView, load_bookmark_entries};

impl BookmarksView {
    /// Reloads rendered bookmark rows while preserving the selected bookmark name when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_bookmark_entries)
    }

    /// Clamps the current selection to the available rows.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    /// Reloads rows and restores selection by bookmark name before falling back to index.
    pub(crate) fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<BookmarkItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_bookmark_name = self
            .selected_entry()
            .map(|entry| entry.bookmark_name().to_owned());

        self.entries = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_bookmark_name,
        );
        Ok(())
    }
}

/// Restores selection by exact bookmark name before falling back to the previous index.
fn restore_selection(
    selection: &mut Selection,
    entries: &[BookmarkItem],
    previous_index: usize,
    previous_bookmark_name: Option<String>,
) {
    if let Some(bookmark_name) = previous_bookmark_name
        && let Some(index) = entries
            .iter()
            .position(|entry| entry.bookmark_name() == bookmark_name.as_str())
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}
