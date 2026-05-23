//! App-owned binding execution after keyboard dispatch has chosen a binding.
//!
//! Keyboard routing and prefix resolution live in `app/keyboard.rs`. This root owns the app-level
//! execution boundary once a concrete binding has already been selected.

use color_eyre::Result;

use super::App;
use crate::actions::{JjBookmarkMutationKind, JjWorkingCopyNavigationKind};
use crate::command::{Binding, Command, ViewCommand};
use crate::modes::InteractionMode;

impl App {
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
                let _ = viewport_height;
                self.open_copy_menu();
                Ok(true)
            }
            Command::ViewFormat => {
                self.open_view_menu();
                Ok(true)
            }
            Command::Refresh => {
                let _ = viewport_height;
                self.refresh();
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
                let effect = self.execute_view(command);
                self.apply_view_effect(effect, viewport_height)
            }
        }
    }
}
