//! Key reducers for app-owned modal screens.
//!
//! `app.rs` keeps the event loop and normal command dispatch. This module owns how active
//! `InteractionMode` values consume key presses and which app-level side effect follows when a
//! modal selection or prompt is accepted.

use std::time::Instant;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::DefaultTerminal;

use crate::action_menu::ActionKind;
use crate::action_output::{
    ActionOutputKey, action_output_visible_lines, handle_action_output_key,
};
use crate::app_screen::{InteractionMode, view_menu_options};
use crate::app_status::StatusLine;
use crate::clipboard;
use crate::command::{
    Binding, BindingMatch, ViewCommand, ViewEffect, help_binding_prefix_next_labels,
    match_help_binding_sequence,
};
use crate::jj_actions::{
    JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, validate_bookmark_rename_new_name,
};
use crate::search::SearchQuery;

use super::{APP_BINDINGS, App, COMMAND_PREFIX_TIMEOUT, PendingCommand, prefix_status_message};

mod reducers;

use reducers::{
    ConfirmationKey, MenuKey, TextPromptKey, bookmark_mutation_plan, is_help_close_key,
    is_help_scroll_key, reduce_confirmation_key, reduce_menu_key, reduce_text_prompt_key,
    reduce_view_menu_key,
};
pub(in crate::app) use reducers::{rebase_plan_from_prompt, squash_plan_from_prompt};

