//! Shared terminal chrome.
//!
//! View slices render their own main content. This module only owns the title
//! bar, status bar, and modal overlays that frame those views.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui_macros::{line, span, vertical};

use crate::app::{StatusKind, StatusLine, ViewFormatOption};
use crate::command::HelpSection;
use crate::copy::CopyOption;

#[derive(Clone, Copy, Debug)]
pub enum StatusHints {
    Graph,
    ShowDocument,
    DiffDocument,
    Status,
    Bookmarks,
    OperationLog,
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
    frame.render_widget(status_line(status), areas.status);
}

pub fn render_overlay(frame: &mut Frame<'_>, _status: &StatusLine, overlay: Overlay<'_>) {
    match overlay {
        Overlay::None => {}
        Overlay::Help { sections } => {
            let content = help_overlay_text(&sections);
            let area = centered_area(frame.area(), 62, content.lines.len() as u16 + 2);
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
        options: &'a [ViewFormatOption],
        selected: usize,
    },
}

fn title_bar(status: &StatusLine) -> Paragraph<'_> {
    Paragraph::new(line![
        span!(Style::default().fg(Color::DarkGray); "{title}", title = status.title())
    ])
}

fn status_line(status: &StatusLine) -> Paragraph<'_> {
    let line = match status.hints() {
        StatusHints::Graph => line![
            key("q"),
            " quit  ",
            key("f"),
            " fetch  ",
            key("r"),
            " refresh  ",
            key("j/k"),
            " move  ",
            key("h/l"),
            " back/open  ",
            key("s"),
            " show  ",
            key("d"),
            " diff  ",
            key("c"),
            " new/undo  ",
            key("w"),
            " mode  ",
            key("W"),
            " revset  ",
            key("L"),
            " log  ",
            key("J"),
            " jj  ",
            key("v"),
            " view  ",
            key("?"),
            " help  ",
            span!(status_style(status); "{message}", message = status.message()),
        ],
        StatusHints::ShowDocument => line![
            key("q"),
            " quit  ",
            key("f"),
            " fetch  ",
            key("r"),
            " refresh  ",
            key("j/k"),
            " scroll  ",
            key("Space/C-b"),
            " page  ",
            key("g/G"),
            " ends  ",
            key("[/]"),
            " file  ",
            key("h"),
            " back  ",
            key("d"),
            " diff  ",
            key("v"),
            " view  ",
            key("?"),
            " help  ",
            span!(status_style(status); "{message}", message = status.message()),
        ],
        StatusHints::DiffDocument => line![
            key("q"),
            " quit  ",
            key("f"),
            " fetch  ",
            key("r"),
            " refresh  ",
            key("j/k"),
            " scroll  ",
            key("Space/C-b"),
            " page  ",
            key("g/G"),
            " ends  ",
            key("[/]"),
            " file  ",
            key("h"),
            " back  ",
            key("s"),
            " show  ",
            key("v"),
            " view  ",
            key("?"),
            " help  ",
            span!(status_style(status); "{message}", message = status.message()),
        ],
        StatusHints::Status => line![
            key("q"),
            " quit  ",
            key("f"),
            " fetch  ",
            key("r"),
            " refresh  ",
            key("j/k"),
            " scroll  ",
            key("Space/C-b"),
            " page  ",
            key("g/G"),
            " ends  ",
            key("/"),
            " search  ",
            key("y"),
            " copy  ",
            key("h"),
            " back  ",
            key("L"),
            " log  ",
            key("J"),
            " jj  ",
            key("?"),
            " help  ",
            span!(status_style(status); "{message}", message = status.message()),
        ],
        StatusHints::Bookmarks => line![
            key("q"),
            " quit  ",
            key("f"),
            " fetch  ",
            key("r"),
            " refresh  ",
            key("j/k"),
            " move  ",
            key("g/G"),
            " ends  ",
            key("/"),
            " search  ",
            key("y"),
            " copy  ",
            key("s/Enter"),
            " show  ",
            key("h"),
            " back  ",
            key("L"),
            " log  ",
            key("J"),
            " jj  ",
            key("?"),
            " help  ",
            span!(status_style(status); "{message}", message = status.message()),
        ],
        StatusHints::OperationLog => line![
            key("q"),
            " quit  ",
            key("f"),
            " fetch  ",
            key("r"),
            " refresh  ",
            key("j/k"),
            " move  ",
            key("g/G"),
            " ends  ",
            key("/"),
            " search  ",
            key("y"),
            " copy id  ",
            key("s"),
            " show  ",
            key("d"),
            " diff  ",
            key("h"),
            " back  ",
            key("L"),
            " log  ",
            key("J"),
            " jj  ",
            key("?"),
            " help  ",
            span!(status_style(status); "{message}", message = status.message()),
        ],
    };
    Paragraph::new(line)
}

fn help_overlay(content: Text<'static>) -> Paragraph<'static> {
    Paragraph::new(content).block(Block::bordered().title("Help"))
}

fn help_overlay_text(sections: &[HelpSection]) -> Text<'static> {
    let mut lines = Vec::new();

    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            lines.push(Line::default());
        }
        lines.push(Line::styled(
            section.title().to_owned(),
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        ));
        for row in section.rows() {
            lines.push(line![
                span!(Modifier::BOLD; "{keys}", keys = row.keys()),
                "  ",
                span!("{action}", action = row.action()),
            ]);
        }
    }

    Text::from(lines)
}

fn copy_menu(options: &[CopyOption], selected: usize) -> List<'static> {
    let items = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let style = if index == selected {
                Style::default().bg(Color::Rgb(52, 54, 62))
            } else {
                Style::default()
            };
            ListItem::new(Line::from(option.label().to_owned())).style(style)
        })
        .collect::<Vec<_>>();
    List::new(items).block(Block::bordered().title("Copy"))
}

fn view_menu(options: &[ViewFormatOption], selected: usize) -> List<'static> {
    let items = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let style = if index == selected {
                Style::default().bg(Color::Rgb(52, 54, 62))
            } else {
                Style::default()
            };
            ListItem::new(Line::from(option.label().to_owned())).style(style)
        })
        .collect::<Vec<_>>();
    List::new(items).block(Block::bordered().title("View"))
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

fn key(label: &str) -> ratatui::text::Span<'_> {
    span!(Modifier::BOLD; "{label}")
}

fn status_style(status: &StatusLine) -> Style {
    match status.kind() {
        StatusKind::Ready => Style::default().fg(Color::DarkGray),
        StatusKind::Error => Style::default().fg(Color::Red),
    }
}

#[cfg(test)]
mod tests {
    use crate::command::{HelpRow, HelpSectionKind};

    use super::*;

    #[test]
    fn help_overlay_text_renders_generated_sections() {
        let text = help_overlay_text(&[
            HelpSection::new(
                HelpSectionKind::Global,
                vec![HelpRow::new("q, Esc", "quit"), HelpRow::new("?", "help")],
            ),
            HelpSection::new(
                HelpSectionKind::Direct,
                vec![
                    HelpRow::new("w", "cycle view mode"),
                    HelpRow::new("-", "none yet"),
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
        Global
        q, Esc  quit
        ?  help

        Direct Actions
        w  cycle view mode
        -  none yet
        ");
    }
}
