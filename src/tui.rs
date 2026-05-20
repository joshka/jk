//! Shared terminal chrome.
//!
//! View slices render their own main content. This module only owns the title
//! bar, status bar, and modal overlays that frame those views.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui_macros::{line, span, text, vertical};

use crate::app::{StatusKind, StatusLine, ViewFormatOption};
use crate::copy::CopyOption;

#[derive(Clone, Copy, Debug)]
pub enum StatusHints {
    Graph,
    ShowDocument,
    DiffDocument,
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

pub fn render_overlay(frame: &mut Frame<'_>, status: &StatusLine, overlay: Overlay<'_>) {
    match overlay {
        Overlay::None => {}
        Overlay::Help => {
            let area = centered_area(frame.area(), 62, 12);
            frame.render_widget(Clear, area);
            frame.render_widget(help_overlay(status.hints()), area);
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
    Help,
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
    };
    Paragraph::new(line)
}

fn help_overlay(hints: StatusHints) -> Paragraph<'static> {
    let lines = match hints {
        StatusHints::Graph => text![
            "q/Esc quit    r refresh    ? close help",
            "j/k move      g/G ends     h back",
            "l/s show      d diff       w mode",
            "W revset      J jj         v view"
        ],
        StatusHints::ShowDocument => text![
            "q/Esc quit    r refresh    ? close help",
            "j/k scroll    g/G ends     h back",
            "[/] file      Space/C-f page down",
            "PageUp/C-b page up         d diff    v view"
        ],
        StatusHints::DiffDocument => text![
            "q/Esc quit    r refresh    ? close help",
            "j/k scroll    g/G ends     h back",
            "[/] file      Space/C-f page down",
            "PageUp/C-b page up         s show    v view"
        ],
    };

    Paragraph::new(lines).block(Block::bordered().title("Help"))
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
