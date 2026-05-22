use color_eyre::Result;

use super::{OperationLogItem, OperationLogView};
use crate::jj::ViewSpec;
use crate::selection::{Selection, restore_by_key_or_index};

impl OperationLogView {
    /// Loads rendered operation-log rows and initializes selection at the first row.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            entries: super::super::load_operation_log_entries(&spec)?,
            spec,
            selection: Selection::default(),
        })
    }

    #[cfg(test)]
    pub fn test_new(entries: Vec<OperationLogItem>) -> Self {
        Self {
            spec: ViewSpec::new(crate::jj::JjCommand::OperationLog, Vec::new()),
            entries,
            selection: Selection::default(),
        }
    }

    /// Reloads rendered rows while preserving the selected exact operation id when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(super::super::load_operation_log_entries)
    }

    /// Reloads rows and restores selection by exact operation id before falling back to index.
    pub fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<OperationLogItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_operation_id = self
            .entries
            .get(previous_index)
            .and_then(OperationLogItem::operation_id)
            .map(str::to_owned);
        self.entries = load(&self.spec)?;
        restore_by_key_or_index(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_operation_id.as_deref(),
            OperationLogItem::operation_id,
        );
        Ok(())
    }
}
