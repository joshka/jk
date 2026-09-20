use jk_cli::LogTemplateSelection;
use jk_tui::command_discovery::{BindingContext, discovery_lines_for_width_and_rows, overlay_line};
use jk_tui::command_preview_view::CommandPreviewView;
use jk_tui::styles::{dialog, dialog_accent};
use ratatui::layout::Rect;
use ratatui::prelude::Span;
use ratatui::widgets::{Clear, Paragraph};

use crate::command_mode::{external_command_lines, jj_command_lines};
use crate::menus::{
    action_menu_lines, diff_file_list_lines, template_selector_lines, view_options_lines,
};
use crate::state::{AppState, AppView, BookmarkMutationField, BookmarkMutationKind, InputMode};

pub fn render_app(
    frame: &mut ratatui::Frame<'_>,
    state: &mut AppState,
    template: &LogTemplateSelection,
) {
    let mode = state.modes.active().cloned();
    match state.views.active_mut() {
        AppView::Log(log) => match &mode {
            Some(InputMode::RunOptions { .. }) => {
                log.render(frame);
                clear_overlay_status_row(frame);
                if let Some(InputMode::RunOptions { dialog }) = state.modes.active_mut() {
                    dialog.render(frame);
                }
            }
            Some(InputMode::RebaseDestination { pending }) => {
                log.render(frame);
                clear_overlay_status_row(frame);
                pending.render(frame);
            }
            Some(InputMode::ActionMenu { context, selected }) => {
                log.render(frame);
                let width = usize::from(frame.area().width.saturating_sub(6));
                let lines = action_menu_lines(*context, *selected, width);
                render_mode_overlay(frame, "Actions", &lines);
            }
            Some(InputMode::ViewOptions { context, selected }) => {
                let lines = view_options_lines(*context, *selected, template, None);
                log.render_with_selector(frame, "View Options", &lines);
                clear_overlay_status_row(frame);
            }
            Some(InputMode::LogTemplate { options, selected }) => {
                let lines = template_selector_lines(options, *selected);
                log.render_with_selector(frame, "Log template", &lines);
                clear_overlay_status_row(frame);
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
            Some(InputMode::ExternalCommand { input, error }) => {
                log.render(frame);
                let lines = external_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "external command", &lines);
            }
            Some(InputMode::DescribeMessage { .. }) => {
                log.render(frame);
                clear_overlay_status_row(frame);
                if let Some(InputMode::DescribeMessage { rev, message }) = state.modes.active_mut()
                {
                    message.render(frame, rev);
                }
            }
            Some(InputMode::AbandonConfirmation { .. }) => {
                log.render(frame);
                clear_overlay_status_row(frame);
                if let Some(InputMode::AbandonConfirmation { dialog, .. }) =
                    state.modes.active_mut()
                {
                    dialog.render(frame);
                }
            }
            Some(InputMode::CommandPreview { .. }) => {
                log.render(frame);
                clear_overlay_status_row(frame);
                let Some(InputMode::CommandPreview { pending }) = state.modes.active_mut() else {
                    return;
                };
                let view = CommandPreviewView::new(pending.preview.clone())
                    .with_status(pending.copy_status.clone())
                    .with_details(pending.details.clone())
                    .with_run_options(crate::run_options::available(&pending.preview.spec));
                pending.can_confirm = view.can_confirm(frame.area());
                pending.max_scroll = view.max_scroll(frame.area());
                pending.scroll = pending.scroll.min(pending.max_scroll);
                view.with_scroll(pending.scroll).render(frame);
            }
            _ => log.render(frame),
        },
        AppView::Bookmarks { view } => match &mode {
            Some(InputMode::JjCommand { input, error }) => {
                view.render(frame);
                let lines = jj_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "jj command", &lines);
            }
            Some(InputMode::ExternalCommand { input, error }) => {
                view.render(frame);
                let lines = external_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "external command", &lines);
            }

            Some(InputMode::RemotePicker {
                names,
                selected,
                bookmark,
            }) => {
                view.render(frame);
                let title = bookmark.as_ref().map_or_else(
                    || "Fetch from remote".to_owned(),
                    |name| format!("Push dry-run: {name}"),
                );
                let lines = remote_picker_lines(names, *selected, frame.area().height);
                render_mode_overlay(frame, &title, &lines);
            }
            Some(InputMode::BookmarkMutation {
                kind,
                name,
                revision,
                field,
                error,
            }) => {
                view.render(frame);
                render_mode_overlay(
                    frame,
                    "Bookmark mutation",
                    &bookmark_mutation_lines(*kind, name, revision, *field, error.as_deref()),
                );
            }
            Some(InputMode::CommandPreview { .. }) => {
                view.render(frame);
                clear_overlay_status_row(frame);
                let Some(InputMode::CommandPreview { pending }) = state.modes.active_mut() else {
                    return;
                };
                let preview = CommandPreviewView::new(pending.preview.clone())
                    .with_status(pending.copy_status.clone())
                    .with_details(pending.details.clone());
                pending.can_confirm = preview.can_confirm(frame.area());
                pending.max_scroll = preview.max_scroll(frame.area());
                pending.scroll = pending.scroll.min(pending.max_scroll);
                preview.with_scroll(pending.scroll).render(frame);
            }
            _ => view.render(frame),
        },
        AppView::Diff { view, query } => match &mode {
            Some(InputMode::ViewOptions { context, selected }) => {
                let lines = view_options_lines(*context, *selected, template, Some(query.format()));
                view.render_with_overlay(frame, "View Options", &lines);
                clear_overlay_status_row(frame);
            }
            Some(InputMode::DiffFileList { selected }) => {
                let lines = diff_file_list_lines(view, *selected);
                view.render_with_overlay(frame, "Diff files", &lines);
                clear_overlay_status_row(frame);
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
                clear_overlay_status_row(frame);
            }
            Some(InputMode::ExternalCommand { input, error }) => {
                let lines = external_command_lines(input, error.as_deref());
                view.render_with_overlay(frame, "external command", &lines);
                clear_overlay_status_row(frame);
            }
            _ => view.render(frame),
        },
        AppView::Show { view, .. } => render_inspection(frame, view, &mode, template),
        AppView::Evolog { view, .. } => render_inspection(frame, view, &mode, template),
        AppView::Status { view, .. } => render_inspection(frame, view, &mode, template),
        AppView::Workspaces { view } => match &mode {
            Some(InputMode::ActionMenu { context, selected }) => {
                view.render(frame);
                let width = usize::from(frame.area().width.saturating_sub(6));
                let lines = action_menu_lines(*context, *selected, width);
                render_mode_overlay(frame, "Actions", &lines);
            }
            Some(InputMode::WorkspaceLifecycle { .. }) => {
                view.render(frame);
                clear_overlay_status_row(frame);
                if let Some(InputMode::WorkspaceLifecycle { dialog }) = state.modes.active_mut() {
                    dialog.render(frame);
                }
            }
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
            Some(InputMode::ExternalCommand { input, error }) => {
                view.render(frame);
                let lines = external_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "external command", &lines);
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
            Some(InputMode::ExternalCommand { input, error }) => {
                view.render(frame);
                let lines = external_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "external command", &lines);
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
            Some(InputMode::ExternalCommand { input, error }) => {
                view.render(frame);
                let lines = external_command_lines(input, error.as_deref());
                render_mode_overlay(frame, "external command", &lines);
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

    if let Some(message) = state.toast_message() {
        render_toast(frame, message);
    }
}

fn render_toast(frame: &mut ratatui::Frame<'_>, message: &str) {
    let area = frame.area();
    if area.width < 12 || area.height < 3 {
        return;
    }
    let label = format!("✓ {message}");
    let width = u16::try_from(Span::raw(&label).width().saturating_add(2))
        .unwrap_or(u16::MAX)
        .min(area.width);
    let toast = Rect::new(
        area.right().saturating_sub(width),
        area.bottom() - 2,
        width,
        1,
    );
    frame.render_widget(Clear, toast);
    frame.render_widget(Paragraph::new(label).style(dialog()), toast);
}

fn remote_picker_lines(names: &[String], selected: usize, height: u16) -> Vec<String> {
    let visible = usize::from(height.saturating_sub(8)).max(1);
    let start = selected.saturating_sub(visible.saturating_sub(1));
    let end = start.saturating_add(visible).min(names.len());
    let mut lines = names
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
        .map(|(index, name)| format!("{} {name}", if index == selected { ">" } else { " " }))
        .collect::<Vec<_>>();
    if names.len() > visible {
        lines.push(format!("{}–{} of {} remotes", start + 1, end, names.len()));
    } else {
        lines.push(String::new());
    }
    lines.push("↑/↓ choose  Enter preview  Esc cancel".to_owned());
    lines
}

fn bookmark_mutation_lines(
    kind: BookmarkMutationKind,
    name: &str,
    revision: &str,
    field: BookmarkMutationField,
    error: Option<&str>,
) -> Vec<String> {
    let operation = match kind {
        BookmarkMutationKind::Create => "create",
        BookmarkMutationKind::Move => "move",
    };
    let mut lines = vec![
        format!("Operation: bookmark {operation}"),
        format!(
            "{} Name: {name}",
            if field == BookmarkMutationField::Name {
                ">"
            } else {
                " "
            }
        ),
        format!(
            "{} Revision: {revision}",
            if field == BookmarkMutationField::Revision {
                ">"
            } else {
                " "
            }
        ),
        String::new(),
    ];
    if let Some(error) = error {
        lines.push(error.to_owned());
    }
    lines.push(
        match kind {
            BookmarkMutationKind::Create => "Tab field  Enter preview  Esc cancel",
            BookmarkMutationKind::Move => "Enter preview  Esc cancel",
        }
        .to_owned(),
    );
    lines
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
            clear_overlay_status_row(frame);
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
            clear_overlay_status_row(frame);
        }
        Some(InputMode::ExternalCommand { input, error }) => {
            let lines = external_command_lines(input, error.as_deref());
            view.render_with_overlay(frame, "external command", &lines);
            clear_overlay_status_row(frame);
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
    // Size from the full rendered document, not the current scroll slice, so the Help surface does
    // not jump wider/narrower as different lines scroll into view.
    let sizing_lines =
        discovery_lines_for_width_and_rows(context, query, 0, content_width, usize::MAX);
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
        return area_width.saturating_sub(4);
    }

    area_width
        .saturating_mul(9)
        .saturating_div(10)
        .min(132)
        .saturating_sub(4)
}

