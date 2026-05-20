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

use crate::action_menu::{ActionMenu, RolePrompt};
use crate::action_output::ActionOutput;
use crate::app::{StatusKind, StatusLine, ViewFormatOption};
use crate::command::HelpSection;
use crate::copy::CopyOption;

#[derive(Clone, Copy, Debug)]
pub enum StatusHints {
    Graph,
    ShowDocument,
    DiffDocument,
    Status,
    FileList,
    FileShowDocument,
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
        Overlay::ActionMenu { menu, selected } => {
            let area = centered_area(frame.area(), 64, menu.items().len() as u16 + 3);
            frame.render_widget(Clear, area);
            frame.render_widget(action_menu(menu, selected), area);
        }
        Overlay::PushRemotePrompt { remotes, selected } => {
            let area = centered_area(frame.area(), 46, remotes.len() as u16 + 2);
            frame.render_widget(Clear, area);
            frame.render_widget(push_remote_prompt(remotes, selected), area);
        }
        Overlay::RebasePreview { output } => {
            let title = action_output_title("Rebase", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::RolePrompt { prompt, selected } => {
            let area = centered_area(frame.area(), 54, prompt.options().len() as u16 + 4);
            frame.render_widget(Clear, area);
            frame.render_widget(role_prompt(prompt, selected), area);
        }
        Overlay::PushPreview { output } => {
            let title = action_output_title("Push", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
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
    ActionMenu {
        menu: &'a ActionMenu,
        selected: usize,
    },
    PushRemotePrompt {
        remotes: &'a [String],
        selected: usize,
    },
    PushPreview {
        output: &'a ActionOutput,
    },
    RebasePreview {
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

fn status_line(status: &StatusLine) -> Paragraph<'_> {
    let line = match status.hints() {
        StatusHints::Graph => line![
            key("Space"),
            " toggle select  ",
            key("q"),
            " quit  ",
            key("f"),
            " fetch  ",
            key("p"),
            " push  ",
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
            key("a"),
            " action menu  ",
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
            key("p"),
            " push  ",
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
        StatusHints::FileList => line![
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
            key("Enter/l"),
            " open  ",
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
        StatusHints::FileShowDocument => line![
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
            key("p"),
            " push  ",
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

fn action_menu(menu: &ActionMenu, selected: usize) -> List<'static> {
    let items = menu
        .items()
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let style = if index == selected {
                Style::default().bg(Color::Rgb(52, 54, 62))
            } else {
                Style::default()
            };
            let label = format!(
                "{}  ({})",
                item.label(),
                item.safety_tier().preview_marker()
            );
            ListItem::new(Line::from(label)).style(style)
        })
        .collect::<Vec<_>>();
    List::new(items).block(Block::bordered().title("Action menu"))
}

fn push_remote_prompt(remotes: &[String], selected: usize) -> List<'static> {
    let items = remotes
        .iter()
        .enumerate()
        .map(|(index, remote)| {
            let style = if index == selected {
                Style::default().bg(Color::Rgb(52, 54, 62))
            } else {
                Style::default()
            };
            ListItem::new(Line::from(remote.to_owned())).style(style)
        })
        .collect::<Vec<_>>();

    List::new(items).block(Block::bordered().title("Push remote"))
}

fn action_output_title(action: &str, output: &ActionOutput) -> String {
    if output.completed() {
        format!("{action} result")
    } else {
        format!("{action} preview")
    }
}

fn render_action_output(frame: &mut Frame<'_>, area: Rect, title: &str, output: &ActionOutput) {
    let block = Block::bordered().title(title.to_owned());
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

    Paragraph::new(Line::from(spans)).style(Style::default().fg(Color::Gray))
}

fn action_output_area(area: Rect, title: &str, output: &ActionOutput) -> Rect {
    let lines = output.body_lines();
    let footer = action_output_footer_text(output.completed());
    let width = lines
        .iter()
        .map(|line| line_width(line))
        .chain([line_width(&footer), line_width(title)])
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
                Style::default().bg(Color::Rgb(52, 54, 62))
            } else {
                Style::default()
            };
            ListItem::new(Line::from(option.label())).style(style)
        })
        .collect();

    let preview_hint = prompt.status_message();
    if !preview_hint.is_empty() {
        items.push(ListItem::new(Line::from(preview_hint)).style(Style::default().fg(Color::Gray)));
    }

    List::new(items)
        .block(Block::bordered().title(format!("{} (preview required)", prompt.title())))
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
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use crate::action_output::ActionOutput;
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
}
