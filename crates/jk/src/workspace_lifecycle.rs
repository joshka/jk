//! Borderless workspace lifecycle prompts and confirmations.

use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use jk_cli::{JjWorkspaces, WorkspaceInspectionQuery};
use jk_core::JjCommandSpec;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Span, Style};
use ratatui::widgets::{Clear, Paragraph, Wrap};

const BACKGROUND: Color = Color::Rgb(30, 35, 47);
const HEADER: Color = Color::Rgb(46, 55, 72);
const MUTED: Color = Color::Rgb(161, 174, 190);
const INPUT: Color = Color::Rgb(37, 45, 59);
const DANGER: Color = Color::Rgb(153, 48, 63);
const CONFIRM: Color = Color::Rgb(28, 112, 121);
const MIN_CONFIRM_WIDTH: u16 = 40;
const MIN_CONFIRM_HEIGHT: u16 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceLifecycleKind {
    Add,
    Rename,
    Forget,
    UpdateStale,
}

impl WorkspaceLifecycleKind {
    pub const fn action(self) -> jk_core::SourceAction {
        match self {
            Self::Add => jk_core::SourceAction::WorkspaceAdd,
            Self::Rename => jk_core::SourceAction::WorkspaceRename,
            Self::Forget => jk_core::SourceAction::WorkspaceForget,
            Self::UpdateStale => jk_core::SourceAction::WorkspaceUpdateStale,
        }
    }

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Add => "Add workspace",
            Self::Rename => "Rename workspace",
            Self::Forget => "Forget workspace metadata?",
            Self::UpdateStale => "Update stale workspace?",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceLifecycleDialog {
    pub kind: WorkspaceLifecycleKind,
    workspace_name: String,
    workspace_root: PathBuf,
    input: String,
    command: Option<JjCommandSpec>,
    error: Option<String>,
    scroll: u16,
    can_confirm: bool,
}

pub enum DialogDecision {
    Stay,
    Cancel,
    Run(JjCommandSpec, WorkspaceLifecycleKind, Option<String>),
}

impl WorkspaceLifecycleDialog {
    pub fn new(
        kind: WorkspaceLifecycleKind,
        name: String,
        root: PathBuf,
        source: &JjWorkspaces,
    ) -> Self {
        let command = match kind {
            WorkspaceLifecycleKind::Forget => Some(source.forget_spec(&name)),
            WorkspaceLifecycleKind::UpdateStale => {
                Some(source.update_stale_spec(&WorkspaceInspectionQuery::new(&root)))
            }
            WorkspaceLifecycleKind::Add | WorkspaceLifecycleKind::Rename => None,
        };
        Self {
            kind,
            workspace_name: name,
            workspace_root: root,
            input: String::new(),
            command,
            error: None,
            scroll: 0,
            can_confirm: false,
        }
    }

    pub fn input(&mut self, key: KeyEvent, source: &JjWorkspaces) -> DialogDecision {
        if key.kind != KeyEventKind::Press {
            return DialogDecision::Stay;
        }
        if key.code == KeyCode::Esc
            || matches!(key.code, KeyCode::Char('n' | 'N')) && self.command.is_some()
        {
            return DialogDecision::Cancel;
        }
        if self.command.is_none() {
            match key.code {
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Char(character)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    self.input.push(character)
                }
                KeyCode::Enter => self.prepare(source),
                _ => {}
            }
            return DialogDecision::Stay;
        }
        if self.command.is_some() {
            match key.code {
                KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(8),
                KeyCode::PageDown => self.scroll = self.scroll.saturating_add(8),
                KeyCode::End => self.scroll = u16::MAX,
                KeyCode::Home => self.scroll = 0,
                _ => {}
            }
        }
        if matches!(key.code, KeyCode::Enter | KeyCode::Char('y' | 'Y')) {
            if !self.can_confirm {
                return DialogDecision::Stay;
            }
            let preferred = match self.kind {
                WorkspaceLifecycleKind::Add => self
                    .command
                    .as_ref()
                    .and_then(|_| destination_name(&self.input)),
                WorkspaceLifecycleKind::Rename => Some(self.input.trim().to_owned()),
                WorkspaceLifecycleKind::Forget => None,
                WorkspaceLifecycleKind::UpdateStale => Some(self.workspace_name.clone()),
            };
            let Some(command) = self.command.clone() else {
                return DialogDecision::Stay;
            };
            return DialogDecision::Run(command, self.kind, preferred);
        }
        DialogDecision::Stay
    }

