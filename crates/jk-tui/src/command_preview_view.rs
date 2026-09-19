//! Draw-only confirmation preview for commands that may mutate state.
//!
//! This view renders [`jk_core::CommandPreview`] data and intentionally owns no execution behavior.

use jk_core::{CommandPreview, CommandPreviewWarning, SafetyClass};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};

const PANEL_WIDTH: u16 = 78;
const MIN_PANEL_HEIGHT: u16 = 8;
const BACKGROUND: Color = Color::Rgb(30, 35, 47);
const HEADER: Color = Color::Rgb(58, 72, 90);
const COMMAND: Color = Color::Rgb(22, 27, 38);
const MUTED: Color = Color::Rgb(161, 174, 190);

/// A compact confirmation view for a pending command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPreviewView {
    preview: CommandPreview,
    status: Option<String>,
}

impl CommandPreviewView {
    /// Creates a view for a pending command preview.
    #[must_use]
    pub const fn new(preview: CommandPreview) -> Self {
        Self {
            preview,
            status: None,
        }
    }

    /// Returns the preview being rendered.
    #[must_use]
    pub const fn preview(&self) -> &CommandPreview {
        &self.preview
    }

    /// Sets a temporary footer status message.
    #[must_use]
    pub fn with_status(mut self, status: Option<String>) -> Self {
        self.status = status;
        self
    }

