//! Reusable row selection state.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Selection {
    index: usize,
}

impl Selection {
    pub fn index(self) -> usize {
        self.index
    }

    pub fn first(&mut self) {
        self.index = 0;
    }

    pub fn next(&mut self, len: usize) {
        if self.index + 1 < len {
            self.index += 1;
        }
    }

    pub fn previous(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    pub fn last(&mut self, len: usize) {
        self.index = len.saturating_sub(1);
    }

    pub fn set(&mut self, index: usize, len: usize) {
        self.index = index.min(len.saturating_sub(1));
    }

    pub fn clamp(&mut self, len: usize) {
        self.set(self.index, len);
    }
}

/// Restores selection to the first row matching the previous stable key.
///
/// If there is no previous key or it is no longer present, preserves and clamps the previous index.
/// The caller owns key capture and any view-specific action policy.
pub fn restore_by_key_or_index<T, K: ?Sized + PartialEq>(
    selection: &mut Selection,
    rows: &[T],
    previous_index: usize,
    previous_key: Option<&K>,
    row_key: impl Fn(&T) -> Option<&K>,
) {
    if let Some(key) = previous_key
        && let Some(index) = rows.iter().position(|row| row_key(row) == Some(key))
    {
        selection.set(index, rows.len());
        return;
    }

    selection.set(previous_index, rows.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Row {
        key: Option<&'static str>,
    }

    #[test]
    fn selection_stays_at_zero_for_empty_content() {
        let mut selection = Selection::default();

        selection.next(0);
        assert_eq!(selection.index(), 0);

        selection.last(0);
        assert_eq!(selection.index(), 0);

        selection.set(10, 0);
        assert_eq!(selection.index(), 0);
    }

    #[test]
    fn selection_clamps_to_last_available_item() {
        let mut selection = Selection::default();

        selection.set(10, 3);
        assert_eq!(selection.index(), 2);

        selection.clamp(2);
        assert_eq!(selection.index(), 1);
    }

    #[test]
    fn restore_by_key_or_index_preserves_matching_key() {
        let mut selection = Selection::default();
        selection.set(2, 3);
        let rows = [
            Row { key: Some("gamma") },
            Row { key: Some("beta") },
            Row { key: Some("delta") },
        ];

        restore_by_key_or_index(&mut selection, &rows, 2, Some("beta"), |row| row.key);

        assert_eq!(selection.index(), 1);
    }

    #[test]
    fn restore_by_key_or_index_preserves_index_when_key_disappears() {
        let mut selection = Selection::default();
        let rows = [
            Row { key: Some("alpha") },
            Row { key: Some("beta") },
            Row { key: Some("gamma") },
        ];

        restore_by_key_or_index(&mut selection, &rows, 2, Some("missing"), |row| row.key);

        assert_eq!(selection.index(), 2);
    }

    #[test]
    fn restore_by_key_or_index_clamps_when_key_disappears() {
        let mut selection = Selection::default();
        let rows = [Row { key: Some("alpha") }];

        restore_by_key_or_index(&mut selection, &rows, 3, Some("missing"), |row| row.key);

        assert_eq!(selection.index(), 0);
    }

    #[test]
    fn restore_by_key_or_index_clamps_empty_rows_to_zero() {
        let mut selection = Selection::default();
        let rows: [Row; 0] = [];

        restore_by_key_or_index(&mut selection, &rows, 3, Some("missing"), |row| row.key);

        assert_eq!(selection.index(), 0);
    }
}
