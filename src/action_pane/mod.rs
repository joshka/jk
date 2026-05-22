//! Scroll state and text projection for action preview/result overlays.
//!
//! The active action output is modal state, not view history. It keeps raw command output readable
//! while preserving the underlying view selection and search state.

use crossterm::event::KeyCode;

#[derive(Clone, Debug)]
pub(crate) struct ActionPane {
    /// User-facing label for the command whose preview or result is shown.
    command_label: String,
    /// Raw command output rendered in the overlay body.
    output: String,
    /// Optional context line shown above the raw output.
    status_context: Option<String>,
    /// Whether the command has already finished running.
    completed: bool,
    /// Current vertical scroll offset into the rendered body lines.
    scroll: usize,
}

impl ActionPane {
    /// Build a preview/result pane whose command is still running or awaiting confirmation.
    pub(crate) fn pending(
        command_label: String,
        output: String,
        status_context: Option<String>,
    ) -> Self {
        Self {
            command_label,
            output,
            status_context,
            completed: false,
            scroll: 0,
        }
    }

    /// Build a preview/result pane for a completed command.
    pub(crate) fn finished(
        command_label: String,
        output: String,
        status_context: Option<String>,
    ) -> Self {
        Self {
            command_label,
            output,
            status_context,
            completed: true,
            scroll: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn command_label(&self) -> &str {
        &self.command_label
    }

    /// Return the optional context line shown above the output body.
    pub(crate) fn status_context(&self) -> Option<&String> {
        self.status_context.as_ref()
    }

    /// Return whether the represented command has completed.
    pub(crate) fn completed(&self) -> bool {
        self.completed
    }

    /// Return the current vertical scroll offset.
    pub(crate) fn scroll(&self) -> usize {
        self.scroll
    }

    /// Render the current pane content into the body lines shown by the overlay.
    pub(crate) fn body_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("command: {}", self.command_label)];
        if let Some(context) = &self.status_context {
            lines.push(format!("context: {context}"));
        }

        if self.output.is_empty() {
            lines.push("output unavailable".to_owned());
        } else {
            lines.push("output:".to_owned());
            lines.extend(self.output.lines().map(|line| format!("  {line}")));
        }

        lines
    }

    /// Scroll down by one line without moving past the last visible body line.
    pub(crate) fn scroll_down(&mut self, visible_lines: u16) {
        let max_scroll = self.max_scroll(visible_lines);
        self.scroll = (self.scroll + 1).min(max_scroll);
    }

    /// Scroll up by one line without moving above the first body line.
    pub(crate) fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    /// Scroll down by one page using the current visible line count.
    pub(crate) fn page_down(&mut self, visible_lines: u16) {
        let max_scroll = self.max_scroll(visible_lines);
        self.scroll = (self.scroll + usize::from(visible_lines).max(1)).min(max_scroll);
    }

    /// Scroll up by one page using the current visible line count.
    pub(crate) fn page_up(&mut self, visible_lines: u16) {
        self.scroll = self
            .scroll
            .saturating_sub(usize::from(visible_lines).max(1));
    }

    /// Jump to the first body line.
    pub(crate) fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    /// Jump to the last scroll position that still fills the viewport.
    pub(crate) fn scroll_to_bottom(&mut self, visible_lines: u16) {
        self.scroll = self.max_scroll(visible_lines);
    }

    /// Return the maximum vertical scroll offset for the current body and viewport size.
    pub(crate) fn max_scroll(&self, visible_lines: u16) -> usize {
        self.body_lines()
            .len()
            .saturating_sub(usize::from(visible_lines))
    }
}

/// Return the number of body lines available beneath the pane header/status row.
pub(crate) fn action_pane_visible_lines(viewport_height: u16) -> u16 {
    viewport_height.saturating_sub(1).max(1)
}

/// Reduced key outcome for an action preview or result overlay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActionPaneKey {
    /// The primary accept key was pressed.
    Primary,
    /// The cancel/close key was pressed.
    Cancel,
    /// The key changed pane-local scroll state.
    Handled,
    /// The key does not belong to the action pane.
    Ignored,
}

/// Route one key through action-pane scrolling and accept/cancel handling.
pub(crate) fn handle_action_pane_key(
    code: KeyCode,
    output: &mut ActionPane,
    visible_lines: u16,
) -> ActionPaneKey {
    match code {
        KeyCode::Enter => ActionPaneKey::Primary,
        KeyCode::Esc | KeyCode::Char('q') => ActionPaneKey::Cancel,
        KeyCode::Char('j') | KeyCode::Down => {
            output.scroll_down(visible_lines);
            ActionPaneKey::Handled
        }
        KeyCode::Char('k') | KeyCode::Up => {
            output.scroll_up();
            ActionPaneKey::Handled
        }
        KeyCode::Char(' ') | KeyCode::PageDown => {
            output.page_down(visible_lines);
            ActionPaneKey::Handled
        }
        KeyCode::Char('b') | KeyCode::PageUp => {
            output.page_up(visible_lines);
            ActionPaneKey::Handled
        }
        KeyCode::Char('g') | KeyCode::Home => {
            output.scroll_to_top();
            ActionPaneKey::Handled
        }
        KeyCode::Char('G') | KeyCode::End => {
            output.scroll_to_bottom(visible_lines);
            ActionPaneKey::Handled
        }
        _ => ActionPaneKey::Ignored,
    }
}

#[cfg(test)]
mod tests;