const fn command_discovery_visible_rows(content_width: usize, area_height: u16) -> usize {
    let preferred = if content_width < 80 {
        12
    } else if area_height >= 40 {
        34
    } else if area_height >= 28 {
        25
    } else {
        12
    };
    let available = area_height.saturating_sub(8) as usize;
    if preferred < available {
        preferred
    } else {
        available
    }
}

fn render_mode_overlay(
    frame: &mut ratatui::Frame<'_>,
    title: &str,
    lines: &[String],
) -> Option<Rect> {
    render_mode_overlay_with_sizing(frame, title, lines, lines)
}

fn render_mode_overlay_with_sizing(
    frame: &mut ratatui::Frame<'_>,
    title: &str,
    lines: &[String],
    sizing_lines: &[String],
) -> Option<Rect> {
    use ratatui::layout::Rect;
    use ratatui::prelude::Text;
    use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

    let area = frame.area();
    if area.is_empty() {
        return None;
    }

    clear_overlay_status_row(frame);

    let content = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(2),
    };
    let width = u16::try_from(overlay_width(title, sizing_lines, content.width))
        .unwrap_or(u16::MAX)
        .min(content.width);
    let text = Text::from(
        lines
            .iter()
            .map(|line| overlay_line(line))
            .collect::<Vec<_>>(),
    );
    let paragraph = Paragraph::new(text)
        .style(dialog())
        .wrap(Wrap { trim: false });
    let height = u16::try_from(paragraph.line_count(width.saturating_sub(4).max(1)) + 3)
        .unwrap_or(u16::MAX)
        .min(content.height);
    let overlay = Rect {
        x: content.x + content.width.saturating_sub(width) / 2,
        y: content.y + content.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, overlay);
    frame.render_widget(Block::default().style(dialog()), overlay);
    let display_title = if title == "Command discovery" {
        "Help"
    } else {
        title
    };
    let header = Rect::new(
        overlay.x.saturating_add(2),
        overlay.y,
        overlay.width.saturating_sub(4),
        overlay.height.min(1),
    );
    frame.render_widget(Paragraph::new(display_title).style(dialog_accent()), header);
    let body = Rect::new(
        overlay.x.saturating_add(2),
        overlay.y.saturating_add(2),
        overlay.width.saturating_sub(4),
        overlay.height.saturating_sub(3),
    );
    frame.render_widget(paragraph, body);
    Some(overlay)
}

