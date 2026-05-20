//! Scroll state and text projection for action preview/result overlays.
//!
//! The active action output is modal state, not view history. It keeps raw command output readable
//! while preserving the underlying view selection and search state.

use crossterm::event::KeyCode;

#[derive(Clone, Debug)]
pub(crate) struct ActionOutput {
    command_label: String,
    output: String,
    status_context: Option<String>,
    completed: bool,
    scroll: usize,
}

impl ActionOutput {
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

    pub(crate) fn status_context(&self) -> Option<&String> {
        self.status_context.as_ref()
    }

    pub(crate) fn completed(&self) -> bool {
        self.completed
    }

    pub(crate) fn scroll(&self) -> usize {
        self.scroll
    }

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

    pub(crate) fn scroll_down(&mut self, visible_lines: u16) {
        let max_scroll = self.max_scroll(visible_lines);
        self.scroll = (self.scroll + 1).min(max_scroll);
    }

    pub(crate) fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub(crate) fn page_down(&mut self, visible_lines: u16) {
        let max_scroll = self.max_scroll(visible_lines);
        self.scroll = (self.scroll + usize::from(visible_lines).max(1)).min(max_scroll);
    }

    pub(crate) fn page_up(&mut self, visible_lines: u16) {
        self.scroll = self
            .scroll
            .saturating_sub(usize::from(visible_lines).max(1));
    }

    pub(crate) fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub(crate) fn scroll_to_bottom(&mut self, visible_lines: u16) {
        self.scroll = self.max_scroll(visible_lines);
    }

    pub(crate) fn max_scroll(&self, visible_lines: u16) -> usize {
        self.body_lines()
            .len()
            .saturating_sub(usize::from(visible_lines))
    }
}

pub(crate) fn action_output_visible_lines(viewport_height: u16) -> u16 {
    viewport_height.saturating_sub(1).max(1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActionOutputKey {
    Primary,
    Cancel,
    Handled,
    Ignored,
}

pub(crate) fn handle_action_output_key(
    code: KeyCode,
    output: &mut ActionOutput,
    visible_lines: u16,
) -> ActionOutputKey {
    match code {
        KeyCode::Enter => ActionOutputKey::Primary,
        KeyCode::Esc | KeyCode::Char('q') => ActionOutputKey::Cancel,
        KeyCode::Char('j') | KeyCode::Down => {
            output.scroll_down(visible_lines);
            ActionOutputKey::Handled
        }
        KeyCode::Char('k') | KeyCode::Up => {
            output.scroll_up();
            ActionOutputKey::Handled
        }
        KeyCode::Char(' ') | KeyCode::PageDown => {
            output.page_down(visible_lines);
            ActionOutputKey::Handled
        }
        KeyCode::Char('b') | KeyCode::PageUp => {
            output.page_up(visible_lines);
            ActionOutputKey::Handled
        }
        KeyCode::Char('g') | KeyCode::Home => {
            output.scroll_to_top();
            ActionOutputKey::Handled
        }
        KeyCode::Char('G') | KeyCode::End => {
            output.scroll_to_bottom(visible_lines);
            ActionOutputKey::Handled
        }
        _ => ActionOutputKey::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_with_lines(count: usize) -> ActionOutput {
        ActionOutput::pending(
            "jj action --preview".to_owned(),
            (0..count)
                .map(|line| format!("line {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            Some("context".to_owned()),
        )
    }

    #[test]
    fn scroll_clamps_to_readable_boundaries() {
        let mut output = output_with_lines(8);

        output.page_down(4);
        output.page_down(4);
        output.page_down(4);

        assert_eq!(output.scroll(), output.max_scroll(4));

        output.page_up(4);
        output.page_up(4);
        output.page_up(4);

        assert_eq!(output.scroll(), 0);
    }

    #[test]
    fn body_lines_keep_command_context_and_multiline_output() {
        let output = ActionOutput::pending(
            "jj git push --preview".to_owned(),
            "first\n\nthird".to_owned(),
            Some("status push uses jj default target".to_owned()),
        );

        assert_eq!(
            output.body_lines(),
            [
                "command: jj git push --preview",
                "context: status push uses jj default target",
                "output:",
                "  first",
                "  ",
                "  third",
            ]
        );
    }

    #[test]
    fn key_handling_maps_preview_commands_and_scrolls_output() {
        let mut output = output_with_lines(8);

        assert_eq!(
            handle_action_output_key(KeyCode::PageDown, &mut output, 4),
            ActionOutputKey::Handled
        );
        assert_eq!(output.scroll(), 4);

        assert_eq!(
            handle_action_output_key(KeyCode::Enter, &mut output, 4),
            ActionOutputKey::Primary
        );
        assert_eq!(
            handle_action_output_key(KeyCode::Esc, &mut output, 4),
            ActionOutputKey::Cancel
        );
        assert_eq!(
            handle_action_output_key(KeyCode::Char('x'), &mut output, 4),
            ActionOutputKey::Ignored
        );
    }

    #[test]
    fn visible_lines_never_drop_below_one() {
        assert_eq!(action_output_visible_lines(0), 1);
        assert_eq!(action_output_visible_lines(1), 1);
        assert_eq!(action_output_visible_lines(5), 4);
    }
}
