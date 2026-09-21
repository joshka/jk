//! Ratatui views and interaction state for `jk`.
//!
//! [`log_view`] and [`diff_view`] retain jj-rendered content with separate cursor and mark
//! indicators. Other views present workspaces, bookmarks, operations, command history, and command
//! previews. Views consume caller-provided snapshots; the calling application executes commands and
//! supplies refreshed data.
//!
//! [`command_discovery`] supplies contextual action/help metadata and formatting. [`styles`]
//! defines the shared canvas, dialog, focus, hint, and action roles. These roles style jk-owned
//! controls while preserving the colors supplied by jj in rendered content.

pub mod bookmark_view;
pub mod command_history_view;
pub mod command_preview_view;
pub mod diff_view;
pub mod log_view;
pub mod operation_log_view;
pub mod rendered_view;
pub mod workspaces_view;

mod ansi_text;
mod chrome;
mod diff_state;
mod keymap;
mod log_state;
mod rendered_log;
mod rendered_state;
mod selected_row;
pub mod styles;

pub use selected_row::content_width;

/// Contextual command-help metadata and popup formatting.
pub mod command_discovery {
    pub use crate::chrome::overlay_line;
    pub use crate::keymap::{
        ActionMenuAction, ActionMenuGroup, ActionMenuRow, ActionMenuSafety, BindingContext,
        CommandFamily, DiscoveryRow, action_menu_rows, discovery_len, discovery_lines,
        discovery_lines_for_width, discovery_lines_for_width_and_rows, discovery_rows,
        discovery_scroll_limit,
    };
}
