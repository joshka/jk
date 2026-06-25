use jk_cli::LogTemplateSelection;
use jk_tui::command_discovery::{BindingContext, discovery_lines_for_width_and_rows};
use jk_tui::command_preview_view::CommandPreviewView;
use ratatui::prelude::{Color, Line, Modifier, Span, Style};

use crate::command_mode::jj_command_lines;
use crate::menus::{diff_file_list_lines, template_selector_lines, view_options_lines};
use crate::mutation_preview::describe_message_lines;
use crate::state::{AppState, AppView, InputMode};

pub fn render_app(
    frame: &mut ratatui::Frame<'_>,
    state: &mut AppState,
    template: &LogTemplateSelection,
) {
    let mode = state.modes.active().cloned();
    match state.views.active_mut() {
        AppView::Log(log) => match &mode {
            Some(InputMode::ViewOptions { context, selected }) => {
                let lines = view_options_lines(*context, *selected, template, None);
                log.render_with_selector(frame, "View Options", &lines);
            }
            Some(InputMode::LogTemplate { options, selected }) => {
                let lines = template_selector_lines(options, *selected);
                log.render_with_selector(frame, "Log template", &lines);
            }
            Some(InputMode::CommandDiscovery {
                context,
                query,
                scroll_offset,
            }) => {
                log.render(frame);
                render_command_discovery_overlay(frame, *context, query, *scroll_offset);
            }
            Some(InputMode::JjCommand { input, error }) => {
                log.render(frame);
                let lines = jj_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "jj command", &lines);
            }
            Some(InputMode::DescribeMessage { rev, message }) => {
                log.render(frame);
                let lines = describe_message_lines(rev, message);
                render_mode_overlay(frame, "Describe revision", &lines);
            }
            Some(InputMode::CommandPreview { pending }) => {
                log.render(frame);
                CommandPreviewView::new(pending.preview.clone())
                    .with_status(pending.copy_status.clone())
                    .render(frame);
            }
            _ => log.render(frame),
        },
        AppView::Diff { view, query } => match &mode {
            Some(InputMode::ViewOptions { context, selected }) => {
                let lines = view_options_lines(*context, *selected, template, Some(query.format()));
                view.render_with_overlay(frame, "View Options", &lines);
            }
            Some(InputMode::DiffFileList { selected }) => {
                let lines = diff_file_list_lines(view, *selected);
                view.render_with_overlay(frame, "Diff files", &lines);
            }
            Some(InputMode::DiffSearch { query }) => {
                let status = format!("/{query}");
                view.render_with_status(frame, &status);
            }
            Some(InputMode::CommandDiscovery {
                context,
                query,
                scroll_offset,
            }) => {
                view.render(frame);
                render_command_discovery_overlay(frame, *context, query, *scroll_offset);
            }
            Some(InputMode::JjCommand { input, error }) => {
                let lines = jj_command_lines(input, error.as_deref());
                view.render_with_overlay(frame, "jj command", &lines);
            }
            _ => view.render(frame),
        },
        AppView::Show { view, .. } => render_inspection(frame, view, &mode, template),
        AppView::Evolog { view, .. } => render_inspection(frame, view, &mode, template),
        AppView::Status { view, .. } => render_inspection(frame, view, &mode, template),
        AppView::Workspaces { view } => match &mode {
            Some(InputMode::ViewOptions { context, selected }) => {
                let lines = view_options_lines(*context, *selected, template, None);
                view.render(frame);
                render_mode_overlay(frame, "View Options", &lines);
            }
            Some(InputMode::CommandDiscovery {
                context,
                query,
                scroll_offset,
            }) => {
                view.render(frame);
                render_command_discovery_overlay(frame, *context, query, *scroll_offset);
            }
            Some(InputMode::JjCommand { input, error }) => {
                view.render(frame);
                let lines = jj_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "jj command", &lines);
            }
            _ => view.render(frame),
        },
        AppView::CommandHistory { view } => match &mode {
            Some(InputMode::CommandDiscovery {
                context,
                query,
                scroll_offset,
            }) => {
                view.render(frame);
                render_command_discovery_overlay(frame, *context, query, *scroll_offset);
            }
            Some(InputMode::JjCommand { input, error }) => {
                view.render(frame);
                let lines = jj_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "jj command", &lines);
            }
            _ => view.render(frame),
        },
        AppView::OperationLog { view } => match &mode {
            Some(InputMode::ViewOptions { context, selected }) => {
                let lines = view_options_lines(*context, *selected, template, None);
                view.render(frame);
                render_mode_overlay(frame, "View Options", &lines);
            }
            Some(InputMode::CommandDiscovery {
                context,
                query,
                scroll_offset,
            }) => {
                view.render(frame);
                render_command_discovery_overlay(frame, *context, query, *scroll_offset);
            }
            Some(InputMode::JjCommand { input, error }) => {
                view.render(frame);
                let lines = jj_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "jj command", &lines);
            }
            _ => view.render(frame),
        },
        AppView::CommandHistoryDetails { view }
        | AppView::CommandOutput { view, .. }
        | AppView::WorkspaceLog { view, .. }
        | AppView::WorkspaceStatus { view, .. }
        | AppView::WorkspaceDiff { view, .. }
        | AppView::OperationShow { view, .. }
        | AppView::OperationDiff { view, .. } => render_inspection(frame, view, &mode, template),
    }
}

