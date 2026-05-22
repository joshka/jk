//! Operation recovery and exact operation target plans.
//!
//! This module keeps repository-wide undo/redo separate from exact selected
//! operation restore/revert plans. Recovery actions never take a selected
//! operation id, while exact targets always act on the selected operation-log
//! row's one operation id.

use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::run_direct_args;

// Operation recovery plans keep global undo/redo separate from selected
// operation restore/revert, which target one exact operation id.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjOperationRecoveryKind {
    /// Globally undo the most recent repository operation.
    Undo,
    /// Globally redo the most recently undone repository operation.
    Redo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjOperationTargetKind {
    /// Restore repository state to one selected exact operation by creating a new operation.
    Restore,
    /// Revert one selected exact operation by applying its inverse.
    Revert,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjOperationRecovery {
    /// Repository-wide undo/redo kind that does not take a selected operation id.
    kind: JjOperationRecoveryKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjOperationTarget {
    /// Exact selected-operation action to run.
    kind: JjOperationTargetKind,
    /// Exact operation id taken from the selected operation-log row.
    operation_id: String,
}

impl JjOperationRecovery {
    /// Builds one repository-wide undo/redo recovery plan.
    pub fn new(kind: JjOperationRecoveryKind) -> Self {
        Self { kind }
    }

    #[cfg(test)]
    pub fn kind(&self) -> JjOperationRecoveryKind {
        self.kind
    }

    /// Returns the user-facing `jj` command label for this recovery action.
    pub fn command_label(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "jj undo",
            JjOperationRecoveryKind::Redo => "jj redo",
        }
    }

    /// Returns argv for the repository-wide recovery command.
    pub fn command_argv(&self) -> Vec<String> {
        match self.kind {
            JjOperationRecoveryKind::Undo => vec!["undo".to_owned()],
            JjOperationRecoveryKind::Redo => vec!["redo".to_owned()],
        }
    }

    /// Returns preview text explaining what the recovery action does and does not target.
    pub fn preview_text(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => concat!(
                "effect: globally undo the last operation in the current repository\n",
                "selection: the selected operation-log row is not an argument\n",
                "redo path: jj redo\n",
                "confirmation: press Enter to run jj undo",
            ),
            JjOperationRecoveryKind::Redo => concat!(
                "effect: globally redo the most recently undone operation in the current ",
                "repository\n",
                "selection: the selected operation-log row is not an argument\n",
                "failure path: jj may report that no redo is available\n",
                "confirmation: press Enter to run jj redo",
            ),
        }
    }

    /// Returns the follow-up recovery hint shown after success.
    pub fn success_hint(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "jj redo",
            JjOperationRecoveryKind::Redo => "jj undo",
        }
    }

    /// Returns the status verb used in app messaging for this action.
    pub fn status_action(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "undo",
            JjOperationRecoveryKind::Redo => "redo",
        }
    }

    /// Runs the repository-wide undo or redo command through the direct `jj` boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            self.command_label(),
            self.status_action(),
        )
    }
}

impl JjOperationTargetKind {
    /// Returns the action verb used in menu labels and status messages.
    pub fn label(self) -> &'static str {
        match self {
            Self::Restore => "restore",
            Self::Revert => "revert",
        }
    }

    /// Returns fallback success wording when `jj` output does not provide one.
    fn success_fallback(self) -> &'static str {
        match self {
            Self::Restore => "restored operation",
            Self::Revert => "reverted operation",
        }
    }
}

impl JjOperationTarget {
    /// Builds an exact selected-operation restore plan.
    pub fn restore(operation_id: impl Into<String>) -> Self {
        Self {
            kind: JjOperationTargetKind::Restore,
            operation_id: operation_id.into(),
        }
    }

    /// Builds an exact selected-operation revert plan.
    pub fn revert(operation_id: impl Into<String>) -> Self {
        Self {
            kind: JjOperationTargetKind::Revert,
            operation_id: operation_id.into(),
        }
    }

    #[cfg(test)]
    pub fn kind(&self) -> JjOperationTargetKind {
        self.kind
    }

