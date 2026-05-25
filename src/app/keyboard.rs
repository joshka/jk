//! App-owned keyboard dispatch and key-prefix resolution.
//!
//! This root owns key interpretation beneath terminal event routing:
//! modal-first dispatch, normal binding dispatch, and prefix replay.

use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::status_line::StatusLine;
use crate::app::{APP_BINDINGS, App, PendingCommand};
use crate::command::{BindingMatch, binding_prefix_next_labels, match_binding_sequence};
use crate::modes::InteractionMode;

/// Keep multi-key prefixes responsive without making fallback bindings feel hair-trigger.
pub(super) const COMMAND_PREFIX_TIMEOUT: Duration = Duration::from_millis(700);

impl App {
    /// Route one pressed key through modal dispatch before falling back to normal bindings.
    ///
    /// The active mode gets the first chance to consume the key. Only when the mode reports that
    /// it did not handle the key does normal binding dispatch run and optionally refresh ready
    /// status text.
    pub fn handle_key_press(&mut self, key: KeyEvent) -> Result<()> {
        if self.handle_mode_key_event_inner(key)? {
            return Ok(());
        }

        let refresh_status = self.handle_normal_key(key)?;
        if refresh_status && self.status.is_ready() {
            self.status = StatusLine::ready(&self.view);
        }

        Ok(())
    }

