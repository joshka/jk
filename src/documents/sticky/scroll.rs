use crate::documents::PinnedDocument;

use super::render::line_text;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct StickyScroll {
    /// Current vertical offset into the rendered document.
    offset: usize,
}

impl StickyScroll {
    pub(super) fn offset(self) -> usize {
        self.offset
    }

    pub(super) fn set(&mut self, offset: usize, max_offset: usize) {
        self.offset = offset.min(max_offset);
    }

    pub(super) fn move_to_top(&mut self) {
        self.offset = 0;
    }

    pub(super) fn move_to_bottom(
        &mut self,
        max_offset: usize,
        viewport_height: u16,
        project: impl Fn(usize) -> PinnedDocument,
    ) {
        self.offset = previous_meaningful_offset(max_offset, viewport_height, project);
    }

    pub(super) fn down(
        &mut self,
        amount: usize,
        max_offset: usize,
        viewport_height: u16,
        project: impl Fn(usize) -> PinnedDocument,
    ) {
        for _ in 0..amount {
            self.offset =
                next_meaningful_offset(self.offset, max_offset, viewport_height, &project);
        }
        self.clamp(max_offset);
    }

    pub(super) fn up(
        &mut self,
        amount: usize,
        viewport_height: u16,
        project: impl Fn(usize) -> PinnedDocument,
    ) {
        for _ in 0..amount {
            self.offset = previous_meaningful_offset(self.offset, viewport_height, &project);
        }
    }

    pub(super) fn clamp(&mut self, max_offset: usize) {
        self.offset = self.offset.min(max_offset);
    }
}

pub(super) fn next_meaningful_offset(
    current_offset: usize,
    max_offset: usize,
    viewport_height: u16,
    project: impl Fn(usize) -> PinnedDocument,
) -> usize {
    // Skip offsets that render the same visible projection. Otherwise a key can
    // mutate hidden scroll state while the terminal appears unchanged.
    let current_key = projection_key(&project(current_offset), viewport_height);
    ((current_offset + 1)..=max_offset)
        .find(|offset| projection_key(&project(*offset), viewport_height) != current_key)
        .unwrap_or(max_offset)
}

pub(super) fn previous_meaningful_offset(
    current_offset: usize,
    viewport_height: u16,
    project: impl Fn(usize) -> PinnedDocument,
) -> usize {
    let current_key = projection_key(&project(current_offset), viewport_height);
    (0..current_offset)
        .rev()
        .find(|offset| projection_key(&project(*offset), viewport_height) != current_key)
        .unwrap_or(0)
}

fn projection_key(document: &PinnedDocument, viewport_height: u16) -> Vec<String> {
    let body_height = viewport_height.saturating_sub(document.sticky_height()) as usize;
    document
        .fixed_lines()
        .iter()
        .chain(
            document
                .body_lines()
                .iter()
                .skip(document.body_scroll_offset())
                .take(body_height),
        )
        .map(line_text)
        .collect()
}
