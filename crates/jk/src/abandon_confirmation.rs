//! The decision to abandon a change: inspect its contents, cancel, or explicitly confirm.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use jk_cli::{AbandonDetails, JjCommandRunner, JjLog, SystemJjCommandRunner};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Span, Style, Stylize};
use ratatui::widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

use crate::mutations::confirm_command_preview_with_runner;
use crate::state::{AppState, InputMode};

const BACKGROUND: Color = Color::Rgb(30, 35, 47);
const MUTED: Color = Color::Rgb(161, 174, 190);

/// Retains the decision and scroll position while the user inspects the patch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbandonConfirmation {
    revision: String,
    details: Result<AbandonDetails, String>,
    probe_failed: bool,
    selected: Button,
    page: Page,
    summary_scroll: usize,
    diff_scroll: usize,
    max_scroll: usize,
    page_size: usize,
    can_review: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Button {
    ViewDiff,
    Cancel,
    Abandon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Page {
    Summary,
    Diff,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Decision {
    Stay,
    Cancel,
    Abandon,
}

impl AbandonConfirmation {
    pub fn new(
        revision: String,
        details: Result<AbandonDetails, String>,
        probe_failed: bool,
    ) -> Self {
        Self {
            revision,
            details,
            probe_failed,
            selected: Button::Cancel,
            page: Page::Summary,
            summary_scroll: 0,
            diff_scroll: 0,
            max_scroll: 0,
            page_size: 1,
            can_review: false,
        }
    }

    fn input(&mut self, key: KeyEvent) -> Decision {
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return Decision::Stay;
        }
        if matches!(key.code, KeyCode::Char('n' | 'N')) {
            return Decision::Cancel;
        }
        if !self.can_review && key.code == KeyCode::Esc {
            return Decision::Cancel;
        }
        if self.page == Page::Diff {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => self.page = Page::Summary,
                _ => self.scroll(key.code),
            }
            return Decision::Stay;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('n' | 'N') => return Decision::Cancel,
            _ if !self.can_review => return Decision::Stay,
            KeyCode::Char('y' | 'Y') => return Decision::Abandon,
            KeyCode::Left | KeyCode::BackTab => self.move_button(false),
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => self.move_button(false),
            KeyCode::Right | KeyCode::Tab => self.move_button(true),
            KeyCode::Enter | KeyCode::Char(' ') => match self.selected {
                Button::Cancel => return Decision::Cancel,
                Button::Abandon => return Decision::Abandon,
                Button::ViewDiff => self.page = Page::Diff,
            },
            _ => self.scroll(key.code),
        }
        Decision::Stay
    }

    fn move_button(&mut self, forward: bool) {
        self.selected = match (self.selected, forward, self.details.is_ok()) {
            (Button::ViewDiff, true, _) | (Button::Abandon, false, _) => Button::Cancel,
            (Button::Cancel, true, _) | (Button::ViewDiff, false, _) => Button::Abandon,
            (Button::Cancel, false, true) | (Button::Abandon, true, true) => Button::ViewDiff,
            (Button::Cancel, false, false) => Button::Abandon,
            (Button::Abandon, true, false) => Button::Cancel,
        };
    }

    fn scroll(&mut self, key: KeyCode) {
        let offset = match self.page {
            Page::Summary => &mut self.summary_scroll,
            Page::Diff => &mut self.diff_scroll,
        };
        *offset = match key {
            KeyCode::Up => offset.saturating_sub(1),
            KeyCode::Down => offset.saturating_add(1),
            KeyCode::PageUp => offset.saturating_sub(self.page_size),
            KeyCode::PageDown => offset.saturating_add(self.page_size),
            KeyCode::Home => 0,
            KeyCode::End => self.max_scroll,
            _ => *offset,
        }
        .min(self.max_scroll);
    }

    /// Draws a content-sized panel. Only the middle scrolls; the decision controls stay in place.
    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let preferred_width = self.details.as_ref().map_or(72, |details| {
            let file_width = details
                .files
                .iter()
                .map(|file| {
                    Span::raw(&file.path).width()
                        + format!("+{} -{}", file.added, file.removed).len()
                        + 5
                })
                .max()
                .unwrap_or(0);
            (file_width.max(62) + 6).min(86) as u16
        });
        let width = area.width.saturating_sub(4).min(preferred_width);
        let stacked = width < 66;
        self.can_review = width >= 34 && area.height >= if stacked { 24 } else { 19 };
        if !self.can_review {
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new("Enlarge terminal to review this change.\nEsc / n cancel")
                    .style(Style::new().fg(Color::White).bg(BACKGROUND)),
                area,
            );
            return;
        }
        let content_width = width.saturating_sub(6);
        let header = self.header(content_width);
        let consequences = self.consequences(content_width);
        let body = self.body(content_width);
        let footer_height = if self.page == Page::Diff {
            5
        } else if stacked {
            11
        } else {
            7
        };
        let fixed_height = header.len() + consequences.len() + footer_height + 4;
        if usize::from(area.height.saturating_sub(2)) <= fixed_height {
            self.can_review = false;
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new("Enlarge terminal to review this change.\nEsc / n cancel")
                    .style(Style::new().fg(Color::White).bg(BACKGROUND)),
                area,
            );
            return;
        }
        let height = fixed_height
            .saturating_add(body.len().max(1))
            .min(usize::from(area.height.saturating_sub(2)));
        let panel = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height as u16) / 2,
            width,
            height as u16,
        );
        let title = if self.page == Page::Diff {
            " Changes to abandon "
        } else {
            " Abandon this change? "
        };
        frame.render_widget(Clear, panel);
        frame.render_widget(
            Block::bordered()
                .title(title)
                .style(Style::new().fg(Color::White).bg(BACKGROUND)),
            panel,
        );
        let inner = Rect::new(
            panel.x + 3,
            panel.y + 2,
            content_width,
            panel.height.saturating_sub(4),
        );
        let body_height = inner
            .height
            .saturating_sub((header.len() + consequences.len() + footer_height) as u16);
        let body_area = Rect::new(
            inner.x,
            inner.y + header.len() as u16,
            inner.width,
            body_height,
        );
        frame.render_widget(
            Paragraph::new(header),
            Rect::new(inner.x, inner.y, inner.width, body_area.y - inner.y),
        );
        self.page_size = usize::from(body_height).max(1);
        self.max_scroll = body.len().saturating_sub(self.page_size);
        let offset = match self.page {
            Page::Summary => &mut self.summary_scroll,
            Page::Diff => &mut self.diff_scroll,
        };
        *offset = (*offset).min(self.max_scroll);
        let visible: Vec<_> = body
            .iter()
            .skip(*offset)
            .take(self.page_size)
            .cloned()
            .collect();
        frame.render_widget(Paragraph::new(visible), body_area);
        if self.max_scroll > 0 {
            let mut scrollbar = ScrollbarState::new(body.len())
                .position(*offset)
                .viewport_content_length(self.page_size);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                Rect::new(panel.x + 1, body_area.y, panel.width - 2, body_area.height),
                &mut scrollbar,
            );
        }
        let consequence_area = Rect::new(
            inner.x,
            body_area.bottom(),
            inner.width,
            consequences.len() as u16,
        );
        frame.render_widget(Paragraph::new(consequences), consequence_area);
        self.render_footer(
            frame,
            Rect::new(
                inner.x,
                consequence_area.bottom(),
                inner.width,
                footer_height as u16,
            ),
            stacked,
        );
    }

    fn header(&self, width: u16) -> Vec<Line<'static>> {
        let label = match &self.details {
            Ok(details) => format!(
                "{}{}  {}",
                if details.is_working_copy { "@ " } else { "" },
                details.change_id.chars().take(8).collect::<String>(),
                details
                    .description
                    .lines()
                    .next()
                    .filter(|s| !s.is_empty())
                    .unwrap_or("(no description)")
            ),
            Err(_) => self.revision.clone(),
        };
        vec![
            Line::from(middle_truncate(&label, width as usize)).bold(),
            Line::from(""),
        ]
    }

    fn body(&self, width: u16) -> Vec<Line<'static>> {
        match &self.details {
            Ok(details) if self.page == Page::Diff => {
                details.diff.lines().flat_map(|line| {
                    let color = if line.starts_with('+') { Color::Green } else if line.starts_with('-') { Color::Red } else { Color::White };
                    wrap_cells(line, width).into_iter().map(move |s| Line::from(s).fg(color))
                }).collect()
            }
            Ok(details) => {
                let mut lines = Vec::new();
                if self.probe_failed {
                    lines.extend(wrap("Couldn't check whether this change is empty. Review its contents before abandoning.", width).into_iter().map(|s| Line::from(s).yellow()));
                } else {
                    lines.push(Line::from(format!("Changes to abandon ({} file{}):", details.files.len(), if details.files.len() == 1 { "" } else { "s" })).bold());
                }
                lines.push(Line::from(""));
                if details.description.lines().count() > 1 {
                    lines.extend(details.description.lines().flat_map(|s| wrap(s, width)).map(Line::from));
                    lines.push(Line::from(""));
                }
                let count_width = details.files.iter().map(|file| format!("+{} −{}", file.added, file.removed).len()).max().unwrap_or(0);
                let path_width = usize::from(width).saturating_sub(count_width + 5);
                for file in &details.files {
                    let color = match file.status.as_str() { "A" => Color::Green, "D" => Color::Red, "M" => Color::Yellow, _ => Color::White };
                    let path = middle_truncate(&file.path.escape_debug().to_string(), path_width);
                    let padding = " ".repeat(path_width.saturating_sub(Span::raw(&path).width()));
                    lines.push(Line::from(vec![Span::styled(format!("{}  ", file.status), Style::new().fg(color)),
                        Span::raw(format!("{path}{padding}  ")), Span::styled(format!("+{}", file.added), Style::new().fg(Color::Green)),
                        Span::styled(format!(" −{}", file.removed), Style::new().fg(Color::Red))]));
                }
                lines
            }
            Err(error) => wrap(&format!("Couldn't load the change details.\n{error}\n\nCancel to inspect first, or choose Abandon change to proceed without a preview."), width)
                .into_iter().map(|s| Line::from(s).yellow()).collect(),
        }
    }

    fn consequences(&self, width: u16) -> Vec<Line<'static>> {
        if self.page == Page::Diff {
            return vec![Line::from("")];
        }
        let mut lines = vec![Line::from("")];
        let impact = match &self.details {
            Ok(details) if details.descendant_count == 0 => {
                "No descendant changes will be rebased.".to_owned()
            }
            Ok(details) => format!(
                "{} descendant change{} will be rebased onto its parents.",
                details.descendant_count,
                if details.descendant_count == 1 {
                    ""
                } else {
                    "s"
                }
            ),
            Err(_) => "Descendant changes may be rebased onto its parents.".to_owned(),
        };
        lines.extend(
            wrap(
                "Abandon removes this change and its edits from history.",
                width,
            )
            .into_iter()
            .map(Line::from),
        );
        lines.extend(wrap(&impact, width).into_iter().map(Line::from));
        lines.extend(
            wrap("Recover with Undo in the action menu.", width)
                .into_iter()
                .map(|s| Line::from(s).fg(MUTED)),
        );
        lines
    }

    fn render_footer(&self, frame: &mut Frame<'_>, area: Rect, stacked: bool) {
        if self.page == Page::Diff {
            draw_button(
                frame,
                Rect::new(area.x, area.y + 1, 14, 3),
                "Back",
                true,
                false,
                false,
            );
            frame.render_widget(
                Paragraph::new("↑/↓ PgUp/PgDn scroll · Esc/Enter back").fg(MUTED),
                Rect::new(area.x, area.y + 4, area.width, 1),
            );
            return;
        }
        let buttons = [
            (Button::ViewDiff, "View diff", 17),
            (Button::Cancel, "Cancel", 14),
            (Button::Abandon, "Abandon change", 22),
        ];
        let mut x = area.x;
        for (i, (button, label, width)) in buttons.into_iter().enumerate() {
            let rect = if stacked {
                Rect::new(area.x, area.y + 1 + (i * 2) as u16, area.width.min(24), 1)
            } else {
                Rect::new(x, area.y + 1, width, 3)
            };
            draw_button(
                frame,
                rect,
                label,
                self.selected == button,
                button == Button::Abandon,
                button == Button::ViewDiff && self.details.is_err(),
            );
            x += width + 2;
        }
        let scroll_hint = if self.max_scroll > 0 {
            "↑/↓ PgUp/PgDn scroll"
        } else {
            ""
        };
        let hints = if stacked {
            format!("Tab/←/→ choose · Enter/Space select\ny abandon · n/Esc cancel\n{scroll_hint}")
        } else {
            format!("Tab/←/→ choose · Enter/Space select\ny abandon · n/Esc cancel   {scroll_hint}")
        };
        frame.render_widget(
            Paragraph::new(hints).fg(MUTED),
            Rect::new(
                area.x,
                area.y + if stacked { 8 } else { 5 },
                area.width,
                if stacked { 3 } else { 2 },
            ),
        );
    }
}

