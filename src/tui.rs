//! Shared terminal chrome.
//!
//! View slices render their own main content. This module only owns the title
//! bar, status bar, and modal overlays that frame those views.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui_macros::{line, span, vertical};

use crate::action_menu::{ActionMenu, RolePrompt};
use crate::action_output::ActionOutput;
use crate::app_screen::ViewMenuOption;
use crate::app_status::{StatusKind, StatusLine};
use crate::command::HelpSection;
use crate::copy::CopyOption;
use crate::theme;

#[derive(Clone, Copy, Debug)]
pub enum StatusHints {
    Graph,
    ShowDocument,
    DiffDocument,
    Status,
    Resolve,
    FileList,
    FileShowDocument,
    Bookmarks,
    Workspaces,
    OperationLog,
    OperationDetailDocument,
}

#[derive(Clone, Copy, Debug)]
pub struct Areas {
    pub title: Rect,
    pub main: Rect,
    pub status: Rect,
}

pub fn areas(area: Rect) -> Areas {
    let [title, main, status] = vertical![==1, >=1, ==1].areas(area);
    Areas {
        title,
        main,
        status,
    }
}

pub fn render_chrome(frame: &mut Frame<'_>, areas: Areas, status: &StatusLine) {
    frame.render_widget(title_bar(status), areas.title);
    frame.render_widget(status_line(status, areas.status.width), areas.status);
}

