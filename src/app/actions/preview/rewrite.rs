use crate::actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjRebasePlan, JjRestorePlan, JjRevertPlan,
    JjSquashPlan,
};
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;

use super::super::super::App;
use super::super::ActionPane;
use super::super::shared::short_id;

impl App {
    /// Open the rebase preview for one prepared multi-source rewrite plan.
    pub fn open_rebase_preview(&mut self, rebase: JjRebasePlan) {
        let status_context = Some(format!(
            "rebase from {} source(s) into {} from {}",
            rebase.sources().len(),
            rebase.destination(),
            self.view.spec().app_label()
        ));
        let status_context = with_rewrite_source_context(status_context, rebase.sources());

        let command_label = rebase.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            rebase.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::RebasePreview { rebase, output };
    }

    /// Open the restore preview for one revision or path restore plan.
    pub fn open_restore_preview(&mut self, restore: JjRestorePlan) {
        let target = restore
            .path()
            .map(|path| format!("path {path} from {}", restore.revision()))
            .unwrap_or_else(|| format!("revision {}", restore.revision()));
        let status_context = Some(format!(
            "restore {target} from {}",
            self.view.spec().app_label()
        ));

        let command_label = restore.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            self.services.load_restore_preview(&restore),
            std::convert::identity,
            status_context,
        );
        self.mode = InteractionMode::RestorePreview { restore, output };
    }

    /// Open the revert preview for one exact revision revert plan.
    pub fn open_revert_preview(&mut self, revert: JjRevertPlan) {
        let status_context = Some(format!(
            "revert revision {} into @ from {}",
            revert.revision(),
            self.view.spec().app_label()
        ));

        let command_label = revert.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            self.services.load_revert_preview(&revert),
            std::convert::identity,
            status_context,
        );
        self.mode = InteractionMode::RevertPreview { revert, output };
    }

    /// Open the squash preview for one prepared multi-source squash plan.
    pub fn open_squash_preview(&mut self, squash: JjSquashPlan) {
        let status_context = Some(format!(
            "squash from {} source(s) into {} from {}",
            squash.sources().len(),
            squash.destination(),
            self.view.spec().app_label()
        ));
        let status_context = with_rewrite_source_context(status_context, squash.sources());

        let command_label = squash.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            squash.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::SquashPreview { squash, output };
    }

    /// Open the absorb preview for one prepared absorb plan, or reject empty destinations.
    pub fn open_absorb_preview(&mut self, absorb: JjAbsorbPlan) {
        if absorb.destinations().is_empty() {
            self.status = StatusLine::error(
                &self.view,
                "absorb requires at least one selected exact candidate destination".to_owned(),
            );
            return;
        }

        let destination_labels = absorb
            .destinations()
            .iter()
            .map(|destination| short_id(destination))
            .collect::<Vec<_>>()
            .join(", ");
        let status_context = Some(format!(
            "absorb source {} into {} selected candidate destination(s) from {} | candidate(s): {}",
            absorb.source(),
            absorb.destinations().len(),
            self.view.spec().app_label(),
            destination_labels
        ));

        let command_label = absorb.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            absorb.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::AbsorbPreview { absorb, output };
    }

    /// Open the abandon preview and preserve any preview-load failure on the same pane surface.
    pub fn open_abandon_preview(&mut self, abandon: JjAbandonPlan) {
        let status_context = Some(format!(
            "abandon exact revision {} from {}",
            abandon.revision(),
            self.view.spec().app_label()
        ));

        match self.services.load_abandon_preview(&abandon) {
            Ok(preview) => {
                let command_label = abandon.command_label();
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    output: ActionPane::pending(
                        command_label,
                        preview.preview_text(),
                        status_context,
                    ),
                    preview,
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = abandon.diff_summary_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    preview: JjAbandonPreview::new(String::new(), None, String::new()),
                    output: ActionPane::finished(command_label, message, status_context),
                };
            }
        }
    }
}

/// Append abbreviated rewrite sources to a status context when any exist.
fn with_rewrite_source_context(
    status_context: Option<String>,
    sources: &[String],
) -> Option<String> {
    let source_labels = sources
        .iter()
        .map(|source| short_id(source))
        .collect::<Vec<_>>()
        .join(", ");

    if source_labels.is_empty() {
        status_context
    } else {
        status_context
            .map(|status_context| format!("{status_context} | source(s): {source_labels}"))
    }
}