/// Routes only explicit confirmation through the existing recorded mutation gateway.
pub fn handle_input(state: &mut AppState, source: &mut JjLog, key: KeyEvent) {
    handle_input_with_runner(state, source, key, SystemJjCommandRunner);
}

fn handle_input_with_runner(
    state: &mut AppState,
    source: &mut JjLog,
    key: KeyEvent,
    runner: impl JjCommandRunner,
) {
    let Some(InputMode::AbandonConfirmation { dialog, .. }) = state.modes.active_mut() else {
        return;
    };
    match dialog.input(key) {
        Decision::Stay => {}
        Decision::Cancel => {
            state.modes.pop();
        }
        Decision::Abandon => {
            if let Some(InputMode::AbandonConfirmation { pending, .. }) = state.modes.pop() {
                confirm_command_preview_with_runner(state, source, pending, runner);
            }
        }
    }
}

fn draw_button(
    frame: &mut Frame<'_>,
    area: Rect,
    label: &str,
    focused: bool,
    destructive: bool,
    disabled: bool,
) {
    let style = if disabled {
        Style::new().fg(Color::DarkGray).bg(Color::Rgb(40, 45, 55))
    } else if focused {
        Style::new().fg(Color::Black).bg(Color::LightCyan).bold()
    } else if destructive {
        Style::new()
            .fg(Color::White)
            .bg(Color::Rgb(120, 45, 50))
            .bold()
    } else {
        Style::new().fg(Color::White).bg(Color::Rgb(58, 72, 90))
    };
    frame.render_widget(Block::default().style(style), area);
    let text = if focused {
        format!("> {label} <")
    } else {
        label.to_owned()
    };
    frame.render_widget(
        Paragraph::new(text).centered().style(style),
        Rect::new(area.x, area.y + area.height / 2, area.width, 1),
    );
}