pub fn render_overlay(frame: &mut Frame<'_>, _status: &StatusLine, overlay: Overlay<'_>) {
    match overlay {
        Overlay::None => {}
        Overlay::Help { sections } => {
            let content = help_overlay_text(&sections);
            let area = centered_area(frame.area(), 84, content.lines.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(help_overlay(content), area);
        }
        Overlay::CopyMenu { options, selected } => {
            let area = centered_area(frame.area(), 54, options.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(copy_menu(options, selected), area);
        }
        Overlay::ViewMenu { options, selected } => {
            let area = centered_area(frame.area(), 54, options.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(view_menu(options, selected), area);
        }
        Overlay::ActionMenu { menu, selected } => {
            let area = centered_area(frame.area(), 64, menu.items().len() as u16 + 3);
            frame.render_widget(Clear, area);
            frame.render_widget(action_menu(menu, selected), area);
        }
        Overlay::PushRemotePrompt { remotes, selected } => {
            let area = centered_area(frame.area(), 46, remotes.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(remote_prompt("Push remote", remotes, selected), area);
        }
        Overlay::FetchRemotePrompt { remotes, selected } => {
            let area = centered_area(frame.area(), 46, remotes.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(remote_prompt("Fetch remote", remotes, selected), area);
        }
        Overlay::ActionOutput { title, output } => {
            let title = action_output_title(title, output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::AbandonConfirm { input, output } => {
            let title = "Abandon confirm";
            let area = action_output_area_with_footer(
                frame.area(),
                title,
                output,
                &abandon_confirm_footer_text(input),
            );
            frame.render_widget(Clear, area);
            render_abandon_confirm(frame, area, title, input, output);
        }
        Overlay::RolePrompt { prompt, selected } => {
            let area = centered_area(frame.area(), 54, prompt.options().len() as u16 + 4);
            frame.render_widget(Clear, area);
            frame.render_widget(role_prompt(prompt, selected), area);
        }
    }
}

pub enum Overlay<'a> {
    None,
    Help {
        sections: Vec<HelpSection>,
    },
    CopyMenu {
        options: &'a [CopyOption],
        selected: usize,
    },
    ViewMenu {
        options: &'a [ViewMenuOption],
        selected: usize,
    },
    ActionMenu {
        menu: &'a ActionMenu,
        selected: usize,
    },
    PushRemotePrompt {
        remotes: &'a [String],
        selected: usize,
    },
    FetchRemotePrompt {
        remotes: &'a [String],
        selected: usize,
    },
    ActionOutput {
        title: &'static str,
        output: &'a ActionOutput,
    },
    AbandonConfirm {
        input: &'a str,
        output: &'a ActionOutput,
    },
    RolePrompt {
        prompt: &'a RolePrompt,
        selected: usize,
    },
}

fn title_bar(status: &StatusLine) -> Paragraph<'_> {
    Paragraph::new(line![
        span!(Style::default().fg(Color::DarkGray); "{title}", title = status.title())
    ])
}

fn status_line(status: &StatusLine, width: u16) -> Paragraph<'_> {
    Paragraph::new(status_line_text(status, width))
}

fn status_line_text(status: &StatusLine, width: u16) -> Line<'_> {
    let mut spans = line![span!(
        status_style(status);
        "{message}",
        message = status.message()
    )]
    .spans;
    let message_width = line_width(status.message());
    let available_hint_width = usize::from(width)
        .saturating_sub(message_width)
        .saturating_sub(2) as u16;
    let hint_spans = status_hint_spans(status.hints(), available_hint_width).spans;

    if !hint_spans.is_empty() {
        spans.extend(line!["  "].spans);
        spans.extend(hint_spans);
    }

    Line::from(spans)
}

fn status_hint_spans(hints: StatusHints, width: u16) -> Line<'static> {
    let mut spans = Vec::new();
    let mut used_width = 0;

    for hint in status_hint_candidates(hints) {
        let separator_width = if spans.is_empty() { 0 } else { 2 };
        let item_width = line_width(hint.key) + 1 + line_width(hint.label);
        if used_width + separator_width + item_width > usize::from(width) {
            break;
        }

        if !spans.is_empty() {
            spans.push(Span::raw("  "));
        }
        spans.push(key(hint.key));
        spans.push(Span::raw(" "));
        spans.push(Span::raw(hint.label));
        used_width += separator_width + item_width;
    }

    Line::from(spans)
}

#[derive(Clone, Copy)]
struct StatusHint {
    key: &'static str,
    label: &'static str,
}

impl StatusHint {
    const fn new(key: &'static str, label: &'static str) -> Self {
        Self { key, label }
    }
}

const GRAPH_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("PgUp/PgDn", "page"),
    StatusHint::new("Enter/l", "open"),
    StatusHint::new("Space", "select"),
    StatusHint::new("a", "action"),
    StatusHint::new("S", "status"),
    StatusHint::new("f/p/r", "sync"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const DOCUMENT_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("g/G", "ends"),
    StatusHint::new("[/]", "file"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const STATUS_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("a", "actions"),
    StatusHint::new("D/C", "describe/commit @"),
    StatusHint::new("b/=/m", "bookmark @"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const RESOLVE_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("Enter/l", "inspect"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const FILE_LIST_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("Enter/l", "open"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("a", "action"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const FILE_SHOW_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("/", "search"),
    StatusHint::new("a", "action"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const OPERATION_DETAIL_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "scroll"),
    StatusHint::new("Space/C-b", "page"),
    StatusHint::new("s/d", "show/diff"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const BOOKMARKS_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("Enter/s", "show"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("x", "delete"),
    StatusHint::new("br", "rename"),
    StatusHint::new("bf", "forget"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const WORKSPACES_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy"),
    StatusHint::new("h", "back"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];
const OPERATION_LOG_STATUS_HINTS: &[StatusHint] = &[
    StatusHint::new("j/k", "move"),
    StatusHint::new("u", "undo"),
    StatusHint::new("C-r", "redo"),
    StatusHint::new("s", "show"),
    StatusHint::new("d", "diff"),
    StatusHint::new("a", "action"),
    StatusHint::new("/", "search"),
    StatusHint::new("y", "copy id"),
    StatusHint::new("q", "quit"),
    StatusHint::new("?", "help"),
];

fn status_hint_candidates(hints: StatusHints) -> &'static [StatusHint] {
    match hints {
        StatusHints::Graph => GRAPH_STATUS_HINTS,
        StatusHints::ShowDocument | StatusHints::DiffDocument => DOCUMENT_STATUS_HINTS,
        StatusHints::Status => STATUS_STATUS_HINTS,
        StatusHints::Resolve => RESOLVE_STATUS_HINTS,
        StatusHints::FileList => FILE_LIST_STATUS_HINTS,
        StatusHints::FileShowDocument => FILE_SHOW_STATUS_HINTS,
        StatusHints::OperationDetailDocument => OPERATION_DETAIL_STATUS_HINTS,
        StatusHints::Bookmarks => BOOKMARKS_STATUS_HINTS,
        StatusHints::Workspaces => WORKSPACES_STATUS_HINTS,
        StatusHints::OperationLog => OPERATION_LOG_STATUS_HINTS,
    }
}

fn help_overlay(content: Text<'static>) -> Paragraph<'static> {
    Paragraph::new(content)
        .style(theme::overlay_background_style())
        .block(overlay_block("Command menu"))
}

fn help_overlay_text(sections: &[HelpSection]) -> Text<'static> {
    let split = sections.len().div_ceil(2);
    let mut left = menu_help_lines();
    for section in &sections[..split] {
        append_help_section_lines(&mut left, section, true);
    }

    let mut right = Vec::new();
    for (index, section) in sections[split..].iter().enumerate() {
        append_help_section_lines(&mut right, section, index > 0);
    }

    Text::from(join_help_column_lines(&left, &right))
}

fn menu_help_lines() -> Vec<Line<'static>> {
    vec![
        Line::styled(
            "Menu".to_owned(),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ),
        line![
            span!(theme::key_style(); "Esc, q, ?"),
            "  ",
            span!("close menu"),
        ],
    ]
}

fn append_help_section_lines(
    lines: &mut Vec<Line<'static>>,
    section: &HelpSection,
    leading_blank: bool,
) {
    if leading_blank {
        lines.push(Line::default());
    }
    lines.push(Line::styled(
        section.title().to_owned(),
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    ));
    for row in section.rows() {
        lines.push(help_row_line(row));
    }
}

fn help_row_line(row: &crate::command::HelpRow) -> Line<'static> {
    line![
        span!(theme::key_style(); "{keys}", keys = row.keys()),
        "  ",
        span!("{action}", action = row.action()),
    ]
}

fn join_help_column_lines(left: &[Line<'static>], right: &[Line<'static>]) -> Vec<Line<'static>> {
    let line_count = left.len().max(right.len());
    (0..line_count)
        .map(|index| join_help_columns(left.get(index), right.get(index)))
        .collect()
}

fn join_help_columns(left: Option<&Line<'static>>, right: Option<&Line<'static>>) -> Line<'static> {
    let Some(left) = left else {
        return right.cloned().unwrap_or_default();
    };
    let Some(right) = right else {
        return left.clone();
    };

    let mut spans = left.spans.clone();
    let padding = 38_usize.saturating_sub(line_display_width(left)) + 4;
    spans.push(Span::raw(" ".repeat(padding)));
    spans.extend(right.spans.clone());
    Line::from(spans)
}

fn line_display_width(line: &Line<'_>) -> usize {
    line.spans
        .iter()
        .map(|span| line_width(span.content.as_ref()))
        .sum()
}

fn copy_menu(options: &[CopyOption], selected: usize) -> List<'static> {
    let items = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let style = if index == selected {
                theme::active_row_style()
            } else {
                Style::default()
            };
            ListItem::new(Line::from(option.label().to_owned())).style(style)
        })
        .collect::<Vec<_>>();
    List::new(items).block(overlay_block("Copy"))
}

fn view_menu(options: &[ViewMenuOption], selected: usize) -> List<'static> {
    let items = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let style = if index == selected {
                theme::active_row_style()
            } else {
                Style::default()
            };
            ListItem::new(Line::from(option.label().to_owned())).style(style)
        })
        .collect::<Vec<_>>();
    List::new(items).block(overlay_block("View"))
}

