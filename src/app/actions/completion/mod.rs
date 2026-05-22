//! Confirmed action execution and result-pane handling for app-owned mutation screens.
//!
//! This subtree keeps completion policy grouped by action family. The root owns only the stacked
//! repo refresh helper shared by operation-target completion.

use color_eyre::Result;

use crate::jj::JjCommand;

use super::super::{App, current_viewport_width};

mod mutation;
mod operation;
mod rewrite;
mod sync;
mod working_copy;

impl App {
    /// Refresh any repo-backed views stored on the app stack after an operation-level mutation.
    pub fn refresh_stacked_repo_views(&mut self, viewport_height: u16) -> Result<()> {
        for view in &mut self.stack {
            match view.command() {
                JjCommand::Default
                | JjCommand::Log
                | JjCommand::Status
                | JjCommand::Bookmarks
                | JjCommand::Workspaces
                | JjCommand::OperationLog => {
                    self.services.refresh_view(view)?;
                    view.clamp(viewport_height, current_viewport_width());
                }
                JjCommand::Show
                | JjCommand::Diff
                | JjCommand::Resolve
                | JjCommand::FileList
                | JjCommand::FileShow
                | JjCommand::OperationShow
                | JjCommand::OperationDiff => {}
            }
        }
        Ok(())
    }
}
