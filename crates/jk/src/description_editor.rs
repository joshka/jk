//! Description editing using ratatui-textarea, with a native terminal cursor and app-owned save
//! keys.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui_textarea::{CursorMove, TextArea};

struct CursorStop {
    row: usize,
    col: usize,
    line: usize,
    character: usize,
}

#[derive(Clone, Debug)]
pub struct DescriptionEditor {
    textarea: TextArea<'static>,
    scroll_top: usize,
    page_height: usize,
    wrap_width: usize,
}

// Mode comparisons describe the visible editing state, not undo history or cached layout.
impl PartialEq for DescriptionEditor {
    fn eq(&self, other: &Self) -> bool {
        self.textarea.lines() == other.textarea.lines()
            && self.textarea.cursor() == other.textarea.cursor()
            && self.textarea.selection_range() == other.textarea.selection_range()
    }
}
impl Eq for DescriptionEditor {}

impl From<String> for DescriptionEditor {
    fn from(text: String) -> Self {
        Self::new(&text)
    }
}

impl DescriptionEditor {
    pub fn new(text: &str) -> Self {
        let mut textarea = TextArea::new(text.split('\n').map(str::to_owned).collect());
        textarea.move_cursor(CursorMove::Bottom);
        textarea.move_cursor(CursorMove::End);
        Self {
            textarea,
            scroll_top: 0,
            page_height: 1,
            wrap_width: 77,
        }
    }

    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn input(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        let control = key.modifiers.contains(KeyModifiers::CONTROL)
            && !key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if matches!(key.code, KeyCode::Left | KeyCode::Right)
            && !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            self.prepare_selection(shift);
            let cursor = self.textarea.cursor();
            let (_, _, _, stops) = self.display_rows(self.wrap_width);
            let position = (cursor.0, cursor.1);
            let stop = if key.code == KeyCode::Left {
                stops
                    .iter()
                    .rev()
                    .find(|stop| (stop.line, stop.character) < position)
            } else {
                stops
                    .iter()
                    .find(|stop| (stop.line, stop.character) > position)
            };
            if let Some(stop) = stop {
                self.textarea.move_cursor(CursorMove::Jump(
                    u16::try_from(stop.line).unwrap_or(u16::MAX),
                    u16::try_from(stop.character).unwrap_or(u16::MAX),
                ));
            }
            return;
        }
        if matches!(key.code, KeyCode::Home | KeyCode::End) && control {
            self.prepare_selection(shift);
            if key.code == KeyCode::Home {
                self.textarea.move_cursor(CursorMove::Top);
                self.textarea.move_cursor(CursorMove::Head);
            } else {
                self.textarea.move_cursor(CursorMove::Bottom);
                self.textarea.move_cursor(CursorMove::End);
            }
            return;
        }
        if matches!(key.code, KeyCode::Char('z' | 'Z')) && control {
            if shift || key.code == KeyCode::Char('Z') {
                self.textarea.redo();
            } else {
                self.textarea.undo();
            }
            return;
        }
        if matches!(key.code, KeyCode::PageUp | KeyCode::PageDown)
            || (matches!(key.code, KeyCode::Char('v' | 'V'))
                && key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT))
        {
            self.prepare_selection(shift);
        }
        if matches!(key.code, KeyCode::Up | KeyCode::Down)
            && !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            self.prepare_selection(shift);
            self.move_visual(if key.code == KeyCode::Up { -1 } else { 1 });
            return;
        }
        match (key.code, key.modifiers) {
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.textarea.clear();
            }
            (KeyCode::Enter, modifiers) if modifiers != KeyModifiers::NONE => {
                self.textarea.insert_newline()
            }
            (KeyCode::Char('j'), KeyModifiers::CONTROL) => self.textarea.insert_newline(),
            // Page movement stays within the description instead of scrolling the underlying log.
            (KeyCode::PageDown, _) => self.move_page(CursorMove::Down),
            (KeyCode::PageUp, _) => self.move_page(CursorMove::Up),
            (KeyCode::Char('v' | 'V'), modifiers)
                if modifiers.contains(KeyModifiers::CONTROL)
                    && !modifiers.contains(KeyModifiers::ALT) =>
            {
                self.move_page(CursorMove::Down)
            }
            (KeyCode::Char('v' | 'V'), modifiers)
                if modifiers.contains(KeyModifiers::ALT)
                    && !modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.move_page(CursorMove::Up)
            }
            _ => {
                self.textarea.input(key);
            }
        }
    }

    fn prepare_selection(&mut self, shift: bool) {
        if shift && !self.textarea.is_selecting() {
            self.textarea.start_selection();
        } else if !shift {
            self.textarea.cancel_selection();
        }
    }

    fn move_page(&mut self, direction: CursorMove) {
        let distance = self.page_height as isize;
        self.move_visual(if matches!(direction, CursorMove::Up) {
            -distance
        } else {
            distance
        });
    }

    fn move_visual(&mut self, distance: isize) {
        let (rows, row, col, stops) = self.display_rows(self.wrap_width);
        let target = row.saturating_add_signed(distance).min(rows.len() - 1);
        if let Some(stop) = stops
            .iter()
            .filter(|stop| stop.row == target)
            .min_by_key(|stop| stop.col.abs_diff(col))
        {
            self.textarea.move_cursor(CursorMove::Jump(
                u16::try_from(stop.line).unwrap_or(u16::MAX),
                u16::try_from(stop.character).unwrap_or(u16::MAX),
            ));
        }
    }

    /// Build one grapheme-aware map for both text and native cursor placement. Textarea owns edits
    /// and history; its renderer currently has no public viewport offset and counts some joined
    /// emoji differently from the terminal.
    fn display_rows(&self, width: usize) -> (Vec<Line<'static>>, usize, usize, Vec<CursorStop>) {
        let cursor = self.textarea.cursor();
        let mut rows = Vec::new();
        let mut stops = Vec::new();
        let mut cursor_position = (0, 0);
        for (line_index, line) in self.textarea.lines().iter().enumerate() {
            let span = Span::raw(line);
            let mut row = Vec::new();
            let mut cells = 0;
            let mut chars = 0;
            for grapheme in span.styled_graphemes(Style::default()) {
                let size = Span::raw(grapheme.symbol).width();
                if cells + size > width && !row.is_empty() {
                    rows.push(Line::from(std::mem::take(&mut row)));
                    cells = 0;
                }
                stops.push(CursorStop {
                    row: rows.len(),
                    col: cells,
                    line: line_index,
                    character: chars,
                });
                let next_char = chars + grapheme.symbol.chars().count();
                if line_index == cursor.0 && (chars..next_char).contains(&cursor.1) {
                    cursor_position = (rows.len(), cells);
                }
                let selected = self.textarea.selection_range().is_some_and(|(start, end)| {
                    start <= (line_index, chars) && (line_index, chars) < end
                });
                let style = if selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                row.push(Span::styled(grapheme.symbol.to_owned(), style));
                cells += size;
                chars = next_char;
            }
            stops.push(CursorStop {
                row: rows.len(),
                col: cells,
                line: line_index,
                character: chars,
            });
            if line_index == cursor.0 && cursor.1 >= chars {
                cursor_position = (rows.len(), cells);
            }
            rows.push(Line::from(row));
        }
        (rows, cursor_position.0, cursor_position.1, stops)
    }

    pub fn render(&mut self, frame: &mut Frame<'_>, rev: &str) {
        let screen = frame.area();
        if screen.width < 12 || screen.height < 8 {
            return;
        }
        let width = screen.width.min(80);
        self.wrap_width = usize::from(width - 3);
        let (rows, cursor_row, cursor_col, _) = self.display_rows(self.wrap_width);
        let height = u16::try_from(rows.len().saturating_add(6))
            .unwrap_or(u16::MAX)
            .max(8)
            .min(screen.height.saturating_sub(2));
        let panel = Rect::new(
            screen.x + (screen.width - width) / 2,
            screen.y + (screen.height - height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, panel);
        frame.render_widget(Block::bordered().title("Describe revision"), panel);
        frame.render_widget(
            Paragraph::new(format!("Revision: {rev}")),
            Rect::new(panel.x + 1, panel.y + 1, width - 2, 1),
        );
        // Reserve an insertion cell at the right edge, including at exact-width line endings.
        let editor = Rect::new(panel.x + 1, panel.y + 3, width - 3, height - 5);

        self.page_height = usize::from(editor.height);
        self.scroll_top = self.scroll_top.min(cursor_row);
        if cursor_row >= self.scroll_top + self.page_height {
            self.scroll_top = cursor_row + 1 - self.page_height;
        }
        let visible = rows
            .into_iter()
            .skip(self.scroll_top)
            .take(self.page_height)
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(visible), editor);
        frame.set_cursor_position((
            editor.x + cursor_col.min(usize::from(editor.width)) as u16,
            editor.y
                + cursor_row
                    .saturating_sub(self.scroll_top)
                    .min(self.page_height - 1) as u16,
        ));
        let hint = if width >= 72 {
            "Enter save · Esc cancel · Ctrl-j newline · Ctrl-u clear · arrows move"
        } else if width >= 44 {
            "Enter save · Esc cancel · Ctrl-j newline"
        } else {
            "Enter save · Esc cancel"
        };
        frame.render_widget(
            Paragraph::new(hint),
            Rect::new(panel.x + 1, panel.bottom() - 2, width - 2, 1),
        );
    }
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn press(editor: &mut DescriptionEditor, code: KeyCode, modifiers: KeyModifiers) {
        editor.input(KeyEvent::new(code, modifiers));
    }

    #[test]
    fn edits_at_cursor_and_supports_word_navigation_delete_and_undo() {
        let mut editor = DescriptionEditor::new("fix cache later");
        press(&mut editor, KeyCode::Left, KeyModifiers::CONTROL);
        press(&mut editor, KeyCode::Char('X'), KeyModifiers::NONE);
        assert_eq!(editor.text(), "fix cache Xlater");
        press(&mut editor, KeyCode::Char('z'), KeyModifiers::CONTROL);
        assert_eq!(editor.text(), "fix cache later");
        press(&mut editor, KeyCode::Home, KeyModifiers::NONE);
        press(&mut editor, KeyCode::Delete, KeyModifiers::NONE);
        assert_eq!(editor.text(), "ix cache later");
        press(&mut editor, KeyCode::End, KeyModifiers::NONE);
        press(&mut editor, KeyCode::Backspace, KeyModifiers::NONE);
        assert_eq!(editor.text(), "ix cache late");
    }

    #[test]
    fn newline_and_clear_are_undoable_and_preserve_blank_lines() {
        let mut editor = DescriptionEditor::new("summary\n\nbody\n");
        assert_eq!(editor.text(), "summary\n\nbody\n");
        press(&mut editor, KeyCode::Char('j'), KeyModifiers::CONTROL);
        assert_eq!(editor.text(), "summary\n\nbody\n\n");
        press(&mut editor, KeyCode::Char('u'), KeyModifiers::CONTROL);
        assert_eq!(editor.text(), "");
        press(&mut editor, KeyCode::Char('z'), KeyModifiers::CONTROL);
        assert_eq!(editor.text(), "summary\n\nbody\n\n");
    }

    #[test]
    fn release_events_do_not_edit_and_shift_navigation_selects() {
        let mut editor = DescriptionEditor::new("first\nsecond\nthird");
        editor.input(KeyEvent::new_with_kind(
            KeyCode::Char('u'),
            KeyModifiers::CONTROL,
            KeyEventKind::Release,
        ));
        assert_eq!(editor.text(), "first\nsecond\nthird");
        press(
            &mut editor,
            KeyCode::Home,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert!(editor.textarea.is_selecting());
        let (rows, _, _, _) = editor.display_rows(40);
        assert!(
            rows[0].spans[0]
                .style
                .add_modifier
                .contains(Modifier::REVERSED)
        );
        press(&mut editor, KeyCode::Char('v'), KeyModifiers::NONE);
        assert_eq!(editor.text(), "v");
        press(&mut editor, KeyCode::Char('z'), KeyModifiers::CONTROL);
        // Textarea records selection deletion and replacement insertion separately.
        assert_eq!(editor.text(), "");
        press(
            &mut editor,
            KeyCode::Char('z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        assert_eq!(editor.text(), "v");
    }

    #[test]
    fn page_navigation_extends_and_cancels_selection() {
        let mut editor = DescriptionEditor::new("first\nsecond\nthird");
        press(&mut editor, KeyCode::PageUp, KeyModifiers::SHIFT);
        assert!(editor.textarea.is_selecting());
        press(&mut editor, KeyCode::PageUp, KeyModifiers::NONE);
        assert!(!editor.textarea.is_selecting());
        assert_eq!(editor.textarea.cursor().0, 0);
    }

    #[test]
    fn navigation_follows_visual_rows_and_whole_graphemes() {
        let mut editor = DescriptionEditor::new("abcdefghijklmnopqrst");
        editor.wrap_width = 5;
        press(&mut editor, KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(editor.textarea.cursor().1, 14);
        press(&mut editor, KeyCode::PageUp, KeyModifiers::NONE);
        assert_eq!(editor.textarea.cursor().1, 9);
        let mut editor = DescriptionEditor::new("e\u{301}👩‍💻");
        press(&mut editor, KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(editor.textarea.cursor().1, 2);
        press(&mut editor, KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(editor.textarea.cursor().1, 0);
        press(&mut editor, KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(editor.textarea.cursor().1, 2);
    }

    #[test]
    fn native_cursor_matches_inserted_text_across_wrapping_and_scrolling() {
        for (width, height) in [(18, 8), (32, 12), (80, 24)] {
            for text in [
                "",
                "words that wrap differently than a paragraph would wrap them",
                "界界e\u{301}👩‍💻",
                "first\n\nlast",
                &"line\n".repeat(40),
            ] {
                let mut editor = DescriptionEditor::new(text);
                let mut terminal =
                    Terminal::new(TestBackend::new(width, height)).expect("terminal");
                terminal
                    .draw(|frame| editor.render(frame, "abc123"))
                    .expect("draw");
                press(&mut editor, KeyCode::Char('X'), KeyModifiers::NONE);
                terminal
                    .draw(|frame| editor.render(frame, "abc123"))
                    .expect("draw");
                let cursor = terminal.backend().cursor_position();
                assert!(cursor.x > 0 && cursor.x < width - 1);
                assert!(cursor.y > 0 && cursor.y < height - 1);
                assert_eq!(
                    terminal.backend().buffer()[(cursor.x - 1, cursor.y)].symbol(),
                    "X",
                    "{width}x{height}: {text:?}"
                );
                press(&mut editor, KeyCode::Home, KeyModifiers::CONTROL);
                terminal
                    .draw(|frame| editor.render(frame, "abc123"))
                    .expect("draw");
                press(&mut editor, KeyCode::Char('Y'), KeyModifiers::NONE);
                terminal
                    .draw(|frame| editor.render(frame, "abc123"))
                    .expect("draw");
                let cursor = terminal.backend().cursor_position();
                assert_eq!(
                    terminal.backend().buffer()[(cursor.x - 1, cursor.y)].symbol(),
                    "Y"
                );
            }
        }
    }

    #[test]
    fn selection_replacement_and_resize_keep_cursor_visible() {
        let mut editor = DescriptionEditor::new(&"some text ".repeat(60));
        for (width, height) in [(80, 24), (32, 12), (80, 24)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
            terminal
                .draw(|frame| editor.render(frame, "abc123"))
                .expect("draw");
            press(&mut editor, KeyCode::PageUp, KeyModifiers::NONE);
            terminal
                .draw(|frame| editor.render(frame, "abc123"))
                .expect("draw");
            press(&mut editor, KeyCode::Left, KeyModifiers::SHIFT);
            press(&mut editor, KeyCode::Char('X'), KeyModifiers::NONE);
            terminal
                .draw(|frame| editor.render(frame, "abc123"))
                .expect("draw");
            let cursor = terminal.backend().cursor_position();
            assert_eq!(
                terminal.backend().buffer()[(cursor.x - 1, cursor.y)].symbol(),
                "X"
            );
        }
    }
}