fn action_menu(menu: &ActionMenu, selected: usize) -> List<'static> {
    let items = menu
        .items()
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let style = if index == selected {
                theme::active_row_style()
            } else {
                Style::default()
            };
            ListItem::new(line![
                span!(theme::key_style(); "{shortcut}", shortcut = item.shortcut()),
                "  ",
                span!("{label}", label = item.label()),
            ])
            .style(style)
        })
        .collect::<Vec<_>>();
    let title = if menu
        .items()
        .first()
        .is_some_and(|item| !item.safety_tier().preview_marker().is_empty())
    {
        "Action menu (preview required)"
    } else {
        "Action menu"
    };
    List::new(items).block(overlay_block(title))
}

fn remote_prompt(title: &'static str, remotes: &[String], selected: usize) -> List<'static> {
    let items = remotes
        .iter()
        .enumerate()
        .map(|(index, remote)| {
            let style = if index == selected {
                theme::active_row_style()
            } else {
                Style::default()
            };
            ListItem::new(Line::from(remote.to_owned())).style(style)
        })
        .collect::<Vec<_>>();

    List::new(items).block(overlay_block(title))
}

fn action_output_title(action: &str, output: &ActionOutput) -> String {
    if output.completed() {
        format!("{action} result")
    } else {
        format!("{action} preview")
    }
}

fn render_action_output(frame: &mut Frame<'_>, area: Rect, title: &str, output: &ActionOutput) {
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
            Paragraph::new(output.body_lines().join("\n")).scroll((scroll, 0)),
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
        frame.render_widget(action_output_footer(output.completed()), footer_area);
    }
}

