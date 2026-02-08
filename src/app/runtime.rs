//! Runtime loop, rendering, and state transitions.

use std::time::Duration;

use crate::error::JkError;
use crate::flow::FlowAction;
use crate::jj;
use ansi_to_tui::IntoText as _;
use crossterm::event::{self, Event};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

use super::preview::confirmation_preview_tokens;
#[cfg(test)]
use super::selection::trim_to_width;
use super::selection::{
    derive_row_revision_map, extract_revision, looks_like_graph_commit_row, startup_action,
    strip_ansi,
};
use super::terminal::{AppTerminal, TerminalSession};
use super::view::decorate_command_output;
use super::{App, Mode};

impl App {
    /// Enter terminal session, run startup action, then drive draw/input loop until quit.
    pub fn run(&mut self, startup_tokens: Vec<String>) -> Result<(), JkError> {
        let mut terminal = TerminalSession::enter()?;
        self.apply_startup_tokens(startup_tokens)?;

        while !self.should_quit {
            self.draw(terminal.terminal_mut())?;

            if event::poll(Duration::from_millis(120))?
                && let Event::Key(key) = event::read()?
            {
                self.handle_key(key)?;
            }
        }

        Ok(())
    }

    /// Execute startup command tokens or default startup flow.
    ///
    /// Local view commands are handled without invoking `jj`.
    pub(super) fn apply_startup_tokens(
        &mut self,
        startup_tokens: Vec<String>,
    ) -> Result<(), JkError> {
        if !startup_tokens.is_empty() {
            let startup_command = startup_tokens.join(" ");
            if let Some(action) = self.local_view_action(&startup_command) {
                let intent = startup_command
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(" ");
                self.record_intent(&intent);
                return self.apply_flow_action(action);
            }
        }

        let action = startup_action(&startup_tokens);
        if !startup_tokens.is_empty() {
            let intent = startup_tokens
                .iter()
                .take(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" ");
            self.record_intent(&intent);
        }
        self.apply_flow_action(action)
    }

    /// Apply a planned action and enforce mode/state transition invariants.
    ///
    /// Render actions reset cursor/scroll and clear row metadata so selection cannot reference stale
    /// rows. Prompt actions enter prompt mode without side effects. Execute actions are routed
    /// through confirmation policy before subprocess execution.
    pub(super) fn apply_flow_action(&mut self, action: FlowAction) -> Result<(), JkError> {
        match action {
            FlowAction::Quit => {
                self.should_quit = true;
                Ok(())
            }
            FlowAction::Render { lines, status } => {
                self.lines = lines;
                self.row_revision_map = vec![None; self.lines.len()];
                self.cursor = 0;
                self.scroll = 0;
                self.status_line = status;
                Ok(())
            }
            FlowAction::Status(message) => {
                self.status_line = message;
                Ok(())
            }
            FlowAction::Execute(tokens) => self.execute_with_confirmation(tokens),
            FlowAction::Prompt(request) => {
                self.start_prompt(request);
                Ok(())
            }
        }
    }

    /// Execute a concrete `jj` command and replace rendered output state.
    ///
    /// Successful `log` runs refresh `last_log_tokens` so patch toggle can re-run with the same
    /// base arguments. This path also rebuilds row-revision metadata so selection-based shortcuts
    /// stay aligned with decorated output.
    pub(super) fn execute_tokens(&mut self, tokens: Vec<String>) -> Result<(), JkError> {
        let result = jj::run(&tokens)?;
        if Self::is_navigable_view_tokens(&result.command) {
            self.record_view_visit(&result.command.join(" "));
        }
        self.record_intent_from_tokens(&result.command);
        self.update_onboarding_progress(&result.command);
        if matches!(result.command.first().map(String::as_str), Some("log")) {
            self.last_log_tokens = result.command.clone();
        }
        self.row_revision_map = derive_row_revision_map(&result.command, &result.output);
        self.lines = decorate_command_output(&result.command, result.output);
        self.cursor = 0;
        self.scroll = 0;
        self.last_command = result.command;
        self.status_line = if result.success {
            format!("ok: jj {}", self.last_command.join(" "))
        } else {
            format!("error: jj {}", self.last_command.join(" "))
        };
        Ok(())
    }

    /// Move selection cursor up and keep viewport aligned.
    pub(super) fn move_cursor_up(&mut self) {
        if self.has_item_navigation() {
            let _ = self.move_cursor_to_previous_item();
        } else if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.ensure_cursor_visible(self.viewport_rows.max(1));
    }

