//! Working-copy action plans.
//!
//! This module owns working-copy creation, duplication, split, and navigation
//! plans. These commands share the same boundary: log selection supplies an
//! exact change id only for exact-target actions, while `@` remains the stable
//! target for current-working-copy and topology-relative movement.

use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::actions::CommandOutput;
use crate::jj::exact_change_id_revset;
use crate::jj::{interactive_jj_command, run_direct_args};
use crate::terminal_process::InteractiveCommand;
use crate::terminal_process::run_with_ratatui_terminal;

// Working-copy creation and copy plans produce a new or duplicated change from
// selected log context; split is the interactive patch-selection variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjNewPlan {
    parents: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjDuplicatePlan {
    source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjSplitTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjSplitPlan {
    target: JjSplitTarget,
}

// Working-copy navigation plans move @ through jj topology commands and keep
// selection-sensitive edit distinct from selection-independent next/prev.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjWorkingCopyNavigationKind {
    Edit,
    Next,
    Prev,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjWorkingCopyNavigationPlan {
    kind: JjWorkingCopyNavigationKind,
    target_change_id: Option<String>,
}

impl JjNewPlan {
    pub fn new(parents: Vec<String>) -> Self {
        Self { parents }.normalize()
    }

    pub fn parents(&self) -> &[String] {
        &self.parents
    }

    pub fn command_label(&self) -> String {
        let label_args = self
            .command_argv()
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = vec!["new".to_owned()];
        argv.extend(self.parents.iter().cloned());
        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            "created new change",
        )
    }

    pub fn preview_summary(&self) -> String {
        let parents = self
            .parents
            .iter()
            .map(|parent| format!("parent: {parent}"))
            .collect::<Vec<_>>()
            .join("\n");
        let graph_effect = if self.parents.len() == 1 {
            "graph effect: creates a new working-copy change from the selected parent"
        } else {
            "graph effect: creates a new working-copy merge change from the selected parents"
        };

        format!(
            "command: {}\n\n{}\n\n{}\n\nconfirmation: press Enter to run jj new\nundo path: jj undo",
            self.command_label(),
            parents,
            graph_effect,
        )
    }

    fn normalize(mut self) -> Self {
        self.parents.retain(|parent| !parent.trim().is_empty());
        self
    }
}

impl JjDuplicatePlan {
    pub fn exact_change(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec!["duplicate".to_owned(), exact_change_id_revset(&self.source)]
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "duplicated")
    }

    pub fn preview_summary(&self) -> String {
        [
            format!("command: {}", self.command_label()),
            String::new(),
            format!("source revision: {}", self.source),
            "source count: 1 exact selected change; multi-source duplicate is intentionally not exposed".to_owned(),
            "effect: creates one new change with the same content as the selected source".to_owned(),
            "placement: jj duplicates onto the source's existing parents because no destination is passed".to_owned(),
            "description: jj controls duplicate description behavior through its own configuration".to_owned(),
            "after run: jk refreshes and selects the source in recent work as a fallback because it does not parse duplicate output for the new change id".to_owned(),
            "confirmation: press Enter to run jj duplicate".to_owned(),
            "cancel: press Esc to return without running jj duplicate".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }
}

impl JjSplitTarget {
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    #[cfg(test)]
    pub fn exact_change_id(&self) -> Option<&str> {
        match self {
            Self::ExactChange(change_id) => Some(change_id),
            Self::CurrentWorkingCopy => None,
        }
    }

    fn command_args(&self) -> Vec<String> {
        match self {
            Self::ExactChange(change_id) => vec![
                "split".to_owned(),
                "--revision".to_owned(),
                exact_change_id_revset(change_id),
            ],
            Self::CurrentWorkingCopy => vec!["split".to_owned()],
        }
    }

    fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => {
                format!("exact selected log revision {change_id}")
            }
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }

    fn status_context(&self) -> String {
        match self {
            Self::ExactChange(change_id) => {
                format!("split exact log revision {change_id}")
            }
            Self::CurrentWorkingCopy => "split current working-copy change (@)".to_owned(),
        }
    }
}

impl JjSplitPlan {
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self {
            target: JjSplitTarget::exact_change(change_id),
        }
    }

    pub fn current_working_copy() -> Self {
        Self {
            target: JjSplitTarget::current_working_copy(),
        }
    }

    pub fn target(&self) -> &JjSplitTarget {
        &self.target
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        self.target.command_args()
    }

    pub fn status_context(&self) -> String {
        self.target.status_context()
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run_interactive(&self, terminal: &mut DefaultTerminal) -> Result<CommandOutput> {
        let command = self.interactive_command();
        let result = run_with_ratatui_terminal(terminal, &command)?;
        Ok(CommandOutput::new(
            self.success_result_message(result.status().description()),
        ))
    }

    pub(crate) fn interactive_command(&self) -> InteractiveCommand {
        interactive_jj_command(self.command_argv(), &self.command_label())
    }

    pub fn success_result_message(&self, status: &str) -> String {
        [
            "result: split command completed successfully".to_owned(),
            format!("child exit status: {status}"),
            "output visibility: jj's diff editor and child output were shown live while jk's terminal was suspended; jk did not capture that output".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }

    pub fn failure_result_message(&self, error: &str) -> String {
        [
            "result: split command failed or did not complete".to_owned(),
            format!("runner status: {error}"),
            "output visibility: jj's diff editor and child output were live terminal output while jk's terminal was suspended; jk did not capture stderr for this result".to_owned(),
            "recovery: if jj recorded an operation, use jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }

    pub fn preview_summary(&self) -> String {
        [
            format!("command: {}", self.command_label()),
            String::new(),
            format!("target: {}", self.target.preview_target()),
            "handoff: jj split opens jj's diff editor to choose patch content".to_owned(),
            "description: jj may also invoke description editing after patch selection".to_owned(),
            "honesty: jk is not an in-app patch editor and does not choose hunks or lines".to_owned(),
            "fileset: no fileset is passed; patch selection is delegated to jj".to_owned(),
            "confirmation: press Enter to suspend jk and run jj split".to_owned(),
            "cancel: press Esc to return without running jj split".to_owned(),
            "after run: jk will refresh and show an app-owned result because inherited child output may disappear when the alternate screen is restored".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj op show -p".to_owned(),
        ]
        .join("\n")
    }
}

impl JjWorkingCopyNavigationPlan {
    pub fn edit(change_id: impl Into<String>) -> Self {
        Self {
            kind: JjWorkingCopyNavigationKind::Edit,
            target_change_id: Some(change_id.into()),
        }
    }

    pub fn next() -> Self {
        Self {
            kind: JjWorkingCopyNavigationKind::Next,
            target_change_id: None,
        }
    }

    pub fn prev() -> Self {
        Self {
            kind: JjWorkingCopyNavigationKind::Prev,
            target_change_id: None,
        }
    }

    pub fn kind(&self) -> JjWorkingCopyNavigationKind {
        self.kind
    }

    pub fn target_change_id(&self) -> Option<&str> {
        self.target_change_id.as_deref()
    }

    pub fn overlay_title(&self) -> &'static str {
        match self.kind {
            JjWorkingCopyNavigationKind::Edit => "Edit",
            JjWorkingCopyNavigationKind::Next => "Next",
            JjWorkingCopyNavigationKind::Prev => "Prev",
        }
    }

    pub fn cancel_message(&self) -> &'static str {
        match self.kind {
            JjWorkingCopyNavigationKind::Edit => "edit cancelled",
            JjWorkingCopyNavigationKind::Next => "next cancelled",
            JjWorkingCopyNavigationKind::Prev => "prev cancelled",
        }
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

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

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

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

#[cfg(test)]
mod tests;
