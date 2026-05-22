//! App-owned normal-mode and prefix dispatch.
//!
//! The event loop lives in `app/mod.rs`, but this module owns the two smaller
//! state machines beneath it:
//!
//! - prefix resolution and fallback replay for multi-key bindings
//! - app-level binding execution after a concrete binding has been chosen

use std::time::Instant;

use color_eyre::Result;
use crossterm::event::KeyCode;

use crate::actions::{JjBookmarkMutationKind, JjWorkingCopyNavigationKind};
use crate::app::status_line::{StatusKind, StatusLine};
use crate::command::{
    Binding, BindingMatch, Command, ViewCommand, binding_prefix_next_labels, match_binding_sequence,
};
use crate::modes::InteractionMode;

use super::{APP_BINDINGS, App, COMMAND_PREFIX_TIMEOUT, PendingCommand};

impl App {
    /// Continue or complete a multi-key prefix that was already in progress.
    pub fn handle_pending_command_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if self
            .pending_command
            .as_ref()
            .is_some_and(|pending| now >= pending.deadline)
        {
            self.run_pending_fallback(viewport_height)?;
            return self.handle_key_after_prefix_fallback(key, viewport_height, now);
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
                    self.run_binding_with_status_refresh(fallback, viewport_height)?;
                    self.handle_key_after_prefix_fallback(key, viewport_height, now)
                } else {
                    self.status = StatusLine::with_message(&self.view, "unknown command prefix");
                    Ok(false)
                }
            }
        }
    }

    /// Execute any prefix fallback whose timeout expired without another key.
    pub fn flush_expired_pending_command(&mut self, viewport_height: u16) -> Result<()> {
        let Some(pending) = self.pending_command.as_ref() else {
            return Ok(());
        };
        if Instant::now() < pending.deadline {
            return Ok(());
        }

        self.run_pending_fallback(viewport_height)?;
        Ok(())
    }

    /// Run the exact binding that a prefix should fall back to when the longer
    /// match fails.
    pub fn run_pending_fallback(&mut self, viewport_height: u16) -> Result<()> {
        let fallback = self
            .pending_command
            .take()
            .and_then(|pending| pending.fallback);
        if matches!(self.mode, InteractionMode::Help) {
            if let Some(binding) = fallback {
                self.execute_help_binding(binding, viewport_height)?;
            } else {
                self.status = StatusLine::with_message(&self.view, "unknown help command prefix");
            }
        } else if let Some(binding) = fallback {
            self.run_binding_with_status_refresh(binding, viewport_height)?;
        } else {
            self.status = StatusLine::ready(&self.view);
        }
        Ok(())
    }

    /// Execute one binding and refresh ready status text if the binding says
    /// status is stale.
    pub fn run_binding_with_status_refresh(
        &mut self,
        binding: Binding,
        viewport_height: u16,
    ) -> Result<()> {
        let refresh_status = self.execute_binding(binding, viewport_height)?;
        if refresh_status && matches!(self.status.kind(), StatusKind::Ready) {
            self.status = StatusLine::ready(&self.view);
        }
        Ok(())
    }

    /// Replay the current key after a prefix fallback has already run.
    ///
    /// This preserves the user expectation that the suffix key is still
    /// interpreted after the shorter binding consumed the prefix.
    pub fn handle_key_after_prefix_fallback(
        &mut self,
        key: crossterm::event::KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if matches!(self.mode, InteractionMode::Normal) {
            self.handle_normal_key_at(key, viewport_height, now)
        } else {
            self.handle_mode_key_event(key, viewport_height)
        }
    }

    /// Execute one app-level or view-level binding at the app root.
    ///
    /// This is the boundary where global commands, app-owned previews/prompts,
    /// and view-reported commands converge before any view effect is
    /// interpreted.
    pub fn execute_binding(&mut self, binding: Binding, viewport_height: u16) -> Result<bool> {
        match binding.command() {
            Command::Quit => {
                self.should_quit = true;
                Ok(false)
            }
            Command::Help => {
                self.mode = InteractionMode::Help;
                Ok(true)
            }
            Command::SearchPrompt => {
                self.mode = InteractionMode::SearchPrompt(String::new());
                Ok(true)
            }
            Command::PromptLogRevset => {
                self.open_log_revset_prompt();
                Ok(true)
            }
            Command::OpenStatus => {
                self.open_status()?;
                Ok(true)
            }
            Command::OpenResolve => {
                self.open_resolve()?;
                Ok(true)
            }
            Command::OpenBookmarks => {
                self.open_bookmarks()?;
                Ok(true)
            }
            Command::OpenWorkspaces => {
                self.open_workspaces()?;
                Ok(true)
            }
            Command::OpenOperationLog => {
                self.open_operation_log()?;
                Ok(true)
            }
            Command::OperationUndo | Command::OperationRedo => {
                if let Some(kind) = binding.command().operation_recovery() {
                    self.open_operation_recovery_preview(kind);
                }
                Ok(false)
            }
            Command::Edit => {
                self.open_log_working_copy_navigation_preview(JjWorkingCopyNavigationKind::Edit);
                Ok(false)
            }
            Command::NextEdit => {
                self.open_log_working_copy_navigation_preview(JjWorkingCopyNavigationKind::Next);
                Ok(false)
            }
            Command::PrevEdit => {
                self.open_log_working_copy_navigation_preview(JjWorkingCopyNavigationKind::Prev);
                Ok(false)
            }
            Command::Describe => {
                self.open_describe_prompt();
                Ok(false)
            }
            Command::Commit => {
                self.open_commit_prompt();
                Ok(false)
            }
            Command::BookmarkCreate => {
                self.open_bookmark_name_prompt(JjBookmarkMutationKind::Create);
                Ok(false)
            }
            Command::BookmarkSet => {
                self.open_bookmark_name_prompt(JjBookmarkMutationKind::Set);
                Ok(false)
            }
            Command::BookmarkMove => {
                self.open_bookmark_name_prompt(JjBookmarkMutationKind::Move);
                Ok(false)
            }
            Command::BookmarkRename => {
                self.open_bookmark_rename_prompt();
                Ok(false)
            }
            Command::BookmarkDelete => {
                self.open_bookmark_delete_preview();
                Ok(false)
            }
            Command::BookmarkForget => {
                self.open_bookmark_forget_preview();
                Ok(false)
            }
            Command::BookmarkTrack => {
                self.open_bookmark_tracking_preview(JjBookmarkMutationKind::Track);
                Ok(false)
            }
            Command::BookmarkUntrack => {
                self.open_bookmark_tracking_preview(JjBookmarkMutationKind::Untrack);
                Ok(false)
            }
            Command::Fetch => {
                self.fetch(viewport_height);
                Ok(false)
            }
            Command::FetchRemote => {
                self.open_fetch_remote_prompt();
                Ok(false)
            }
            Command::Push => self.open_push_prompt(),
            Command::Copy => {
                self.open_copy_menu(viewport_height);
                Ok(true)
            }
            Command::ViewFormat => {
                self.open_view_menu();
                Ok(true)
            }
            Command::Refresh => {
                self.refresh(viewport_height);
                Ok(false)
            }
            Command::Back => {
                self.pop_view();
                Ok(true)
            }
            Command::SwitchLog => {
                self.switch_to_log()?;
                Ok(true)
            }
            Command::SwitchDefault => {
                self.switch_to_default()?;
                Ok(true)
            }
            Command::View(ViewCommand::OpenActionMenu) => self.open_action_menu(viewport_height),
            Command::View(command) => {
                let effect = self.execute_view(command, viewport_height);
                self.apply_view_effect(effect, viewport_height)
            }
        }
    }
}

/// Format a pressed key sequence the same way prefix status text presents it.
fn binding_key_label(keys: &[crossterm::event::KeyEvent]) -> String {
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
pub fn prefix_status_message(
    prefix: &str,
    keys: &[crossterm::event::KeyEvent],
    next: &[String],
) -> String {
    let keys = binding_key_label(keys);
    if next.is_empty() {
        format!("{prefix}: {keys}")
    } else {
        format!("{prefix}: {keys} -> {}", next.join("/"))
    }
}