    /// Route one key through the active mode and then run any queued interactive handoff.
    pub fn handle_mode_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        let handled = self.handle_mode_key_event_inner(key)?;
        self.run_pending_interactive_action(None)?;
        Ok(handled)
    }

    /// Route one key through modal dispatch without running any queued interactive terminal
    /// handoff.
    pub fn handle_mode_key_event_inner(&mut self, key: KeyEvent) -> Result<bool> {
        let viewport_height = self.main_viewport_height();
        if matches!(self.mode, InteractionMode::Help) {
            return self.handle_help_key(key, viewport_height);
        }

        let code = key.code;
        if self.handle_common_action_preview_key(code, viewport_height) {
            return Ok(true);
        }

        self.handle_active_mode_key(code)
    }

    /// Dispatch a key to the currently active non-preview interaction mode.
    fn handle_active_mode_key(&mut self, code: KeyCode) -> Result<bool> {
        let viewport_height = self.main_viewport_height();
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

    /// Dispatch one normal-mode key using the current time for prefix resolution.
    pub fn handle_normal_key(&mut self, key: KeyEvent) -> Result<bool> {
        self.handle_normal_key_at_viewport_height(key, self.main_viewport_height(), Instant::now())
    }

    /// Dispatch one normal-mode key with an explicit prefix-resolution timestamp.
    ///
    /// Tests call this variant with a controlled timestamp so prefix fallback behavior stays
    /// deterministic.
    pub fn handle_normal_key_at_viewport_height(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if self.pending_command.is_some() {
            return self.handle_pending_command_key_at_viewport_height(key, viewport_height, now);
        }

        let keys = [key];
        let Some(binding_match) =
            match_binding_sequence(&[APP_BINDINGS, self.view.bindings()], &keys)
        else {
            return Ok(true);
        };

        match binding_match {
            BindingMatch::Exact(binding) => self.execute_binding(binding, viewport_height),
            BindingMatch::Prefix { fallback } => {
                self.pending_command = Some(PendingCommand {
                    keys: keys.to_vec(),
                    fallback,
                    deadline: now + COMMAND_PREFIX_TIMEOUT,
                });
                self.status = StatusLine::with_message(
                    &self.view,
                    prefix_status_message(
                        "prefix",
                        &keys,
                        &binding_prefix_next_labels(&[APP_BINDINGS, self.view.bindings()], &keys),
                    ),
                );
                Ok(false)
            }
        }
    }

    /// Continue or complete a multi-key prefix that was already in progress.
    pub fn handle_pending_command_key_at_viewport_height(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if self
            .pending_command
            .as_ref()
            .is_some_and(|pending| now >= pending.deadline)
        {
            self.run_pending_fallback_at_viewport_height(viewport_height)?;
            return self.handle_key_after_prefix_fallback_at_viewport_height(
                key,
                viewport_height,
                now,
            );
        }

        if key.code == KeyCode::Esc {
            self.pending_command = None;
            self.status = StatusLine::with_message(&self.view, "prefix cancelled");
            return Ok(false);
        }

        let Some(mut pending) = self.pending_command.take() else {
            return Ok(true);
        };
        pending.keys.push(key);

        match match_binding_sequence(&[APP_BINDINGS, self.view.bindings()], &pending.keys) {
            Some(BindingMatch::Exact(binding)) => self.execute_binding(binding, viewport_height),
            Some(BindingMatch::Prefix { fallback }) => {
                self.status = StatusLine::with_message(
                    &self.view,
                    prefix_status_message(
                        "prefix",
                        &pending.keys,
                        &binding_prefix_next_labels(
                            &[APP_BINDINGS, self.view.bindings()],
                            &pending.keys,
                        ),
                    ),
                );
                self.pending_command = Some(PendingCommand {
                    keys: pending.keys,
                    fallback,
                    deadline: now + COMMAND_PREFIX_TIMEOUT,
                });
                Ok(false)
            }
            None => {
                if let Some(fallback) = pending.fallback {
                    self.run_binding_with_status_refresh_at_viewport_height(
                        fallback,
                        viewport_height,
                    )?;
                    self.handle_key_after_prefix_fallback_at_viewport_height(
                        key,
                        viewport_height,
                        now,
                    )
                } else {
                    self.status = StatusLine::with_message(&self.view, "unknown command prefix");
                    Ok(false)
                }
            }
        }
    }

    /// Execute any prefix fallback whose timeout expired without another key.
    pub fn flush_expired_pending_command(&mut self) -> Result<()> {
        let Some(pending) = self.pending_command.as_ref() else {
            return Ok(());
        };
        if Instant::now() < pending.deadline {
            return Ok(());
        }

        self.run_pending_fallback()?;
        Ok(())
    }

    /// Run the exact binding that a prefix should fall back to when the longer match fails.
    pub fn run_pending_fallback(&mut self) -> Result<()> {
        let viewport_height = self.main_viewport_height();
        let fallback = self
            .pending_command
            .take()
            .and_then(|pending| pending.fallback);

        if matches!(self.mode, InteractionMode::Help) {
            let Some(binding) = fallback else {
                self.status = StatusLine::with_message(&self.view, "unknown help command prefix");
                return Ok(());
            };
            self.execute_help_binding(binding, viewport_height)?;
            return Ok(());
        }

        let Some(binding) = fallback else {
            self.status = StatusLine::ready(&self.view);
            return Ok(());
        };
        self.run_binding_with_status_refresh_at_viewport_height(binding, viewport_height)?;

        Ok(())
    }

    /// Execute one binding and refresh ready status text if the binding says status is stale.
    pub fn run_binding_with_status_refresh_at_viewport_height(
        &mut self,
        binding: crate::command::Binding,
        viewport_height: u16,
    ) -> Result<()> {
        let refresh_status = self.execute_binding(binding, viewport_height)?;
        if refresh_status && self.status.is_ready() {
            self.status = StatusLine::ready(&self.view);
        }
        Ok(())
    }

    /// Replay the current key after a prefix fallback has already run.
    ///
    /// This preserves the user expectation that the suffix key is still interpreted after the
    /// shorter binding consumed the prefix.
    pub fn handle_key_after_prefix_fallback_at_viewport_height(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if matches!(self.mode, InteractionMode::Normal) {
            self.handle_normal_key_at_viewport_height(key, viewport_height, now)
        } else {
            self.handle_mode_key_event_at_viewport_height(key, viewport_height)
        }
    }

    #[cfg(test)]
    pub fn handle_mode_key_at_viewport_height(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> Result<bool> {
        let handled = self.handle_mode_key_event_inner_at_viewport_height(
            KeyEvent::new(code, crossterm::event::KeyModifiers::NONE),
            viewport_height,
        )?;
        self.run_pending_interactive_action(None)?;
        Ok(handled)
    }

    #[cfg(test)]
    pub fn handle_normal_key_at_viewport_height_for_test(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
    ) -> Result<bool> {
        self.handle_normal_key_at_viewport_height(key, viewport_height, Instant::now())
    }

    #[cfg(test)]
    pub fn flush_expired_pending_command_at_viewport_height(
        &mut self,
        viewport_height: u16,
    ) -> Result<()> {
        let Some(pending) = self.pending_command.as_ref() else {
            return Ok(());
        };
        if Instant::now() < pending.deadline {
            return Ok(());
        }

        self.run_pending_fallback_at_viewport_height(viewport_height)
    }

    fn handle_mode_key_event_at_viewport_height(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
    ) -> Result<bool> {
        let handled = self.handle_mode_key_event_inner_at_viewport_height(key, viewport_height)?;
        self.run_pending_interactive_action(None)?;
        Ok(handled)
    }

    fn handle_mode_key_event_inner_at_viewport_height(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
    ) -> Result<bool> {
        if matches!(self.mode, InteractionMode::Help) {
            return self.handle_help_key(key, viewport_height);
        }

        let code = key.code;
        if self.handle_common_action_preview_key(code, viewport_height) {
            return Ok(true);
        }

        self.handle_active_mode_key_at_viewport_height(code, viewport_height)
    }

    fn handle_active_mode_key_at_viewport_height(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> Result<bool> {
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

    pub fn run_pending_fallback_at_viewport_height(&mut self, viewport_height: u16) -> Result<()> {
        let fallback = self
            .pending_command
            .take()
            .and_then(|pending| pending.fallback);

        if matches!(self.mode, InteractionMode::Help) {
            let Some(binding) = fallback else {
                self.status = StatusLine::with_message(&self.view, "unknown help command prefix");
                return Ok(());
            };
            self.execute_help_binding(binding, viewport_height)?;
            return Ok(());
        }

        let Some(binding) = fallback else {
            self.status = StatusLine::ready(&self.view);
            return Ok(());
        };
        self.run_binding_with_status_refresh_at_viewport_height(binding, viewport_height)?;

        Ok(())
    }
}

/// Format a pressed key sequence the same way prefix status text presents it.
fn binding_key_label(keys: &[KeyEvent]) -> String {
    keys.iter()
        .map(|key| match key.code {
            KeyCode::Char(character) if key.modifiers.is_empty() => character.to_string(),
            KeyCode::Char(character) => format!("{:?}-{character}", key.modifiers),
            _ => format!("{:?}", key.code),
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Build the status-line message shown while a multi-key prefix is still pending.
pub fn prefix_status_message(prefix: &str, keys: &[KeyEvent], next: &[String]) -> String {
    let keys = binding_key_label(keys);
    if next.is_empty() {
        format!("{prefix}: {keys}")
    } else {
        format!("{prefix}: {keys} -> {}", next.join("/"))
    }
}
