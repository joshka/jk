//! Preview opening and immediate action execution for app-owned action flows.
//!
//! This subtree keeps preview-opening policy grouped by action family. The root owns only the
//! common pane/error helper shared across preview-open paths.

use color_eyre::Result;

use crate::app::App;
use crate::app::actions::ActionPane;

mod mutation;
mod operation;
mod rewrite;
mod sync;
mod working_copy;

impl App {
    // Keep mode construction at each caller; this helper owns only pane state and error status.
    fn preview_output_with_error_status<T>(
        &mut self,
        command_label: String,
        preview_result: Result<T>,
        preview_text: impl FnOnce(T) -> String,
        status_context: Option<String>,
    ) -> ActionPane {
        match preview_result {
            Ok(output) => ActionPane::pending(command_label, preview_text(output), status_context),
            Err(error) => {
                let message = error.to_string();
                self.status =
                    crate::app::status_line::StatusLine::error(&self.view, message.clone());
                ActionPane::finished(command_label, message, status_context)
            }
        }
    }
}