    /// Move selection cursor down and keep viewport aligned.
    pub(super) fn move_cursor_down(&mut self) {
        if self.has_item_navigation() {
            let _ = self.move_cursor_to_next_item();
        } else if self.cursor + 1 < self.lines.len() {
            self.cursor += 1;
        }
        self.ensure_cursor_visible(self.viewport_rows.max(1));
    }

    /// Move cursor one page up using viewport-aware step size.
    pub(super) fn page_up(&mut self) {
        let step = self.viewport_rows.saturating_sub(1).max(1);

        if self.has_item_navigation() {
            let starts = self.selectable_indices();
            if starts.is_empty() {
                return;
            }
            let target = self.cursor.saturating_sub(step);
            self.cursor = starts
                .iter()
                .copied()
                .rev()
                .find(|index| *index <= target)
                .unwrap_or(starts[0]);
        } else {
            self.cursor = self.cursor.saturating_sub(step);
        }

        self.ensure_cursor_visible(self.viewport_rows.max(1));
    }

    /// Move cursor one page down using viewport-aware step size.
    pub(super) fn page_down(&mut self) {
        let step = self.viewport_rows.saturating_sub(1).max(1);

        if self.has_item_navigation() {
            let starts = self.selectable_indices();
            if starts.is_empty() {
                return;
            }
            let last_row = self.lines.len().saturating_sub(1);
            let target = self.cursor.saturating_add(step).min(last_row);
            self.cursor = starts
                .iter()
                .copied()
                .find(|index| *index >= target)
                .unwrap_or_else(|| *starts.last().expect("starts cannot be empty"));
        } else if !self.lines.is_empty() {
            self.cursor = (self.cursor + step).min(self.lines.len().saturating_sub(1));
        }

        self.ensure_cursor_visible(self.viewport_rows.max(1));
    }

    /// Jump cursor to top row (or first selectable item) and reset scroll.
    pub(super) fn move_cursor_top(&mut self) {
        self.cursor = self.first_selectable_index().unwrap_or(0);
        self.scroll = 0;
    }

    /// Jump cursor to bottom row (or last selectable item) and align viewport.
    pub(super) fn move_cursor_bottom(&mut self) {
        if self.lines.is_empty() {
            self.cursor = 0;
            self.scroll = 0;
            return;
        }

        self.cursor = self
            .last_selectable_index()
            .unwrap_or_else(|| self.lines.len().saturating_sub(1));
        self.ensure_cursor_visible(self.viewport_rows.max(1));
    }

    /// Return whether current rendered content supports item-oriented revision navigation.
    fn has_item_navigation(&self) -> bool {
        !self.selectable_indices().is_empty()
    }

    /// Return whether log-like graph rows can be navigated even without metadata map entries.
    fn has_graph_row_navigation(&self) -> bool {
        matches!(self.last_command.first().map(String::as_str), Some("log"))
            && self
                .lines
                .iter()
                .any(|line| looks_like_graph_commit_row(line))
    }

    /// Return first selectable row index for item-based views.
    fn first_selectable_index(&self) -> Option<usize> {
        self.selectable_indices().first().copied()
    }

    /// Return last selectable row index for item-based views at item boundary.
    fn last_selectable_index(&self) -> Option<usize> {
        self.selectable_indices().last().copied()
    }

    /// Return row indices for selectable revision items.
    ///
    /// For metadata-backed maps, this returns item-start boundaries (first row where revision
    /// changes). If metadata is unavailable, graph-row starts are used as a fallback.
    fn selectable_indices(&self) -> Vec<usize> {
        if self.row_revision_map.len() == self.lines.len() {
            let mut starts = Vec::new();
            let mut previous: Option<&str> = None;

            for (index, revision) in self.row_revision_map.iter().enumerate() {
                let current = revision.as_deref();
                if let Some(revision) = current
                    && previous != Some(revision)
                {
                    starts.push(index);
                }
                previous = current;
            }

            if !starts.is_empty() {
                return starts;
            }
        }

        if self.has_graph_row_navigation() {
            return self
                .lines
                .iter()
                .enumerate()
                .filter_map(|(index, line)| looks_like_graph_commit_row(line).then_some(index))
                .collect();
        }

        Vec::new()
    }

    /// Move cursor to previous revision item. Returns true when movement occurs.
    fn move_cursor_to_previous_item(&mut self) -> bool {
        if self.cursor == 0 || !self.has_item_navigation() {
            return false;
        }
        let starts = self.selectable_indices();
        if let Some(target) = starts
            .iter()
            .copied()
            .rev()
            .find(|index| *index < self.cursor)
        {
            self.cursor = target;
            return true;
        }
        false
    }

