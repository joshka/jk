//! Draw-only confirmation preview for commands that may change state.
//!
//! This view renders [`jk_core::CommandPreview`] data and intentionally owns no execution behavior.

use jk_core::{CommandPreview, CommandPreviewWarning};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use crate::styles::{TITLE, dialog, dialog_accent, dialog_supporting, dialog_warning};

const MAX_PANEL_WIDTH: u16 = 78;
const MIN_CONFIRM_WIDTH: u16 = 40;
const MIN_CONFIRM_HEIGHT: u16 = 8;
const PANEL_PADDING: u16 = 2;
const CHROME_HEIGHT: u16 = 4;

/// A compact confirmation view for a pending command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPreviewView {
    preview: CommandPreview,
    status: Option<String>,
    details: Vec<String>,
    scroll: u16,
    run_options: bool,
}

impl CommandPreviewView {
    /// Creates a view for a pending command preview.
    #[must_use]
    pub const fn new(preview: CommandPreview) -> Self {
        Self {
            preview,
            status: None,
            details: Vec::new(),
            scroll: 0,
            run_options: false,
        }
    }

    /// Returns the preview being rendered.
    #[must_use]
    pub const fn preview(&self) -> &CommandPreview {
        &self.preview
    }

    /// Sets a temporary status message beneath the reviewed command.
    #[must_use]
    pub fn with_status(mut self, status: Option<String>) -> Self {
        self.status = status;
        self
    }

    /// Adds command-specific details that should be reviewed before confirmation.
    #[must_use]
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    /// Sets the rendered-line offset for long previews.
    #[must_use]
    pub const fn with_scroll(mut self, scroll: u16) -> Self {
        self.scroll = scroll;
        self
    }

    /// Shows the Run Options control when the pending command supports it.
    #[must_use]
    pub const fn with_run_options(mut self, available: bool) -> Self {
        self.run_options = available;
        self
    }

    /// Returns the greatest rendered-line offset available in `area`.
    #[must_use]
    pub fn max_scroll(&self, area: Rect) -> u16 {
        let body = self.body();
        let panel = self.panel(area, &body);
        let content_width = panel.width.saturating_sub(PANEL_PADDING).max(1);
        let body_height = panel
            .height
            .saturating_sub(self.chrome_height(content_width));
        text_height(&body, content_width).saturating_sub(body_height)
    }

    /// Reports whether the viewport can show all confirmation controls.
    #[must_use]
    pub const fn can_confirm(&self, area: Rect) -> bool {
        area.width >= MIN_CONFIRM_WIDTH && area.height >= MIN_CONFIRM_HEIGHT
    }

    /// Renders the command preview without executing anything.
    pub fn render(&self, frame: &mut Frame<'_>) {
        let area = frame.area();
        if area.is_empty() {
            return;
        }
        if !self.can_confirm(area) {
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new("Enlarge terminal to review.\nEsc cancel").style(dialog()),
                area,
            );
            return;
        }

        let body = self.body();
        let panel = self.panel(area, &body);
        let content_width = panel.width.saturating_sub(PANEL_PADDING).max(1);
        let body_height = panel
            .height
            .saturating_sub(self.chrome_height(content_width));
        let max_scroll = text_height(&body, content_width).saturating_sub(body_height);
        frame.render_widget(Clear, panel);
        frame.render_widget(Block::default().style(dialog()), panel);