fn render_abandon_confirm(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    input: &str,
    output: &ActionOutput,
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
            Paragraph::new(output.body_lines().join("\n")).scroll((scroll, 0)),
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

fn action_output_footer(completed: bool) -> Paragraph<'static> {
    let primary = if completed {
        line![key("Enter"), " close  "]
    } else {
        line![key("Enter"), " confirm  "]
    };
    let mut spans = primary.spans;
    if completed {
        spans.extend(line![key("Esc/q"), " close  "].spans);
    } else {
        spans.extend(line![key("Esc/q"), " cancel  "].spans);
    }
    spans.extend(line![key("j/k"), " scroll  "].spans);
    spans.extend(line![key("PgUp/PgDn"), " page  "].spans);
    spans.extend(line![key("g/G"), " ends"].spans);

    Paragraph::new(Line::from(spans)).style(theme::muted_style())
}

fn abandon_confirm_footer(input: &str) -> Paragraph<'static> {
    Paragraph::new(Line::from(abandon_confirm_footer_text(input))).style(theme::muted_style())
}

fn action_output_area(area: Rect, title: &str, output: &ActionOutput) -> Rect {
    let footer = action_output_footer_text(output.completed());
    action_output_area_with_footer(area, title, output, &footer)
}

fn action_output_area_with_footer(
    area: Rect,
    title: &str,
    output: &ActionOutput,
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

fn action_output_footer_text(completed: bool) -> String {
    if completed {
        "Enter close  Esc/q close  j/k scroll  PgUp/PgDn page  g/G ends".to_owned()
    } else {
        "Enter confirm  Esc/q cancel  j/k scroll  PgUp/PgDn page  g/G ends".to_owned()
    }
}

fn abandon_confirm_footer_text(input: &str) -> String {
    format!("type exact id: {input}  Enter confirm  Esc cancel  arrows/page scroll")
}

fn line_width(line: &str) -> usize {
    line.chars().count()
}

fn role_prompt(prompt: &RolePrompt, selected: usize) -> List<'static> {
    let mut items: Vec<ListItem<'static>> = prompt
        .options()
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let style = if index == selected {
                theme::active_row_style()
            } else {
                Style::default()
            };
            ListItem::new(Line::from(option.label())).style(style)
        })
        .collect();

    let preview_hint = prompt.preview_required_message();
    if !preview_hint.is_empty() {
        items.push(ListItem::new(Line::from(preview_hint.to_owned())).style(theme::muted_style()));
    }

    List::new(items).block(overlay_block(format!(
        "{} (preview required)",
        prompt.title()
    )))
}

