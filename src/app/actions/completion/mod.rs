//! Confirmed action execution and result-pane handling for app-owned mutation screens.
//!
//! This subtree keeps completion policy grouped by action family. The root owns only the stacked
//! repo refresh helper shared by operation-target completion.

use color_eyre::Result;

use crate::app::{App, clamp_view_to_current_viewport};
use crate::jj;

mod mutation;
mod operation;
mod rewrite;
mod sync;
mod working_copy;

impl App {
    /// Refresh any repo-backed views stored on the app stack after an operation-level mutation.
    pub fn refresh_stacked_repo_views(&mut self) -> Result<()> {
        for view in &mut self.stack {
            match view.command() {
                jj::Command::Default
                | jj::Command::Log
                | jj::Command::Status
                | jj::Command::Bookmarks
                | jj::Command::Workspaces
                | jj::Command::OperationLog => {
                    self.services.refresh_view(view)?;
                    clamp_view_to_current_viewport(view);
                }
                jj::Command::Show
                | jj::Command::Diff
                | jj::Command::Resolve
                | jj::Command::FileList
                | jj::Command::FileShow
                | jj::Command::OperationShow
                | jj::Command::OperationDiff => {}
            }
        }
        Ok(())
    }
}