    fn prepare(&mut self, source: &JjWorkspaces) {
        let value = self.input.trim();
        if value.is_empty() || value.contains('\0') {
            self.error = Some("Enter a non-empty value without NUL characters.".to_owned());
            return;
        }
        self.command = match self.kind {
            WorkspaceLifecycleKind::Add => {
                let destination = resolve_destination(&self.workspace_root, value);
                let Some(name) = destination
                    .file_name()
                    .and_then(|part| part.to_str())
                    .filter(|name| !name.is_empty())
                else {
                    self.error = Some("Destination must end in a valid workspace name.".to_owned());
                    return;
                };
                Some(source.add_spec(&destination, name))
            }
            WorkspaceLifecycleKind::Rename => Some(
                source.rename_spec(&WorkspaceInspectionQuery::new(&self.workspace_root), value),
            ),
            _ => self.command.clone(),
        };
        self.error = None;
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.can_confirm = self.command.is_some()
            && area.width >= MIN_CONFIRM_WIDTH
            && area.height >= MIN_CONFIRM_HEIGHT;
        let width = area.width.saturating_sub(4).min(78);
        let preferred_height = if self.command.is_some() { 21 } else { 13 };
        let height = preferred_height.min(area.height.saturating_sub(2));
        let panel = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, panel);
        frame.render_widget(Paragraph::new("").style(Style::new().bg(BACKGROUND)), panel);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("  {}", self.kind.title()),
                Style::new().fg(Color::White).bold(),
            )))
            .style(
                Style::new().bg(if self.kind == WorkspaceLifecycleKind::Forget {
                    DANGER
                } else {
                    HEADER
                }),
            ),
            Rect::new(panel.x, panel.y, panel.width, 2.min(panel.height)),
        );
        let inner = Rect::new(
            panel.x + 2,
            panel.y + 3,
            panel.width.saturating_sub(4),
            panel.height.saturating_sub(5),
        );
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Workspace  ", Style::new().fg(MUTED)),
                Span::raw(&self.workspace_name),
            ]),
            Line::from(vec![
                Span::styled("Path       ", Style::new().fg(MUTED)),
                Span::raw(self.workspace_root.display().to_string()),
            ]),
            Line::from(""),
        ];
        if let Some(command) = &self.command {
            match self.kind {
                WorkspaceLifecycleKind::Add => {
                    let destination = resolve_destination(&self.workspace_root, self.input.trim());
                    lines.push(Line::from(vec![
                        Span::styled("Destination ", Style::new().fg(MUTED)),
                        Span::raw(destination.display().to_string()),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled("Name        ", Style::new().fg(MUTED)),
                        Span::raw(destination_name(&self.input).unwrap_or_default()),
                    ]));
                    lines.push(Line::from(""));
                }
                WorkspaceLifecycleKind::Rename => {
                    lines.push(Line::from(vec![
                        Span::styled("New name    ", Style::new().fg(MUTED)),
                        Span::raw(self.input.trim().to_owned()),
                    ]));
                    lines.push(Line::from(""));
                }
                _ => {}
            }
            if self.kind == WorkspaceLifecycleKind::Forget {
                lines.push(Line::from(Span::styled(
                    "Files are not deleted. Only jj workspace metadata is forgotten.",
                    Style::new().fg(Color::LightYellow).bold(),
                )));
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "Exact command",
                Style::new().fg(MUTED),
            )));
            lines.extend(wrap_command_line(
                &command.command_preview().command_line,
                inner.width,
            ));
        } else {
            let label = if self.kind == WorkspaceLifecycleKind::Add {
                "Destination path"
            } else {
                "New workspace name"
            };
            lines.push(Line::from(Span::styled(label, Style::new().fg(MUTED))));
            lines.push(Line::from(Span::styled(
                format!(" {}_", self.input),
                Style::new().fg(Color::White).bg(INPUT),
            )));
            if let Some(error) = &self.error {
                lines.push(Line::from(Span::styled(
                    error,
                    Style::new().fg(Color::LightRed),
                )));
            }
        }
        let paragraph = Paragraph::new(lines)
            .style(Style::new().fg(Color::White).bg(BACKGROUND))
            .wrap(Wrap { trim: false });
        let content_height = paragraph.line_count(inner.width.max(1));
        let max_scroll = content_height.saturating_sub(usize::from(inner.height));
        self.scroll = self.scroll.min(max_scroll.try_into().unwrap_or(u16::MAX));
        frame.render_widget(paragraph.scroll((self.scroll, 0)), inner);
        let footer = if self.command.is_some() {
            if area.width < 56 {
                "  ↑↓ review  enter/y  esc/n  "
            } else {
                "  ↑/↓ Review    Enter / y Confirm    Esc / n Cancel  "
            }
        } else {
            "  Enter  Review command    Esc  Cancel  "
        };
        frame.render_widget(
            Paragraph::new(footer).style(Style::new().fg(Color::White).bg(
                if self.kind == WorkspaceLifecycleKind::Forget {
                    DANGER
                } else {
                    CONFIRM
                },
            )),
            Rect::new(
                panel.x + 2,
                panel.y + panel.height.saturating_sub(2),
                panel.width.saturating_sub(4),
                1,
            ),
        );
    }
}

