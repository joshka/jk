use crossterm::event::KeyCode;

use crate::app::actions::ActionPane;

/// Outcome of one confirmation-pane key in a pure reducer context.
pub(in crate::app) enum ConfirmationKey {
    Cancel,
    Accept,
    Handled,
    Ignored,
}

/// Reduce a confirmation-pane key, updating typed text and pane scrolling in place.
pub(in crate::app) fn reduce_confirmation_key(
    input: &mut String,
    output: &mut ActionPane,
    visible_lines: u16,
    code: KeyCode,
) -> ConfirmationKey {
    match code {
        KeyCode::Esc => ConfirmationKey::Cancel,
        KeyCode::Enter => ConfirmationKey::Accept,
        KeyCode::Backspace => {
            input.pop();
            ConfirmationKey::Handled
        }
        KeyCode::Char(character) => {
            input.push(character);
            ConfirmationKey::Handled
        }
        KeyCode::Down => {
            output.scroll_down(visible_lines);
            ConfirmationKey::Handled
        }
        KeyCode::Up => {
            output.scroll_up();
            ConfirmationKey::Handled
        }
        KeyCode::PageDown => {
            output.page_down(visible_lines);
            ConfirmationKey::Handled
        }
        KeyCode::PageUp => {
            output.page_up(visible_lines);
            ConfirmationKey::Handled
        }
        KeyCode::Home => {
            output.scroll_to_top();
            ConfirmationKey::Handled
        }
        KeyCode::End => {
            output.scroll_to_bottom(visible_lines);
            ConfirmationKey::Handled
        }
        _ => ConfirmationKey::Ignored,
    }
}
