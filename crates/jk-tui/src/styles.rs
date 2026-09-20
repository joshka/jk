//! Shared roles for jk-owned controls; repository output retains the styles supplied by jj.

use std::sync::OnceLock;

use ratatui::prelude::{Color, Modifier, Style};

/// Leaves repository content and the surrounding canvas in the terminal's own palette.
pub const CANVAS: Style = Style::new().fg(Color::Reset).bg(Color::Reset);
/// Compatibility name for terminal-default canvas chrome; floating dialogs use [`dialog`].
pub const SURFACE: Style = CANVAS;
/// Emphasizes a short surface title without a filled badge.
pub const TITLE: Style = Style::new().add_modifier(Modifier::BOLD);
/// Emphasizes active controls while inheriting the surface's readable foreground.
///
/// Cursor symbols, marks, and control labels supply the distinction without depending on an ANSI
/// accent color that may have poor contrast in a light or customized terminal palette.
pub const FOCUS: Style = Style::new().add_modifier(Modifier::BOLD);
/// Keeps contextual text readable in the containing surface's palette.
pub const SUPPORTING: Style = Style::new();
/// Adds semantic emphasis to an explicit warning on the terminal canvas.
pub const WARNING: Style = Style::new().fg(Color::Yellow);

/// The coordinated palette for opaque jk-owned dialogs, independent of jj's output colors.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DialogTheme {
    /// Light text on a restrained dark surface; also the safe fallback when detection is
    /// unavailable.
    #[default]
    Dark,
    /// Dark text on a pale surface for a light terminal background.
    Light,
}

static DIALOG_THEME: OnceLock<DialogTheme> = OnceLock::new();

/// Chooses the dialog palette once, before the application starts reading terminal input.
///
/// Later calls leave the original choice intact. Rendering without initialization uses the dark
/// palette so unit tests and terminals without color-query support remain deterministic.
pub fn initialize_dialog_theme(theme: DialogTheme) {
    let _ = DIALOG_THEME.set(theme);
}

/// Returns the paired foreground and opaque background for a floating dialog.
///
/// Paint the whole dialog area with this style first. Child controls inherit its foreground and
/// background unless they deliberately supply another paired surface, such as a danger button.
#[must_use]
pub fn dialog() -> Style {
    dialog_style(dialog_theme())
}

/// Returns a warning foreground with readable contrast on the active dialog background.
#[must_use]
pub fn dialog_warning() -> Style {
    warning_style(dialog_theme())
}

/// Returns an error foreground with readable contrast on the active dialog background.
#[must_use]
pub fn dialog_error() -> Style {
    let foreground = match dialog_theme() {
        DialogTheme::Dark => Color::Rgb(255, 135, 130),
        DialogTheme::Light => Color::Rgb(150, 35, 30),
    };
    Style::new().fg(foreground)
}

/// Highlights short dialog titles, keyboard keys, and focused markers without filling their row.
#[must_use]
pub fn dialog_accent() -> Style {
    let foreground = match dialog_theme() {
        DialogTheme::Dark => Color::Rgb(101, 212, 207),
        DialogTheme::Light => Color::Rgb(0, 103, 103),
    };
    Style::new().fg(foreground).add_modifier(Modifier::BOLD)
}

/// Gives supporting dialog text lower emphasis while retaining readable contrast.
#[must_use]
pub fn dialog_supporting() -> Style {
    let foreground = match dialog_theme() {
        DialogTheme::Dark => Color::Rgb(161, 174, 190),
        DialogTheme::Light => Color::Rgb(82, 97, 114),
    };
    Style::new().fg(foreground)
}

/// Supplies a paired neutral surface for an actual dialog action region.
#[must_use]
pub fn dialog_action() -> Style {
    match dialog_theme() {
        DialogTheme::Dark => Style::new()
            .fg(Color::Rgb(224, 230, 238))
            .bg(Color::Rgb(58, 72, 90)),
        DialogTheme::Light => Style::new()
            .fg(Color::Rgb(28, 35, 45))
            .bg(Color::Rgb(198, 209, 222)),
    }
}

/// Supplies a paired accent surface for a focused ordinary action or a confirmation control.
#[must_use]
pub fn dialog_confirm() -> Style {
    match dialog_theme() {
        DialogTheme::Dark => Style::new()
            .fg(Color::Rgb(15, 29, 35))
            .bg(Color::Rgb(82, 196, 192)),
        DialogTheme::Light => Style::new()
            .fg(Color::Rgb(255, 255, 255))
            .bg(Color::Rgb(0, 105, 104)),
    }
}

/// Keeps destructive action regions burgundy, including while they are focused.
///
/// Add an explicit marker and bold text for focus so the consequence retains its visual identity.
#[must_use]
pub fn dialog_danger() -> Style {
    match dialog_theme() {
        DialogTheme::Dark => Style::new()
            .fg(Color::Rgb(255, 235, 235))
            .bg(Color::Rgb(120, 45, 50)),
        DialogTheme::Light => Style::new()
            .fg(Color::Rgb(255, 255, 255))
            .bg(Color::Rgb(151, 44, 51)),
    }
}

fn dialog_theme() -> DialogTheme {
    DIALOG_THEME.get().copied().unwrap_or_default()
}

const fn dialog_style(theme: DialogTheme) -> Style {
    match theme {
        DialogTheme::Dark => Style::new()
            .fg(Color::Rgb(224, 230, 238))
            .bg(Color::Rgb(36, 44, 58)),
        DialogTheme::Light => Style::new()
            .fg(Color::Rgb(28, 35, 45))
            .bg(Color::Rgb(225, 230, 237)),
    }
}

const fn warning_style(theme: DialogTheme) -> Style {
    let foreground = match theme {
        DialogTheme::Dark => Color::Rgb(234, 184, 85),
        DialogTheme::Light => Color::Rgb(116, 71, 0),
    };
    Style::new().fg(foreground)
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::layout::Rect;
    use ratatui::prelude::{Line, Span};
    use ratatui::widgets::{Block, Paragraph};

    use super::*;

    #[test]
    fn both_dialog_palettes_are_opaque_and_preserve_surrounding_repository_cells() {
        for theme in [DialogTheme::Dark, DialogTheme::Light] {
            let style = dialog_style(theme);
            let mut terminal = Terminal::new(TestBackend::new(12, 6)).expect("terminal");
            terminal
                .draw(|frame| {
                    frame.render_widget(Paragraph::new("jj graph").style(CANVAS), frame.area());
                    frame.render_widget(Block::default().style(style), Rect::new(2, 2, 8, 3));
                    frame.render_widget(
                        Paragraph::new(Line::from(vec![Span::styled("› Run", FOCUS)])),
                        Rect::new(3, 3, 6, 1),
                    );
                })
                .expect("draw");
            let buffer = terminal.backend().buffer();
            assert_eq!(buffer[(0, 0)].symbol(), "j");
            assert_eq!(buffer[(0, 0)].bg, Color::Reset);
            assert_eq!(buffer[(2, 2)].symbol(), " ");
            assert_eq!(Some(buffer[(2, 2)].bg), style.bg);
            assert_eq!(Some(buffer[(3, 3)].fg), style.fg);
            assert!(buffer[(3, 3)].modifier.contains(Modifier::BOLD));
            assert_eq!(buffer[(10, 3)].bg, Color::Reset);
            assert_ne!(style.fg, style.bg);
            assert_ne!(warning_style(theme).fg, style.bg);
        }
    }
}
