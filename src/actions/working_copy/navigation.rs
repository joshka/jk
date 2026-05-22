use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{exact_change_id_revset, run_direct_args};

/// Working-copy navigation command family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjWorkingCopyNavigationKind {
    /// Move `@` to edit one exact selected revision.
    Edit,
    /// Move `@` to the next editable change relative to the current working copy.
    Next,
    /// Move `@` to the previous editable change relative to the current working copy.
    Prev,
}

/// Working-copy navigation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjWorkingCopyNavigationPlan {
    /// Topology movement or edit action owned by this plan.
    kind: JjWorkingCopyNavigationKind,
    /// Exact target change id only for `jj edit`.
    target_change_id: Option<String>,
}

impl JjWorkingCopyNavigationPlan {
    /// Builds an exact-target `jj edit` plan.
    pub fn edit(change_id: impl Into<String>) -> Self {
        Self {
            kind: JjWorkingCopyNavigationKind::Edit,
            target_change_id: Some(change_id.into()),
        }
    }

    /// Builds a `jj next --edit` plan relative to the current working copy.
    pub fn next() -> Self {
        Self {
            kind: JjWorkingCopyNavigationKind::Next,
            target_change_id: None,
        }
    }

    /// Builds a `jj prev --edit` plan relative to the current working copy.
    pub fn prev() -> Self {
        Self {
            kind: JjWorkingCopyNavigationKind::Prev,
            target_change_id: None,
        }
    }

    /// Returns the working-copy navigation kind owned by this plan.
    pub fn kind(&self) -> JjWorkingCopyNavigationKind {
        self.kind
    }

    /// Returns the exact edit target change id, if this is an edit plan.
    pub fn target_change_id(&self) -> Option<&str> {
        self.target_change_id.as_deref()
    }

    /// Returns the overlay title shown by the app for this navigation action.
    pub fn overlay_title(&self) -> &'static str {
        match self.kind {
            JjWorkingCopyNavigationKind::Edit => "Edit",
            JjWorkingCopyNavigationKind::Next => "Next",
            JjWorkingCopyNavigationKind::Prev => "Prev",
        }
    }

    /// Returns the app-owned cancellation message for this navigation action.
    pub fn cancel_message(&self) -> &'static str {
        match self.kind {
            JjWorkingCopyNavigationKind::Edit => "edit cancelled",
            JjWorkingCopyNavigationKind::Next => "next cancelled",
            JjWorkingCopyNavigationKind::Prev => "prev cancelled",
        }
    }

    /// Returns the user-facing `jj` command label for this navigation plan.
    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    /// Returns argv for the underlying working-copy navigation command.
    pub fn command_argv(&self) -> Vec<String> {
        match self.kind {
            JjWorkingCopyNavigationKind::Edit => vec![
                "edit".to_owned(),
                exact_change_id_revset(
                    self.target_change_id
                        .as_deref()
                        .expect("edit requires an exact target change id"),
                ),
            ],
            JjWorkingCopyNavigationKind::Next => {
                vec!["next".to_owned(), "--edit".to_owned()]
            }
            JjWorkingCopyNavigationKind::Prev => {
                vec!["prev".to_owned(), "--edit".to_owned()]
            }
        }
    }

    /// Returns preview text without mutating repository state.
    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    /// Runs the underlying working-copy navigation command.
    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            match self.kind {
                JjWorkingCopyNavigationKind::Edit => "moved @ to edit the selected revision",
                JjWorkingCopyNavigationKind::Next => "moved @ to the next change for editing",
                JjWorkingCopyNavigationKind::Prev => "moved @ to the previous change for editing",
            },
        )
    }

    /// Returns the preview summary shown before confirming this navigation action.
    pub fn preview_summary(&self) -> String {
        match self.kind {
            JjWorkingCopyNavigationKind::Edit => {
                let target = self
                    .target_change_id
                    .as_deref()
                    .expect("edit requires an exact target change id");

                format!(
                    concat!(
                        "command: {}\n\n",
                        "target: exact selected log revision {}\n",
                        "effect: moves @ to edit that revision directly\n",
                        "selection: the selected log row becomes the exact jj edit argument\n",
                        "confirmation: press Enter to run {}\n",
                        "undo path: jj undo"
                    ),
                    self.command_label(),
                    target,
                    self.command_label(),
                )
            }
            JjWorkingCopyNavigationKind::Next => format!(
                concat!(
                    "command: {}\n\n",
                    "target: current working-copy change (@)\n",
                    "selection: the highlighted log row is not an argument to jj next --edit\n",
                    "effect: runs jj topology movement relative to @ and opens the next change ",
                    "for editing directly\n",
                    "ambiguity: jj may fail if the next editable change is ambiguous or ",
                    "unavailable\n",
                    "confirmation: press Enter to run jj next --edit\n",
                    "undo path: jj undo"
                ),
                self.command_label(),
            ),
            JjWorkingCopyNavigationKind::Prev => format!(
                concat!(
                    "command: {}\n\n",
                    "target: current working-copy change (@)\n",
                    "selection: the highlighted log row is not an argument to jj prev --edit\n",
                    "effect: runs jj topology movement relative to @ and opens the previous ",
                    "change for editing directly\n",
                    "ambiguity: jj may fail if the previous editable change is ambiguous or ",
                    "unavailable\n",
                    "confirmation: press Enter to run jj prev --edit\n",
                    "undo path: jj undo"
                ),
                self.command_label(),
            ),
        }
    }
}
