//! Normal-mode key handling.

use crate::error::JkError;
use crossterm::event::KeyEvent;

use super::super::preview::toggle_patch_flag;
use super::super::selection::matches_any;
use super::{App, Mode};

impl App {
    /// Handle normal-mode keybindings in priority order.
    ///
    /// Branch order defines precedence when bindings overlap. Most branches short-circuit after
    /// mutating selection or dispatching a command, so exactly one action is applied per key event.
    /// Dangerous commands are still funneled through confirmation gating via downstream execution.
    pub(super) fn handle_normal_key(&mut self, key: KeyEvent) -> Result<(), JkError> {
        if matches_any(&self.keybinds.normal.quit, key) {
            self.should_quit = true;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.refresh, key) {
            let tokens = if self.last_command.is_empty() {
                vec!["log".to_string()]
            } else {
                self.last_command.clone()
            };
            self.execute_tokens(tokens)?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.command_mode, key) {
            self.mode = Mode::Command;
            self.command_input.clear();
            self.command_history_index = None;
            self.command_history_draft.clear();
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.help, key) {
            self.execute_command_line("commands")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.keymap, key) {
            self.execute_command_line("keys")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.aliases, key) {
            self.execute_command_line("aliases")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.repeat_last, key) {
            let tokens = if self.last_command.is_empty() {
                vec!["log".to_string()]
            } else {
                self.last_command.clone()
            };
            self.execute_tokens(tokens)?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.up, key) {
            self.move_cursor_up();
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.down, key) {
            self.move_cursor_down();
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.top, key) {
            self.cursor = 0;
            self.scroll = 0;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.bottom, key) {
            if !self.lines.is_empty() {
                self.cursor = self.lines.len() - 1;
                self.ensure_cursor_visible(20);
            }
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.show, key) {
            if let Some(revision) = self.selected_revision() {
                self.execute_with_confirmation(vec!["show".to_string(), revision])?;
            } else {
                self.status_line = "No revision selected on this line".to_string();
            }
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.diff, key) {
            if let Some(revision) = self.selected_revision() {
                self.execute_with_confirmation(vec![
                    "diff".to_string(),
                    "-r".to_string(),
                    revision,
                ])?;
            } else {
                self.status_line = "No revision selected on this line".to_string();
            }
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.status, key) {
            self.execute_command_line("status")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.log, key) {
            self.execute_command_line("log")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.operation_log, key) {
            self.execute_command_line("operation log")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.bookmark_list, key) {
            self.execute_command_line("bookmark list")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.resolve_list, key) {
            self.execute_command_line("resolve -l")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.file_list, key) {
            self.execute_command_line("file list")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.tag_list, key) {
            self.execute_command_line("tag list")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.root, key) {
            self.execute_command_line("root")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.toggle_patch, key) {
            if !matches!(
                self.last_log_tokens.first().map(String::as_str),
                Some("log")
            ) {
                self.status_line = "Patch toggle is available after running log".to_string();
                return Ok(());
            }

            let tokens = toggle_patch_flag(&self.last_log_tokens);
            self.execute_tokens(tokens)?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.fetch, key) {
            self.execute_command_line("gf")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.push, key) {
            self.execute_command_line("gp")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.rebase_main, key) {
            self.execute_command_line("rbm")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.rebase_trunk, key) {
            self.execute_command_line("rbt")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.new, key) {
            self.execute_command_line("new")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.next, key) {
            self.execute_command_line("next")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.prev, key) {
            self.execute_command_line("prev")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.edit, key) {
            self.execute_command_line("edit")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.commit, key) {
            self.execute_command_line("commit")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.describe, key) {
            self.execute_command_line("describe")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.bookmark_set, key) {
            self.execute_command_line("bookmark set")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.abandon, key) {
            self.execute_command_line("abandon")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.rebase, key) {
            self.execute_command_line("rebase")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.squash, key) {
            self.execute_command_line("squash")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.split, key) {
            self.execute_command_line("split")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.restore, key) {
            self.execute_command_line("restore")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.revert, key) {
            self.execute_command_line("revert")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.undo, key) {
            self.execute_command_line("undo")?;
            return Ok(());
        }

        if matches_any(&self.keybinds.normal.redo, key) {
            self.execute_command_line("redo")?;
            return Ok(());
        }

        Ok(())
    }
}
