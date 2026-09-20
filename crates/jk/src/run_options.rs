//! Command-local execution options edited between operand selection and final confirmation.
//!
//! The underlying preview remains on the mode stack. Cancelling discards the draft; applying
//! replaces only that preview's spec and resets its confirmation guard. No option persists into
//! another command or changes the repository that supplied the selected operands.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use jk_core::{
    ExecutionMode, GlobalOptions, ImmutabilityPolicy, JjCommandSpec, OperationIntegrationPolicy,
    OperationLoadPolicy, SafetyClass, WorkingCopyPolicy,
};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Line, Modifier, Style, Text};
use ratatui::widgets::{Clear, Paragraph, Wrap};

use crate::state::{AppState, InputMode, InputModeResult};

/// The bounded first slice supports explicit local content/history previews only.
pub(crate) fn available(spec: &JjCommandSpec) -> bool {
    matches!(spec.mode(), ExecutionMode::ConfirmMutation)
        && matches!(
            spec.safety(),
            SafetyClass::LocalRewrite | SafetyClass::DestructiveLocal
        )
        && spec.argv().first().is_some_and(|command| {
            matches!(command.to_str(), Some("rebase" | "squash" | "restore"))
        })
        && matches!(
            spec.global_options().operation_integration(),
            OperationIntegrationPolicy::Integrate
        )
}

/// Opens an isolated draft for the command currently awaiting confirmation.
pub(crate) fn open(state: &mut AppState) {
    let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
        return;
    };
    let Some(dialog) = RunOptionsDialog::new(pending.preview.spec.clone()) else {
        return;
    };
    state.modes.push(InputMode::RunOptions {
        dialog: Box::new(dialog),
    });
}

/// Applies or discards draft options without invoking a command runner.
pub(crate) fn handle_input(state: &mut AppState, key: KeyEvent) -> InputModeResult {
    let Some(InputMode::RunOptions { dialog }) = state.modes.active_mut() else {
        return InputModeResult::Unhandled;
    };
    match dialog.input(key) {
        Decision::Stay => {}
        Decision::Cancel => {
            state.modes.pop();
        }
        Decision::Apply(spec) => {
            state.modes.pop();
            if let Some(InputMode::CommandPreview { pending }) = state.modes.active_mut() {
                pending.preview = spec.command_preview();
                pending.can_confirm = false;
                pending.scroll = 0;
                pending.copy_status = None;
            }
        }
    }
    InputModeResult::Handled
}

/// Describes skipped working-copy updates without changing a future command's success text.
pub(crate) fn success_message(spec: &JjCommandSpec, ordinary: &'static str) -> &'static str {
    if matches!(
        spec.global_options().operation(),
        OperationLoadPolicy::AtOperation(_)
    ) {
        "Completed at historical operation · latest history refreshed · C history"
    } else if spec.global_options().working_copy() == WorkingCopyPolicy::Ignore {
        "Completed without updating working-copy files · C history"
    } else {
        ordinary
    }
}

/// One row supplies its label and contextual help to both navigation and rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OptionRow {
    WorkingCopy,
    Operation,
    Immutable,
}

const ROWS: &[OptionRow] = &[
    OptionRow::WorkingCopy,
    OptionRow::Operation,
    OptionRow::Immutable,
];