        let header_area = Rect::new(panel.x + 1, panel.y + 1, content_width, 1);
        let body_area = Rect::new(
            panel.x + 1,
            header_area.bottom(),
            content_width,
            body_height,
        );
        let footer_area = Rect::new(
            panel.x + 1,
            body_area.bottom(),
            content_width,
            self.chrome_height(content_width) - 3,
        );
        frame.render_widget(
            Paragraph::new(self.preview.title.as_str()).style(dialog_accent()),
            header_area,
        );
        frame.render_widget(
            Paragraph::new(body)
                .style(dialog())
                .wrap(Wrap { trim: false })
                .scroll((self.scroll.min(max_scroll), 0)),
            body_area,
        );
        frame.render_widget(
            Paragraph::new(footer_text(self.run_options, max_scroll > 0, content_width))
                .style(dialog()),
            footer_area,
        );
    }

    fn body(&self) -> Text<'_> {
        let mut lines = vec![Line::default()];
        if !self.details.is_empty() {
            lines.extend(self.details.iter().map(|detail| Line::raw(detail.as_str())));
            lines.push(Line::default());
        }

        // Consequences follow operand roles and precede the exact executable command. Do not repeat
        // the generic safety classification: these warnings describe the actual pending effects.
        if !self.preview.warnings.is_empty() {
            lines.extend(self.preview.warnings.iter().map(warning_line));
            lines.push(Line::default());
        }

        lines.push(Line::from(Span::styled("Command", TITLE)));
        lines.push(Line::raw(self.preview.command_line.as_str()));
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            self.status.as_deref().unwrap_or_default(),
            dialog_supporting(),
        )));
        Text::from(lines)
    }

    fn panel(&self, area: Rect, body: &Text<'_>) -> Rect {
        // Keep repository context visible around the decision whenever the remaining panel can
        // still satisfy the minimum control layout. Very small terminals retain the full viewport.
        let horizontal_margin = if area.width >= MIN_CONFIRM_WIDTH + 4 {
            4
        } else {
            0
        };
        let vertical_margin = if area.height >= MIN_CONFIRM_HEIGHT + 4 {
            4
        } else {
            0
        };
        let available_width = area.width.saturating_sub(horizontal_margin);
        let available_height = area.height.saturating_sub(vertical_margin);
        let preferred_width = body
            .width()
            .max(Line::raw(self.preview.title.as_str()).width())
            .max(footer_text(self.run_options, false, u16::MAX).width())
            .max(33);
        let width = u16::try_from(preferred_width)
            .unwrap_or(u16::MAX)
            .saturating_add(PANEL_PADDING)
            .min(MAX_PANEL_WIDTH)
            .min(available_width);
        let content_width = width.saturating_sub(PANEL_PADDING).max(1);
        let height = text_height(body, content_width)
            .saturating_add(self.chrome_height(content_width))
            .min(available_height);
        Rect::new(
            area.x + area.width.saturating_sub(width) / 2,
            area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        )
    }

    fn chrome_height(&self, content_width: u16) -> u16 {
        // Reserve the optional control row regardless of scrolling so footer geometry is stable.
        CHROME_HEIGHT + u16::from(self.run_options && content_width < 51)
    }
}

fn footer_text(run_options: bool, scrolls: bool, width: u16) -> Text<'static> {
    let mut spans = vec![
        Span::styled("enter", dialog_accent()),
        Span::raw(" run  "),
        Span::styled("y", dialog_accent()),
        Span::raw(" copy  "),
        Span::styled("esc", dialog_accent()),
        Span::raw(" cancel"),
    ];
    let mut lines = Vec::new();
    if run_options {
        if width < 51 {
            lines.push(Line::from(spans));
            spans = Vec::new();
        } else {
            spans.push(Span::raw("  "));
        }
        spans.extend([Span::styled("o", dialog_accent()), Span::raw(" options")]);
    }
    if scrolls {
        let hint = if run_options || width >= 40 {
            "  ↑↓ scroll"
        } else {
            "  ↑↓"
        };
        spans.push(Span::styled(hint, dialog_supporting()));
    }
    lines.push(Line::from(spans));
    Text::from(lines)
}

fn warning_line(warning: &CommandPreviewWarning) -> Line<'_> {
    Line::from(Span::styled(
        format!("! {}", warning_label(warning)),
        dialog_warning(),
    ))
}

fn warning_label(warning: &CommandPreviewWarning) -> String {
    match warning {
        CommandPreviewWarning::LocalMetadata => "Changes local repository metadata.".to_owned(),
        CommandPreviewWarning::LocalRewrite => "Rewrites local history.".to_owned(),
        CommandPreviewWarning::DestructiveLocal => "Changes local files or history.".to_owned(),
        CommandPreviewWarning::NetworkWrite => "Writes to a remote service.".to_owned(),
        CommandPreviewWarning::ExternalCommand => "Runs another program.".to_owned(),
        CommandPreviewWarning::IgnoresWorkingCopy => {
            "Ignores the current working-copy snapshot.".to_owned()
        }
        CommandPreviewWarning::AtOperation(operation) => format!("Runs at operation {operation}."),
        CommandPreviewWarning::DoesNotIntegrateOperation => {
            "Does not integrate the loaded operation.".to_owned()
        }
        CommandPreviewWarning::IgnoresImmutableCommits => {
            "May rewrite immutable commits.".to_owned()
        }
        _ => "Review this command before running.".to_owned(),
    }
}

