use crate::actions::{JjOperationRecovery, JjOperationRecoveryKind, JjOperationTarget};
use crate::app::App;
use crate::app::actions::ActionPane;
use crate::modes::InteractionMode;

impl App {
    /// Open the global undo/redo preview that does not depend on a selected operation id.
    pub fn open_operation_recovery_preview(&mut self, kind: JjOperationRecoveryKind) {
        let recovery = JjOperationRecovery::new(kind);
        let status_context = Some(format!(
            "global current-repo {} from {}",
            recovery.status_action(),
            self.view.spec().app_label()
        ));
        self.mode = InteractionMode::OperationRecoveryPreview {
            output: ActionPane::pending(
                recovery.command_label().to_owned(),
                recovery.preview_text().to_owned(),
                status_context,
            ),
            recovery,
        };
    }

    /// Open the restore/revert preview for one exact operation id.
    pub fn open_operation_target_preview(&mut self, target: JjOperationTarget) {
        let status_context = Some(format!(
            "operation {} exact id {} from {}",
            target.status_action(),
            target.operation_id(),
            self.view.spec().app_label()
        ));

        let command_label = target.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            target.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::OperationTargetPreview { target, output };
    }
}