impl OptionRow {
    const fn label(self) -> &'static str {
        match self {
            Self::WorkingCopy => "Working copy",
            Self::Operation => "Operation",
            Self::Immutable => "Immutable commits",
        }
    }

    const fn help(self) -> &'static str {
        match self {
            Self::WorkingCopy => "Ignore skips both snapshotting and updating working-copy files.",
            Self::Operation => {
                "Enter an operation ID (12+ hex digits), or clear it for latest. Historical edits create concurrent history and leave working-copy files unchanged."
            }
            Self::Immutable => {
                "Allow explicitly disables jj's immutable-commit protection for this command."
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunOptionsDialog {
    spec: JjCommandSpec,
    draft: GlobalOptions,
    operation_input: String,
    selected: usize,
    editing: bool,
    scroll: u16,
    max_scroll: u16,
    can_apply: bool,
    error: Option<String>,
}

enum Decision {
    Stay,
    Cancel,
    Apply(JjCommandSpec),
}

impl RunOptionsDialog {
    fn new(spec: JjCommandSpec) -> Option<Self> {
        if !available(&spec) {
            return None;
        }
        let draft = spec.global_options().clone();
        let operation_input = match draft.operation() {
            OperationLoadPolicy::AtOperation(operation) => operation.clone(),
            _ => String::new(),
        };
        Some(Self {
            spec,
            draft,
            operation_input,
            selected: 0,
            editing: false,
            scroll: 0,
            max_scroll: 0,
            can_apply: false,
            error: None,
        })
    }

    /// Historical context cannot silently retarget restore's symbolic working-copy destination.
    fn supports_history(&self) -> bool {
        self.spec
            .argv()
            .first()
            .is_some_and(|command| command != "restore")
    }

    fn command(&self) -> Result<JjCommandSpec, String> {
        let operation = self.operation_input.trim();
        let policy = if operation.is_empty() {
            OperationLoadPolicy::Latest
        } else if !self.supports_history() {
            return Err("Restore keeps its @ destination in the latest operation.".to_owned());
        } else if operation.len() < 12 || !operation.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Use an operation ID with at least 12 hexadecimal digits.".to_owned());
        } else {
            OperationLoadPolicy::AtOperation(operation.to_owned())
        };
        let mut globals = self.draft.clone().with_operation(policy);
        if !operation.is_empty() {
            globals = globals.with_working_copy(WorkingCopyPolicy::Ignore);
        }
        Ok(self.spec.clone().with_global_options(globals))
    }

    fn input(&mut self, key: KeyEvent) -> Decision {
        if key.kind != KeyEventKind::Press {
            return Decision::Stay;
        }
        if key.code == KeyCode::Esc {
            return Decision::Cancel;
        }
        if self.editing {
            self.edit_operation(key);
            return Decision::Stay;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.checked_sub(1).unwrap_or(ROWS.len() - 1);
                self.scroll = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % ROWS.len();
                self.scroll = 0;
            }
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(5).min(self.max_scroll),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(5),
            KeyCode::Enter | KeyCode::Char(' ') => self.activate(),
            KeyCode::Char('a') if self.can_apply && key.modifiers == KeyModifiers::NONE => {
                match self.command() {
                    Ok(spec) => return Decision::Apply(spec),
                    Err(error) => self.error = Some(error),
                }
            }
            _ => {}
        }
        Decision::Stay
    }

    fn activate(&mut self) {
        self.error = None;
        self.scroll = 0;
        match ROWS[self.selected] {
            OptionRow::WorkingCopy => {
                let policy = if self.draft.working_copy() == WorkingCopyPolicy::Ignore {
                    WorkingCopyPolicy::SnapshotAndUpdate
                } else {
                    WorkingCopyPolicy::Ignore
                };
                self.draft = self.draft.clone().with_working_copy(policy);
            }
            OptionRow::Operation if self.supports_history() => self.editing = true,
            OptionRow::Operation => {
                self.error =
                    Some("Restore's @ destination stays in the latest operation.".to_owned());
            }
            OptionRow::Immutable => {
                let policy = if self.draft.immutability() == ImmutabilityPolicy::Ignore {
                    ImmutabilityPolicy::Enforce
                } else {
                    ImmutabilityPolicy::Ignore
                };
                self.draft = self.draft.clone().with_immutability(policy);
            }
        }
    }

    fn edit_operation(&mut self, key: KeyEvent) {
        self.error = None;
        match key.code {
            KeyCode::Enter => self.editing = false,
            KeyCode::Backspace => {
                self.operation_input.pop();
            }
            KeyCode::Char('u') if key.modifiers == KeyModifiers::CONTROL => {
                self.operation_input.clear()
            }
            KeyCode::Char(character)
                if key.modifiers == KeyModifiers::NONE && character.is_ascii_hexdigit() =>
            {
                self.operation_input.push(character);
            }
            _ => {}
        }
    }

    fn row_value(&self, row: OptionRow) -> &'static str {
        match row {
            OptionRow::WorkingCopy if !self.operation_input.is_empty() => "ignored (history)",
            OptionRow::WorkingCopy if self.draft.working_copy() == WorkingCopyPolicy::Ignore => {
                "ignore"
            }
            OptionRow::WorkingCopy => "snapshot/update",
            OptionRow::Operation if self.editing => "editing ID",
            OptionRow::Operation if !self.operation_input.is_empty() => "specific ID below",
            OptionRow::Operation => "latest",
            OptionRow::Immutable if self.draft.immutability() == ImmutabilityPolicy::Ignore => {
                "ALLOW rewriting"
            }
            OptionRow::Immutable => "protected",
        }
    }

    fn body(&self) -> Text<'static> {
        let repository = self.spec.repository().or(self.spec.cwd()).map_or_else(
            || {
                std::env::current_dir().map_or_else(
                    |_| "current workspace (path unavailable)".to_owned(),
                    |path| path.display().to_string(),
                )
            },
            |path| path.display().to_string(),
        );
        let mut lines = vec![
            Line::from(ROWS[self.selected].help()),
            Line::from(""),
            Line::from(format!("Repository: {repository}")),
            Line::from("Fixed to the repository that supplied these operands."),
            Line::from(
                "Options affect this command only. Normal operation integration stays enabled.",
            ),
        ];
        if !self.operation_input.is_empty() {
            lines.push(Line::from(format!("Operation: {}", self.operation_input)));
            lines.push(Line::from(
                "Working-copy files are ignored. Refresh returns to latest history.",
            ));
        }
        if self.draft.immutability() == ImmutabilityPolicy::Ignore {
            lines.push(Line::styled(
                "Warning: immutable commits may be rewritten.",
                Style::new().fg(Color::Yellow),
            ));
        }
        if let Some(error) = &self.error {
            lines.push(Line::styled(error.clone(), Style::new().fg(Color::Red)));
        }
        lines.push(Line::from(""));
        match self.command() {
            Ok(spec) => lines.push(Line::from(spec.command_preview().command_line)),
            Err(error) => lines.push(Line::styled(error, Style::new().fg(Color::Red))),
        }
        Text::from(lines)
    }

    /// Draws flat terminal-default controls and a scrollable, untruncated scope/command body.
    pub(crate) fn render(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        self.can_apply = area.width >= 40 && area.height >= 14;
        if !self.can_apply {
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new("Enlarge terminal to edit run options.\nEsc cancel"),
                area,
            );
            return;
        }
        let width = area.width.min(82);
        let content_width = width - 2;
        let body_height = Paragraph::new(self.body())
            .wrap(Wrap { trim: false })
            .line_count(content_width);
        let help_height = |row: OptionRow| {
            Paragraph::new(row.help())
                .wrap(Wrap { trim: false })
                .line_count(content_width)
        };
        let longest_help = ROWS.iter().map(|row| help_height(*row)).max().unwrap_or(0);
        let stable_height = body_height + longest_help - help_height(ROWS[self.selected]) + 9;
        let height = u16::try_from(stable_height)
            .unwrap_or(u16::MAX)
            .min(area.height - 2);
        let panel = Rect::new(
            area.x + (area.width - width) / 2,
            area.y + (area.height - height) / 2,
            width,
            height,
        );
        frame.render_widget(Clear, panel);
        let inner = Rect::new(panel.x + 1, panel.y, panel.width - 2, panel.height);
        frame.render_widget(
            Paragraph::new("Run options · this command only")
                .style(Style::new().add_modifier(Modifier::BOLD)),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
        self.render_rows(frame, inner);
        let body_area = Rect::new(inner.x, inner.y + 6, inner.width, inner.height - 9);
        let body = self.body();
        let paragraph = Paragraph::new(body).wrap(Wrap { trim: false });
        self.max_scroll = u16::try_from(paragraph.line_count(body_area.width))
            .unwrap_or(u16::MAX)
            .saturating_sub(body_area.height);
        self.scroll = self.scroll.min(self.max_scroll);
        frame.render_widget(paragraph.scroll((self.scroll, 0)), body_area);
        self.render_footer(frame, inner);
    }

    fn render_rows(&self, frame: &mut Frame<'_>, inner: Rect) {
        for (index, row) in ROWS.iter().copied().enumerate() {
            let selected = self.selected == index;
            let marker = if selected { ">" } else { " " };
            let line = format!("{marker} {:<17} {}", row.label(), self.row_value(row));
            let style = if selected {
                Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::new()
            };
            frame.render_widget(
                Paragraph::new(line).style(style),
                Rect::new(inner.x, inner.y + 2 + index as u16, inner.width, 1),
            );
        }
    }

    fn render_footer(&self, frame: &mut Frame<'_>, inner: Rect) {
        let footer_y = inner.bottom() - 2;
        if self.editing {
            let value_width = usize::from(inner.width.saturating_sub(5));
            let offset = self.operation_input.len().saturating_sub(value_width);
            let input = self
                .operation_input
                .get(offset..)
                .unwrap_or(&self.operation_input);
            frame.render_widget(
                Paragraph::new(format!("ID: {input}")),
                Rect::new(inner.x, footer_y, inner.width, 1),
            );
            frame.render_widget(
                Paragraph::new("Enter done  Ctrl-u latest  Esc cancel"),
                Rect::new(inner.x, footer_y + 1, inner.width, 1),
            );
            frame.set_cursor_position((
                inner.x + 4 + u16::try_from(input.len()).unwrap_or(0),
                footer_y,
            ));
        } else {
            let scroll = if self.max_scroll > 0 {
                "  PgUp/PgDn scroll"
            } else {
                ""
            };
            frame.render_widget(
                Paragraph::new(format!("↑↓ select  Enter edit{scroll}")),
                Rect::new(inner.x, footer_y, inner.width, 1),
            );
            frame.render_widget(
                Paragraph::new("a apply to preview  Esc cancel"),
                Rect::new(inner.x, footer_y + 1, inner.width, 1),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use jk_core::{ColorPolicy, CommandPreviewWarning, ConfigOverlay, OutputPolicy};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;
    use crate::mutation_preview::PendingCommandPreview;
    use crate::test_support::{buffer_line, log_app_view};

    fn rebase_spec() -> JjCommandSpec {
        JjCommandSpec::confirm_mutation(
            [
                "rebase",
                "--revision",
                "0123456789abcdef",
                "--onto",
                "fedcba9876543210",
            ],
            SafetyClass::LocalRewrite,
        )
        .with_repository("/fixture with spaces")
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn state_with_preview() -> AppState {
        let mut state = AppState::new(log_app_view("selected"));
        state.modes.push(InputMode::CommandPreview {
            pending: PendingCommandPreview::rebase(rebase_spec().command_preview(), Vec::new()),
        });
        state
    }

    fn render(dialog: &mut RunOptionsDialog, width: u16, height: u16) -> Terminal<TestBackend> {
        let mut terminal =
            Terminal::new(TestBackend::new(width, height)).expect("terminal fixture");
        terminal
            .draw(|frame| dialog.render(frame))
            .expect("render options");
        terminal
    }

    #[test]
    fn cancel_keeps_original_preview_and_history_untouched() {
        let mut state = state_with_preview();
        let original = state.modes.active().cloned();
        open(&mut state);
        handle_input(&mut state, key(KeyCode::Enter));
        handle_input(&mut state, key(KeyCode::Esc));
        assert_eq!(state.modes.active(), original.as_ref());
        assert!(state.history.records().next().is_none());
    }

    #[test]
    fn apply_rebuilds_exact_preview_without_execution_and_requires_fresh_confirmation() {
        let mut state = state_with_preview();
        open(&mut state);
        let Some(InputMode::RunOptions { dialog }) = state.modes.active_mut() else {
            panic!("run options opened");
        };
        render(dialog, 80, 24);
        dialog.input(key(KeyCode::Enter));
        handle_input(&mut state, key(KeyCode::Char('a')));
        let Some(InputMode::CommandPreview { pending }) = state.modes.active() else {
            panic!("returned to preview");
        };
        assert_eq!(pending.preview.spec.argv(), rebase_spec().argv());
        assert_eq!(
            pending.preview.spec.repository(),
            rebase_spec().repository()
        );
        assert!(
            pending
                .preview
                .command_line
                .contains("--ignore-working-copy rebase")
        );
        assert_eq!(pending.preview, pending.preview.spec.command_preview());
        assert!(!pending.can_confirm);
        assert!(state.history.records().next().is_none());
    }

    #[test]
    fn historical_and_immutable_overrides_preserve_config_output_and_command_order() {
        let globals = GlobalOptions::default()
            .with_repository("/fixture with spaces")
            .with_output(OutputPolicy {
                color: ColorPolicy::Never,
                ..OutputPolicy::default()
            })
            .with_config_overlay(ConfigOverlay::Inline {
                name_value: "ui.quiet=true".to_owned(),
            });
        let original = rebase_spec().with_global_options(globals);
        let mut dialog = RunOptionsDialog::new(original.clone()).expect("local preview");
        dialog.operation_input = "012345abcdef".to_owned();
        dialog.selected = 2;
        dialog.activate();
        let spec = dialog.command().expect("valid options");
        assert_eq!(spec.argv(), original.argv());
        let expected_globals = original
            .global_options()
            .clone()
            .with_operation(OperationLoadPolicy::AtOperation("012345abcdef".to_owned()))
            .with_working_copy(WorkingCopyPolicy::Ignore)
            .with_immutability(ImmutabilityPolicy::Ignore);
        assert_eq!(spec.global_options(), &expected_globals);
        let command = spec.command_preview();
        assert!(
            command
                .command_line
                .find("--at-operation")
                .expect("operation option")
                < command
                    .command_line
                    .find(" rebase ")
                    .expect("command family")
        );
        assert!(
            command
                .warnings
                .contains(&CommandPreviewWarning::IgnoresWorkingCopy)
        );
        assert!(
            command
                .warnings
                .contains(&CommandPreviewWarning::IgnoresImmutableCommits)
        );
        assert!(
            !command.command_line.contains("--ignore-working-copy"),
            "at-operation implies this flag"
        );
    }

    #[test]
    fn symbolic_history_and_restore_retargeting_are_rejected() {
        let mut dialog = RunOptionsDialog::new(rebase_spec()).expect("preview");
        dialog.operation_input = "@-".to_owned();
        assert!(dialog.command().is_err());
        dialog.operation_input = "abc".to_owned();
        assert!(dialog.command().is_err());

        let spec = JjCommandSpec::confirm_mutation(
            ["restore", "--from", "abc123", "--into", "@"],
            SafetyClass::DestructiveLocal,
        );
        let mut dialog = RunOptionsDialog::new(spec).expect("restore options");
        dialog.operation_input = "012345abcdef".to_owned();
        assert!(dialog.command().is_err());
    }

    #[test]
    fn network_external_workspace_and_detached_operations_cannot_open_options() {
        for spec in [
            rebase_spec().with_safety(SafetyClass::NetworkWrite),
            rebase_spec().with_mode(ExecutionMode::ConfirmExternalTool),
            JjCommandSpec::confirm_mutation(["git", "push"], SafetyClass::NetworkWrite),
            JjCommandSpec::confirm_mutation(
                ["workspace", "add", "../other"],
                SafetyClass::LocalMetadata,
            ),
            rebase_spec().with_global_options(
                GlobalOptions::default()
                    .with_operation_integration(OperationIntegrationPolicy::DoNotIntegrate),
            ),
        ] {
            assert!(RunOptionsDialog::new(spec).is_none());
        }
    }

    #[test]
    fn controls_remain_visible_and_scope_command_body_scrolls_at_narrow_sizes() {
        for (width, height) in [(40, 14), (80, 24), (120, 40)] {
            let mut dialog = RunOptionsDialog::new(rebase_spec()).expect("preview");
            let terminal = render(&mut dialog, width, height);
            let lines = (0..height)
                .map(|row| buffer_line(terminal.backend().buffer(), row))
                .collect::<Vec<_>>();
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("a apply to preview  Esc cancel"))
            );
            assert!(lines.iter().any(|line| line.contains("Immutable commits")));
            assert!(
                lines
                    .iter()
                    .all(|line| !line.contains(['│', '─', '┌', '┐']))
            );
            assert!(dialog.can_apply);
            if width == 40 {
                assert!(dialog.max_scroll > 0);
                dialog.input(key(KeyCode::PageDown));
                assert!(dialog.scroll > 0);
            }
        }
    }

    #[test]
    fn unrendered_or_too_small_drawer_cannot_apply_and_repeat_is_ignored() {
        let mut dialog = RunOptionsDialog::new(rebase_spec()).expect("preview");
        assert!(matches!(
            dialog.input(key(KeyCode::Char('a'))),
            Decision::Stay
        ));
        render(&mut dialog, 30, 10);
        assert!(matches!(
            dialog.input(key(KeyCode::Char('a'))),
            Decision::Stay
        ));
        render(&mut dialog, 80, 24);
        let repeat =
            KeyEvent::new_with_kind(KeyCode::Enter, KeyModifiers::NONE, KeyEventKind::Repeat);
        dialog.input(repeat);
        assert_eq!(
            dialog.draft.working_copy(),
            WorkingCopyPolicy::SnapshotAndUpdate
        );
    }

    #[test]
    fn clearing_operation_restores_latest_and_success_copy_tracks_effective_options() {
        let mut dialog = RunOptionsDialog::new(rebase_spec()).expect("preview");
        dialog.selected = 1;
        dialog.activate();
        for ch in "012345abcdef".chars() {
            dialog.input(key(KeyCode::Char(ch)));
        }
        let spec = dialog.command().expect("historical options");
        assert!(success_message(&spec, "normal success").contains("historical operation"));
        dialog.input(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        dialog.input(key(KeyCode::Enter));
        let spec = dialog.command().expect("latest options");
        assert_eq!(
            spec.global_options().operation(),
            &OperationLoadPolicy::Latest
        );
        assert_eq!(success_message(&spec, "normal success"), "normal success");
    }
}