fn text_height(text: &Text<'_>, content_width: u16) -> u16 {
    // Match Paragraph's word wrapping, not a character-count estimate: role labels and long command
    // arguments can wrap earlier than the right edge.
    let text_height = Paragraph::new(text.clone())
        .wrap(Wrap { trim: false })
        .line_count(content_width.max(1));
    text_height.try_into().unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use jk_core::{GlobalOptions, JjCommandSpec, SafetyClass, WorkingCopyPolicy};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::style::{Color, Style};

    use super::*;

    #[test]
    fn preview_orders_operands_consequences_and_exact_command() {
        let global_options = GlobalOptions::default().with_working_copy(WorkingCopyPolicy::Ignore);
        let preview = JjCommandSpec::confirm_mutation(
            ["describe", "--message", "Update preview renderer"],
            SafetyClass::LocalRewrite,
        )
        .with_global_options(global_options)
        .with_title("Describe workspace")
        .command_preview();
        let view = CommandPreviewView::new(preview).with_details(vec!["Revision: @".to_owned()]);
        let buffer = render(&view, 120, 30);
        let rendered = buffer_to_string(&buffer);

        let title = rendered.find("Describe workspace");
        let operand = rendered.find("Revision: @");
        let consequence = rendered.find("Rewrites local history.");
        let command = rendered.find("jj --no-pager --color always --ignore-working-copy describe");
        assert!(matches!(
            (title, operand, consequence, command),
            (Some(title), Some(operand), Some(consequence), Some(command))
                if title < operand && operand < consequence && consequence < command
        ));
        assert!(rendered.contains("'Update preview renderer'"));
        assert!(rendered.contains("Ignores the current working-copy snapshot."));
        assert!(rendered.contains("enter run  y copy  esc cancel"));
        assert!(!rendered.contains("Confirm command"));
        assert!(!rendered.contains("Effect:"));
        assert!(!rendered.contains("Safety"));
        assert!(!rendered.contains("Backend"));
    }

    #[test]
    fn read_only_preview_does_not_add_generic_warning_chrome() {
        let preview = JjCommandSpec::render_read_only(["log"])
            .with_title("Refresh log")
            .command_preview();
        let view = CommandPreviewView::new(preview);
        let buffer = render(&view, 80, 14);
        let rendered = buffer_to_string(&buffer);

        assert!(rendered.contains("jj --no-pager --color always log"));
        assert!(!rendered.contains("warnings"));
        assert!(!rendered.contains("Effect:"));
        assert!(rendered.contains("enter run  y copy  esc cancel"));
        assert!(!rendered.contains("↑↓"));
    }

    #[test]
    fn copy_status_keeps_controls_and_panel_position_stable() {
        let preview = JjCommandSpec::render_read_only(["undo"])
            .with_title("Undo latest operation")
            .command_preview();
        let view = CommandPreviewView::new(preview);
        let area = Rect::new(0, 0, 80, 14);
        let panel = view.panel(area, &view.body());
        let copied = view.with_status(Some("copied command".to_owned()));
        assert_eq!(panel, copied.panel(area, &copied.body()));
        let buffer = render(&copied, area.width, area.height);
        let rendered = buffer_to_string(&buffer);

        assert!(rendered.contains("copied command"));
        assert!(rendered.contains("enter run  y copy  esc cancel"));
    }

    #[test]
    fn preview_has_opaque_dialog_colors_without_borders() {
        let preview = JjCommandSpec::confirm_mutation(
            ["restore", "--from", "abc123", "--into", "@"],
            SafetyClass::DestructiveLocal,
        )
        .with_title("Restore all paths")
        .command_preview();
        let view = CommandPreviewView::new(preview).with_details(vec![
            "Source: abc123".to_owned(),
            "Destination: @ (working copy)".to_owned(),
            "Affected content: all paths".to_owned(),
        ]);
        let buffer = render(&view, 100, 30);
        let rendered = buffer_to_string(&buffer);

        assert!(!rendered.chars().any(|symbol| "┌┐└┘─│".contains(symbol)));
        let panel = view.panel(Rect::new(0, 0, 100, 30), &view.body());
        for y in buffer.area.top()..buffer.area.bottom() {
            for x in buffer.area.left()..buffer.area.right() {
                let expected = if panel.contains((x, y).into()) {
                    dialog().bg
                } else {
                    Some(Color::Reset)
                };
                assert_eq!(Some(buffer[(x, y)].bg), expected);
            }
        }
        assert!(
            buffer
                .content
                .iter()
                .any(|cell| Some(cell.fg) == dialog_warning().fg)
        );
        assert!(buffer.content.iter().all(|cell| cell.fg == Color::Reset
            || Some(cell.fg) == dialog().fg
            || Some(cell.fg) == dialog_warning().fg
            || Some(cell.fg) == dialog_accent().fg
            || Some(cell.fg) == dialog_supporting().fg));
    }

    #[test]
    fn short_preview_sizes_to_content() {
        let preview = JjCommandSpec::render_read_only(["log"])
            .with_title("Refresh log")
            .command_preview();
        let view = CommandPreviewView::new(preview);
        let area = Rect::new(0, 0, 120, 30);
        let body = view.body();
        let panel = view.panel(area, &body);

        assert!(panel.width < MAX_PANEL_WIDTH);
        assert_eq!(
            panel.height,
            text_height(&body, panel.width - PANEL_PADDING) + CHROME_HEIGHT
        );
        assert_eq!(panel.x, (area.width - panel.width) / 2);
        assert_eq!(panel.y, (area.height - panel.height) / 2);
    }

    #[test]
    fn short_preview_exposes_scrolling_and_keeps_controls_visible() {
        let preview = JjCommandSpec::confirm_mutation(
            ["squash", "--from", "abc123", "--into", "@"],
            SafetyClass::LocalRewrite,
        )
        .with_title("Squash changes")
        .command_preview();
        let view = CommandPreviewView::new(preview).with_details(vec![
            "Source: abc123".to_owned(),
            "Destination: @".to_owned(),
        ]);
        let area = Rect::new(0, 0, 78, 10);
        let panel = view.panel(area, &view.body());
        let buffer = render(&view, area.width, area.height);
        let rendered = buffer_to_string(&buffer);
        assert!(rendered.contains("Source: abc123"));
        assert!(rendered.contains("Rewrites local history."));
        assert!(rendered.contains("enter run  y copy  esc cancel  ↑↓ scroll"));

        let scroll = view.max_scroll(area);
        let scrolled = view.with_scroll(scroll);
        assert_eq!(panel, scrolled.panel(area, &scrolled.body()));
        let buffer = render(&scrolled, area.width, area.height);
        let rendered = buffer_to_string(&buffer);
        assert!(rendered.contains("jj --no-pager --color always squash --from abc123 --into @"));
        assert!(rendered.contains("enter run  y copy  esc cancel"));
    }

    #[test]
    fn narrow_preview_keeps_the_complete_command_inspectable() {
        let long_argument = format!("start{}finish", "abcdefghij".repeat(15));
        let preview = JjCommandSpec::confirm_mutation(
            ["describe", "--message", long_argument.as_str()],
            SafetyClass::LocalRewrite,
        )
        .with_title("Describe change")
        .command_preview();
        let expected = without_whitespace(&preview.command_line);
        let view = CommandPreviewView::new(preview);
        for (width, height) in [(40, 8), (60, 16), (80, 24)] {
            let area = Rect::new(0, 0, width, height);
            let max_scroll = view.max_scroll(area);
            let panel = view.panel(area, &view.body());
            let body_top = panel.y + 2;
            let body_height = panel.height - CHROME_HEIGHT;
            let mut all_body_lines = String::new();
            if width == MIN_CONFIRM_WIDTH {
                assert!(max_scroll > 0);
            }

            for scroll in 0..=max_scroll {
                let scrolled = view.clone().with_scroll(scroll);
                let buffer = render(&scrolled, area.width, area.height);
                let rendered = buffer_to_string(&buffer);
                assert!(rendered.contains("enter run  y copy  esc cancel"));
                if max_scroll > 0 {
                    assert!(rendered.contains("↑↓"));
                }
                let rows = if scroll == max_scroll { body_height } else { 1 };
                for y in body_top..body_top + rows {
                    for x in panel.x + 1..panel.right() - 1 {
                        all_body_lines.push_str(buffer[(x, y)].symbol());
                    }
                }
            }

            assert!(without_whitespace(&all_body_lines).contains(&expected));
        }
    }

    #[test]
    fn compact_preview_preserves_repository_context_and_visible_controls() {
        let preview = JjCommandSpec::confirm_mutation(
            ["squash", "--from", "source", "--into", "destination"],
            SafetyClass::LocalRewrite,
        )
        .command_preview();
        let view = CommandPreviewView::new(preview)
            .with_run_options(true)
            .with_details(vec![
                "Source: a full revision identifier to review"
                    .to_owned();
                12
            ]);
        for (width, height) in [(60, 16), (80, 24)] {
            let area = Rect::new(0, 0, width, height);
            let panel = view.panel(area, &view.body());
            assert!(panel.x >= 2 && panel.y >= 2);
            assert!(panel.right() <= width - 2 && panel.bottom() <= height - 2);
            let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
            terminal
                .draw(|frame| {
                    frame.render_widget(
                        Paragraph::new(Text::from(vec![
                            Line::raw("jk jj log"),
                            Line::styled(
                                "@ source  keep this repository context",
                                Style::new().fg(Color::Green),
                            ),
                        ])),
                        frame.area(),
                    );
                    view.render(frame);
                })
                .expect("preview over repository");
            let buffer = terminal.backend().buffer();
            assert_eq!(buffer[(0, 0)].symbol(), "j");
            assert_eq!(buffer[(0, 1)].symbol(), "@");
            assert_eq!(buffer[(0, 1)].fg, Color::Green);
            assert_eq!(buffer[(0, 1)].bg, Color::Reset);
            assert_eq!(Some(buffer[(panel.x, panel.y)].bg), dialog().bg);
            let rendered = buffer_to_string(buffer);
            assert!(rendered.contains("enter run  y copy  esc cancel"));
            assert!(rendered.contains("o options"));
            assert!(rendered.contains("↑↓ scroll"));
        }
    }

    #[test]
    fn overlay_clears_panel_text_and_retains_surrounding_repository() {
        let preview = JjCommandSpec::render_read_only(["log"]).command_preview();
        let view = CommandPreviewView::new(preview);
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    Paragraph::new("x".repeat(80 * 20)).wrap(Wrap { trim: false }),
                    frame.area(),
                );
                view.render(frame);
            })
            .unwrap();
        let panel = view.panel(Rect::new(0, 0, 80, 20), &view.body());
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(0, 0)].symbol(), "x");
        assert_eq!(buffer[(0, 0)].bg, Color::Reset);
        for y in panel.top()..panel.bottom() {
            for x in panel.left()..panel.right() {
                assert_ne!(buffer[(x, y)].symbol(), "x");
                assert_eq!(Some(buffer[(x, y)].bg), dialog().bg);
            }
        }
    }

    #[test]
    fn run_options_stay_readable_beside_primary_controls_at_narrow_widths() {
        let preview = JjCommandSpec::confirm_mutation(
            ["rebase", "--source", "abc123", "--destination", "xyz789"],
            SafetyClass::LocalRewrite,
        )
        .with_title("Rebase changes")
        .command_preview();
        let view = CommandPreviewView::new(preview).with_run_options(true);

        for width in [40, 50, 60, 80] {
            let area = Rect::new(0, 0, width, MIN_CONFIRM_HEIGHT);
            assert!(view.can_confirm(area));
            assert!(view.max_scroll(area) > 0);
            let buffer = render(&view, width, area.height);
            let rendered = buffer_to_string(&buffer);
            assert!(rendered.contains("enter run  y copy  esc cancel"));
            assert!(rendered.contains("o options"));
            assert!(rendered.contains("↑↓ scroll"));
            let panel = view.panel(area, &view.body());
            let footer = footer_text(true, true, panel.width - PANEL_PADDING);
            assert!(
                footer
                    .lines
                    .iter()
                    .all(|line| line.width() <= usize::from(panel.width - PANEL_PADDING))
            );
        }
    }

    #[test]
    fn tiny_viewport_cannot_confirm() {
        let preview = JjCommandSpec::render_read_only(["log"]).command_preview();
        let view = CommandPreviewView::new(preview);

        assert!(view.can_confirm(Rect::new(0, 0, MIN_CONFIRM_WIDTH, MIN_CONFIRM_HEIGHT)));
        for (width, height) in [(MIN_CONFIRM_WIDTH - 1, 20), (80, MIN_CONFIRM_HEIGHT - 1)] {
            assert!(!view.can_confirm(Rect::new(0, 0, width, height)));
            let rendered = buffer_to_string(&render(&view, width, height));
            assert!(rendered.contains("Enlarge terminal to review."));
            assert!(rendered.contains("Esc cancel"));
            assert!(!rendered.contains("enter run"));
        }
    }

    fn render(view: &CommandPreviewView, width: u16, height: u16) -> Buffer {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| view.render(frame)).unwrap();
        terminal.backend().buffer().clone()
    }

    fn without_whitespace(text: &str) -> String {
        text.chars()
            .filter(|character| !character.is_whitespace())
            .collect()
    }

    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut text = String::new();
        for y in buffer.area.top()..buffer.area.bottom() {
            for x in buffer.area.left()..buffer.area.right() {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }
}