fn overlay_block(title: impl Into<String>) -> Block<'static> {
    Block::bordered()
        .style(theme::overlay_background_style())
        .border_style(theme::overlay_border_style())
        .title_style(theme::overlay_title_style())
        .title(title.into())
}

fn centered_area(area: ratatui::layout::Rect, width: u16, height: u16) -> ratatui::layout::Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    ratatui::layout::Rect {
        x,
        y,
        width,
        height,
    }
}

fn key(label: &'static str) -> ratatui::text::Span<'static> {
    span!(theme::key_style(); "{label}")
}

fn status_style(status: &StatusLine) -> Style {
    match status.kind() {
        StatusKind::Ready => Style::default().fg(Color::DarkGray),
        StatusKind::Error => Style::default().fg(Color::Red),
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use crate::action_output::ActionOutput;
    use crate::app_status::{StatusKind, StatusLine};
    use crate::command::{HelpRow, HelpSectionKind};

    use super::*;

    fn render_chrome_snapshot(status: &StatusLine, width: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, 3)).unwrap();
        terminal
            .draw(|frame| {
                let areas = areas(frame.area());
                render_chrome(frame, areas, status);
            })
            .unwrap();

        let title = row_text(terminal.backend().buffer(), 0, width);
        let status = row_text(terminal.backend().buffer(), 2, width);

        format!("title|{title}\nstatus|{status}")
    }

    fn row_text(buffer: &ratatui::buffer::Buffer, row: u16, width: u16) -> String {
        (0..width)
            .map(|column| buffer[(column, row)].symbol())
            .collect::<String>()
            .trim_end()
            .to_owned()
    }

    fn render_widget_rows(
        width: u16,
        height: u16,
        render: impl FnOnce(&mut ratatui::Frame<'_>),
    ) -> String {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(render).unwrap();

        (0..height)
            .map(|row| row_text(terminal.backend().buffer(), row, width))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn help_overlay_text_renders_generated_sections() {
        let text = help_overlay_text(&[
            HelpSection::new(
                HelpSectionKind::Views,
                vec![HelpRow::new("S", "status"), HelpRow::new("v", "view menu")],
            ),
            HelpSection::new(
                HelpSectionKind::Actions,
                vec![
                    HelpRow::new("D", "describe selected revision"),
                    HelpRow::new("p", "push selected revision"),
                ],
            ),
        ]);

        let rendered = text
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        insta::assert_snapshot!(rendered, @r"
        Menu                                      Action Previews
        Esc, q, ?  close menu                     D  describe selected revision
                                                  p  push selected revision
        View Switching
        S  status
        v  view menu
        ");
    }

    #[test]
    fn help_overlay_has_background_and_colored_key_labels() {
        let text = help_overlay_text(&[HelpSection::new(
            HelpSectionKind::Views,
            vec![HelpRow::new("S", "status")],
        )]);
        let mut terminal = Terminal::new(TestBackend::new(84, 8)).unwrap();

        terminal
            .draw(|frame| {
                frame.render_widget(help_overlay(text), frame.area());
            })
            .unwrap();

        let background = theme::overlay_background_style().bg.unwrap();
        assert_eq!(terminal.backend().buffer()[(1, 1)].bg, background);
        assert_eq!(terminal.backend().buffer()[(1, 2)].fg, Color::Yellow);
        assert_eq!(terminal.backend().buffer()[(1, 5)].fg, Color::Yellow);
    }

    #[test]
    fn status_chrome_renders_compact_hints_on_narrow_width() {
        let status = StatusLine::test(
            "jk log",
            "push cancelled",
            StatusKind::Error,
            StatusHints::Graph,
        );

        assert_snapshot!(render_chrome_snapshot(&status, 48), @r"
        title|jk log
        status|push cancelled  j/k move  PgUp/PgDn page
        ");
    }

    #[test]
    fn status_chrome_renders_core_hints_on_normal_width() {
        let status = StatusLine::test(
            "jk operation-log",
            "19 operations",
            StatusKind::Ready,
            StatusHints::OperationLog,
        );

        assert_snapshot!(render_chrome_snapshot(&status, 120), @r"
        title|jk operation-log
        status|19 operations  j/k move  u undo  C-r redo  s show  d diff  a action  / search  y copy id  q quit  ? help
        ");
    }

    #[test]
    fn file_list_status_hints_do_not_advertise_delete() {
        let status = StatusLine::test(
            "jk file list",
            "1 files",
            StatusKind::Ready,
            StatusHints::FileList,
        );

        assert_snapshot!(render_chrome_snapshot(&status, 100), @r"
        title|jk file list
        status|1 files  j/k move  Enter/l open  / search  y copy  a action  q quit  ? help
        ");
    }

    #[test]
    fn file_show_status_hints_advertise_file_actions() {
        let status = StatusLine::test(
            "jk file show src/main.rs",
            "4 lines",
            StatusKind::Ready,
            StatusHints::FileShowDocument,
        );

        assert_snapshot!(render_chrome_snapshot(&status, 120), @r"
        title|jk file show src/main.rs
        status|4 lines  j/k scroll  Space/C-b page  / search  a action  h back  q quit  ? help
        ");
    }

    #[test]
    fn resolve_status_hints_advertise_inspect_without_external_resolve() {
        let status = StatusLine::test(
            "jk resolve",
            "1 conflicts",
            StatusKind::Ready,
            StatusHints::Resolve,
        );

        assert_snapshot!(render_chrome_snapshot(&status, 100), @r"
        title|jk resolve
        status|1 conflicts  j/k move  Enter/l inspect  / search  y copy  q quit  ? help
        ");
    }

    #[test]
    fn action_menu_renders_shortcuts_and_preview_policy() {
        let menu = crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        );

        let rendered = render_widget_rows(48, 9, |frame| {
            frame.render_widget(action_menu(&menu, 1), frame.area());
        });

        assert_snapshot!(rendered, @r"
        ┌Action menu (preview required)────────────────┐
        │e  edit selected revision change-a            │
        │n  new child of change-a                      │
        │s  split selected revision change-a           │
        │x  abandon selected revision change-a         │
        │d  duplicate selected revision change-a       │
        │r  restore selected revision change-a         │
        │v  revert selected revision change-a into @   │
        └──────────────────────────────────────────────┘
        ");
    }

    #[test]
    fn action_menu_selected_row_has_visible_fallback_style() {
        let menu = crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        );
        let mut terminal = Terminal::new(TestBackend::new(48, 8)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(action_menu(&menu, 1), frame.area());
            })
            .unwrap();

        let selected_cell = &terminal.backend().buffer()[(1, 2)];
        let style = theme::active_row_style();
        assert_eq!(selected_cell.bg, style.bg.unwrap());
        assert!(!selected_cell.modifier.contains(Modifier::REVERSED));
        assert!(selected_cell.modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn action_menu_keeps_shortcuts_visible_on_narrow_terminals() {
        let menu = crate::action_menu::build_action_menu(
            &crate::action_menu::ExactActionContext::with_current("change-a"),
        );

        let rendered = render_widget_rows(28, 5, |frame| {
            frame.render_widget(action_menu(&menu, 0), frame.area());
        });

        assert_snapshot!(rendered, @r"
        ┌Action menu (preview requi┐
        │e  edit selected revision │
        │n  new child of change-a  │
        │s  split selected revision│
        └──────────────────────────┘
        ");
    }

    #[test]
    fn role_prompt_uses_shared_popover_presentation() {
        let prompt = RolePrompt::new(
            "confirm role assignment",
            vec![
                crate::action_menu::RolePromptOption::new("source", "source-a"),
                crate::action_menu::RolePromptOption::new("destination", "dest-a"),
            ],
            "Preview required before execution.",
        );

        let rendered = render_widget_rows(50, 6, |frame| {
            frame.render_widget(role_prompt(&prompt, 0), frame.area());
        });

        assert_snapshot!(rendered, @r"
        ┌confirm role assignment (preview required)──────┐
        │source: source-a                                │
        │destination: dest-a                             │
        │Preview required before execution.              │
        │                                                │
        └────────────────────────────────────────────────┘
        ");
    }

    #[test]
    fn action_output_render_keeps_footer_visible_while_body_scrolls() {
        let mut output = ActionOutput::pending(
            "jj action --preview".to_owned(),
            (0..8)
                .map(|line| format!("line {line}"))
                .collect::<Vec<_>>()
                .join("\n"),
            None,
        );
        output.scroll_down(5);
        output.scroll_down(5);

        let mut terminal = Terminal::new(TestBackend::new(36, 8)).unwrap();
        terminal
            .draw(|frame| {
                render_action_output(
                    frame,
                    Rect {
                        x: 0,
                        y: 0,
                        width: 36,
                        height: 8,
                    },
                    "Push preview",
                    &output,
                );
            })
            .unwrap();

        let rendered = (1..7)
            .map(|y| {
                (1..35)
                    .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n");

        insta::assert_snapshot!(rendered, @r"
          line 0
          line 1
          line 2
          line 3
          line 4
        Enter confirm  Esc/q cancel  j/k s
        ");
    }

    #[test]
    fn action_output_overlay_renders_common_preview_title_and_footer() {
        let output = ActionOutput::pending(
            "jj git fetch --remote exact:\"origin\"".to_owned(),
            "fetch preview".to_owned(),
            None,
        );
        let status = StatusLine::test("jk log", "ready", StatusKind::Ready, StatusHints::Graph);

        let rendered = render_widget_rows(80, 8, |frame| {
            render_overlay(
                frame,
                &status,
                Overlay::ActionOutput {
                    title: "Fetch",
                    output: &output,
                },
            );
        });

        assert!(rendered.contains("Fetch preview"));
        assert!(rendered.contains("command: jj git fetch --remote exact:\"origin\""));
        assert!(rendered.contains("Enter confirm  Esc/q cancel"));
        assert!(!rendered.contains("type exact id"));
    }

    #[test]
    fn action_output_overlay_renders_common_result_title_and_footer() {
        let output = ActionOutput::finished(
            "jj git fetch".to_owned(),
            "fetched".to_owned(),
            Some("default fetch uses jj git fetch remote resolution".to_owned()),
        );
        let status = StatusLine::test("jk log", "ready", StatusKind::Ready, StatusHints::Graph);

        let rendered = render_widget_rows(80, 8, |frame| {
            render_overlay(
                frame,
                &status,
                Overlay::ActionOutput {
                    title: "Fetch",
                    output: &output,
                },
            );
        });

        assert!(rendered.contains("Fetch result"));
        assert!(rendered.contains("command: jj git fetch"));
        assert!(rendered.contains("default fetch uses jj git fetch remote resolution"));
        assert!(rendered.contains("Enter close  Esc/q close"));
        assert!(!rendered.contains("type exact id"));
    }

    #[test]
    fn abandon_confirm_render_shows_typed_exact_id_footer() {
        let output = ActionOutput::pending(
            "jj abandon change-a".to_owned(),
            "change: change-a\ntitle: Edit change\ndiff summary:\nM src/main.rs\n\nundo path: jj undo"
                .to_owned(),
            None,
        );

        let mut terminal = Terminal::new(TestBackend::new(64, 8)).unwrap();
        terminal
            .draw(|frame| {
                render_abandon_confirm(
                    frame,
                    Rect {
                        x: 0,
                        y: 0,
                        width: 64,
                        height: 8,
                    },
                    "Abandon confirm",
                    "change",
                    &output,
                );
            })
            .unwrap();

        let rendered = (1..7)
            .map(|y| {
                (1..63)
                    .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n");

        insta::assert_snapshot!(rendered, @r"
        command: jj abandon change-a
        output:
          change: change-a
          title: Edit change
          diff summary:
        type exact id: change  Enter confirm  Esc cancel  arrows/page
        ");
    }
}
