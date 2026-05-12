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

#[cfg(test)]
mod tests {
    use super::*;

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
}