fn overlay_width(title: &str, lines: &[String], area_width: u16) -> usize {
    let display_title = if title == "Command discovery" {
        "Help"
    } else {
        title
    };
    lines
        .iter()
        .map(|line| Span::raw(line).width())
        .chain(std::iter::once(Span::raw(display_title).width()))
        .max()
        .unwrap_or(0)
        .saturating_add(4)
        .min(132)
        .min(usize::from(area_width))
}

#[cfg(test)]
mod tests {
    #[test]
    fn remote_picker_scrolls_selection_without_hiding_controls() {
        let names = (0..40)
            .map(|index| format!("remote-{index}"))
            .collect::<Vec<_>>();
        for height in [16, 24, 40, 50] {
            for selected in [0, 20, 39] {
                let lines = super::remote_picker_lines(&names, selected, height);
                assert!(
                    lines
                        .iter()
                        .any(|line| line == &format!("> remote-{selected}"))
                );
                assert_eq!(
                    lines.last().map(String::as_str),
                    Some("↑/↓ choose  Enter preview  Esc cancel")
                );
                assert!(lines.len() + 6 <= usize::from(height));
            }
        }
    }
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn preview_owns_controls_until_closed_in_log_and_bookmarks() {
        use jk_core::{JjCommandSpec, SafetyClass};
        use jk_tui::bookmark_view::{BookmarkView, BookmarkViewSnapshot};

        use crate::mutation_preview::PendingCommandPreview;
        use crate::test_support::{buffer_line, log_app_view};

        for app_view in [
            log_app_view("change"),
            AppView::Bookmarks {
                view: BookmarkView::new(BookmarkViewSnapshot::new(Vec::new())),
            },
        ] {
            let mut state = AppState::new(app_view);
            let preview = JjCommandSpec::confirm_mutation(
                [
                    "describe",
                    "--message",
                    "review before changing the description",
                ],
                SafetyClass::LocalRewrite,
            )
            .command_preview();
            state.modes.push(InputMode::CommandPreview {
                pending: PendingCommandPreview::describe(preview),
            });
            let mut terminal = Terminal::new(TestBackend::new(80, 24)).expect("terminal");
            terminal
                .draw(|frame| render_app(frame, &mut state, &LogTemplateSelection::Configured))
                .expect("preview");
            assert!(
                buffer_line(terminal.backend().buffer(), 23)
                    .trim()
                    .is_empty()
            );

            let mut narrow = Terminal::new(TestBackend::new(40, 8)).expect("narrow terminal");
            narrow
                .draw(|frame| render_app(frame, &mut state, &LogTemplateSelection::Configured))
                .expect("narrow preview");
            let text = narrow
                .backend()
                .buffer()
                .content()
                .iter()
                .map(ratatui::buffer::Cell::symbol)
                .collect::<String>();
            assert!(text.contains("enter run  y copy  esc cancel"));
            assert!(!text.contains("? help"));

            state.modes.pop();
            terminal
                .draw(|frame| render_app(frame, &mut state, &LogTemplateSelection::Configured))
                .expect("restored view");
            assert!(buffer_line(terminal.backend().buffer(), 23).contains("? help"));
        }
    }

