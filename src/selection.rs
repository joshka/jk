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
