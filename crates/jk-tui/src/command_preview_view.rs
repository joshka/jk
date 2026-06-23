//! Draw-only confirmation preview for commands that may mutate state.
//!
//! This view renders [`jk_core::CommandPreview`] data and intentionally owns no execution behavior.

use jk_core::{CommandPreview, CommandPreviewWarning, ExecutionMode, RefreshPlan, SafetyClass};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

const PANEL_WIDTH: u16 = 78;
const MIN_PANEL_HEIGHT: u16 = 8;

/// A compact confirmation view for a pending command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPreviewView {
    preview: CommandPreview,
}

impl CommandPreviewView {
    /// Creates a view for a pending command preview.
    #[must_use]
    pub const fn new(preview: CommandPreview) -> Self {
        Self { preview }
    }

    /// Returns the preview being rendered.
    #[must_use]
    pub const fn preview(&self) -> &CommandPreview {
        &self.preview
    }

    /// Renders the command preview without executing anything.
    pub fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }

        let text = self.body_text();
        let panel_width = PANEL_WIDTH.min(area.width);
        let content_width = panel_width.saturating_sub(2).max(1);
        let panel = centered_panel(area, panel_height(&text, content_width).saturating_add(1));
        frame.render_widget(Clear, panel);

        let block = Block::bordered()
            .title(" Confirm command ")
            .style(Style::new().fg(Color::White).bg(Color::Black));
        let inner = block.inner(panel);
        frame.render_widget(block, panel);
        if inner.is_empty() {
            return;
        }

        let footer_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width,
            height: 1,
        };
        let body_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(1),
        };

        if !body_area.is_empty() {
            let paragraph = Paragraph::new(text)
                .style(Style::new().fg(Color::White).bg(Color::Black))
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, body_area);
        }
        frame.render_widget(
            Paragraph::new(footer_line()).style(Style::new().fg(Color::White).bg(Color::Black)),
            footer_area,
        );
    }

    fn body_text(&self) -> Text<'_> {
        let mut lines = vec![
            Line::from(Span::styled(
                self.preview.title.as_str(),
                Style::new().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Command",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                self.preview.command_line.as_str(),
                Style::new().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Summary",
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("Safety: {}", safety_label(self.preview.safety))),
            Line::from(format!(
                "Execution: {}",
                execution_label(self.preview.execution_mode)
            )),
            Line::from(format!(
                "Refresh: {}",
                refresh_label(self.preview.refresh_plan)
            )),
        ];

        if self.preview.warnings.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "No warnings for this command.",
                Style::new().fg(Color::Green),
            )));
        } else {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Warnings",
                Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            lines.extend(self.preview.warnings.iter().map(warning_line));
        }

        Text::from(lines)
    }
}

fn footer_line() -> Line<'static> {
    Line::from(vec![
        Span::styled(
            "enter run",
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled(
            "esc cancel",
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn warning_line(warning: &CommandPreviewWarning) -> Line<'_> {
    Line::from(vec![
        Span::styled("! ", Style::new().fg(Color::Red)),
        Span::raw(warning_label(warning)),
    ])
}

fn safety_label(safety: SafetyClass) -> &'static str {
    match safety {
        SafetyClass::ReadOnly => "read-only",
        SafetyClass::LocalMetadata => "local metadata",
        SafetyClass::LocalRewrite => "local rewrite",
        SafetyClass::DestructiveLocal => "destructive local",
        SafetyClass::NetworkRead => "network read",
        SafetyClass::NetworkWrite => "network write",
        SafetyClass::ExternalCommand => "external command",
        _ => "unknown",
    }
}

fn execution_label(mode: ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::RenderReadOnly => "render read-only",
        ExecutionMode::ConfirmMutation => "confirm mutation",
        ExecutionMode::ConfirmExternalTool => "confirm external tool",
        ExecutionMode::DryRunThenConfirm => "dry-run then confirm",
        ExecutionMode::CommandMode => "command mode",
        _ => "unknown",
    }
}

fn refresh_label(refresh_plan: RefreshPlan) -> &'static str {
    match refresh_plan {
        RefreshPlan::None => "none",
        RefreshPlan::ReRunSpec => "re-run current command",
        _ => "unknown",
    }
}

