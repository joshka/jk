//! Shared terminal style fallbacks for app-owned UI surfaces.
//!
//! Rendered `jj` output keeps its original styles. These helpers cover app chrome, selected rows,
//! and popovers so low-color terminals still show the interaction state through modifiers instead
//! of color alone.

use ratatui::style::{Color, Modifier, Style};

/// Returns the style for the actively selected row.
pub fn active_row_style() -> Style {
    Style::default()
        .bg(Color::Rgb(48, 52, 60))
        .add_modifier(Modifier::BOLD)
}

/// Returns the style for marked rows that are selected for preview actions.
pub fn marked_row_style() -> Style {
    Style::default()
        .bg(Color::Rgb(32, 47, 48))
        .add_modifier(Modifier::BOLD)
}

/// Returns the background style for overlays and popovers.
pub fn overlay_background_style() -> Style {
    Style::default().bg(Color::Rgb(18, 20, 24))
}

/// Returns the border style for overlays and popovers.
pub fn overlay_border_style() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Returns the title style for overlays and popovers.
pub fn overlay_title_style() -> Style {
    Style::default()
        .fg(Color::LightCyan)
        .add_modifier(Modifier::BOLD)
}

/// Returns the muted text style for secondary chrome.
pub fn muted_style() -> Style {
    Style::default().fg(Color::Gray)
}

/// Returns the emphasized key-label style used in help and popovers.
pub fn key_style() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_row_style_has_non_color_fallback() {
        let style = active_row_style();

        assert_eq!(style.fg, None);
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn marked_row_style_preserves_foreground() {
        let style = marked_row_style();

        assert_eq!(style.fg, None);
        assert!(style.bg.is_some());
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }
}