    /// Returns the exact selected operation id owned by this plan.
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Returns the status verb used in app messaging for this action.
    pub fn status_action(&self) -> &'static str {
        self.kind.label()
    }

    /// Returns the full `jj` command label for this exact operation action.
    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for the exact operation action.
    pub fn command_argv(&self) -> Vec<String> {
        let action = match self.kind {
            JjOperationTargetKind::Restore => "restore",
            JjOperationTargetKind::Revert => "revert",
        };
        vec![
            "operation".to_owned(),
            action.to_owned(),
            self.operation_id.clone(),
        ]
    }

    /// Returns preview output without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs the exact operation action through the direct `jj` boundary.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            self.kind.success_fallback(),
        )
    }

    /// Returns the preview summary shown before confirming the action.
    pub fn preview_summary(&self) -> String {
        let effect = match self.kind {
            JjOperationTargetKind::Restore => {
                "effect: restore the repository state to the selected operation by creating a new operation"
            }
            JjOperationTargetKind::Revert => {
                "effect: revert exactly the selected operation by applying its inverse"
            }
        };

        [
            format!("command: {}", self.command_label()),
            String::new(),
            format!("operation id: {}", self.operation_id),
            effect.to_owned(),
            "selection: the selected operation-log row supplies this exact operation id".to_owned(),
            format!("confirmation: press Enter to run {}", self.command_label()),
            "recovery: jj undo".to_owned(),
        ]
        .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_undo_command_has_no_operation_id_argument() {
        let recovery = JjOperationRecovery::new(JjOperationRecoveryKind::Undo);
        let selected_operation_id = operation_id('c');

        assert_eq!(recovery.command_label(), "jj undo");
        assert_eq!(recovery.command_argv(), ["undo"]);
        assert!(!recovery.command_argv().contains(&selected_operation_id));
        assert!(
            recovery
                .preview_text()
                .contains("selected operation-log row is not an argument")
        );
    }

    #[test]
    fn operation_redo_command_has_no_operation_id_argument() {
        let recovery = JjOperationRecovery::new(JjOperationRecoveryKind::Redo);
        let selected_operation_id = operation_id('d');

        assert_eq!(recovery.command_label(), "jj redo");
        assert_eq!(recovery.command_argv(), ["redo"]);
        assert!(!recovery.command_argv().contains(&selected_operation_id));
        assert!(
            recovery
                .preview_text()
                .contains("selected operation-log row is not an argument")
        );
    }

    #[test]
    fn operation_restore_command_targets_exact_operation_id() {
        let operation_id = operation_id('e');
        let target = JjOperationTarget::restore(operation_id.clone());

        assert_eq!(target.kind(), JjOperationTargetKind::Restore);
        assert_eq!(target.operation_id(), operation_id.as_str());
        assert_eq!(
            target.command_argv(),
            ["operation", "restore", operation_id.as_str()]
        );
        assert_eq!(
            target.command_label(),
            format!("jj operation restore {operation_id}")
        );
        assert!(
            target
                .preview_summary()
                .contains(&format!("operation id: {operation_id}"))
        );
        assert!(target.preview_summary().contains(
            "restore the repository state to the selected operation by creating a new operation"
        ));
        assert!(target.preview_summary().contains(&format!(
            "confirmation: press Enter to run jj operation restore {operation_id}"
        )));
    }

    #[test]
    fn operation_revert_command_targets_exact_operation_id() {
        let operation_id = operation_id('f');
        let target = JjOperationTarget::revert(operation_id.clone());

        assert_eq!(target.kind(), JjOperationTargetKind::Revert);
        assert_eq!(target.operation_id(), operation_id.as_str());
        assert_eq!(
            target.command_argv(),
            ["operation", "revert", operation_id.as_str()]
        );
        assert_eq!(
            target.command_label(),
            format!("jj operation revert {operation_id}")
        );
        assert!(
            target
                .preview_summary()
                .contains(&format!("operation id: {operation_id}"))
        );
        assert!(
            target
                .preview_summary()
                .contains("revert exactly the selected operation by applying its inverse")
        );
        assert!(target.preview_summary().contains(&format!(
            "confirmation: press Enter to run jj operation revert {operation_id}"
        )));
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }
}
