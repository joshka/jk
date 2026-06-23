use jk_cli::LogTemplateSelection;
use jk_tui::command_discovery::discovery_lines;
use jk_tui::command_preview_view::CommandPreviewView;

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
                selected,
            }) => {
                let lines = discovery_lines(*context, query, *selected);
                log.render_with_selector(frame, "Command discovery", &lines);
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
                selected,
            }) => {
                let lines = discovery_lines(*context, query, *selected);
                view.render_with_overlay(frame, "Command discovery", &lines);
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
                selected,
            }) => {
                let lines = discovery_lines(*context, query, *selected);
                view.render(frame);
                render_mode_overlay(frame, "Command discovery", &lines);
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
                selected,
            }) => {
                let lines = discovery_lines(*context, query, *selected);
                view.render(frame);
                render_mode_overlay(frame, "Command discovery", &lines);
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
                selected,
            }) => {
                let lines = discovery_lines(*context, query, *selected);
                view.render(frame);
                render_mode_overlay(frame, "Command discovery", &lines);
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
            selected,
        }) => {
            let lines = discovery_lines(*context, query, *selected);
            view.render_with_overlay(frame, "Command discovery", &lines);
        }
        Some(InputMode::JjCommand { input, error }) => {
            let lines = jj_command_lines(input, error.as_deref());
            view.render_with_overlay(frame, "jj command", &lines);
        }
        _ => view.render(frame),
    }
}

fn render_mode_overlay(frame: &mut ratatui::Frame<'_>, title: &str, lines: &[String]) {
    use ratatui::layout::Rect;
    use ratatui::prelude::{Color, Line, Modifier, Span, Style, Text};
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
    let width = 72_u16.min(content.width);
    let height = u16::try_from(lines.len().saturating_add(4))
        .unwrap_or(u16::MAX)
        .min(content.height);
    let overlay = Rect {
        x: content.x + content.width.saturating_sub(width) / 2,
        y: content.y + content.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, overlay);

    let text = Text::from(
        std::iter::once(Line::from(Span::styled(
            title,
            Style::new().add_modifier(Modifier::BOLD),
        )))
        .chain(std::iter::once(Line::from("")))
        .chain(lines.iter().map(|line| Line::from(line.as_str())))
        .collect::<Vec<_>>(),
    );
    let paragraph = Paragraph::new(text)
        .block(Block::bordered())
        .style(Style::new().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, overlay);
}