    /// Renders the command preview without executing anything.
    pub fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }

        let mut text = self.body_text();
        let panel_width = PANEL_WIDTH.min(area.width);
        let content_width = panel_width.saturating_sub(2).max(1);
        if let Some(title) = text.lines.first_mut() {
            title.spans.push(Span::raw(
                " ".repeat(usize::from(content_width).saturating_sub(title.width())),
            ));
        }
        let panel = centered_panel(area, panel_height(&text, content_width).saturating_add(3));
        frame.render_widget(Clear, panel);

        let block = Block::default()
            .padding(Padding::uniform(1))
            .style(Style::new().fg(Color::White).bg(BACKGROUND));
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
                .style(Style::new().fg(Color::White).bg(BACKGROUND))
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, body_area);
        }
        frame.render_widget(
            Paragraph::new(footer_line(
                self.status.as_deref(),
                self.preview.safety == SafetyClass::DestructiveLocal,
            ))
            .style(Style::new().fg(Color::White).bg(HEADER)),
            footer_area,
        );
    }

    fn body_text(&self) -> Text<'_> {
        let mut lines = vec![
            Line::from(Span::styled(
                format!("  {}", self.preview.title),
                Style::new()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .style(Style::new().bg(HEADER)),
            Line::from(""),
            Line::from(Span::styled("Review command", Style::new().fg(MUTED))),
            Line::from(Span::styled(
                self.preview.command_line.as_str(),
                Style::new().fg(Color::White),
            ))
            .style(Style::new().bg(COMMAND)),
            Line::from(""),
            Line::from(Span::styled(
                effect_label(&self.preview),
                Style::new().fg(MUTED),
            )),
        ];

        lines.push(Line::from(""));
        if !self.preview.warnings.is_empty() {
            lines.push(Line::from(Span::styled(
                "Before you continue",
                Style::new()
                    .fg(Color::LightRed)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.extend(self.preview.warnings.iter().map(warning_line));
        }

        Text::from(lines)
    }
}

fn footer_line(status: Option<&str>, destructive: bool) -> Line<'static> {
    if let Some(status) = status {
        return Line::from(vec![
            Span::styled(
                status.to_owned(),
                Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled("enter run", Style::new().fg(Color::Green)),
            Span::raw("    "),
            Span::styled("esc cancel", Style::new().fg(Color::Red)),
        ]);
    }

    Line::from(vec![
        Span::styled(
            " enter run ",
            Style::new()
                .fg(if destructive {
                    Color::White
                } else {
                    Color::Black
                })
                .bg(if destructive {
                    Color::Rgb(120, 45, 50)
                } else {
                    Color::LightCyan
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled("y copy", Style::new().fg(MUTED)),
        Span::raw("    "),
        Span::styled("esc cancel", Style::new().fg(Color::White)),
    ])
}

fn warning_line(warning: &CommandPreviewWarning) -> Line<'_> {
    Line::from(vec![
        Span::styled("! ", Style::new().fg(Color::Red)),
        Span::raw(warning_label(warning)),
    ])
}

fn effect_label(preview: &CommandPreview) -> &'static str {
    if preview.title.contains("push") && preview.title.contains("dry-run") {
        return "Contacts the remote to check the push. Remote bookmarks stay unchanged.";
    }
    match preview.safety {
        SafetyClass::NetworkRead if preview.title == "jj git fetch" => {
            "Fetches remote changes and refreshes your local bookmark list."
        }
        SafetyClass::NetworkRead => "Reads from the selected remote.",
        SafetyClass::NetworkWrite => "Updates the selected remote.",
        SafetyClass::DestructiveLocal if preview.title == "jj bookmark delete" => {
            "Deletes the local bookmark. A later push can delete its remote copy."
        }
        SafetyClass::DestructiveLocal => "Deletes or replaces local repository data.",
        SafetyClass::LocalMetadata => "Updates this repository's local metadata.",
        SafetyClass::LocalRewrite => "Changes local history.",
        _ => "Review the command before continuing.",
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
    text_height.saturating_add(2).try_into().unwrap_or(u16::MAX)
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
        assert!(rendered.contains("Describe workspace"));
        assert!(rendered.contains("jj --no-pager --color always --ignore-working-copy describe"));
        assert!(rendered.contains("--message"));
        assert!(rendered.contains("'Update preview renderer'"));
        assert!(!rendered.contains("Safety:"));
        assert!(!rendered.contains("Execution:"));
        assert!(!rendered.contains("Refresh:"));
        assert!(!rendered.contains('┌'));
        assert!(
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .any(|cell| cell.bg == HEADER)
        );
        assert!(rendered.contains("Rewrites local history."));
        assert!(rendered.contains("Ignores the current working-copy snapshot."));
        assert!(rendered.contains("enter"));
        assert!(rendered.contains("run"));
        assert!(rendered.contains('y'));
        assert!(rendered.contains("copy"));
        assert!(rendered.contains("esc"));
        assert!(rendered.contains("cancel"));
    }

    #[test]
    fn command_preview_omits_empty_warning_sections() {
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
        assert!(!rendered.contains("No warnings"));
        assert!(rendered.contains("enter run"));
        assert!(rendered.contains("y copy"));
        assert!(rendered.contains("esc cancel"));
    }

    #[test]
    fn command_preview_status_replaces_copy_hint() {
        let preview = JjCommandSpec::render_read_only(["undo"])
            .with_title("Undo latest operation")
            .command_preview();
        let view = CommandPreviewView::new(preview).with_status(Some("copied command".to_owned()));
        let backend = TestBackend::new(80, 14);
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => match error {},
        };

        let draw_result = terminal.draw(|frame| view.render(frame));
        assert!(draw_result.is_ok());

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("copied command"));
        assert!(rendered.contains("enter run"));
        assert!(!rendered.contains("y copy"));
    }

    #[test]
    fn destructive_preview_uses_red_action_region_without_borders() {
        let preview = JjCommandSpec::confirm_mutation(
            ["bookmark", "delete", "--", "exact:topic"],
            SafetyClass::DestructiveLocal,
        )
        .with_title("jj bookmark delete")
        .command_preview();
        for (width, height) in [(40, 18), (80, 24), (120, 30)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
            terminal
                .draw(|frame| CommandPreviewView::new(preview.clone()).render(frame))
                .expect("draw");
            let buffer = terminal.backend().buffer();
            let rendered = buffer_to_string(buffer);
            assert!(!rendered.chars().any(|ch| "┌┐└┘│─".contains(ch)));
            assert!(rendered.contains("esc cancel"));
            assert!(
                buffer
                    .content
                    .iter()
                    .any(|cell| cell.bg == Color::Rgb(120, 45, 50))
            );
        }
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