    #[test]
    fn bookmark_preview_requires_visible_controls_and_fresh_enter() {
        use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
        use jk_cli::{BookmarkMutation, JjBookmarks, JjLog};
        use jk_tui::bookmark_view::{BookmarkView, BookmarkViewSnapshot};

        let mut state = AppState::new(AppView::Bookmarks {
            view: BookmarkView::new(BookmarkViewSnapshot::new(Vec::new())),
        });
        let bookmarks = JjBookmarks::default();
        let mut log = JjLog::default();
        crate::bookmark_routes::open_bookmark_preview(
            &mut state,
            &bookmarks,
            BookmarkMutation::Delete {
                name: "topic".into(),
            },
        );

        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        crate::handle_command_preview_mode(&mut state, &mut log, &bookmarks, enter);
        assert!(matches!(
            state.modes.active(),
            Some(InputMode::CommandPreview { .. })
        ));
        assert_eq!(state.history.records().count(), 0);

        let mut terminal = Terminal::new(TestBackend::new(30, 6)).expect("small terminal");
        terminal
            .draw(|frame| render_app(frame, &mut state, &LogTemplateSelection::Configured))
            .expect("small preview");
        assert!(
            matches!(state.modes.active(), Some(InputMode::CommandPreview { pending })
            if !pending.can_confirm)
        );
        crate::handle_command_preview_mode(&mut state, &mut log, &bookmarks, enter);
        assert_eq!(state.history.records().count(), 0);

        let mut terminal = Terminal::new(TestBackend::new(90, 24)).expect("wide terminal");
        terminal
            .draw(|frame| render_app(frame, &mut state, &LogTemplateSelection::Configured))
            .expect("wide preview");
        assert!(
            matches!(state.modes.active(), Some(InputMode::CommandPreview { pending })
            if pending.can_confirm)
        );
        let repeated_enter = KeyEvent {
            kind: KeyEventKind::Repeat,
            ..enter
        };
        crate::handle_command_preview_mode(&mut state, &mut log, &bookmarks, repeated_enter);
        assert!(matches!(
            state.modes.active(),
            Some(InputMode::CommandPreview { .. })
        ));
        assert_eq!(state.history.records().count(), 0);
        crate::handle_command_preview_mode(
            &mut state,
            &mut log,
            &bookmarks,
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        );
        assert!(state.modes.active().is_none());
    }