    /// Move cursor to next revision item. Returns true when movement occurs.
    fn move_cursor_to_next_item(&mut self) -> bool {
        if self.cursor + 1 >= self.lines.len() || !self.has_item_navigation() {
            return false;
        }
        let starts = self.selectable_indices();
        if let Some(target) = starts.iter().copied().find(|index| *index > self.cursor) {
            self.cursor = target;
            return true;
        }
        false
    }

    /// Adjust scroll so selected row stays within the visible content window.
    pub(super) fn ensure_cursor_visible(&mut self, content_height: usize) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
            return;
        }

        if self.cursor >= self.scroll.saturating_add(content_height) {
            self.scroll = self.cursor.saturating_sub(content_height.saturating_sub(1));
        }
    }

    /// Resolve selected revision nearest to cursor.
    ///
    /// Metadata row mapping is preferred and plain-text parsing is used as a fallback.
    pub(super) fn selected_revision(&self) -> Option<String> {
        if !self.row_revision_map.is_empty() {
            for line_index in (0..=self.cursor).rev() {
                if let Some(Some(revision)) = self.row_revision_map.get(line_index) {
                    return Some(revision.clone());
                }
            }
        }

        for line_index in (0..=self.cursor).rev() {
            if let Some(line) = self.lines.get(line_index)
                && let Some(revision) = extract_revision(line)
            {
                return Some(revision);
            }
        }

        None
    }

    /// Draw one frame using ratatui widgets.
    fn draw(&mut self, terminal: &mut AppTerminal) -> Result<(), JkError> {
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(area);

            let chrome_style = Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD);
            let mode_badge_style = Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD);
            let header_widget = Paragraph::new(Line::from(vec![
                Span::styled(" jk ", chrome_style),
                Span::styled(
                    format!("[{}]", self.mode_label().to_ascii_uppercase()),
                    mode_badge_style,
                ),
                Span::styled(" ", chrome_style),
            ]));
            frame.render_widget(header_widget, layout[0]);

            let content_height = layout[1].height as usize;
            self.viewport_rows = content_height.max(1);
            self.ensure_cursor_visible(content_height.max(1));

            let mut body_lines = Vec::with_capacity(content_height);
            for idx in 0..content_height {
                let line_index = self.scroll + idx;
                let raw_line = self.lines.get(line_index).map(String::as_str).unwrap_or("");
                let next_line = self
                    .lines
                    .get(line_index + 1)
                    .map(String::as_str)
                    .unwrap_or("");
                let mut content = self.display_line_for_tui(raw_line);
                let section_heading = self.is_legacy_underline(next_line);
                let selected = line_index == self.cursor && self.mode == Mode::Normal;

                if section_heading {
                    content = content.patch_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    );
                }
                if selected {
                    content = content.patch_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    );
                }

                let prefix_style = if selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let prefix = if selected { "▸ " } else { "  " };
                let mut spans = Vec::with_capacity(content.spans.len() + 1);
                spans.push(Span::styled(prefix, prefix_style));
                spans.extend(content.spans);
                content.spans = spans;
                body_lines.push(content);
            }

            frame.render_widget(Paragraph::new(Text::from(body_lines)), layout[1]);

            let footer = match self.mode {
                Mode::Normal => {
                    let mut segments = Vec::new();
                    if !self.onboarding.complete {
                        segments.push(format!(
                            "onboarding: {}",
                            self.onboarding_next_step_hint()
                        ));
                    }
                    segments.push(format!("next: {}", self.primary_next_action_hint()));
                    if let Some(actions) = self.log_quick_actions_hint() {
                        segments.push(actions);
                    }
                    if let Some(history) = self.view_history_hint() {
                        segments.push(history);
                    }
                    segments.push(
                        "nav j/k ↑/↓ PgUp/PgDn Ctrl+u/d  |  back/forward ←/→ Ctrl+o/i  |  views l s o L v f t w  |  ? help"
                            .to_string(),
                    );
                    let hints = segments.join("  |  ");
                    if self.show_status_line_in_footer() {
                        format!("{}  |  {hints}", self.status_line)
                    } else {
                        hints.to_string()
                    }
                }
                Mode::Command => {
                    let suggestions = self.ranked_command_suggestions(&self.command_input, 3);
                    let suggestions_label = if suggestions.is_empty() {
                        "suggest: (none)".to_string()
                    } else {
                        format!("suggest: {}", suggestions.join(", "))
                    };
                    let recent = self.recent_intent_labels(3);
                    let recent_label = if recent.is_empty() {
                        String::new()
                    } else {
                        format!("  |  recent: {}", recent.join(", "))
                    };
                    format!(
                        ":{}  (Enter run, Esc cancel, Up/Down history)  |  {}{}",
                        self.command_input, suggestions_label, recent_label
                    )
                }
                Mode::Confirm => {
                    let pending = self.pending_confirm.clone().unwrap_or_default();
                    if confirmation_preview_tokens(&pending).is_some() {
                        format!("Run `jj {}` ? [y/n/Esc/d dry-run]", pending.join(" "))
                    } else {
                        format!("Run `jj {}` ? [y/n/Esc]", pending.join(" "))
                    }
                }
                Mode::Prompt => {
                    if let Some(prompt) = &self.pending_prompt {
                        format!(
                            "{} > {}  (Enter submit, Esc cancel)",
                            prompt.label, prompt.input
                        )
                    } else {
                        "prompt unavailable".to_string()
                    }
                }
            };

            frame.render_widget(
                Paragraph::new(format!(" {footer} ")).style(chrome_style),
                layout[2],
            );
        })?;
        Ok(())
    }

    /// Convert legacy content lines into cleaner TUI display lines.
    ///
    /// ANSI styling from command output is preserved using `ansi-to-tui`.
    fn display_line_for_tui(&self, line: &str) -> Line<'static> {
        if self.is_legacy_underline(line) {
            return Line::default();
        }

        line.into_text()
            .ok()
            .and_then(|text| text.lines.into_iter().next())
            .unwrap_or_else(|| Line::raw(strip_ansi(line)))
    }

    /// Return whether a line is a legacy ASCII underline separator.
    fn is_legacy_underline(&self, line: &str) -> bool {
        let trimmed = strip_ansi(line).trim().to_string();
        trimmed.len() > 2 && trimmed.chars().all(|ch| ch == '=' || ch == '-')
    }

    /// Build current frame rows including header, visible content slice, and mode-specific footer.
    #[cfg(test)]
    fn render_for_display(&mut self, width: usize, height: usize) -> Vec<String> {
        let header = format!("jk [{}]", self.mode_label());

        let content_height = height.saturating_sub(2);
        self.ensure_cursor_visible(content_height.max(1));

        let mut rows = Vec::with_capacity(height.max(1));
        rows.push(trim_to_width(&header, width));

        for idx in 0..content_height {
            let line_index = self.scroll + idx;
            let content = if let Some(line) = self.lines.get(line_index) {
                let marker = if line_index == self.cursor && self.mode == Mode::Normal {
                    ">"
                } else {
                    " "
                };
                format!("{marker} {}", line)
            } else {
                String::new()
            };
            rows.push(trim_to_width(&content, width));
        }

        let footer = match self.mode {
            Mode::Normal => {
                let mut segments = Vec::new();
                if !self.onboarding.complete {
                    segments.push(format!("onboarding: {}", self.onboarding_next_step_hint()));
                }
                segments.push(format!("next: {}", self.primary_next_action_hint()));
                if let Some(actions) = self.log_quick_actions_hint() {
                    segments.push(actions);
                }
                if let Some(history) = self.view_history_hint() {
                    segments.push(history);
                }
                segments.push(
                    "nav j/k ↑/↓ PgUp/PgDn Ctrl+u/d  |  back/forward ←/→ Ctrl+o/i  |  views l s o L v f t w  |  ? help"
                        .to_string(),
                );
                let hints = segments.join("  |  ");
                if self.show_status_line_in_footer() {
                    format!("{}  |  {hints}", self.status_line)
                } else {
                    hints
                }
            }
            Mode::Command => {
                let suggestions = self.ranked_command_suggestions(&self.command_input, 2);
                if suggestions.is_empty() {
                    format!(":{} (suggest: none)", self.command_input)
                } else {
                    format!(
                        ":{} (suggest: {})",
                        self.command_input,
                        suggestions.join(", ")
                    )
                }
            }
            Mode::Confirm => {
                let pending = self.pending_confirm.clone().unwrap_or_default();
                if confirmation_preview_tokens(&pending).is_some() {
                    format!("Run `jj {}` ? [y/n/d]", pending.join(" "))
                } else {
                    format!("Run `jj {}` ? [y/n]", pending.join(" "))
                }
            }
            Mode::Prompt => {
                if let Some(prompt) = &self.pending_prompt {
                    format!("{} > {}", prompt.label, prompt.input)
                } else {
                    "prompt unavailable".to_string()
                }
            }
        };
        rows.push(trim_to_width(&footer, width));

        rows
    }

    /// Return short label used in header for current input mode.
    fn mode_label(&self) -> &'static str {
        match self.mode {
            Mode::Normal => "normal",
            Mode::Command => "command",
            Mode::Confirm => "confirm",
            Mode::Prompt => "prompt",
        }
    }

    /// Return whether status line adds meaningful context beyond header/footer hints.
    fn show_status_line_in_footer(&self) -> bool {
        self.status_line.starts_with("error:")
            || self.status_line.starts_with("No ")
            || self.status_line.contains("canceled")
            || self.status_line.contains("required")
            || self.status_line.contains("unavailable")
    }

    /// Update first-run onboarding progress from executed command tokens.
    fn update_onboarding_progress(&mut self, tokens: &[String]) {
        let head = tokens.first().map(String::as_str).unwrap_or_default();
        let sub = tokens.get(1).map(String::as_str).unwrap_or_default();

        if matches!(head, "log" | "show" | "diff" | "evolog" | "interdiff") {
            self.onboarding.inspect = true;
        }

        if matches!(
            head,
            "new"
                | "describe"
                | "commit"
                | "rebase"
                | "squash"
                | "split"
                | "abandon"
                | "restore"
                | "revert"
                | "bookmark"
        ) || (head == "git" && sub == "push")
        {
            self.onboarding.act = true;
        }

        if matches!(head, "status" | "log" | "show" | "diff")
            || (head == "git" && matches!(sub, "fetch" | "push"))
        {
            self.onboarding.verify = true;
        }

        if matches!(head, "undo" | "redo") || (head == "operation" && sub == "log") {
            self.onboarding.recover = true;
        }

        if !self.onboarding.complete
            && self.onboarding.inspect
            && self.onboarding.act
            && self.onboarding.verify
            && self.onboarding.recover
        {
            self.onboarding.complete = true;
            self.status_line =
                "Onboarding complete: inspect -> act -> verify -> recover".to_string();
        }
    }

    /// Return next required onboarding step hint.
    fn onboarding_next_step_hint(&self) -> &'static str {
        if !self.onboarding.inspect {
            "inspect (`Enter`/`d`)"
        } else if !self.onboarding.act {
            "act (`D`/`B`/`S`/`X`/`a`)"
        } else if !self.onboarding.verify {
            "verify (`s`/`log`)"
        } else if !self.onboarding.recover {
            "recover (`o` then `u`/`U`)"
        } else {
            "complete"
        }
    }

    /// Return primary next action hint for the active view.
    fn primary_next_action_hint(&self) -> &'static str {
        let command = self.current_view_command.as_str();
        if command.starts_with("commands") || command.starts_with("help") {
            return ":help inspect|rewrite|sync|recover";
        }
        if command.starts_with("status") {
            return "F fetch or P push";
        }
        if command.starts_with("show") || command.starts_with("diff") {
            return "Left back to prior screen";
        }
        if command.starts_with("operation") {
            return "u/U undo or redo from op context";
        }
        if command.starts_with("log")
            || (command.is_empty()
                && matches!(self.last_command.first().map(String::as_str), Some("log")))
        {
            return "Enter show selected revision";
        }
        "l return to log home"
    }

    /// Return quick actions for log-like screens when a revision is selected.
    fn log_quick_actions_hint(&self) -> Option<String> {
        if !matches!(self.last_command.first().map(String::as_str), Some("log")) {
            return None;
        }
        self.selected_revision().map(|revision| {
            format!("quick ({revision}): Enter show, d diff, D describe, a abandon")
        })
    }

    /// Return back/forward context hint when view history exists.
    fn view_history_hint(&self) -> Option<String> {
        if self.view_back_stack.is_empty() && self.view_forward_stack.is_empty() {
            return None;
        }
        let back = self
            .view_back_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "-".to_string());
        let forward = self
            .view_forward_stack
            .last()
            .cloned()
            .unwrap_or_else(|| "-".to_string());
        Some(format!("history: back {back} | fwd {forward}"))
    }

    #[cfg(test)]
    /// Render deterministic frame text for snapshot tests.
    pub fn render_for_snapshot(&mut self, width: usize, height: usize) -> String {
        self.render_for_display(width, height).join("\n")
    }
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use crate::config::KeybindConfig;

    use super::App;

    #[test]
    fn display_line_for_tui_preserves_ansi_colors() {
        let app = App::new(KeybindConfig::load().expect("keybind config should parse"));
        let rendered = app.display_line_for_tui("\u{1b}[31mred\u{1b}[0m plain");
        let colored_span = rendered
            .spans
            .iter()
            .find(|span| span.content == "red")
            .expect("red span should exist");

        assert_eq!(colored_span.style.fg, Some(Color::Red));
    }
}
