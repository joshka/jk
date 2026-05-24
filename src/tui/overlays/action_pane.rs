use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;

use crate::app::actions::ActionPane;
use crate::tui::overlays::menus::{centered_area, overlay_block};
use crate::tui::theme;

#[derive(Clone, Copy)]
enum ActionPaneMode {
    Preview,
    Result,
}

/// Choose the preview versus result title suffix for an action pane.
pub fn action_pane_title(action: &str, output: &ActionPane) -> String {
    if matches!(action_pane_mode(output), ActionPaneMode::Result) {
        format!("{action} result")
    } else {
        format!("{action} preview")
    }
}

/// Render the shared preview/result pane body plus footer into the given modal area.
pub fn render_action_pane(frame: &mut Frame, area: Rect, title: &str, output: &ActionPane) {
    let block = overlay_block(title.to_owned());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let footer_height = u16::from(inner.height > 1);
    let body_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(footer_height),
    };
    if body_area.height > 0 {
        let scroll = output.scroll().min(usize::from(u16::MAX)) as u16;
        frame.render_widget(
            ratatui::widgets::Paragraph::new(output.body_lines().join("\n")).scroll((scroll, 0)),
            body_area,
        );
    }

    if footer_height > 0 {
        let footer_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(action_pane_footer(action_pane_mode(output)), footer_area);
    }
}

/// Render the abandon confirmation overlay, reusing the action-pane body above a typed footer.
pub fn render_abandon_confirm(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    input: &str,
    output: &ActionPane,
) {
    let block = overlay_block(title.to_owned());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let footer_height = u16::from(inner.height > 1);
    let body_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: inner.height.saturating_sub(footer_height),
    };
    if body_area.height > 0 {
        let scroll = output.scroll().min(usize::from(u16::MAX)) as u16;
        frame.render_widget(
            ratatui::widgets::Paragraph::new(output.body_lines().join("\n")).scroll((scroll, 0)),
            body_area,
        );
    }

    if footer_height > 0 {
        let footer_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(abandon_confirm_footer(input), footer_area);
    }
}

pub fn action_pane_area(area: Rect, title: &str, output: &ActionPane) -> Rect {
    let footer = action_pane_footer_text(action_pane_mode(output));
    action_pane_area_with_footer(area, title, output, &footer)
}

pub fn action_pane_area_with_footer(
    area: Rect,
    title: &str,
    output: &ActionPane,
    footer: &str,
) -> Rect {
    let lines = output.body_lines();
    let width = lines
        .iter()
        .map(|line| line_width(line))
        .chain([line_width(footer), line_width(title)])
        .max()
        .unwrap_or(0)
        .max(44)
        .min(usize::from(area.width)) as u16;
    let available_body_height = area.height.saturating_sub(3);
    let body_height = lines.len().min(usize::from(available_body_height)).max(1) as u16;
    let height = body_height.saturating_add(3).min(area.height);
    centered_area(area, width, height)
}

pub fn abandon_confirm_footer_text(input: &str) -> String {
    format!("type exact id: {input}  Enter confirm  Esc cancel  arrows/page scroll")
}

fn action_pane_footer(mode: ActionPaneMode) -> ratatui::widgets::Paragraph<'static> {
    let primary = if matches!(mode, ActionPaneMode::Result) {
        keyed_line("Enter", "close")
    } else {
        keyed_line("Enter", "confirm")
    };
    let mut spans = primary.spans;
    if matches!(mode, ActionPaneMode::Result) {
        spans.extend(keyed_line("Esc/q", "close").spans);
    } else {
        spans.extend(keyed_line("Esc/q", "cancel").spans);
    }
    spans.extend(keyed_line("j/k", "scroll").spans);
    spans.extend(keyed_line("PgUp/PgDn", "page").spans);
    spans.extend(keyed_line("g/G", "ends").spans);

    ratatui::widgets::Paragraph::new(Line::from(spans)).style(theme::muted_style())
}

fn abandon_confirm_footer(input: &str) -> ratatui::widgets::Paragraph<'static> {
    ratatui::widgets::Paragraph::new(Line::from(abandon_confirm_footer_text(input)))
        .style(theme::muted_style())
}

fn action_pane_footer_text(mode: ActionPaneMode) -> String {
    if matches!(mode, ActionPaneMode::Result) {
        "Enter close  Esc/q close  j/k scroll  PgUp/PgDn page  g/G ends".to_owned()
    } else {
        "Enter confirm  Esc/q cancel  j/k scroll  PgUp/PgDn page  g/G ends".to_owned()
    }
}

fn action_pane_mode(output: &ActionPane) -> ActionPaneMode {
    if output.completed() {
        ActionPaneMode::Result
    } else {
        ActionPaneMode::Preview
    }
}

fn keyed_line(key: &'static str, label: &'static str) -> Line<'static> {
    let mut spans = Vec::new();
    spans.push(ratatui::text::Span::styled(
        key.to_owned(),
        theme::key_style(),
    ));
    spans.push(ratatui::text::Span::raw(format!(" {label}  ")));
    Line::from(spans)
}

fn line_width(line: &str) -> usize {
    line.chars().count()
}