fn warning_label(warning: &CommandPreviewWarning) -> String {
    match warning {
        CommandPreviewWarning::LocalMetadata => "Changes local repository metadata.".to_owned(),
        CommandPreviewWarning::LocalRewrite => "Rewrites local history.".to_owned(),
        CommandPreviewWarning::DestructiveLocal => {
            "Performs a destructive local operation.".to_owned()
        }
        CommandPreviewWarning::NetworkWrite => "Writes to a remote or network service.".to_owned(),
        CommandPreviewWarning::ExternalCommand => "Runs an external command.".to_owned(),
        CommandPreviewWarning::IgnoresWorkingCopy => {
            "Ignores the current working-copy snapshot.".to_owned()
        }
        CommandPreviewWarning::AtOperation(operation) => {
            format!("Runs at operation {operation}.")
        }
        CommandPreviewWarning::DoesNotIntegrateOperation => {
            "Does not integrate the loaded operation.".to_owned()
        }
        CommandPreviewWarning::IgnoresImmutableCommits => {
            "May rewrite immutable commits.".to_owned()
        }
        _ => "Review this command before running.".to_owned(),
    }
}

fn centered_panel(area: Rect, preferred_height: u16) -> Rect {
    let width = PANEL_WIDTH.min(area.width);
    let height = preferred_height.max(MIN_PANEL_HEIGHT).min(area.height);

    Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    }
}

fn panel_height(text: &Text<'_>, content_width: u16) -> u16 {
    let content_width = usize::from(content_width.max(1));
    let text_height = text
        .lines
        .iter()
        .map(|line| line.width().div_ceil(content_width).max(1))
        .sum::<usize>();
    text_height.saturating_add(2) as u16
}

#[cfg(test)]
mod tests {
    use jk_core::{
        ExecutionMode, GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass, WorkingCopyPolicy,
    };
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn command_preview_renders_command_warning_and_key_hints() {
        let global_options = GlobalOptions::default().with_working_copy(WorkingCopyPolicy::Ignore);
        let preview = JjCommandSpec::confirm_mutation(
            ["describe", "--message", "Update preview renderer"],
            SafetyClass::LocalRewrite,
        )
        .with_global_options(global_options)
        .with_title("Describe workspace")
        .command_preview();
        let view = CommandPreviewView::new(preview);
        let backend = TestBackend::new(120, 30);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Confirm command"));
        assert!(rendered.contains("Describe workspace"));
        assert!(rendered.contains("jj --no-pager --color always --ignore-working-copy describe"));
        assert!(rendered.contains("--message"));
        assert!(rendered.contains("'Update preview renderer'"));
        assert!(rendered.contains("Safety: local rewrite"));
        assert!(rendered.contains("Execution: confirm mutation"));
        assert!(rendered.contains("Rewrites local history."));
        assert!(rendered.contains("Ignores the current working-copy snapshot."));
        assert!(rendered.contains("enter"));
        assert!(rendered.contains("run"));
        assert!(rendered.contains("esc"));
        assert!(rendered.contains("cancel"));
    }

    #[test]
    fn command_preview_without_warnings_says_so() {
        let preview = JjCommandSpec::render_read_only(["log"])
            .with_mode(ExecutionMode::RenderReadOnly)
            .with_safety(SafetyClass::ReadOnly)
            .with_refresh_plan(RefreshPlan::None)
            .with_title("Refresh log")
            .command_preview();
        let view = CommandPreviewView::new(preview);
        let backend = TestBackend::new(80, 14);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("jj --no-pager --color always log"));
        assert!(rendered.contains("Safety: read-only"));
        assert!(rendered.contains("No warnings for this command."));
        assert!(rendered.contains("enter run"));
        assert!(rendered.contains("esc cancel"));
    }

    fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
        let area = buffer.area;
        let mut text = String::new();

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }

        text
    }
}
