use color_eyre::Result;

use super::{WorkspaceContext, WorkspaceItem, WorkspacesView, load_workspace_context};
use crate::jj::ViewSpec;

impl WorkspacesView {
    /// Reloads the workspace context while preserving the selected workspace name when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_workspace_context)
    }

    /// Clamps the current selection to the available rows.
    pub fn clamp(&mut self) {
        self.selection.clamp(self.context.entries().len());
    }

    /// Reloads the context and restores selection by workspace name before falling back to index.
    pub fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<WorkspaceContext>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_name = self
            .selected_entry()
            .and_then(WorkspaceItem::name)
            .map(str::to_owned);

        self.context = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            self.context.entries(),
            previous_index,
            previous_name,
        );
        Ok(())
    }
}

/// Restores selection by exact workspace name before falling back to the previous index.
fn restore_selection(
    selection: &mut crate::selection::Selection,
    entries: &[WorkspaceItem],
    previous_index: usize,
    previous_name: Option<String>,
) {
    if let Some(name) = previous_name
        && let Some(index) = entries
            .iter()
            .position(|entry| entry.name() == Some(name.as_str()))
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}
