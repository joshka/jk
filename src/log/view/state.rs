use color_eyre::Result;

use super::{LogItem, LogView};
use crate::jj::{LogViewMode, ViewSpec};

impl LogView {
    /// Load the default/log view from a parsed `ViewSpec`.
    ///
    /// Startup and log-mode switches both come through here. The loader keeps
    /// the `ViewSpec`, derives the current log mode from it, and fetches the
    /// rendered log rows before any navigation state is applied.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        let home_command = spec.command();
        let mode = LogViewMode::from_spec(&spec);

        Ok(Self {
            home_command,
            mode,
            entries: super::super::load_entries(&spec)?,
            spec,
            selection: crate::selection::Selection::default(),
            selected_change_ids: Vec::new(),
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(super::super::load_entries)
    }

    fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_change_id = self
            .entries
            .get(previous_index)
            .and_then(LogItem::action_id)
            .map(str::to_owned);

        self.entries = load(&self.spec)?;
        retain_selected_change_ids(&mut self.selected_change_ids, &self.entries);
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_change_id,
        );
        Ok(())
    }

    #[cfg(test)]
    pub fn test_refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<()> {
        self.refresh_with_loader(load)
    }

    pub fn select_change_id(&mut self, change_id: &str) -> bool {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.action_id() == Some(change_id))
        else {
            return false;
        };
        self.selection.set(index, self.entries.len());
        true
    }

    pub fn reveal_change_id(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
    ) -> Result<bool> {
        self.reveal_change_id_with_loader(change_id, fallback_mode, super::super::load_entries)
    }

    pub fn set_mode(&mut self, mode: LogViewMode) -> Result<()> {
        self.switch_mode_with_loader(mode, super::super::load_entries)
    }

    pub fn cycle_mode(&mut self) -> Result<LogViewMode> {
        let next_mode = self.mode.next();
        self.set_mode(next_mode.clone())?;
        Ok(next_mode)
    }

    fn reveal_change_id_with_loader(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<bool> {
        if self.select_change_id(change_id) {
            return Ok(false);
        }

        self.switch_mode_with_loader(fallback_mode, load)?;
        if self.select_change_id(change_id) {
            Ok(true)
        } else {
            Err(color_eyre::eyre::eyre!(
                "refreshed log did not include the new working-copy change"
            ))
        }
    }

    #[cfg(test)]
    pub fn test_reveal_change_id_with_loader(
        &mut self,
        change_id: &str,
        fallback_mode: LogViewMode,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<bool> {
        self.reveal_change_id_with_loader(change_id, fallback_mode, load)
    }

    fn switch_mode_with_loader(
        &mut self,
        mode: LogViewMode,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<()> {
        let previous_spec = self.spec.clone();
        let previous_mode = self.mode.clone();
        let previous_index = self.selection.index();
        let previous_change_id = self
            .entries
            .get(previous_index)
            .and_then(LogItem::action_id)
            .map(str::to_owned);
        let spec = ViewSpec::for_log_mode(self.home_command, &mode);
        let entries = match load(&spec) {
            Ok(entries) => entries,
            Err(error) => {
                self.spec = previous_spec;
                self.mode = previous_mode;
                return Err(error);
            }
        };

        self.spec = spec;
        self.mode = mode;
        self.entries = entries;
        retain_selected_change_ids(&mut self.selected_change_ids, &self.entries);
        restore_selection(
            &mut self.selection,
            &self.entries,
            previous_index,
            previous_change_id,
        );
        Ok(())
    }

    #[cfg(test)]
    pub fn test_switch_mode_with_loader(
        &mut self,
        mode: LogViewMode,
        load: impl Fn(&ViewSpec) -> Result<Vec<LogItem>>,
    ) -> Result<()> {
        self.switch_mode_with_loader(mode, load)
    }

    #[cfg(test)]
    pub fn test_selected_change_ids(&self) -> &[String] {
        &self.selected_change_ids
    }
}

fn restore_selection(
    selection: &mut crate::selection::Selection,
    entries: &[LogItem],
    previous_index: usize,
    previous_change_id: Option<String>,
) {
    if let Some(change_id) = previous_change_id
        && let Some(index) = entries
            .iter()
            .position(|entry| entry.action_id() == Some(change_id.as_str()))
    {
        selection.set(index, entries.len());
        return;
    }

    selection.set(previous_index, entries.len());
}

#[cfg(test)]
pub fn test_restore_selection(
    selection: &mut crate::selection::Selection,
    entries: &[LogItem],
    previous_index: usize,
    previous_change_id: Option<String>,
) {
    restore_selection(selection, entries, previous_index, previous_change_id);
}

fn retain_selected_change_ids(selected_change_ids: &mut Vec<String>, entries: &[LogItem]) {
    let retained = selected_change_ids
        .iter()
        .filter(|selected| {
            entries
                .iter()
                .any(|entry| entry.action_id() == Some(selected.as_str()))
        })
        .cloned()
        .collect();
    *selected_change_ids = retained;
}
