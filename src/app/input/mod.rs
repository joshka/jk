//! Key reducers for app-owned modal screens.
//!
//! `app.rs` keeps the event loop and normal command dispatch. This root owns
//! how active `InteractionMode` values consume key presses and which app-level
//! side effect follows when a modal selection or prompt is accepted.

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::DefaultTerminal;

use crate::app::status_line::StatusLine;
use crate::command::ViewEffect;
use crate::modes::InteractionMode;

use super::{APP_BINDINGS, App, COMMAND_PREFIX_TIMEOUT, PendingCommand};

mod abandon;
mod help;
mod menus;
mod prompts;

#[cfg(test)]
pub(in crate::app) use super::reducers::{rebase_plan_from_prompt, squash_plan_from_prompt};

impl App {
    /// Open the copy menu by asking the active view for copyable options.
    pub(super) fn open_copy_menu(&mut self, viewport_height: u16) {
        let options = match self.execute_view(crate::command::ViewCommand::Copy, viewport_height) {
            ViewEffect::CopyOptions(options) => options,
            _ => Vec::new(),
        };
        if options.is_empty() {
            self.status = StatusLine::with_message(&self.view, "nothing to copy");
        } else {
            self.mode = InteractionMode::CopyMenu {
                options,
                selected: 0,
            };
        }
    }

    #[cfg(test)]
    pub(super) fn handle_mode_key(&mut self, code: KeyCode, viewport_height: u16) -> Result<bool> {
        self.handle_mode_key_event_with_terminal(
            KeyEvent::new(code, crossterm::event::KeyModifiers::NONE),
            viewport_height,
            None,
        )
    }

    pub(super) fn handle_mode_key_event(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
    ) -> Result<bool> {
        self.handle_mode_key_event_with_terminal(key, viewport_height, None)
    }

    /// Route one key through modal dispatch, including action previews that may need terminal
    /// handoff.
    pub(super) fn handle_mode_key_event_with_terminal(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
        terminal: Option<&mut DefaultTerminal>,
    ) -> Result<bool> {
        if matches!(self.mode, InteractionMode::Help) {
            return self.handle_help_key(key, viewport_height);
        }

        let code = key.code;
        if self.handle_common_action_preview_key(code, viewport_height, terminal) {
            return Ok(true);
        }

        self.handle_active_mode_key(code, viewport_height)
    }

    /// Dispatch a key to the currently active non-preview interaction mode.
    fn handle_active_mode_key(&mut self, code: KeyCode, viewport_height: u16) -> Result<bool> {
        match &mut self.mode {
            InteractionMode::Normal => Ok(false),
            InteractionMode::Help => unreachable!("help mode is handled before borrowing mode"),
            InteractionMode::SearchPrompt(_) => {
                self.handle_search_prompt_key(code, viewport_height)
            }
            InteractionMode::LogRevsetPrompt(_) => self.handle_log_revset_prompt_key(code),
            InteractionMode::CopyMenu { .. } => self.handle_copy_menu_key(code),
            InteractionMode::ViewMenu { .. } => self.handle_view_menu_key(code, viewport_height),
            InteractionMode::ActionMenu { .. } => self.handle_action_menu_key(code),
            InteractionMode::RolePrompt { .. } => self.handle_role_prompt_key(code),
            InteractionMode::DescribePrompt { .. } => self.handle_describe_prompt_key(code),
            InteractionMode::CommitPrompt(_) => self.handle_commit_prompt_key(code),
            InteractionMode::BookmarkNamePrompt { .. } => {
                self.handle_bookmark_name_prompt_key(code)
            }
            InteractionMode::BookmarkRenamePrompt { .. } => {
                self.handle_bookmark_rename_prompt_key(code)
            }
            InteractionMode::AbandonPreview { .. } => {
                self.handle_abandon_preview_key(code, viewport_height)
            }
            InteractionMode::AbandonConfirm { .. } => {
                self.handle_abandon_confirm_key(code, viewport_height)
            }
            InteractionMode::PushRemotePrompt { .. } => self.handle_push_remote_prompt_key(code),
            InteractionMode::FetchRemotePrompt { .. } => self.handle_fetch_remote_prompt_key(code),
            InteractionMode::DescribePreview { .. }
            | InteractionMode::CommitPreview { .. }
            | InteractionMode::BookmarkMutationPreview { .. }
            | InteractionMode::FileMutationPreview { .. }
            | InteractionMode::NewPreview { .. }
            | InteractionMode::DuplicatePreview { .. }
            | InteractionMode::RebasePreview { .. }
            | InteractionMode::SplitPreview { .. }
            | InteractionMode::RestorePreview { .. }
            | InteractionMode::RevertPreview { .. }
            | InteractionMode::SquashPreview { .. }
            | InteractionMode::AbsorbPreview { .. }
            | InteractionMode::FetchPreview { .. }
            | InteractionMode::PushPreview { .. }
            | InteractionMode::OperationRecoveryPreview { .. }
            | InteractionMode::OperationTargetPreview { .. }
            | InteractionMode::WorkingCopyNavigationPreview { .. } => {
                unreachable!("common action preview modes are handled before borrowing mode")
            }
        }
    }
}