fn resolve_destination(root: &Path, input: &str) -> PathBuf {
    let path = PathBuf::from(input);
    if path.is_absolute() {
        path
    } else {
        normalize_lexically(&root.join(path))
    }
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn destination_name(input: &str) -> Option<String> {
    Path::new(input.trim())
        .file_name()?
        .to_str()
        .map(ToOwned::to_owned)
}

fn wrap_command_line(command: &str, width: u16) -> Vec<Line<'static>> {
    let width = usize::from(width.max(1));
    let mut lines = Vec::new();
    let mut line = String::new();

    for character in command.chars() {
        let mut candidate = line.clone();
        candidate.push(character);
        if !line.is_empty() && Line::from(candidate.as_str()).width() > width {
            lines.push(Line::from(Span::styled(
                std::mem::take(&mut line),
                Style::new().fg(Color::LightCyan),
            )));
        }
        line.push(character);
    }

    lines.push(Line::from(Span::styled(
        line,
        Style::new().fg(Color::LightCyan),
    )));
    lines
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn render_dialog(dialog: &mut WorkspaceLifecycleDialog, width: u16, height: u16) {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| dialog.render(frame))
            .expect("dialog renders");
    }

    #[test]
    fn add_input_resolves_relative_destination_before_review() {
        let source = JjWorkspaces::default().with_repository("/repo/default");
        let mut dialog = WorkspaceLifecycleDialog::new(
            WorkspaceLifecycleKind::Add,
            "default".to_owned(),
            PathBuf::from("/repo/default"),
            &source,
        );
        for character in "../new workspace".chars() {
            assert!(matches!(
                dialog.input(key(KeyCode::Char(character)), &source),
                DialogDecision::Stay
            ));
        }
        assert!(matches!(
            dialog.input(key(KeyCode::Enter), &source),
            DialogDecision::Stay
        ));
        render_dialog(&mut dialog, 80, 24);
        assert!(dialog.can_confirm);

        let DialogDecision::Run(spec, kind, preferred) = dialog.input(key(KeyCode::Enter), &source)
        else {
            panic!("second enter should confirm the reviewed command");
        };
        assert_eq!(kind, WorkspaceLifecycleKind::Add);
        assert_eq!(preferred.as_deref(), Some("new workspace"));
        assert!(
            spec.command_preview()
                .command_line
                .contains("'/repo/new workspace'")
        );
    }

    #[test]
    fn forget_is_explicit_and_cancelable_without_a_command() {
        let source = JjWorkspaces::default().with_repository("/repo/default");
        let mut dialog = WorkspaceLifecycleDialog::new(
            WorkspaceLifecycleKind::Forget,
            "scratch".to_owned(),
            PathBuf::from("/repo/scratch"),
            &source,
        );

        assert!(matches!(
            dialog.input(key(KeyCode::Char('n')), &source),
            DialogDecision::Cancel
        ));
        render_dialog(&mut dialog, 80, 24);
        assert!(dialog.can_confirm);
        let DialogDecision::Run(spec, kind, preferred) =
            dialog.input(key(KeyCode::Char('y')), &source)
        else {
            panic!("y should explicitly confirm forget");
        };
        assert_eq!(kind, WorkspaceLifecycleKind::Forget);
        assert_eq!(preferred, None);
        assert!(
            spec.command_preview()
                .command_line
                .ends_with("workspace forget scratch")
        );
    }

    #[test]
    fn repeat_and_release_cannot_confirm_a_reviewed_command() {
        let source = JjWorkspaces::default().with_repository("/repo/default");
        let mut dialog = WorkspaceLifecycleDialog::new(
            WorkspaceLifecycleKind::Forget,
            "scratch".to_owned(),
            PathBuf::from("/repo/scratch"),
            &source,
        );
        render_dialog(&mut dialog, 80, 24);
        assert!(dialog.can_confirm);

        let mut repeat = key(KeyCode::Enter);
        repeat.kind = KeyEventKind::Repeat;
        assert!(matches!(
            dialog.input(repeat, &source),
            DialogDecision::Stay
        ));

        let mut release = key(KeyCode::Char('y'));
        release.kind = KeyEventKind::Release;
        assert!(matches!(
            dialog.input(release, &source),
            DialogDecision::Stay
        ));
        assert!(matches!(
            dialog.input(key(KeyCode::Char('y')), &source),
            DialogDecision::Run(..)
        ));
    }

    #[test]
    fn narrow_confirmation_renders_in_bounds_and_can_scroll_to_the_command() {
        let source = JjWorkspaces::default().with_repository("/repo/default");
        let mut dialog = WorkspaceLifecycleDialog::new(
            WorkspaceLifecycleKind::Forget,
            "scratch".to_owned(),
            PathBuf::from("/repo/scratch"),
            &source,
        );
        for _ in 0..8 {
            assert!(matches!(
                dialog.input(key(KeyCode::Down), &source),
                DialogDecision::Stay
            ));
        }

        let backend = TestBackend::new(28, 8);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| dialog.render(frame))
            .expect("narrow confirmation renders");
        assert!(!dialog.can_confirm);
        assert!(matches!(
            dialog.input(key(KeyCode::Enter), &source),
            DialogDecision::Stay
        ));
        assert!(matches!(
            dialog.input(key(KeyCode::Char('y')), &source),
            DialogDecision::Stay
        ));

        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(rendered.contains("Forget"));
        assert!(!rendered.contains('┌'));
    }

    #[test]
    fn wrapped_command_uses_rendered_lines_for_narrow_review() {
        let lines = wrap_command_line("jj workspace forget scratch", 10);
        let rendered = lines.iter().map(Line::to_string).collect::<Vec<_>>();

        assert_eq!(rendered, ["jj workspa", "ce forget ", "scratch"]);

        let root = PathBuf::from(format!("/repo/{}", "long directory ".repeat(20)));
        let source = JjWorkspaces::default().with_repository(&root);
        let mut dialog = WorkspaceLifecycleDialog::new(
            WorkspaceLifecycleKind::Forget,
            "tailmarker".to_owned(),
            root,
            &source,
        );
        render_dialog(&mut dialog, 40, 12);
        let _ = dialog.input(key(KeyCode::End), &source);
        let mut terminal = Terminal::new(TestBackend::new(40, 12)).expect("test terminal");
        terminal
            .draw(|frame| dialog.render(frame))
            .expect("long command renders");
        let rendered = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(
            rendered.contains("tailmarker"),
            "the exact command's end remains reachable"
        );
        let end = dialog.scroll;
        assert!(end > 0);
        let _ = dialog.input(key(KeyCode::Up), &source);
        render_dialog(&mut dialog, 40, 12);
        assert_eq!(dialog.scroll, end - 1);
    }
}