impl App {
    pub(super) fn open_copy_menu(&mut self, viewport_height: u16) {
        let options = match self.execute_view(ViewCommand::Copy, viewport_height) {
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

    fn handle_active_mode_key(&mut self, code: KeyCode, viewport_height: u16) -> Result<bool> {
        match &mut self.mode {
            InteractionMode::Normal => Ok(false),
            InteractionMode::Help => unreachable!("help mode is handled before borrowing mode"),
            InteractionMode::SearchPrompt(input) => {
                match reduce_text_prompt_key(input, code) {
                    TextPromptKey::Cancel => self.mode = InteractionMode::Normal,
                    TextPromptKey::Accept => {
                        self.search = SearchQuery::new(input.clone());
                        self.mode = InteractionMode::Normal;
                        self.status = if self.search.is_some() {
                            match self.execute_view(ViewCommand::StartSearch, viewport_height) {
                                ViewEffect::SearchStarted { matches } => StatusLine::with_message(
                                    &self.view,
                                    format!("{matches} matches"),
                                ),
                                _ => StatusLine::ready(&self.view),
                            }
                        } else {
                            StatusLine::ready(&self.view)
                        };
                    }
                    TextPromptKey::Edited | TextPromptKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::LogRevsetPrompt(input) => {
                match reduce_text_prompt_key(input, code) {
                    TextPromptKey::Cancel => self.mode = InteractionMode::Normal,
                    TextPromptKey::Accept => {
                        let revset = std::mem::take(input);
                        self.mode = InteractionMode::Normal;
                        self.apply_custom_log_revset(revset);
                    }
                    TextPromptKey::Edited | TextPromptKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::CopyMenu { options, selected } => {
                match reduce_menu_key(selected, options.len(), code) {
                    MenuKey::Cancel => self.mode = InteractionMode::Normal,
                    MenuKey::Accept => {
                        if let Some(option) = options.get(*selected) {
                            match clipboard::copy(option.value()) {
                                Ok(()) => {
                                    self.status = StatusLine::with_message(
                                        &self.view,
                                        format!("copied {}", option.label()),
                                    );
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                }
                            }
                        }
                        self.mode = InteractionMode::Normal;
                    }
                    MenuKey::Shortcut(_) | MenuKey::Other => {}
                }
                Ok(true)
            }
            InteractionMode::ViewMenu { selected } => {
                match reduce_view_menu_key(selected, code) {
                    MenuKey::Cancel => self.mode = InteractionMode::Normal,
                    MenuKey::Accept => {
                        let action = view_menu_options()[*selected].action();
                        self.mode = InteractionMode::Normal;
                        self.apply_view_menu_action(action, viewport_height)?;
                    }
                    MenuKey::Shortcut(_) | MenuKey::Other => {}
                }
                Ok(true)
            }
            InteractionMode::ActionMenu { menu, selected } => {
                match reduce_menu_key(selected, menu.items().len(), code) {
                    MenuKey::Cancel => self.mode = InteractionMode::Normal,
                    MenuKey::Accept => {
                        if let Some(action) = menu.items().get(*selected).cloned() {
                            self.apply_action_menu_item(action);
                        } else {
                            self.mode = InteractionMode::Normal;
                        }
                    }
                    MenuKey::Shortcut(shortcut) => {
                        if let Some(action) = menu.item_for_shortcut(shortcut).cloned() {
                            self.apply_action_menu_item(action);
                        }
                    }
                    MenuKey::Other => {}
                }
                Ok(true)
            }
            InteractionMode::RolePrompt {
                action,
                prompt,
                selected,
            } => {
                match reduce_menu_key(selected, prompt.options().len(), code) {
                    MenuKey::Cancel => self.mode = InteractionMode::Normal,
                    MenuKey::Accept => {
                        let next_status = prompt.status_message();
                        let action = *action;
                        let rebase_plan = match action {
                            ActionKind::Rebase => rebase_plan_from_prompt(prompt),
                            _ => None,
                        };
                        let squash_plan = match action {
                            ActionKind::Squash => squash_plan_from_prompt(prompt),
                            _ => None,
                        };

                        self.mode = InteractionMode::Normal;

                        match action {
                            ActionKind::Rebase => match rebase_plan {
                                Some(rebase) => self.open_rebase_preview(rebase),
                                None => {
                                    self.status =
                                        StatusLine::error(&self.view, next_status.to_owned());
                                }
                            },
                            ActionKind::Squash => match squash_plan {
                                Some(squash) => self.open_squash_preview(squash),
                                None => {
                                    self.status =
                                        StatusLine::error(&self.view, next_status.to_owned());
                                }
                            },
                            ActionKind::Edit
                            | ActionKind::New
                            | ActionKind::Split
                            | ActionKind::Duplicate
                            | ActionKind::Restore
                            | ActionKind::Revert
                            | ActionKind::Abandon
                            | ActionKind::Absorb
                            | ActionKind::FileTrack
                            | ActionKind::FileUntrack
                            | ActionKind::FileChmodExecutable
                            | ActionKind::FileChmodNormal => {
                                self.status =
                                    StatusLine::with_message(&self.view, next_status.to_owned());
                            }
                        }
                    }
                    MenuKey::Shortcut(_) | MenuKey::Other => {}
                }
                Ok(true)
            }
            InteractionMode::DescribePrompt { target, input } => {
                match reduce_text_prompt_key(input, code) {
                    TextPromptKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        self.status =
                            StatusLine::with_message(&self.view, "describe cancelled".to_owned());
                    }
                    TextPromptKey::Accept => {
                        let message = input.trim().to_owned();
                        let target = target.clone();
                        self.mode = InteractionMode::Normal;
                        if message.is_empty() {
                            self.status = StatusLine::with_message(
                                &self.view,
                                "describe cancelled: empty description".to_owned(),
                            );
                        } else {
                            self.open_describe_preview(JjDescribePlan::new(target, message));
                        }
                    }
                    TextPromptKey::Edited | TextPromptKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::CommitPrompt(input) => {
                match reduce_text_prompt_key(input, code) {
                    TextPromptKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        self.status =
                            StatusLine::with_message(&self.view, "commit cancelled".to_owned());
                    }
                    TextPromptKey::Accept => {
                        let message = input.trim().to_owned();
                        self.mode = InteractionMode::Normal;
                        if message.is_empty() {
                            self.status = StatusLine::with_message(
                                &self.view,
                                "commit cancelled: empty description".to_owned(),
                            );
                        } else {
                            self.open_commit_preview(JjCommitPlan::new(message));
                        }
                    }
                    TextPromptKey::Edited | TextPromptKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::BookmarkNamePrompt {
                kind,
                target,
                input,
            } => {
                match reduce_text_prompt_key(input, code) {
                    TextPromptKey::Cancel => {
                        let kind = *kind;
                        self.mode = InteractionMode::Normal;
                        self.status = StatusLine::with_message(
                            &self.view,
                            format!("bookmark {} cancelled", kind.label()),
                        );
                    }
                    TextPromptKey::Accept => {
                        let name = input.trim().to_owned();
                        let kind = *kind;
                        let target = target.clone();
                        self.mode = InteractionMode::Normal;
                        if name.is_empty() {
                            self.status = StatusLine::with_message(
                                &self.view,
                                format!("bookmark {} cancelled: empty bookmark name", kind.label()),
                            );
                        } else {
                            self.open_bookmark_mutation_preview(bookmark_mutation_plan(
                                kind, name, target,
                            ));
                        }
                    }
                    TextPromptKey::Edited | TextPromptKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::BookmarkRenamePrompt { old_name, input } => {
                match reduce_text_prompt_key(input, code) {
                    TextPromptKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        self.status =
                            StatusLine::with_message(&self.view, "bookmark rename cancelled");
                    }
                    TextPromptKey::Accept => {
                        let old_name = old_name.clone();
                        let new_name = std::mem::take(input);
                        self.mode = InteractionMode::Normal;
                        match validate_bookmark_rename_new_name(&old_name, &new_name) {
                            Ok(()) => {
                                self.open_bookmark_mutation_preview(
                                    JjBookmarkMutationPlan::rename(old_name, new_name),
                                );
                            }
                            Err(reason) => {
                                self.status = StatusLine::with_message(
                                    &self.view,
                                    format!("bookmark rename cancelled: {reason}"),
                                );
                            }
                        }
                    }
                    TextPromptKey::Edited | TextPromptKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::AbandonPreview {
                abandon,
                preview,
                output,
            } => {
                let (abandon, preview, status_context, completed) = {
                    (
                        abandon.clone(),
                        preview.clone(),
                        output.status_context().cloned(),
                        output.completed(),
                    )
                };
                let visible_lines = action_output_visible_lines(viewport_height);
                match handle_action_output_key(code, output, visible_lines) {
                    ActionOutputKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        if !completed {
                            self.status = StatusLine::with_message(
                                &self.view,
                                "abandon cancelled".to_owned(),
                            );
                        }
                    }
                    ActionOutputKey::Primary => {
                        if completed {
                            self.mode = InteractionMode::Normal;
                            return Ok(true);
                        }

                        if preview.is_empty_change() {
                            self.confirm_empty_abandon_after_recheck(
                                abandon,
                                status_context,
                                viewport_height,
                            );
                        } else {
                            self.mode = InteractionMode::AbandonConfirm {
                                abandon,
                                input: String::new(),
                                output: output.clone(),
                            };
                        }
                    }
                    ActionOutputKey::Handled | ActionOutputKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::AbandonConfirm {
                abandon,
                input,
                output,
            } => {
                let (abandon_plan, status_context) =
                    (abandon.clone(), output.status_context().cloned());
                let visible_lines = action_output_visible_lines(viewport_height);
                match reduce_confirmation_key(input, output, visible_lines, code) {
                    ConfirmationKey::Cancel => {
                        self.mode = InteractionMode::Normal;
                        self.status =
                            StatusLine::with_message(&self.view, "abandon cancelled".to_owned());
                    }
                    ConfirmationKey::Accept => {
                        if input == abandon.revision() {
                            self.confirm_abandon(abandon_plan, status_context, viewport_height);
                        } else {
                            self.status = StatusLine::error(
                                &self.view,
                                "confirmation did not match; abandon not run".to_owned(),
                            );
                        }
                    }
                    ConfirmationKey::Handled | ConfirmationKey::Ignored => {}
                }
                Ok(true)
            }
            InteractionMode::PushRemotePrompt {
                target,
                remotes,
                selected,
            } => {
                match reduce_menu_key(selected, remotes.len(), code) {
                    MenuKey::Cancel => self.mode = InteractionMode::Normal,
                    MenuKey::Accept => {
                        let target = target.clone();
                        let selected_remote = remotes.get(*selected).cloned();
                        self.mode = InteractionMode::Normal;
                        match selected_remote {
                            Some(remote) => self.open_push_preview(target, remote),
                            None => {
                                self.status = StatusLine::error(
                                    &self.view,
                                    "no remote selected for push".to_owned(),
                                );
                            }
                        }
                    }
                    MenuKey::Shortcut(_) | MenuKey::Other => {}
                }
                Ok(true)
            }
            InteractionMode::FetchRemotePrompt { remotes, selected } => {
                match reduce_menu_key(selected, remotes.len(), code) {
                    MenuKey::Cancel => self.mode = InteractionMode::Normal,
                    MenuKey::Accept => {
                        let selected_remote = remotes.get(*selected).cloned();
                        self.mode = InteractionMode::Normal;
                        match selected_remote {
                            Some(remote) => self.open_fetch_preview(remote),
                            None => {
                                self.status = StatusLine::error(
                                    &self.view,
                                    "no remote selected for fetch".to_owned(),
                                );
                            }
                        }
                    }
                    MenuKey::Shortcut(_) | MenuKey::Other => {}
                }
                Ok(true)
            }
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

    fn handle_help_key(&mut self, key: KeyEvent, viewport_height: u16) -> Result<bool> {
        if is_help_close_key(key) {
            self.pending_command = None;
            self.mode = InteractionMode::Normal;
            return Ok(true);
        }
        if is_help_scroll_key(key) {
            return Ok(true);
        }

        if self.pending_command.is_some() {
            return self.handle_pending_help_key(key, viewport_height, Instant::now());
        }

        let keys = [key];
        let context = self.view.help_context();
        let Some(binding_match) =
            match_help_binding_sequence(&[APP_BINDINGS, self.view.bindings()], &keys, context)
        else {
            self.status = StatusLine::with_message(&self.view, "not available from help menu");
            return Ok(true);
        };

        match binding_match {
            BindingMatch::Exact(binding) => {
                self.execute_help_binding(binding, viewport_height)?;
                Ok(true)
            }
            BindingMatch::Prefix { fallback } => {
                self.pending_command = Some(PendingCommand {
                    keys: keys.to_vec(),
                    fallback,
                    deadline: Instant::now() + COMMAND_PREFIX_TIMEOUT,
                });
                self.status = StatusLine::with_message(
                    &self.view,
                    prefix_status_message(
                        "help",
                        &keys,
                        &help_binding_prefix_next_labels(
                            &[APP_BINDINGS, self.view.bindings()],
                            &keys,
                            context,
                        ),
                    ),
                );
                Ok(true)
            }
        }
    }

    fn handle_pending_help_key(
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
            self.run_pending_fallback(viewport_height)?;
            return self.handle_key_after_prefix_fallback(key, viewport_height, now);
        }

        let Some(mut pending) = self.pending_command.take() else {
            return Ok(true);
        };
        pending.keys.push(key);

        match match_help_binding_sequence(
            &[APP_BINDINGS, self.view.bindings()],
            &pending.keys,
            self.view.help_context(),
        ) {
            Some(BindingMatch::Exact(binding)) => {
                self.execute_help_binding(binding, viewport_height)?;
            }
            Some(BindingMatch::Prefix { fallback }) => {
                self.status = StatusLine::with_message(
                    &self.view,
                    prefix_status_message(
                        "help",
                        &pending.keys,
                        &help_binding_prefix_next_labels(
                            &[APP_BINDINGS, self.view.bindings()],
                            &pending.keys,
                            self.view.help_context(),
                        ),
                    ),
                );
                self.pending_command = Some(PendingCommand {
                    keys: pending.keys,
                    fallback,
                    deadline: now + COMMAND_PREFIX_TIMEOUT,
                });
            }
            None => {
                if let Some(fallback) = pending.fallback {
                    self.execute_help_binding(fallback, viewport_height)?;
                    return self.handle_key_after_prefix_fallback(key, viewport_height, now);
                }

                self.status = StatusLine::with_message(&self.view, "unknown help command prefix");
            }
        }

        Ok(true)
    }

    pub(super) fn execute_help_binding(
        &mut self,
        binding: Binding,
        viewport_height: u16,
    ) -> Result<()> {
        self.pending_command = None;
        self.mode = InteractionMode::Normal;
        self.run_binding_with_status_refresh(binding, viewport_height)
    }
}
