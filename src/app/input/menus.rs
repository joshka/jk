use color_eyre::Result;
use crossterm::event::KeyCode;

use crate::app::status_line::StatusLine;
use crate::clipboard;
use crate::modes::{InteractionMode, view_menu_options};

use super::super::reducers::{
    MenuKey, RolePromptDecision, reduce_menu_key, reduce_role_prompt_accept, reduce_view_menu_key,
};
use super::App;

impl App {
    pub(super) fn handle_copy_menu_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::CopyMenu { options, selected } = &mut self.mode else {
            unreachable!("copy menu key handler requires copy menu mode");
        };

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

    pub(super) fn handle_view_menu_key(
        &mut self,
        code: KeyCode,
        viewport_height: u16,
    ) -> Result<bool> {
        let InteractionMode::ViewMenu { selected } = &mut self.mode else {
            unreachable!("view menu key handler requires view menu mode");
        };

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

    pub(super) fn handle_action_menu_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::ActionMenu { menu, selected } = &mut self.mode else {
            unreachable!("action menu key handler requires action menu mode");
        };

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

    pub(super) fn handle_role_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::RolePrompt {
            action,
            prompt,
            selected,
        } = &mut self.mode
        else {
            unreachable!("role prompt key handler requires role prompt mode");
        };

        match reduce_menu_key(selected, prompt.options().len(), code) {
            MenuKey::Cancel => self.mode = InteractionMode::Normal,
            MenuKey::Accept => {
                let decision = reduce_role_prompt_accept(*action, prompt);
                self.mode = InteractionMode::Normal;

                match decision {
                    RolePromptDecision::Rebase(rebase) => self.open_rebase_preview(rebase),
                    RolePromptDecision::Squash(squash) => self.open_squash_preview(squash),
                    RolePromptDecision::StatusMessage(message) => {
                        self.status = StatusLine::with_message(&self.view, message);
                    }
                    RolePromptDecision::StatusError(message) => {
                        self.status = StatusLine::error(&self.view, message);
                    }
                }
            }
            MenuKey::Shortcut(_) | MenuKey::Other => {}
        }
        Ok(true)
    }

    pub(super) fn handle_push_remote_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::PushRemotePrompt {
            target,
            remotes,
            selected,
        } = &mut self.mode
        else {
            unreachable!("push remote prompt key handler requires push remote prompt mode");
        };

        match reduce_menu_key(selected, remotes.len(), code) {
            MenuKey::Cancel => self.mode = InteractionMode::Normal,
            MenuKey::Accept => {
                let target = target.clone();
                let selected_remote = remotes.get(*selected).cloned();
                self.mode = InteractionMode::Normal;
                match selected_remote {
                    Some(remote) => self.open_push_preview(target, remote),
                    None => {
                        self.status =
                            StatusLine::error(&self.view, "no remote selected for push".to_owned());
                    }
                }
            }
            MenuKey::Shortcut(_) | MenuKey::Other => {}
        }
        Ok(true)
    }

    pub(super) fn handle_fetch_remote_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::FetchRemotePrompt { remotes, selected } = &mut self.mode else {
            unreachable!("fetch remote prompt key handler requires fetch remote prompt mode");
        };

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
}
