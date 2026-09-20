//! Shared roles for jk-owned controls; repository output retains the styles supplied by jj.

use ratatui::prelude::{Color, Modifier, Style};

/// Clears jk-owned surfaces using the terminal foreground and background.
pub const SURFACE: Style = Style::new().fg(Color::Reset).bg(Color::Reset);
/// Emphasizes a short surface title without a filled badge.
pub const TITLE: Style = Style::new().add_modifier(Modifier::BOLD);
/// Identifies the one active control or cursor.
pub const FOCUS: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
/// Keeps contextual text readable in the terminal palette.
pub const SUPPORTING: Style = Style::new();
/// Adds semantic emphasis to an explicit warning.
pub const WARNING: Style = Style::new().fg(Color::Yellow);
