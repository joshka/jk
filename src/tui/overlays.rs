use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui_macros::{line, span};

use crate::action_pane::ActionPane;
use crate::command::HelpSection;
use crate::copy::CopyOption;
use crate::menus::{ActionMenu, RolePrompt};
use crate::modes::ViewMenuOption;
use crate::status_line::StatusLine;
use crate::theme;

/// Borrowed overlay projection for the current interaction mode.
///
/// The enum carries references so drawing never takes ownership of prompt/menu/action state. Add
/// only shared modal presentation here; feature-specific availability and command policy belong in
/// the view, action menu, or action plan that produced the state.
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
    ActionPane {
        title: &'static str,
        output: &'a ActionPane,
    },
    AbandonConfirm {
        input: &'a str,
        output: &'a ActionPane,
    },
    RolePrompt {
        prompt: &'a RolePrompt,
        selected: usize,
    },
}

/// Draw the active modal overlay over an already rendered frame.
///
/// Overlays are presentation-only. Selection indexes and output scroll offsets are owned by
/// `InteractionMode` or `ActionPane`; this function only sizes, clears, and renders the modal.
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
        Overlay::ActionPane { title, output } => {
            let title = action_pane_title(title, output);
            let area = action_pane_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_pane(frame, area, &title, output);
        }
        Overlay::AbandonConfirm { input, output } => {
            let title = "Abandon confirm";
            let area = action_pane_area_with_footer(
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

pub fn help_overlay(content: Text<'static>) -> Paragraph<'static> {
    Paragraph::new(content)
        .style(theme::overlay_background_style())
        .block(overlay_block("Command menu"))
}

pub fn help_overlay_text(sections: &[HelpSection]) -> Text<'static> {
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

pub fn action_menu(menu: &ActionMenu, selected: usize) -> List<'static> {
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

fn action_pane_title(action: &str, output: &ActionPane) -> String {
    if output.completed() {
        format!("{action} result")
    } else {
        format!("{action} preview")
    }
}

pub fn render_action_pane(frame: &mut Frame<'_>, area: Rect, title: &str, output: &ActionPane) {
    let block = overlay_block(title.to_owned());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let footer_height = u16::from(inner.height > 1);
    // `ActionPane` owns scroll position and key behavior. Chrome rendering only projects that
    // offset into a body area and reserves the last usable row for command hints when it exists.
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
        frame.render_widget(action_pane_footer(output.completed()), footer_area);
    }
}

pub fn render_abandon_confirm(
    frame: &mut Frame<'_>,
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
    // Confirmation input is app-owned state. The overlay renders the current value as footer text
    // but does not validate, mutate, or advance the prompt.
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

fn action_pane_footer(completed: bool) -> Paragraph<'static> {
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

fn action_pane_area(area: Rect, title: &str, output: &ActionPane) -> Rect {
    let footer = action_pane_footer_text(output.completed());
    action_pane_area_with_footer(area, title, output, &footer)
}

fn action_pane_area_with_footer(
    area: Rect,
    title: &str,
    output: &ActionPane,
    footer: &str,
) -> Rect {
    let lines = output.body_lines();
    // Size to the widest visible contract text, then clamp to the terminal. This keeps previews and
    // results readable without changing the output content or inventing wrapping policy here.
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

fn action_pane_footer_text(completed: bool) -> String {
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

pub fn role_prompt(prompt: &RolePrompt, selected: usize) -> List<'static> {
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
    // All overlays share one fallback-friendly style contract: bordered, cleared, high-contrast
    // presentation with theme-owned colors. Variant-specific behavior and state stay upstream.
    Block::bordered()
        .style(theme::overlay_background_style())
        .border_style(theme::overlay_border_style())
        .title_style(theme::overlay_title_style())
        .title(title.into())
}

fn centered_area(area: Rect, width: u16, height: u16) -> Rect {
    // Modal geometry is clipped to the current terminal instead of assuming a minimum size. The
    // caller decides desired content dimensions; this helper only keeps the rectangle drawable.
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;

    Rect {
        x,
        y,
        width,
        height,
    }
}

fn key(label: &'static str) -> Span<'static> {
    span!(theme::key_style(); "{label}")
}
