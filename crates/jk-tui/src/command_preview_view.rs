//! Draw-only confirmation preview for commands that may change state.
//!
//! This view renders [`jk_core::CommandPreview`] data and intentionally owns no execution behavior.

use jk_core::{CommandPreview, CommandPreviewWarning, SafetyClass};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Wrap};

const PANEL_WIDTH: u16 = 78;
const MIN_PANEL_HEIGHT: u16 = 5;
const MIN_CONFIRM_WIDTH: u16 = 40;
const MIN_CONFIRM_HEIGHT: u16 = 8;
const BACKGROUND: Color = Color::Rgb(30, 35, 47);
const SURFACE: Color = Color::Rgb(40, 45, 55);
const DETAILS_REGION: Color = Color::Rgb(35, 51, 62);
const ACTION_REGION: Color = Color::Rgb(58, 72, 90);
const WARNING_REGION: Color = Color::Rgb(70, 39, 43);
const SUCCESS_REGION: Color = Color::Rgb(35, 68, 55);
const MUTED: Color = Color::Rgb(161, 174, 190);

struct PreviewRegion<'a> {
    text: Text<'a>,
    background: Color,
}

/// A compact confirmation view for a pending command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandPreviewView {
    preview: CommandPreview,
    status: Option<String>,
    details: Vec<String>,
    scroll: u16,
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

    /// Returns the greatest rendered-line offset available in `area`.
    #[must_use]
    pub fn max_scroll(&self, area: Rect) -> u16 {
        let panel_width = PANEL_WIDTH.min(area.width);
        let content_width = panel_width.saturating_sub(4).max(1);
        let content_height = self.content_height(content_width);
        let panel = centered_panel(area, content_height.saturating_add(2));
        let body_height = panel.height.saturating_sub(2);
        content_height.saturating_sub(body_height)
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
                Paragraph::new("Enlarge terminal to review.\nEsc cancel")
                    .style(Style::new().fg(Color::White).bg(BACKGROUND)),
                area,
            );
            return;
        }

        let panel_width = PANEL_WIDTH.min(area.width);
        let content_width = panel_width.saturating_sub(4).max(1);
        let regions = self.regions();
        let content_height = regions
            .iter()
            .map(|region| text_height(&region.text, content_width))
            .sum::<u16>();
        let panel = centered_panel(area, content_height.saturating_add(2));
        frame.render_widget(Clear, panel);
        frame.render_widget(Block::default().style(Style::new().bg(BACKGROUND)), panel);

        if panel.width < 2 || panel.height < 3 {
            return;
        }

        let header_area = Rect::new(panel.x, panel.y, panel.width, 1);
        let footer_area = Rect::new(panel.x, panel.bottom().saturating_sub(1), panel.width, 1);
        frame.render_widget(
            Paragraph::new(header_line(&self.preview))
                .style(Style::new().bg(ACTION_REGION))
                .block(Block::default().padding(Padding::horizontal(2))),
            header_area,
        );

        let body_area = Rect::new(
            panel.x,
            header_area.bottom(),
            panel.width,
            panel.height.saturating_sub(2),
        );
        self.render_regions(frame, body_area, regions, content_width, content_height);

        let mut footer = footer_line(self.status.as_deref());
        if self.max_scroll(area) > 0 && panel.width >= 58 {
            footer
                .spans
                .push(Span::styled("    ↑↓ scroll", Style::new().fg(MUTED)));
        }
        frame.render_widget(
            Paragraph::new(footer)
                .style(Style::new().bg(ACTION_REGION))
                .block(Block::default().padding(Padding::horizontal(2))),
            footer_area,
        );
    }

    fn render_regions(
        &self,
        frame: &mut Frame<'_>,
        body_area: Rect,
        regions: Vec<PreviewRegion<'_>>,
        content_width: u16,
        content_height: u16,
    ) {
        let scroll = self
            .scroll
            .min(content_height.saturating_sub(body_area.height));
        let visible_end = scroll.saturating_add(body_area.height);
        let mut region_top: u16 = 0;

        for region in regions {
            let region_height = text_height(&region.text, content_width);
            let region_bottom = region_top.saturating_add(region_height);
            let visible_top = region_top.max(scroll);
            let visible_bottom = region_bottom.min(visible_end);

            if visible_top < visible_bottom {
                let area = Rect::new(
                    body_area.x,
                    body_area
                        .y
                        .saturating_add(visible_top.saturating_sub(scroll)),
                    body_area.width,
                    visible_bottom.saturating_sub(visible_top),
                );
                render_region(frame, area, region, visible_top.saturating_sub(region_top));
            }

            region_top = region_bottom;
        }
    }

    fn content_height(&self, content_width: u16) -> u16 {
        self.regions()
            .iter()
            .map(|region| text_height(&region.text, content_width))
            .sum()
    }

    fn regions(&self) -> Vec<PreviewRegion<'_>> {
        let mut regions = vec![PreviewRegion {
            text: Text::from(vec![
                section_heading("Command"),
                Line::from(Span::styled(
                    self.preview.command_line.as_str(),
                    Style::new().fg(Color::Yellow),
                )),
            ]),
            background: SURFACE,
        }];

        // Warnings come before optional details so a short terminal shows the consequences first.
        if self.preview.warnings.is_empty() {
            regions.push(PreviewRegion {
                text: Text::from(Line::from(Span::styled(
                    "No warnings for this command.",
                    Style::new().fg(Color::LightGreen),
                ))),
                background: SUCCESS_REGION,
            });
        } else {
            let mut lines = vec![section_heading("Warnings")];
            lines.extend(self.preview.warnings.iter().map(warning_line));
            regions.push(PreviewRegion {
                text: Text::from(lines),
                background: WARNING_REGION,
            });
        }

        if !self.details.is_empty() {
            let mut lines = vec![section_heading("Details")];
            lines.extend(self.details.iter().map(|detail| {
                Line::from(Span::styled(detail.as_str(), Style::new().fg(Color::White)))
            }));
            regions.push(PreviewRegion {
                text: Text::from(lines),
                background: DETAILS_REGION,
            });
        }

        regions.push(PreviewRegion {
            text: Text::from(effect_line(self.preview.safety)),
            background: SURFACE,
        });

        regions
    }
}

