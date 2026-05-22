use ratatui::text::Line;

/// Display policy for rendered jj document text.
///
/// Wrapped mode preserves the original behavior: Ratatui wraps long lines with
/// `trim: false`, keeping indentation and blank lines visible. No-wrap mode
/// leaves the spans intact, clips at the viewport edge, and uses a horizontal
/// offset owned by the document view state.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DocumentDisplayMode {
    #[default]
    Wrap,
    NoWrap,
}

/// Viewport state for rendered document text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentViewport {
    /// Current wrap-versus-no-wrap display policy.
    display_mode: DocumentDisplayMode,
    /// Current horizontal offset used only in no-wrap mode.
    horizontal_offset: usize,
}

impl DocumentViewport {
    /// Return the current display mode.
    pub fn display_mode(self) -> DocumentDisplayMode {
        self.display_mode
    }

    /// Return the current horizontal offset, or zero in wrapped mode.
    pub fn horizontal_offset(self) -> usize {
        match self.display_mode {
            DocumentDisplayMode::Wrap => 0,
            DocumentDisplayMode::NoWrap => self.horizontal_offset,
        }
    }

    /// Toggle wrapped versus no-wrap mode, resetting horizontal scroll when wrapping.
    pub fn toggle_wrap(&mut self) {
        self.display_mode = match self.display_mode {
            DocumentDisplayMode::Wrap => DocumentDisplayMode::NoWrap,
            DocumentDisplayMode::NoWrap => {
                self.horizontal_offset = 0;
                DocumentDisplayMode::Wrap
            }
        };
    }

    /// Scroll horizontally left in no-wrap mode.
    pub fn scroll_left(&mut self, amount: usize) {
        if self.display_mode == DocumentDisplayMode::NoWrap {
            self.horizontal_offset = self.horizontal_offset.saturating_sub(amount);
        }
    }

    /// Scroll horizontally right in no-wrap mode within content bounds.
    pub fn scroll_right(&mut self, viewport_width: u16, amount: usize, max_line_width: usize) {
        if self.display_mode == DocumentDisplayMode::NoWrap {
            self.horizontal_offset = self
                .horizontal_offset
                .saturating_add(amount)
                .min(max_horizontal_offset(viewport_width, max_line_width));
        }
    }

    /// Clamp horizontal offset to the current viewport and content width.
    pub fn clamp(&mut self, viewport_width: u16, max_line_width: usize) {
        match self.display_mode {
            DocumentDisplayMode::Wrap => self.horizontal_offset = 0,
            DocumentDisplayMode::NoWrap => {
                self.horizontal_offset = self
                    .horizontal_offset
                    .min(max_horizontal_offset(viewport_width, max_line_width));
            }
        }
    }
}

pub(super) fn max_projected_line_width(fixed_lines: &[Line<'_>], body_lines: &[Line<'_>]) -> usize {
    fixed_lines
        .iter()
        .chain(body_lines)
        .map(|line| line.width())
        .max()
        .unwrap_or_default()
}

fn max_horizontal_offset(viewport_width: u16, max_line_width: usize) -> usize {
    max_line_width.saturating_sub(usize::from(viewport_width))
}
