use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui_macros::{line, span};

use crate::command::{HelpRow, HelpSection};
use crate::menus::{ActionMenu, CopyOption, RolePrompt};
use crate::modes::ViewMenuOption;
use crate::tui::theme;

/// Wrap help text in the shared overlay block and background styling.
pub fn help_overlay(content: Text<'static>) -> Paragraph<'static> {
    Paragraph::new(content)
        .style(theme::overlay_background_style())
        .block(overlay_block("Command menu"))
}

/// Project help sections into the two-column text shown in the help overlay.
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

/// Build the copy-menu list for the current copyable options.
pub(super) fn copy_menu(options: &[CopyOption], selected: usize) -> List<'static> {
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

/// Build the top-level view-switching menu.
pub(super) fn view_menu(options: &[ViewMenuOption], selected: usize) -> List<'static> {
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

/// Build the action menu, surfacing preview policy in the title when needed.
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

/// Build a remote-selection prompt for push or fetch.
pub(super) fn remote_prompt(
    title: &'static str,
    remotes: &[String],
    selected: usize,
) -> List<'static> {
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

/// Build the shared role-assignment prompt for rewrite previews.
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

/// Center a modal rectangle inside the available frame, clamping to the frame bounds.
pub(super) fn centered_area(area: Rect, width: u16, height: u16) -> Rect {
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

pub(super) fn overlay_block(title: impl Into<String>) -> Block<'static> {
    Block::bordered()
        .style(theme::overlay_background_style())
        .border_style(theme::overlay_border_style())
        .title_style(theme::overlay_title_style())
        .title(title.into())
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

fn help_row_line(row: &HelpRow) -> Line<'static> {
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

fn line_width(line: &str) -> usize {
    line.chars().count()
}
