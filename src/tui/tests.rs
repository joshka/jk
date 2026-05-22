use insta::assert_snapshot;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::app::actions::ActionPane;
use crate::app::status_line::{StatusKind, StatusLine};
use crate::command::{HelpRow, HelpSection, HelpSectionKind};
use crate::menus::RolePrompt;
use crate::tui::theme;

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
        StatusHints::Log,
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
    let menu = crate::menus::build_action_menu(&crate::menus::ExactActionContext::with_current(
        "change-a",
    ));

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
    let menu = crate::menus::build_action_menu(&crate::menus::ExactActionContext::with_current(
        "change-a",
    ));
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
    let menu = crate::menus::build_action_menu(&crate::menus::ExactActionContext::with_current(
        "change-a",
    ));

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
            crate::menus::RolePromptOption::new("source", "source-a"),
            crate::menus::RolePromptOption::new("destination", "dest-a"),
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
fn action_pane_render_keeps_footer_visible_while_body_scrolls() {
    let mut output = ActionPane::pending(
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
            render_action_pane(
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
fn action_pane_overlay_renders_common_preview_title_and_footer() {
    let output = ActionPane::pending(
        "jj git fetch --remote exact:\"origin\"".to_owned(),
        "fetch preview".to_owned(),
        None,
    );
    let status = StatusLine::test("jk log", "ready", StatusKind::Ready, StatusHints::Log);

    let rendered = render_widget_rows(80, 8, |frame| {
        render_overlay(
            frame,
            &status,
            Overlay::ActionPane {
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
fn action_pane_overlay_renders_common_result_title_and_footer() {
    let output = ActionPane::finished(
        "jj git fetch".to_owned(),
        "fetched".to_owned(),
        Some("default fetch uses jj git fetch remote resolution".to_owned()),
    );
    let status = StatusLine::test("jk log", "ready", StatusKind::Ready, StatusHints::Log);

    let rendered = render_widget_rows(80, 8, |frame| {
        render_overlay(
            frame,
            &status,
            Overlay::ActionPane {
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
    let output = ActionPane::pending(
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