fn render_inspection(
    frame: &mut ratatui::Frame<'_>,
    view: &mut jk_tui::rendered_view::RenderedView,
    mode: &Option<InputMode>,
    template: &LogTemplateSelection,
) {
    match mode {
        Some(InputMode::ViewOptions { context, selected }) => {
            let lines = view_options_lines(*context, *selected, template, None);
            view.render_with_overlay(frame, "View Options", &lines);
        }
        Some(InputMode::InspectionSearch { query }) => {
            let status = format!("/{query}");
            view.render_with_status(frame, &status);
        }
        Some(InputMode::CommandDiscovery {
            context,
            query,
            scroll_offset,
        }) => {
            view.render(frame);
            render_command_discovery_overlay(frame, *context, query, *scroll_offset);
        }
        Some(InputMode::JjCommand { input, error }) => {
            let lines = jj_command_lines(input, error.as_deref());
            view.render_with_overlay(frame, "jj command", &lines);
        }
        _ => view.render(frame),
    }
}

fn render_command_discovery_overlay(
    frame: &mut ratatui::Frame<'_>,
    context: BindingContext,
    query: &str,
    scroll_offset: usize,
) {
    let content_width = command_discovery_format_width(frame);
    let visible_rows = command_discovery_visible_rows(content_width, frame.area().height);
    let lines = discovery_lines_for_width_and_rows(
        context,
        query,
        scroll_offset,
        content_width,
        visible_rows,
    );
    // Size from the full rendered document, not the current scroll slice, so the Help box does not
    // jump wider/narrower as different lines scroll into view.
    let sizing_lines =
        discovery_lines_for_width_and_rows(context, query, 0, content_width, usize::MAX);
    // The overlay body lists close and quit explicitly. Hiding the underlying hotbar prevents
    // duplicated controls from competing with the help content.
    clear_overlay_status_row(frame);
    render_mode_overlay_with_sizing(frame, "Command discovery", &lines, &sizing_lines);
}

fn clear_overlay_status_row(frame: &mut ratatui::Frame<'_>) {
    use ratatui::layout::Rect;
    use ratatui::widgets::Clear;

    let area = frame.area();
    if area.is_empty() {
        return;
    }

    let status_row = Rect {
        x: area.x,
        y: area.y.saturating_add(area.height.saturating_sub(1)),
        width: area.width,
        height: 1,
    };
    frame.render_widget(Clear, status_row);
}

fn command_discovery_format_width(frame: &ratatui::Frame<'_>) -> usize {
    let area_width = usize::from(frame.area().width);
    if area_width < 120 {
        return area_width.saturating_sub(2);
    }

    area_width
        .saturating_mul(9)
        .saturating_div(10)
        .min(132)
        .saturating_sub(2)
}

const fn command_discovery_visible_rows(content_width: usize, area_height: u16) -> usize {
    // Narrow Help reads like a document. Keep a stable viewport even in tall terminals so scrolling
    // remains predictable instead of expanding into a long sheet that changes the user's spatial
    // model.
    if content_width < 80 {
        return 12;
    }

    if area_height >= 40 {
        34
    } else if area_height >= 28 {
        25
    } else {
        12
    }
}

fn render_mode_overlay(frame: &mut ratatui::Frame<'_>, title: &str, lines: &[String]) {
    render_mode_overlay_with_sizing(frame, title, lines, lines);
}

