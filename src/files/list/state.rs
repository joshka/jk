use color_eyre::Result;

use crate::selection::restore_by_key_or_index;

use super::{FileListItem, FileListView, ViewSpec, load_file_list_entries};

impl FileListView {
    /// Reload the file list while preserving the selected path when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_file_list_entries)
    }

    /// Clamp the current selection to the current entry count.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.entries.len());
    }

    /// Reload entries with a caller-supplied loader while restoring selection by exact path first.
    pub fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<FileListItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_path = self.selected_path().map(str::to_owned);

        self.entries = load(&self.spec)?;
        restore_by_key_or_index(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_path.as_deref(),
            |entry| Some(entry.path()),
        );
        Ok(())
    }
}