fn header_line(preview: &CommandPreview) -> Line<'_> {
    Line::from(vec![
        Span::styled(
            "Confirm command",
            Style::new()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ·  ", Style::new().fg(MUTED)),
        Span::styled(
            preview.title.as_str(),
            Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn section_heading(label: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        label,
        Style::new()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn render_region(frame: &mut Frame<'_>, area: Rect, region: PreviewRegion<'_>, scroll: u16) {
    if area.is_empty() {
        return;
    }

    let style = Style::new().fg(Color::White).bg(region.background);
    frame.render_widget(Block::default().style(style), area);
    frame.render_widget(
        Paragraph::new(region.text)
            .style(style)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0))
            .block(Block::default().padding(Padding::horizontal(2))),
        area,
    );
}

fn footer_line(status: Option<&str>) -> Line<'static> {
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
            "enter run",
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled(
            "y copy",
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled(
            "esc cancel",
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn warning_line(warning: &CommandPreviewWarning) -> Line<'static> {
    Line::from(vec![
        Span::styled("! ", Style::new().fg(Color::LightRed)),
        Span::styled(warning_label(warning), Style::new().fg(Color::White)),
    ])
}

fn effect_line(safety: SafetyClass) -> Line<'static> {
    Line::from(vec![
        Span::styled("Effect: ", Style::new().fg(MUTED)),
        Span::styled(effect_label(safety), Style::new().fg(effect_color(safety))),
    ])
}

const fn effect_color(safety: SafetyClass) -> Color {
    match safety {
        SafetyClass::DestructiveLocal | SafetyClass::NetworkWrite => Color::LightRed,
        SafetyClass::LocalRewrite | SafetyClass::ExternalCommand => Color::Yellow,
        SafetyClass::ReadOnly | SafetyClass::NetworkRead => Color::Green,
        _ => Color::White,
    }
}

const fn effect_label(safety: SafetyClass) -> &'static str {
    match safety {
        SafetyClass::ReadOnly => "reads information only",
        SafetyClass::LocalMetadata => "changes repository metadata",
        SafetyClass::LocalRewrite => "changes local history",
        SafetyClass::DestructiveLocal => "changes local files or history",
        SafetyClass::NetworkRead => "reads from a remote service",
        SafetyClass::NetworkWrite => "sends changes to a remote service",
        SafetyClass::ExternalCommand => "runs another program",
        _ => "review before running",
    }
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
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| view.render(frame)).unwrap();

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Confirm command"));
        assert!(rendered.contains("Describe workspace"));
        assert!(rendered.contains("jj --no-pager --color always --ignore-working-copy describe"));
        assert!(rendered.contains("--message"));
        assert!(rendered.contains("'Update preview renderer'"));
        assert!(rendered.contains("Effect: changes local history"));
        assert!(rendered.contains("Rewrites local history."));
        assert!(rendered.contains("Ignores the current working-copy snapshot."));
        assert!(rendered.contains("enter"));
        assert!(rendered.contains("run"));
        assert!(rendered.contains('y'));
        assert!(rendered.contains("copy"));
        assert!(rendered.contains("esc"));
        assert!(rendered.contains("cancel"));
        assert!(!rendered.contains('┌'));
        assert!(!rendered.contains('┐'));
        assert!(
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .any(|cell| cell.bg == ACTION_REGION)
        );
        assert!(
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .any(|cell| cell.bg == WARNING_REGION)
        );
    }

    #[test]
    fn command_preview_without_warnings_says_so() {
        let preview = JjCommandSpec::render_read_only(["log"])
            .with_title("Refresh log")
            .command_preview();
        let view = CommandPreviewView::new(preview);
        let backend = TestBackend::new(80, 14);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| view.render(frame)).unwrap();

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("jj --no-pager --color always log"));
        assert!(rendered.contains("Effect: reads information only"));
        assert!(rendered.contains("No warnings for this command."));
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
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| view.render(frame)).unwrap();

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("copied command"));
        assert!(rendered.contains("enter run"));
        assert!(!rendered.contains("y copy"));
    }

    #[test]
    fn command_preview_uses_filled_regions_without_borders() {
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
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| view.render(frame)).unwrap();

        let buffer = terminal.backend().buffer();
        let rendered = buffer_to_string(buffer);
        assert!(!rendered.chars().any(|symbol| "┌┐└┘─│".contains(symbol)));
        assert!(background_count(buffer, ACTION_REGION) > 0);
        assert!(background_count(buffer, SURFACE) > 0);
        assert!(background_count(buffer, WARNING_REGION) > 0);
        assert!(background_count(buffer, DETAILS_REGION) > 0);
    }

    #[test]
    fn compact_preview_keeps_command_warnings_and_controls_visible() {
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
        let backend = TestBackend::new(78, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|frame| view.render(frame)).unwrap();

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("jj --no-pager --color always squash --from abc123 --into @"));
        assert!(rendered.contains("Rewrites local history."));
        assert!(rendered.contains("enter run"));
        assert!(rendered.contains("y copy"));
        assert!(rendered.contains("esc cancel"));
        assert!(rendered.contains("Source: abc123"));
    }

    #[test]
    fn preview_scrolls_to_hidden_details_without_hiding_controls() {
        let preview = JjCommandSpec::confirm_mutation(["squash"], SafetyClass::LocalRewrite)
            .with_title("Squash changes")
            .command_preview();
        let area = Rect::new(0, 0, 78, 8);
        let view = CommandPreviewView::new(preview).with_details(vec![
            "Source: abc123".to_owned(),
            "Destination: @".to_owned(),
            "Affected content: selected changes".to_owned(),
        ]);
        assert!(view.max_scroll(area) > 0);

        let backend = TestBackend::new(area.width, area.height);
        let mut terminal = Terminal::new(backend).unwrap();
        let scroll = view.max_scroll(area);
        terminal
            .draw(|frame| view.with_scroll(scroll).render(frame))
            .unwrap();

        let rendered = buffer_to_string(terminal.backend().buffer());
        assert!(rendered.contains("Affected content: selected changes"));
        assert!(rendered.contains("enter run"));
        assert!(rendered.contains("esc cancel"));
    }

    #[test]
    fn tiny_viewport_cannot_confirm() {
        let preview = JjCommandSpec::render_read_only(["log"]).command_preview();
        let view = CommandPreviewView::new(preview);

        assert!(view.can_confirm(Rect::new(0, 0, MIN_CONFIRM_WIDTH, MIN_CONFIRM_HEIGHT)));
        assert!(!view.can_confirm(Rect::new(0, 0, MIN_CONFIRM_WIDTH - 1, 20)));
        assert!(!view.can_confirm(Rect::new(0, 0, 80, MIN_CONFIRM_HEIGHT - 1)));
    }

    fn background_count(buffer: &ratatui::buffer::Buffer, color: Color) -> usize {
        let area = buffer.area;
        let mut count = 0;
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if buffer[(x, y)].bg == color {
                    count += 1;
                }
            }
        }
        count
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
