//! Scroll state and text projection for action preview/result overlays.
//!
//! The active action output is modal state, not view history. It keeps raw command output readable
//! while preserving the underlying view selection and search state.

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
}