    #[test]
    fn overlay_width_uses_terminal_cells_for_wide_and_combining_characters() {
        let wide = format!("> {}", "界".repeat(29));
        assert_eq!(overlay_width("Actions", &[wide], 80), 64);

        let combining = "e\u{301}".repeat(55);
        assert_eq!(overlay_width("Actions", &[combining], 80), 59);
    }

    #[test]
    fn toast_handles_tiny_terminals_without_panicking() {
        for width in 0..=14 {
            for height in 0..=8 {
                let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                terminal
                    .draw(|frame| render_toast(frame, "Created new change"))
                    .unwrap();
            }
        }
    }

    #[test]
    fn command_overlay_has_opaque_surface_and_semantic_prompt() {
        let lines = vec![
            "! printf hello".to_owned(),
            "error: fixture".to_owned(),
            String::new(),
            "enter run   esc cancel".to_owned(),
        ];
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        let mut overlay = None;
        terminal
            .draw(|frame| overlay = render_mode_overlay(frame, "external command", &lines))
            .unwrap();
        let overlay = overlay.expect("overlay");
        let buffer = terminal.backend().buffer();
        assert_eq!(Some(buffer[(overlay.x, overlay.y)].bg), dialog().bg);
        assert_eq!(
            Some(buffer[(overlay.x + 2, overlay.y + 2)].fg),
            dialog_accent().fg
        );
        assert!(
            buffer[(overlay.x + 2, overlay.y + 2)]
                .modifier
                .contains(ratatui::prelude::Modifier::BOLD)
        );
        assert_eq!(
            Some(buffer[(overlay.x + 2, overlay.y + 3)].fg),
            jk_tui::styles::dialog_error().fg
        );
        assert!(
            !buffer
                .content
                .iter()
                .any(|cell| matches!(cell.symbol(), "┌" | "┐" | "│"))
        );
    }

    #[test]
    fn mode_overlay_marks_only_the_active_control() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).expect("terminal");
        let mut overlay = None;
        terminal
            .draw(|frame| {
                overlay = render_mode_overlay(
                    frame,
                    "Actions",
                    &["> new change".into(), "  describe".into()],
                )
            })
            .expect("draw mode overlay");
        let overlay = overlay.expect("overlay area");
        let buffer = terminal.backend().buffer();
        assert_eq!(
            Some(buffer[(overlay.x + 2, overlay.y + 2)].fg),
            dialog_accent().fg
        );
        assert!(
            buffer[(overlay.x + 2, overlay.y + 2)]
                .modifier
                .contains(ratatui::prelude::Modifier::BOLD)
        );
        for x in overlay.x..overlay.right() {
            assert_eq!(Some(buffer[(x, overlay.y + 2)].bg), dialog().bg);
        }
    }
}