fn render_mode_overlay_with_sizing(
    frame: &mut ratatui::Frame<'_>,
    title: &str,
    lines: &[String],
    sizing_lines: &[String],
) {
    use ratatui::layout::Rect;
    use ratatui::prelude::Text;
    use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

    let area = frame.area();
    if area.is_empty() {
        return;
    }

    let content = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(2),
    };
    let width = u16::try_from(overlay_width(title, sizing_lines, content.width))
        .unwrap_or(u16::MAX)
        .min(content.width);
    let height = u16::try_from(overlay_height(title, lines))
        .unwrap_or(u16::MAX)
        .min(content.height);
    let overlay = Rect {
        x: content.x + content.width.saturating_sub(width) / 2,
        y: content.y + content.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, overlay);

    let command_discovery = title == "Command discovery";
    let display_title = if command_discovery { "Help" } else { title };
    let mut text_lines = Vec::new();
    if !command_discovery {
        text_lines.push(Line::from(Span::styled(
            display_title,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));
        text_lines.push(Line::from(""));
    }
    text_lines.extend(lines.iter().map(|line| overlay_line(line)));
    let text = Text::from(text_lines);
    let mut block = Block::bordered();
    if command_discovery {
        block = block.title(Span::styled(
            display_title,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    }
    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::new().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, overlay);
}

fn overlay_width(title: &str, lines: &[String], area_width: u16) -> usize {
    const COMMAND_DISCOVERY_MAX_WIDTH: usize = 132;

    if title == "Command discovery" {
        let content_width = lines
            .iter()
            .map(|line| line.chars().count())
            .chain(std::iter::once("Help".chars().count()))
            .max()
            .unwrap_or(0);
        let area_width = usize::from(area_width);
        return content_width
            .saturating_add(4)
            .clamp(56, COMMAND_DISCOVERY_MAX_WIDTH)
            .min(area_width);
    }

    let content_width = lines
        .iter()
        .map(|line| line.chars().count())
        .chain(std::iter::once(title.chars().count()))
        .max()
        .unwrap_or(0);
    content_width.saturating_add(4).clamp(56, 96)
}

fn overlay_height(title: &str, lines: &[String]) -> usize {
    if title == "Command discovery" {
        return lines.len().saturating_add(2);
    }

    lines.len().saturating_add(4)
}

fn overlay_line(line: &str) -> Line<'_> {
    if line.ends_with(':') {
        return Line::from(Span::styled(
            line,
            Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    }

    if let Some(line) = command_row_line(line) {
        return line;
    }

    if line.starts_with("Type ")
        || line == "actions, and aliases."
        || line.starts_with("Examples:")
        || line.contains(" closes")
        || line.starts_with("showing ")
        || line.trim_start().starts_with("key ")
    {
        return Line::from(Span::styled(line, Style::new().fg(Color::Gray)));
    }

    if line.starts_with('>') {
        return Line::from(Span::styled(
            line,
            Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));
    }

    Line::from(line)
}

fn command_row_line(line: &str) -> Option<Line<'_>> {
    if !line.starts_with("  ") || line.trim_start().starts_with("no matching") {
        return None;
    }

    let mut spans = Vec::new();
    let mut cursor = 0;
    while let Some(prefix_start) = find_cell_prefix(line, cursor) {
        if prefix_start > cursor {
            spans.push(Span::raw(&line[cursor..prefix_start]));
        }

        let key_start = skip_spaces(line, prefix_start);
        let separator_start = find_space_run(line, key_start, 2)?;
        let separator_end = skip_spaces(line, separator_start);
        let key = &line[key_start..separator_start];
        if key.trim().is_empty() {
            return None;
        }

        spans.push(Span::raw(&line[prefix_start..key_start]));
        spans.push(Span::styled(
            key,
            Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(&line[separator_start..separator_end]));

        let next_prefix = find_next_cell_prefix(line, separator_end);
        let action_end = next_prefix.unwrap_or(line.len());
        spans.push(Span::styled(
            &line[separator_end..action_end],
            Style::new().fg(Color::White),
        ));

        cursor = action_end;
        if next_prefix.is_none() {
            break;
        }
    }

    if cursor < line.len() {
        spans.push(Span::raw(&line[cursor..]));
    }

    Some(Line::from(spans))
}

fn find_next_cell_prefix(line: &str, start: usize) -> Option<usize> {
    let mut cursor = start;
    while let Some(prefix_start) = find_cell_prefix(line, cursor) {
        let key_start = skip_spaces(line, prefix_start);
        if find_space_run(line, key_start, 2).is_some() {
            return Some(prefix_start);
        }
        cursor = key_start.saturating_add(1);
    }
    None
}

fn find_cell_prefix(line: &str, start: usize) -> Option<usize> {
    let mut cursor = start;
    while let Some(space_start) = find_space_run(line, cursor, 2) {
        let key_start = skip_spaces(line, space_start);
        if key_start < line.len() {
            return Some(space_start);
        }
        cursor = key_start;
    }
    None
}

fn find_space_run(line: &str, start: usize, min_len: usize) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() {
        if bytes[cursor] != b' ' {
            cursor += 1;
            continue;
        }

        let run_start = cursor;
        while cursor < bytes.len() && bytes[cursor] == b' ' {
            cursor += 1;
        }
        if cursor.saturating_sub(run_start) >= min_len {
            return Some(run_start);
        }
    }
    None
}

fn skip_spaces(line: &str, start: usize) -> usize {
    let bytes = line.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len() && bytes[cursor] == b' ' {
        cursor += 1;
    }
    cursor
}