/// Hard-wrap by terminal cells so Unicode and long patch lines remain inspectable.
fn wrap_cells(text: &str, width: u16) -> Vec<String> {
    let mut lines = Vec::new();
    for logical in text.lines() {
        let span = Span::raw(logical);
        let mut line = String::new();
        let mut used = 0;
        for grapheme in span.styled_graphemes(Style::default()) {
            let size = Span::raw(grapheme.symbol).width();
            if used + size > usize::from(width.max(1)) && !line.is_empty() {
                lines.push(std::mem::take(&mut line));
                used = 0;
            }
            line.push_str(grapheme.symbol);
            used += size;
        }
        lines.push(line);
    }
    lines
}

fn wrap(text: &str, width: u16) -> Vec<String> {
    let mut lines = Vec::new();
    for logical in text.lines() {
        let mut line = String::new();
        for word in logical.split_whitespace() {
            if !line.is_empty()
                && Span::raw(&line).width() + 1 + Span::raw(word).width() > usize::from(width)
            {
                lines.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        lines.extend(wrap_cells(&line, width));
        if line.is_empty() {
            lines.push(String::new());
        }
    }
    lines
}

fn middle_truncate(text: &str, width: usize) -> String {
    let span = Span::raw(text);
    if span.width() <= width {
        return text.to_owned();
    }
    let graphemes: Vec<_> = span
        .styled_graphemes(Style::default())
        .map(|g| g.symbol)
        .collect();
    let half = width.saturating_sub(1) / 2;
    fn take(symbols: Vec<&str>, budget: usize) -> Vec<&str> {
        let mut used = 0;
        symbols
            .into_iter()
            .take_while(|s| {
                used += Span::raw(*s).width();
                used <= budget
            })
            .collect::<Vec<_>>()
    }
    let prefix = take(graphemes.clone(), half).concat();
    let mut suffix = take(
        graphemes.into_iter().rev().collect(),
        width.saturating_sub(1 + half),
    );
    suffix.reverse();
    format!("{prefix}…{}", suffix.concat())
}

#[cfg(test)]
mod tests {
    use jk_cli::{AbandonFile, AbandonQuery, JjAbandon};
    use jk_core::SourceAction;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;

    use super::*;
    use crate::mutation_preview::PendingCommandPreview;
    use crate::test_support::{SequencedRunner, buffer_line, log_app_view, output};

    fn dialog(file_count: usize) -> AbandonConfirmation {
        let details = AbandonDetails {
            change_id: "tnsrxtnmabcdefgh".to_owned(),
            description: "Add request caching\n".to_owned(),
            is_working_copy: true,
            files: (0..file_count)
                .map(|i| AbandonFile {
                    path: format!("src/cache_{i:02}.rs"),
                    status: "M".to_owned(),
                    added: 84,
                    removed: 3,
                })
                .collect(),
            descendant_count: 2,
            diff: (0..100).map(|i| format!("+cache entry {i}\n")).collect(),
        };
        AbandonConfirmation::new("tnsrxtnm".to_owned(), Ok(details), false)
    }

    fn draw(dialog: &mut AbandonConfirmation, width: u16, height: u16) -> Buffer {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
        terminal.draw(|frame| dialog.render(frame)).expect("render");
        terminal.backend().buffer().clone()
    }

    fn text(buffer: &Buffer) -> String {
        (0..buffer.area.height)
            .map(|y| buffer_line(buffer, y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn press(dialog: &mut AbandonConfirmation, code: KeyCode) -> Decision {
        dialog.input(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn default_enter_and_space_cancel_but_y_confirms() {
        for code in [
            KeyCode::Enter,
            KeyCode::Char(' '),
            KeyCode::Char('n'),
            KeyCode::Esc,
        ] {
            let mut dialog = dialog(3);
            draw(&mut dialog, 100, 30);
            assert_eq!(press(&mut dialog, code), Decision::Cancel);
        }
        let mut dialog = dialog(3);
        draw(&mut dialog, 100, 30);
        assert_eq!(press(&mut dialog, KeyCode::Char('y')), Decision::Abandon);
        assert_eq!(
            dialog.input(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL)),
            Decision::Stay
        );
    }

    #[test]
    fn buttons_navigate_and_patch_returns_to_same_decision() {
        let mut dialog = dialog(30);
        draw(&mut dialog, 100, 30);
        press(&mut dialog, KeyCode::Down);
        assert_eq!(dialog.summary_scroll, 1);
        press(&mut dialog, KeyCode::Left);
        assert_eq!(dialog.selected, Button::ViewDiff);
        assert_eq!(press(&mut dialog, KeyCode::Enter), Decision::Stay);
        draw(&mut dialog, 100, 30);
        press(&mut dialog, KeyCode::PageDown);
        assert!(dialog.diff_scroll > 1);
        assert_eq!(press(&mut dialog, KeyCode::Char('y')), Decision::Stay);
        press(&mut dialog, KeyCode::Esc);
        assert_eq!(dialog.page, Page::Summary);
        assert_eq!(dialog.summary_scroll, 1);
        press(&mut dialog, KeyCode::Tab);
        assert_eq!(dialog.selected, Button::Cancel);
        press(&mut dialog, KeyCode::Right);
        assert_eq!(press(&mut dialog, KeyCode::Char(' ')), Decision::Abandon);
        press(&mut dialog, KeyCode::BackTab);
        assert_eq!(dialog.selected, Button::Cancel);
    }

    #[test]
    fn renders_contents_buttons_and_no_internal_metadata() {
        let buffer = draw(&mut dialog(3), 100, 30);
        let rendered = text(&buffer);
        for expected in [
            "Abandon this change?",
            "@ tnsrxtnm",
            "Add request caching",
            "cache_00.rs",
            "+84",
            "−3",
            "> Cancel <",
            "Abandon change",
            "2 descendant changes",
        ] {
            assert!(rendered.contains(expected), "missing {expected}");
        }
        for forbidden in ["Safety:", "Execution:", "Refresh:", "y copy", "scroll"] {
            assert!(!rendered.contains(forbidden), "unexpected {forbidden}");
        }
        assert!(
            buffer
                .content
                .iter()
                .any(|cell| cell.bg == Color::LightCyan)
        );
        assert!(
            buffer
                .content
                .iter()
                .any(|cell| cell.bg == Color::Rgb(120, 45, 50))
        );
    }

    #[test]
    fn long_details_scroll_without_moving_buttons_and_resize_clamps() {
        let mut dialog = dialog(40);
        let first = draw(&mut dialog, 100, 30);
        press(&mut dialog, KeyCode::End);
        let last = draw(&mut dialog, 100, 30);
        assert!(!text(&last).contains("cache_00.rs"));
        assert!(text(&last).contains("cache_39.rs"));
        for y in 0..first.area.height {
            if buffer_line(&first, y).contains("> Cancel <") {
                assert_eq!(buffer_line(&first, y), buffer_line(&last, y));
            }
        }
        draw(&mut dialog, 100, 80);
        assert_eq!(dialog.summary_scroll, 0);
        assert_eq!(dialog.max_scroll, 0);
    }

    #[test]
    fn narrow_layout_stacks_buttons_and_tiny_layout_only_allows_cancel() {
        let mut dialog = dialog(3);
        let rendered = text(&draw(&mut dialog, 58, 32));
        assert!(rendered.contains("> Cancel <"));
        assert!(rendered.contains("Abandon change"));
        for width in [0, 1, 12, 30, 58, 80] {
            for height in [0, 1, 8, 16] {
                draw(&mut dialog, width, height);
                assert_eq!(press(&mut dialog, KeyCode::Char('y')), Decision::Stay);
                assert_eq!(press(&mut dialog, KeyCode::Esc), Decision::Cancel);
            }
        }
        dialog.page = Page::Diff;
        draw(&mut dialog, 30, 8);
        assert_eq!(press(&mut dialog, KeyCode::Esc), Decision::Cancel);
    }

    #[test]
    fn unavailable_details_do_not_claim_the_change_is_empty() {
        let mut dialog =
            AbandonConfirmation::new("abc".to_owned(), Err("jj failed".to_owned()), true);
        let rendered = text(&draw(&mut dialog, 100, 32));
        assert!(rendered.contains("Couldn't load the change details"));
        assert!(rendered.contains("without a preview"));
        assert!(!rendered.contains("0 files"));
        press(&mut dialog, KeyCode::Left);
        assert_eq!(dialog.selected, Button::Abandon);
        press(&mut dialog, KeyCode::Tab);
        assert_eq!(dialog.selected, Button::Cancel);
    }

    #[test]
    fn explicit_yes_uses_existing_recording_and_refresh() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut dialog = dialog(3);
        draw(&mut dialog, 100, 30);
        let pending = PendingCommandPreview::abandon(
            JjAbandon::default()
                .spec_for(&AbandonQuery::new("abc123"))
                .command_preview(),
        );
        state.modes.push(InputMode::AbandonConfirmation {
            pending,
            dialog: Box::new(dialog),
        });
        let runner = SequencedRunner::successes(vec![
            output(0, "111111111111\n", ""),
            output(0, "", "Abandoned 1 commits.\n"),
            output(0, "222222222222\n", ""),
            output(0, "log\n", ""),
            output(0, "{}\n", ""),
        ]);
        handle_input_with_runner(
            &mut state,
            &mut JjLog::default(),
            KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE),
            runner,
        );
        assert!(state.modes.active().is_none());
        assert_eq!(
            state
                .history
                .records()
                .next()
                .expect("record")
                .source
                .action,
            SourceAction::AbandonRevision
        );
    }

    #[test]
    fn default_cancel_never_invokes_the_runner() {
        let mut state = AppState::new(log_app_view("abc123"));
        let mut dialog = dialog(3);
        draw(&mut dialog, 100, 30);
        let pending = PendingCommandPreview::abandon(
            JjAbandon::default()
                .spec_for(&AbandonQuery::new("abc123"))
                .command_preview(),
        );
        state.modes.push(InputMode::AbandonConfirmation {
            pending,
            dialog: Box::new(dialog),
        });
        handle_input_with_runner(
            &mut state,
            &mut JjLog::default(),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            SequencedRunner::successes(vec![]),
        );
        assert!(state.modes.active().is_none());
        assert_eq!(state.history.records().count(), 0);
    }

    #[test]
    fn long_unicode_paths_keep_both_ends_within_cell_budget() {
        let shortened = middle_truncate("src/日本語/very/long/path/缓存.rs", 22);
        assert!(shortened.starts_with("src/"));
        assert!(shortened.ends_with("缓存.rs"));
        assert!(Span::raw(shortened).width() <= 22);
    }
}
