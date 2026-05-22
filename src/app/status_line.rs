//! App status-line state and per-screen count labels.
//!
//! The app mutates status after commands and view transitions, while this module owns how a
//! status line is constructed from the active view.

use crate::jj::JjCommand;
use crate::tui::StatusHints;
use crate::view_state::ViewState;

#[derive(Clone, Debug)]
pub struct StatusLine {
    /// App/view label shown at the start of the status line.
    title: String,
    /// User-facing status text for the current screen or recent action.
    message: String,
    /// Presentation kind used to choose normal versus error styling.
    kind: StatusKind,
    /// Key-hint projection copied from the active view at status construction time.
    hints: StatusHints,
}

impl StatusLine {
    /// Build the normal status line for the current view position or item count.
    pub(crate) fn ready(view: &ViewState) -> Self {
        let message = if let Some(item_count) = view.item_count() {
            item_count_message(view, item_count)
        } else {
            format!(
                "{}/{} lines",
                view.scroll_offset()
                    .saturating_add(1)
                    .min(view.document_line_count()),
                view.document_line_count()
            )
        };
        Self {
            title: view.spec().app_label(),
            message,
            kind: StatusKind::Ready,
            hints: view.status_hints(),
        }
    }

    /// Build an error status line for the current view.
    pub(crate) fn error(view: &ViewState, message: String) -> Self {
        Self {
            title: view.spec().app_label(),
            message,
            kind: StatusKind::Error,
            hints: view.status_hints(),
        }
    }

    /// Build a normal status line with an explicit message for the current view.
    pub(crate) fn with_message(view: &ViewState, message: impl Into<String>) -> Self {
        Self {
            title: view.spec().app_label(),
            message: message.into(),
            kind: StatusKind::Ready,
            hints: view.status_hints(),
        }
    }

    #[cfg(test)]
    pub(crate) fn test(
        title: impl Into<String>,
        message: impl Into<String>,
        kind: StatusKind,
        hints: StatusHints,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            kind,
            hints,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return the user-facing status text.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Return the presentation kind used by the renderer.
    pub fn kind(&self) -> &StatusKind {
        &self.kind
    }

    /// Return the key-hint projection copied from the active view.
    pub fn hints(&self) -> StatusHints {
        self.hints
    }
}

/// Format the graph/log item-count message, optionally including the current log mode label.
fn graph_status_message(item_count: usize, mode_label: Option<&str>) -> String {
    let base = format!("{item_count} items");
    match mode_label {
        Some(mode_label) => format!("{base} | {mode_label}"),
        None => base,
    }
}

/// Format the ready-status count line for the active view command.
fn item_count_message(view: &ViewState, item_count: usize) -> String {
    match view.command() {
        JjCommand::Resolve => format!("{item_count} conflicts"),
        JjCommand::FileList => format!("{item_count} files"),
        JjCommand::Bookmarks => format!("{item_count} bookmarks"),
        JjCommand::Workspaces => format!("{item_count} workspaces"),
        JjCommand::OperationLog => format!("{item_count} operations"),
        JjCommand::Default | JjCommand::Log => {
            graph_status_message(item_count, view.log_mode_label())
        }
        JjCommand::Show
        | JjCommand::Diff
        | JjCommand::Status
        | JjCommand::FileShow
        | JjCommand::OperationShow
        | JjCommand::OperationDiff => format!("{item_count} items"),
    }
}

#[derive(Clone, Debug)]
pub enum StatusKind {
    /// Normal informational status.
    Ready,
    /// Error status that should render with error emphasis.
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_status_message_includes_mode_label() {
        assert_eq!(
            graph_status_message(4, Some("trunk work")),
            "4 items | trunk work"
        );
        assert_eq!(graph_status_message(4, None), "4 items");
    }
}
