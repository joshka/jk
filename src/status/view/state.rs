use color_eyre::Result;
#[cfg(test)]
use ratatui::text::Line;

use crate::jj::ViewSpec;
#[cfg(test)]
use crate::selection::Selection;

use super::StatusView;
#[cfg(test)]
use crate::jj::JjCommand;
#[cfg(test)]
use crate::status::rows::parse_status_row;
use crate::status::rows::{StatusRow, load_status_rows};

impl StatusView {
    /// Load the status view and derive exact-path action contracts from rendered output.
    pub fn load(spec: ViewSpec) -> Result<Self> {
        Ok(Self {
            rows: load_status_rows(&spec)?,
            spec,
            selection: crate::selection::Selection::default(),
        })
    }

    /// Reload rendered status rows while preserving selection when possible.
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_loader(load_status_rows)?;
        Ok(())
    }

    #[cfg(test)]
    pub fn scroll_to_bottom(&mut self, _viewport_height: u16) {
        self.selection.last(self.rows.len());
    }

    #[cfg(test)]
    pub fn scroll_down(&mut self, _viewport_height: u16, amount: usize) {
        for _ in 0..amount {
            self.selection.next(self.rows.len());
        }
    }

    /// Reload rows with a caller-supplied loader while restoring the best previous selection.
    pub(super) fn refresh_with_loader(
        &mut self,
        load: impl Fn(&ViewSpec) -> Result<Vec<StatusRow>>,
    ) -> Result<()> {
        let previous_index = self.selection.index();
        let previous_path = self
            .rows
            .get(previous_index)
            .and_then(StatusRow::exact_path_option)
            .map(str::to_owned);
        let previous_text = self.rows.get(previous_index).map(StatusRow::row_text);

        self.rows = load(&self.spec)?;
        restore_selection(
            &mut self.selection,
            &self.rows,
            previous_index,
            previous_path,
            previous_text,
        );
        Ok(())
    }
}

/// Restore selection after refresh by exact path first, then row text, then prior index.
fn restore_selection(
    selection: &mut crate::selection::Selection,
    rows: &[StatusRow],
    previous_index: usize,
    previous_path: Option<String>,
    previous_text: Option<String>,
) {
    if let Some(path) = previous_path
        && let Some(index) = rows
            .iter()
            .position(|row| row.exact_path_option() == Some(path.as_str()))
    {
        selection.set(index, rows.len());
        return;
    }

    if let Some(text) = previous_text
        && let Some(index) = rows.iter().position(|row| row.row_text() == text)
    {
        selection.set(index, rows.len());
        return;
    }

    selection.set(previous_index, rows.len());
}

#[cfg(test)]
impl StatusView {
    pub(crate) fn test_new(lines: &[&str]) -> Self {
        Self {
            spec: ViewSpec::new(JjCommand::Status, Vec::new()),
            rows: lines
                .iter()
                .map(|line| parse_status_row(Line::from((*line).to_owned())))
                .collect(),
            selection: Selection::default(),
        }
    }
}
