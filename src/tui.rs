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
    Resolve,
    FileList,
    FileShowDocument,
    Bookmarks,
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
        Overlay::NewPreview { output } => {
            let title = action_output_title("New change", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::DescribePreview { output } => {
            let title = action_output_title("Describe", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::CommitPreview { output } => {
            let title = action_output_title("Commit", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::BookmarkMutationPreview { output } => {
            let title = action_output_title("Bookmark", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::RebasePreview { output } => {
            let title = action_output_title("Rebase", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::RestorePreview { output } => {
            let title = action_output_title("Restore", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::RevertPreview { output } => {
            let title = action_output_title("Revert", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::SquashPreview { output } => {
            let title = action_output_title("Squash", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::AbsorbPreview { output } => {
            let title = action_output_title("Absorb", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::AbandonPreview { output } => {
            let title = action_output_title("Abandon", output);
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
        Overlay::PushPreview { output } => {
            let title = action_output_title("Push", output);
            let area = action_output_area(frame.area(), &title, output);
            frame.render_widget(Clear, area);
            render_action_output(frame, area, &title, output);
        }
        Overlay::OperationRecoveryPreview { output } => {
            let title = action_output_title("Operation recovery", output);
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
    NewPreview {
        output: &'a ActionOutput,
    },
    DescribePreview {
        output: &'a ActionOutput,
    },
    CommitPreview {
        output: &'a ActionOutput,
    },
    BookmarkMutationPreview {
        output: &'a ActionOutput,
    },
    OperationRecoveryPreview {
        output: &'a ActionOutput,
    },
    RebasePreview {
        output: &'a ActionOutput,
    },
    RestorePreview {
        output: &'a ActionOutput,
    },
    RevertPreview {
        output: &'a ActionOutput,
    },
    SquashPreview {
        output: &'a ActionOutput,
    },
    AbsorbPreview {
        output: &'a ActionOutput,
    },
    AbandonPreview {
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
    let hint_spans = status_hint_spans(status.hints(), width).spans;

    if !hint_spans.is_empty() {
        spans.extend(line!["  "].spans);
        spans.extend(hint_spans);
    }

    Line::from(spans)
}

fn status_hint_spans(hints: StatusHints, width: u16) -> Line<'static> {
    let compact = width < 60;

    match (hints, compact) {
        (StatusHints::Graph, true) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("?"),
            " help",
        ],
        (StatusHints::Graph, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("Enter/l"),
            " open  ",
            key("Space"),
            " select  ",
            key("a"),
            " action  ",
            key("D/C"),
            " describe/commit  ",
            key("b/=/m"),
            " bookmark  ",
            key("f/p/r"),
            " fetch/push/refresh  ",
            key("?"),
            " help",
        ],
        (StatusHints::ShowDocument, true)
        | (StatusHints::DiffDocument, true)
        | (StatusHints::FileShowDocument, true)
        | (StatusHints::OperationDetailDocument, true) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " scroll  ",
            key("?"),
            " help",
        ],
        (StatusHints::ShowDocument, false) => line![
            key("q"),
            " quit  ",
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
            key("?"),
            " help",
        ],
        (StatusHints::DiffDocument, false) => line![
            key("q"),
            " quit  ",
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
            key("?"),
            " help",
        ],
        (StatusHints::Status, true) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " scroll  ",
            key("?"),
            " help",
        ],
        (StatusHints::Status, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " scroll  ",
            key("Space/C-b"),
            " page  ",
            key("/"),
            " search  ",
            key("y"),
            " copy  ",
            key("D/C"),
            " describe/commit @  ",
            key("b/=/m"),
            " bookmark @  ",
            key("h"),
            " back  ",
            key("?"),
            " help",
        ],
        (StatusHints::Resolve, true) | (StatusHints::FileList, true) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("?"),
            " help",
        ],
        (StatusHints::Resolve, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("Enter/l"),
            " inspect  ",
            key("/"),
            " search  ",
            key("y"),
            " copy  ",
            key("?"),
            " help",
        ],
        (StatusHints::FileList, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("Enter/l"),
            " open  ",
            key("/"),
            " search  ",
            key("y"),
            " copy  ",
            key("?"),
            " help",
        ],
        (StatusHints::FileShowDocument, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " scroll  ",
            key("Space/C-b"),
            " page  ",
            key("/"),
            " search  ",
            key("h"),
            " back  ",
            key("?"),
            " help",
        ],
        (StatusHints::OperationDetailDocument, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " scroll  ",
            key("Space/C-b"),
            " page  ",
            key("s/d"),
            " show/diff  ",
            key("h"),
            " back  ",
            key("?"),
            " help",
        ],
        (StatusHints::Bookmarks, true) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("?"),
            " help",
        ],
        (StatusHints::Bookmarks, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("Enter/s"),
            " show  ",
            key("/"),
            " search  ",
            key("y"),
            " copy  ",
            key("x"),
            " delete  ",
            key("?"),
            " help",
        ],
        (StatusHints::OperationLog, true) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("u"),
            " undo  ",
            key("C-r"),
            " redo  ",
            key("?"),
            " help",
        ],
        (StatusHints::OperationLog, false) => line![
            key("q"),
            " quit  ",
            key("j/k"),
            " move  ",
            key("u"),
            " undo  ",
            key("C-r"),
            " redo  ",
            key("s"),
            " show  ",
            key("d"),
            " diff  ",
            key("/"),
            " search  ",
            key("y"),
            " copy id  ",
            key("?"),
            " help",
        ],
    }
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

fn render_abandon_confirm(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    input: &str,
    output: &ActionOutput,
) {
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

    Paragraph::new(Line::from(spans)).style(Style::default().fg(Color::Gray))
}

fn abandon_confirm_footer(input: &str) -> Paragraph<'static> {
    Paragraph::new(Line::from(abandon_confirm_footer_text(input)))
        .style(Style::default().fg(Color::Gray))
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
    use insta::assert_snapshot;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use crate::action_output::ActionOutput;
    use crate::app::{StatusKind, StatusLine};
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
    fn status_chrome_renders_compact_hints_on_narrow_width() {
        let status = StatusLine::test(
            "jk log",
            "push cancelled",
            StatusKind::Error,
            StatusHints::Graph,
        );

        assert_snapshot!(render_chrome_snapshot(&status, 48), @r"
        title|jk log
        status|push cancelled  q quit  j/k move  ? help
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
        status|19 operations  q quit  j/k move  u undo  C-r redo  s show  d diff  / search  y copy id  ? help
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
        status|1 files  q quit  j/k move  Enter/l open  / search  y copy  ? help
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
        status|1 conflicts  q quit  j/k move  Enter/l inspect  / search  y copy  ? help
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
