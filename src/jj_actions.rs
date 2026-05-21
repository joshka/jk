//! Preview-first action and mutation plans for `jj` commands.
//!
//! These value types own argv construction, labels, preview summaries, and
//! direct execution for user-confirmed mutation flows. Syntax quoting helpers
//! stay in [`crate::jj_syntax`]; rendered row loading stays in
//! [`crate::jj_rows`] and view-spec command construction stays in
//! [`crate::jj`].

use color_eyre::Result;
use color_eyre::eyre::eyre;
use ratatui::DefaultTerminal;

use crate::interactive_process::InteractiveCommand;
use crate::interactive_process::run_with_ratatui_terminal;
use crate::jj::{
    ColorMode, base_command, interactive_jj_command, run_direct_args, run_direct_args_stdout,
    summarize_output,
};
use crate::jj_syntax::{exact_change_id_revset, exact_string_pattern, root_file_fileset};

mod git_sync;

pub use git_sync::{JjGitFetch, JjGitPush, JjGitPushTarget};

const DESCRIPTION_FIRST_LINE_TEMPLATE: &str = "description.first_line() ++ \"\\n\"";

// Shared result envelope for preview and confirmed command output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    message: String,
}

impl CommandOutput {
    pub(crate) fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

// Operation recovery plans keep global undo/redo separate from selected
// operation restore/revert, which target one exact operation id.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjOperationRecoveryKind {
    Undo,
    Redo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjOperationTargetKind {
    Restore,
    Revert,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjOperationRecovery {
    kind: JjOperationRecoveryKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjOperationTarget {
    kind: JjOperationTargetKind,
    operation_id: String,
}

impl JjOperationRecovery {
    pub fn new(kind: JjOperationRecoveryKind) -> Self {
        Self { kind }
    }

    #[cfg(test)]
    pub fn kind(&self) -> JjOperationRecoveryKind {
        self.kind
    }

    pub fn command_label(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "jj undo",
            JjOperationRecoveryKind::Redo => "jj redo",
        }
    }

    pub fn command_argv(&self) -> Vec<String> {
        match self.kind {
            JjOperationRecoveryKind::Undo => vec!["undo".to_owned()],
            JjOperationRecoveryKind::Redo => vec!["redo".to_owned()],
        }
    }

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

    pub fn success_hint(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "jj redo",
            JjOperationRecoveryKind::Redo => "jj undo",
        }
    }

    pub fn status_action(&self) -> &'static str {
        match self.kind {
            JjOperationRecoveryKind::Undo => "undo",
            JjOperationRecoveryKind::Redo => "redo",
        }
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            self.command_label(),
            self.status_action(),
        )
    }
}

impl JjOperationTargetKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Restore => "restore",
            Self::Revert => "revert",
        }
    }

    fn success_fallback(self) -> &'static str {
        match self {
            Self::Restore => "restored operation",
            Self::Revert => "reverted operation",
        }
    }
}

impl JjOperationTarget {
    pub fn restore(operation_id: impl Into<String>) -> Self {
        Self {
            kind: JjOperationTargetKind::Restore,
            operation_id: operation_id.into(),
        }
    }

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

    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    pub fn status_action(&self) -> &'static str {
        self.kind.label()
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

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

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            self.kind.success_fallback(),
        )
    }

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

// Graph rewrite plans move selected changes relative to an explicit
// destination while leaving final graph simulation to jj.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRebasePlan {
    sources: Vec<String>,
    destination: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjSquashPlan {
    sources: Vec<String>,
    destination: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbsorbPlan {
    source: String,
    destinations: Vec<String>,
}

// Working-copy creation and copy plans produce a new or duplicated change from
// selected graph context; split is the interactive patch-selection variant.
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

// Description finalization plans update messages without opening an editor;
// commit always targets @ and describe may target @ or an exact change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjDescribeTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjDescribePlan {
    target: JjDescribeTarget,
    message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjCommitPlan {
    message: String,
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

// Content mutation plans preview the diff jj will remove or reverse-apply, and
// file mutation plans restrict paths through exact root-file filesets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRestorePlan {
    target: JjRestoreTarget,
    path: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileMutationKind {
    Track,
    Untrack,
    Chmod(JjFileChmodMode),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjFileChmodMode {
    Executable,
    Normal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjFileMutationTarget {
    WorkingCopy { path: String },
    ExactRevision { revision: String, path: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjFileMutationPlan {
    kind: JjFileMutationKind,
    target: JjFileMutationTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum JjRestoreTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRevertPlan {
    revision: String,
}

// Bookmark mutation plans keep local-name changes, remote tracking metadata,
// and forget/delete semantics in one exact-pattern command family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjBookmarkMutationKind {
    Create,
    Set,
    Move,
    Rename,
    Delete,
    Forget,
    Track,
    Untrack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjBookmarkTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjBookmarkForgetTarget {
    Local { tracking: String },
    RemoteOnly { remote: String, tracking: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjBookmarkTrackingTarget {
    local_bookmark: Option<String>,
    remote_bookmark: String,
    remote: String,
    visible_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjBookmarkMutationPlan {
    kind: JjBookmarkMutationKind,
    name: String,
    new_name: Option<String>,
    target: Option<JjBookmarkTarget>,
    forget_target: Option<JjBookmarkForgetTarget>,
    tracking_target: Option<Box<JjBookmarkTrackingTarget>>,
}

// Working-copy creation starts here; keep new/duplicate/split extraction
// together because callers depend on their shared graph-selection contract.
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
                format!("exact selected graph revision {change_id}")
            }
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }

    fn status_context(&self) -> String {
        match self {
            Self::ExactChange(change_id) => {
                format!("split exact graph revision {change_id}")
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

impl JjDescribeTarget {
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    pub fn exact_change_id(&self) -> Option<&str> {
        match self {
            Self::ExactChange(change_id) => Some(change_id),
            Self::CurrentWorkingCopy => None,
        }
    }

    fn command_arg(&self) -> String {
        match self {
            Self::ExactChange(change_id) => exact_change_id_revset(change_id),
            Self::CurrentWorkingCopy => "@".to_owned(),
        }
    }

    fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => format!("exact selected revision {change_id}"),
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }
}

impl JjDescribePlan {
    pub fn new(target: JjDescribeTarget, message: impl Into<String>) -> Self {
        Self {
            target,
            message: message.into(),
        }
    }

    pub fn target(&self) -> &JjDescribeTarget {
        &self.target
    }

    pub fn command_label(&self) -> String {
        format!(
            "jj describe {} --message {}",
            self.target.label(),
            self.message
        )
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "describe".to_owned(),
            self.target.command_arg(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "described")
    }

    pub fn preview_summary(&self) -> String {
        format!(
            "command: {}\n\ntarget: {}\nmessage: {}\n\neffect: updates only the target change description without opening an editor\nconfirmation: press Enter to run jj describe\nundo path: jj undo",
            self.command_label(),
            self.target.preview_target(),
            self.message,
        )
    }
}

impl JjCommitPlan {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn command_label(&self) -> String {
        format!("jj commit --message {}", self.message)
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "commit".to_owned(),
            "--message".to_owned(),
            self.message.clone(),
        ]
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            "committed current working-copy change",
        )
    }

    pub fn preview_summary(&self) -> String {
        format!(
            "command: {}\n\ntarget: current working-copy change (@)\nmessage: {}\n\neffect: updates @ with the message and creates a new working-copy change on top\nselection: selected graph rows are not arguments to jj commit\nconfirmation: press Enter to run jj commit\nundo path: jj undo",
            self.command_label(),
            self.message,
        )
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
                        "target: exact selected graph revision {}\n",
                        "effect: moves @ to edit that revision directly\n",
                        "selection: the selected graph row becomes the exact jj edit argument\n",
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
                    "selection: the highlighted graph row is not an argument to jj next --edit\n",
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
                    "selection: the highlighted graph row is not an argument to jj prev --edit\n",
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

// Content/file mutation plans share exact change/path targeting and preview
// honesty about showing jj's source diff rather than simulating the result.
impl JjRestorePlan {
    pub fn for_revision(revision: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: None,
        }
    }

    pub fn for_path(revision: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::ExactChange(revision.into()),
            path: Some(path.into()),
        }
    }

    pub fn for_working_copy_path(path: impl Into<String>) -> Self {
        Self {
            target: JjRestoreTarget::CurrentWorkingCopy,
            path: Some(path.into()),
        }
    }

    pub fn revision(&self) -> &str {
        self.target.label()
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = match &self.target {
            JjRestoreTarget::ExactChange(revision) => vec![
                "restore".to_owned(),
                "--changes-in".to_owned(),
                exact_change_id_revset(revision),
            ],
            JjRestoreTarget::CurrentWorkingCopy => vec!["restore".to_owned()],
        };
        if let Some(path) = &self.path {
            argv.push(root_file_fileset(path));
        }
        argv
    }

    pub fn preview_diff_label(&self) -> String {
        let label_args = self.preview_diff_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn preview_diff_argv(&self) -> Vec<String> {
        let mut argv = match &self.target {
            JjRestoreTarget::ExactChange(revision) => vec![
                "diff".to_owned(),
                "-r".to_owned(),
                exact_change_id_revset(revision),
            ],
            JjRestoreTarget::CurrentWorkingCopy => vec!["diff".to_owned()],
        };
        if let Some(path) = &self.path {
            argv.push(root_file_fileset(path));
        }
        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "restored")
    }

    pub fn preview_summary(&self, forward_diff: &str) -> String {
        let mut lines = vec![
            format!("target revision: {}", self.revision()),
            format!("command: {}", self.command_label()),
        ];

        match &self.path {
            Some(path) => {
                lines.push(format!("selected path: {path}"));
                lines.push(format!("exact fileset: {}", root_file_fileset(path)));
                lines.push(self.target.path_restore_effect());
            }
            None => lines.push(
                "effect: restore removes the selected revision's forward diff from that exact revision"
                    .to_owned(),
            ),
        }

        lines.extend([
            format!("preview source: {}", self.preview_diff_label()),
            "honesty: the output below is the forward diff that jj restore removes; jk is not simulating the final graph".to_owned(),
            "confirmation: press Enter to run jj restore".to_owned(),
            "undo path: jj undo".to_owned(),
            String::new(),
            "forward diff:".to_owned(),
            forward_diff.trim_end().to_owned(),
        ]);

        lines.join("\n")
    }
}

impl JjRestoreTarget {
    fn label(&self) -> &str {
        match self {
            Self::ExactChange(revision) => revision,
            Self::CurrentWorkingCopy => "@",
        }
    }

    fn path_restore_effect(&self) -> String {
        match self {
            Self::ExactChange(_) => {
                "effect: restore removes the selected path's forward diff from that exact revision"
                    .to_owned()
            }
            Self::CurrentWorkingCopy => {
                "effect: restore removes the selected path's working-copy diff from @".to_owned()
            }
        }
    }
}

impl JjFileChmodMode {
    pub fn command_arg(self) -> &'static str {
        match self {
            Self::Executable => "x",
            Self::Normal => "n",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::Normal => "normal",
        }
    }
}

impl JjFileMutationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Track => "track",
            Self::Untrack => "untrack",
            Self::Chmod(JjFileChmodMode::Executable) => "chmod x",
            Self::Chmod(JjFileChmodMode::Normal) => "chmod n",
        }
    }

    fn success_fallback(self) -> &'static str {
        match self {
            Self::Track => "tracked file",
            Self::Untrack => "untracked file",
            Self::Chmod(JjFileChmodMode::Executable) => "set file executable",
            Self::Chmod(JjFileChmodMode::Normal) => "set file normal",
        }
    }
}

impl JjFileMutationTarget {
    fn path(&self) -> &str {
        match self {
            Self::WorkingCopy { path } | Self::ExactRevision { path, .. } => path,
        }
    }

    fn revision(&self) -> Option<&str> {
        match self {
            Self::WorkingCopy { .. } => None,
            Self::ExactRevision { revision, .. } => Some(revision),
        }
    }

    fn scope_label(&self) -> String {
        match self {
            Self::WorkingCopy { .. } => "working-copy change (@)".to_owned(),
            Self::ExactRevision { revision, .. } => {
                format!("exact selected revision {revision}")
            }
        }
    }
}

impl JjFileMutationPlan {
    pub fn track(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Track,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    pub fn untrack(path: impl Into<String>) -> Self {
        Self {
            kind: JjFileMutationKind::Untrack,
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    pub fn chmod_working_copy(path: impl Into<String>, mode: JjFileChmodMode) -> Self {
        Self {
            kind: JjFileMutationKind::Chmod(mode),
            target: JjFileMutationTarget::WorkingCopy { path: path.into() },
        }
    }

    pub fn chmod_exact_revision(
        revision: impl Into<String>,
        path: impl Into<String>,
        mode: JjFileChmodMode,
    ) -> Self {
        Self {
            kind: JjFileMutationKind::Chmod(mode),
            target: JjFileMutationTarget::ExactRevision {
                revision: revision.into(),
                path: path.into(),
            },
        }
    }

    pub fn kind(&self) -> JjFileMutationKind {
        self.kind
    }

    pub fn path(&self) -> &str {
        self.target.path()
    }

    pub fn revision(&self) -> Option<&str> {
        self.target.revision()
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        let fileset = root_file_fileset(self.path());
        match self.kind {
            JjFileMutationKind::Track => {
                vec![
                    "file".to_owned(),
                    "track".to_owned(),
                    "--".to_owned(),
                    fileset,
                ]
            }
            JjFileMutationKind::Untrack => {
                vec![
                    "file".to_owned(),
                    "untrack".to_owned(),
                    "--".to_owned(),
                    fileset,
                ]
            }
            JjFileMutationKind::Chmod(mode) => {
                let mut argv = vec!["file".to_owned(), "chmod".to_owned()];
                if let Some(revision) = self.revision() {
                    argv.push("-r".to_owned());
                    argv.push(exact_change_id_revset(revision));
                }
                argv.push(mode.command_arg().to_owned());
                argv.push("--".to_owned());
                argv.push(fileset);
                argv
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
            self.kind.success_fallback(),
        )
    }

    pub fn preview_summary(&self) -> String {
        let mut lines = vec![
            format!("command: {}", self.command_label()),
            String::new(),
            format!("selected path: {}", self.path()),
            format!("exact fileset: {}", root_file_fileset(self.path())),
            format!("target scope: {}", self.target.scope_label()),
        ];

        if let JjFileMutationKind::Chmod(mode) = self.kind {
            lines.push(format!(
                "chmod mode: {} ({})",
                mode.command_arg(),
                mode.label()
            ));
        }

        lines.extend([
            self.effect_summary(),
            "confirmation: press Enter to run the command above".to_owned(),
            "output: jj stdout and stderr are preserved in this result pane".to_owned(),
            "recovery: jj undo".to_owned(),
            "review: jj status; jj op show -p".to_owned(),
        ]);
        lines.join("\n")
    }

    fn effect_summary(&self) -> String {
        match self.kind {
            JjFileMutationKind::Track => {
                "effect: starts tracking this exact untracked working-copy path".to_owned()
            }
            JjFileMutationKind::Untrack => {
                "effect: stops tracking this exact working-copy path; jj requires the path to already be ignored".to_owned()
            }
            JjFileMutationKind::Chmod(JjFileChmodMode::Executable) => {
                "effect: marks this exact path executable in the target scope".to_owned()
            }
            JjFileMutationKind::Chmod(JjFileChmodMode::Normal) => {
                "effect: marks this exact path non-executable in the target scope".to_owned()
            }
        }
    }
}

impl JjRevertPlan {
    pub fn new(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
        }
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec![
            "revert".to_owned(),
            "-r".to_owned(),
            exact_change_id_revset(&self.revision),
            "-o".to_owned(),
            "@".to_owned(),
        ]
    }

    pub fn preview_diff_label(&self) -> String {
        let label_args = self.preview_diff_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn preview_diff_argv(&self) -> Vec<String> {
        vec![
            "diff".to_owned(),
            "-r".to_owned(),
            exact_change_id_revset(&self.revision),
        ]
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        let forward_diff =
            run_direct_args_stdout(self.preview_diff_argv(), &self.preview_diff_label())?;
        Ok(CommandOutput::new(self.preview_summary(&forward_diff)))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "reverted")
    }

    pub fn preview_summary(&self, forward_diff: &str) -> String {
        [
            format!("target revision: {}", self.revision),
            format!("command: {}", self.command_label()),
            "effect: revert reverse-applies the selected revision's forward diff into @".to_owned(),
            format!("preview source: {}", self.preview_diff_label()),
            "honesty: the output below is the forward diff that jj revert reverse-applies into @; jk is not simulating the final graph".to_owned(),
            "confirmation: press Enter to run jj revert".to_owned(),
            "undo path: jj undo".to_owned(),
            String::new(),
            "forward diff:".to_owned(),
            forward_diff.trim_end().to_owned(),
        ]
        .join("\n")
    }
}

// Bookmark mutation owns all bookmark subcommand argv so exact-name matching,
// tracking metadata, and preview wording stay consistent.
impl JjBookmarkMutationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Set => "set",
            Self::Move => "move",
            Self::Rename => "rename",
            Self::Delete => "delete",
            Self::Forget => "forget",
            Self::Track => "track",
            Self::Untrack => "untrack",
        }
    }

    fn success_fallback(self) -> &'static str {
        match self {
            Self::Create => "created bookmark",
            Self::Set => "set bookmark",
            Self::Move => "moved bookmark",
            Self::Rename => "renamed bookmark",
            Self::Delete => "deleted bookmark",
            Self::Forget => "forgot bookmark",
            Self::Track => "tracked bookmark",
            Self::Untrack => "untracked bookmark",
        }
    }
}

impl JjBookmarkTarget {
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    fn command_arg(&self) -> String {
        match self {
            Self::ExactChange(change_id) => exact_change_id_revset(change_id),
            Self::CurrentWorkingCopy => "@".to_owned(),
        }
    }

    fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => format!("exact selected revision {change_id}"),
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }
}

impl JjBookmarkMutationPlan {
    pub fn create(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Create,
            name: name.into(),
            new_name: None,
            target: Some(target),
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn set(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Set,
            name: name.into(),
            new_name: None,
            target: Some(target),
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn move_to(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Move,
            name: name.into(),
            new_name: None,
            target: Some(target),
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn rename(old_name: impl Into<String>, new_name: impl Into<String>) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Rename,
            name: old_name.into(),
            new_name: Some(new_name.into()),
            target: None,
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn delete(name: impl Into<String>) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Delete,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn forget(name: impl Into<String>, target: JjBookmarkForgetTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Forget,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: Some(target),
            tracking_target: None,
        }
    }

    pub fn track(name: impl Into<String>, target: JjBookmarkTrackingTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Track,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: None,
            tracking_target: Some(Box::new(target)),
        }
    }

    pub fn untrack(name: impl Into<String>, target: JjBookmarkTrackingTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Untrack,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: None,
            tracking_target: Some(Box::new(target)),
        }
    }

    pub fn kind(&self) -> JjBookmarkMutationKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_name(&self) -> Option<&str> {
        self.new_name.as_deref()
    }

    pub fn target(&self) -> Option<&JjBookmarkTarget> {
        self.target.as_ref()
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
        match self.kind {
            JjBookmarkMutationKind::Create => vec![
                "bookmark".to_owned(),
                "create".to_owned(),
                "--revision".to_owned(),
                self.required_target().command_arg(),
                self.name.clone(),
            ],
            JjBookmarkMutationKind::Set => vec![
                "bookmark".to_owned(),
                "set".to_owned(),
                "--revision".to_owned(),
                self.required_target().command_arg(),
                self.name.clone(),
            ],
            JjBookmarkMutationKind::Move => vec![
                "bookmark".to_owned(),
                "move".to_owned(),
                "--to".to_owned(),
                self.required_target().command_arg(),
                exact_string_pattern(&self.name),
            ],
            JjBookmarkMutationKind::Rename => vec![
                "bookmark".to_owned(),
                "rename".to_owned(),
                self.name.clone(),
                self.required_new_name().to_owned(),
            ],
            JjBookmarkMutationKind::Delete => vec![
                "bookmark".to_owned(),
                "delete".to_owned(),
                exact_string_pattern(&self.name),
            ],
            JjBookmarkMutationKind::Forget => {
                let mut argv = vec!["bookmark".to_owned(), "forget".to_owned()];
                if self.required_forget_target().include_remotes() {
                    argv.push("--include-remotes".to_owned());
                }
                argv.push(exact_string_pattern(&self.name));
                argv
            }
            JjBookmarkMutationKind::Track | JjBookmarkMutationKind::Untrack => {
                let target = self.required_tracking_target();
                vec![
                    "bookmark".to_owned(),
                    self.kind.label().to_owned(),
                    "--remote".to_owned(),
                    exact_string_pattern(target.remote()),
                    exact_string_pattern(target.remote_bookmark()),
                ]
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
            self.kind.success_fallback(),
        )
    }

    pub fn preview_summary(&self) -> String {
        let mut lines = vec![
            format!("command: {}", self.command_label()),
            String::new(),
            format!("bookmark: {}", self.name),
        ];

        match self.kind {
            JjBookmarkMutationKind::Create => {
                lines.extend([
                    "source/current: new local bookmark name".to_owned(),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: creates one local bookmark at the exact destination target".to_owned(),
                    "confirmation: press Enter to run jj bookmark create".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Set => {
                lines.extend([
                    "source/current: jj resolves the literal local bookmark name".to_owned(),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: sets one local bookmark to the exact destination target".to_owned(),
                    "confirmation: press Enter to run jj bookmark set".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Move => {
                lines.extend([
                    format!(
                        "source/current: exact pattern {}",
                        exact_string_pattern(&self.name)
                    ),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: moves one exactly named local bookmark to the destination target"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark move".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Rename => {
                lines.extend([
                    format!("old name: {}", self.name),
                    format!("new name: {}", self.required_new_name()),
                    "target: exact selected local bookmark row; rendered labels are not parsed"
                        .to_owned(),
                    "effect: renames one local bookmark without --overwrite-existing".to_owned(),
                    "duplicate name: jj failure output is preserved if the new name already exists"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark rename".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Delete => {
                lines.extend([
                    format!(
                        "source/current: exact pattern {}",
                        exact_string_pattern(&self.name)
                    ),
                    "destination: none".to_owned(),
                    "effect: deletes one local bookmark; this is delete, not forget".to_owned(),
                    "tracking: track/untrack stay disabled until exact tracking metadata exists"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark delete".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Forget => {
                let target = self.required_forget_target();
                lines.extend([
                    format!(
                        "target: exact bookmark {}",
                        exact_string_pattern(&self.name)
                    ),
                    format!("visible state: {}", target.visible_state()),
                    format!("scope: {}", target.scope_summary()),
                    "effect: forgets tracking relationship metadata; this is forget, not delete"
                        .to_owned(),
                    "output: full jj failure output remains inspectable in this pane".to_owned(),
                    "confirmation: press Enter to run jj bookmark forget".to_owned(),
                    "recovery: jj undo; review: jj op show -p".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Track | JjBookmarkMutationKind::Untrack => {
                let target = self.required_tracking_target();
                lines.extend([
                    format!("local bookmark: {}", target.local_bookmark_label()),
                    format!("remote bookmark: {}", target.remote_bookmark()),
                    format!("remote: {}", target.remote()),
                    format!("remote pattern: {}", exact_string_pattern(target.remote())),
                    format!(
                        "bookmark pattern: {}",
                        exact_string_pattern(target.remote_bookmark())
                    ),
                    format!("visible state: {}", target.visible_state()),
                    target.effect(self.kind),
                    "output: full jj result or failure output remains inspectable in this pane"
                        .to_owned(),
                    format!(
                        "confirmation: press Enter to run jj bookmark {}",
                        self.kind.label()
                    ),
                    "recovery: jj undo; review: jj op show -p".to_owned(),
                ]);
            }
        }

        lines.join("\n")
    }

    fn required_target(&self) -> &JjBookmarkTarget {
        self.target
            .as_ref()
            .expect("bookmark mutation kind requires target")
    }

    fn required_new_name(&self) -> &str {
        self.new_name
            .as_deref()
            .expect("bookmark rename requires new name")
    }

    fn required_forget_target(&self) -> &JjBookmarkForgetTarget {
        self.forget_target
            .as_ref()
            .expect("bookmark forget requires a forget target")
    }

    fn required_tracking_target(&self) -> &JjBookmarkTrackingTarget {
        self.tracking_target
            .as_deref()
            .expect("bookmark track/untrack requires a tracking target")
    }
}

impl JjBookmarkForgetTarget {
    pub fn local(tracking: impl Into<String>) -> Self {
        Self::Local {
            tracking: tracking.into(),
        }
    }

    pub fn remote_only(remote: impl Into<String>, tracking: impl Into<String>) -> Self {
        Self::RemoteOnly {
            remote: remote.into(),
            tracking: tracking.into(),
        }
    }

    fn include_remotes(&self) -> bool {
        matches!(self, Self::RemoteOnly { .. })
    }

    fn visible_state(&self) -> String {
        match self {
            Self::Local { tracking } => format!("local bookmark; {tracking}"),
            Self::RemoteOnly { remote, tracking } => {
                format!("remote-only bookmark on {remote}; {tracking}")
            }
        }
    }

    fn scope_summary(&self) -> &'static str {
        match self {
            Self::Local { .. } => "local tracked bookmark or local bookmark with remote peer",
            Self::RemoteOnly { .. } => "one remote peer and no local peer; includes remotes",
        }
    }
}

impl JjBookmarkTrackingTarget {
    pub fn new(
        local_bookmark: Option<String>,
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self {
            local_bookmark,
            remote_bookmark: remote_bookmark.into(),
            remote: remote.into(),
            visible_state: visible_state.into(),
        }
    }

    pub fn local(
        local_bookmark: impl Into<String>,
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self::new(
            Some(local_bookmark.into()),
            remote_bookmark,
            remote,
            visible_state,
        )
    }

    pub fn remote_only(
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self::new(None, remote_bookmark, remote, visible_state)
    }

    pub fn remote_bookmark(&self) -> &str {
        &self.remote_bookmark
    }

    pub fn remote(&self) -> &str {
        &self.remote
    }

    pub fn visible_state(&self) -> &str {
        &self.visible_state
    }

    fn local_bookmark_label(&self) -> &str {
        self.local_bookmark.as_deref().unwrap_or("absent")
    }

    fn effect(&self, kind: JjBookmarkMutationKind) -> String {
        match kind {
            JjBookmarkMutationKind::Track => {
                "effect: tracks the exact remote bookmark for the exact local bookmark; this does not fetch, push, delete, or rename".to_owned()
            }
            JjBookmarkMutationKind::Untrack => {
                "effect: untracks the exact remote bookmark relationship; this does not delete the local or remote bookmark".to_owned()
            }
            JjBookmarkMutationKind::Create
            | JjBookmarkMutationKind::Set
            | JjBookmarkMutationKind::Move
            | JjBookmarkMutationKind::Rename
            | JjBookmarkMutationKind::Delete
            | JjBookmarkMutationKind::Forget => {
                unreachable!("tracking target effects only apply to track/untrack")
            }
        }
    }
}

pub fn validate_bookmark_rename_new_name(
    old_name: &str,
    new_name: &str,
) -> std::result::Result<(), String> {
    if new_name.is_empty() {
        return Err("empty bookmark name".to_owned());
    }
    if new_name == old_name {
        return Err("new bookmark name is unchanged".to_owned());
    }
    if new_name == "@" {
        return Err("bookmark name must not be @".to_owned());
    }
    if new_name.starts_with('-') {
        return Err("bookmark name must not start with '-'".to_owned());
    }
    if new_name.starts_with('/') || new_name.ends_with('/') || new_name.contains("//") {
        return Err("bookmark name must not contain empty path components".to_owned());
    }
    if new_name.starts_with('.') || new_name.contains("/.") {
        return Err("bookmark name components must not start with '.'".to_owned());
    }
    if new_name.ends_with('.') || new_name.ends_with(".lock") {
        return Err("bookmark name must not end with '.' or '.lock'".to_owned());
    }
    if new_name.contains("..") {
        return Err("bookmark name must not contain '..'".to_owned());
    }
    if new_name
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err("bookmark name must not contain whitespace or control characters".to_owned());
    }
    if new_name
        .chars()
        .any(|character| matches!(character, '@' | ':' | '?' | '*' | '[' | '\\' | '^' | '~'))
    {
        return Err("bookmark name contains a reserved ref character".to_owned());
    }

    Ok(())
}

// Rewrite plans share explicit source/destination roles and avoid parsing or
// predicting jj's final graph shape.
impl JjRebasePlan {
    pub fn new(sources: Vec<String>, destination: impl Into<String>) -> Self {
        Self {
            sources,
            destination: destination.into(),
        }
        .normalize()
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn command_label(&self, _dry_run: bool) -> String {
        let label_args = self
            .command_argv(false)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self, _dry_run: bool) -> Vec<String> {
        let mut argv = vec!["rebase".to_owned()];
        for source in &self.sources {
            argv.push("-r".to_owned());
            argv.push(source.clone());
        }
        argv.push("-o".to_owned());
        argv.push(self.destination.clone());

        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "rebased",
        )
    }

    pub fn preview_summary(&self) -> String {
        let sources = self
            .sources
            .iter()
            .map(|source| format!("source revision: {source}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "command: {}\n\nroles:\n{}\ndestination revision: {}\n\ncurrent graph context:\n- source rows are selected in jk\n- destination is the current row\n\nexpected jj effect:\n- semantics: jj rebase --revision <source> --onto <destination>\n- only listed source revisions are rebased\n- dependencies among listed sources are preserved\n- descendants outside the selected set may be rebased to fill holes\n- destination descendants are not inserted or rebased by -o\n\nnot a graph preview: jk has not run jj and is not simulating the final graph\n\nreview after run: jj op show -p\nundo path: jj undo\nconfirmation: press Enter to run jj rebase",
            self.command_label(false),
            sources,
            self.destination,
        )
    }

    fn normalize(mut self) -> Self {
        self.sources.retain(|source| !source.trim().is_empty());
        self
    }
}

impl JjSquashPlan {
    pub fn new(sources: Vec<String>, destination: impl Into<String>) -> Self {
        Self {
            sources,
            destination: destination.into(),
        }
        .normalize()
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn destination(&self) -> &str {
        &self.destination
    }

    pub fn command_label(&self, _dry_run: bool) -> String {
        let label_args = self
            .command_argv(false)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self, _dry_run: bool) -> Vec<String> {
        let mut argv = vec!["squash".to_owned()];
        for source in &self.sources {
            argv.push("--from".to_owned());
            argv.push(source.clone());
        }
        argv.push("--into".to_owned());
        argv.push(self.destination.clone());
        argv.push("--use-destination-message".to_owned());

        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "squashed",
        )
    }

    pub fn preview_summary(&self) -> String {
        let sources = self
            .sources
            .iter()
            .map(|source| format!("source: {source}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "command: {}\n\n{}\n\ndestination: {}\n\ngraph effect: moves the selected source changes into the destination; jj may abandon emptied sources and rebase descendants\n\ndescription behavior: --use-destination-message keeps the destination description, discards source descriptions, and avoids an editor or prompt\n\nconfirmation: press Enter to run jj squash\nundo path: jj undo",
            self.command_label(false),
            sources,
            self.destination,
        )
    }

    fn normalize(mut self) -> Self {
        self.sources.retain(|source| !source.trim().is_empty());
        self
    }
}

impl JjAbsorbPlan {
    pub fn new(source: impl Into<String>, destinations: Vec<String>) -> Self {
        Self {
            source: source.into(),
            destinations,
        }
        .normalize()
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn destinations(&self) -> &[String] {
        &self.destinations
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
        let mut argv = vec![
            "absorb".to_owned(),
            "--from".to_owned(),
            exact_change_id_revset(&self.source),
        ];
        for destination in &self.destinations {
            argv.push("--into".to_owned());
            argv.push(exact_change_id_revset(destination));
        }

        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "absorbed")
    }

    pub fn preview_summary(&self) -> String {
        let destinations = self
            .destinations
            .iter()
            .map(|destination| format!("candidate destination: {destination}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            concat!(
                "command: {}\n\n",
                "source: {}\n",
                "{}\n\n",
                "selection: selected revisions are candidate destinations; jj absorb only ",
                "considers selected revisions that are ancestors of the source\n\n",
                "effect: jj splits source changes and moves each change to the closest ",
                "selected mutable ancestor where the corresponding lines were last modified\n\n",
                "opacity: jk does not simulate line-level placement or final graph shape\n\n",
                "ambiguity: changes remain in the source when jj cannot choose unambiguously\n\n",
                "source result: the source may become empty or abandoned depending on jj ",
                "semantics\n\n",
                "confirmation: press Enter to run jj absorb\n",
                "recovery: jj undo\n",
                "review: jj op show -p"
            ),
            self.command_label(),
            self.source,
            destinations,
        )
    }

    fn normalize(mut self) -> Self {
        self.destinations
            .retain(|destination| !destination.trim().is_empty() && destination != &self.source);
        self
    }
}

// Abandon safety owns the preflight probes and strong-confirm preview; keep it
// separate from rewrite plans because it classifies destructive risk first.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPlan {
    revision: String,
}

impl JjAbandonPlan {
    pub fn new(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
        }
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn command_label(&self) -> String {
        format!("jj abandon {}", self.revision)
    }

    pub fn command_argv(&self) -> Vec<String> {
        vec!["abandon".to_owned(), self.exact_change_id_revset()]
    }

    pub fn diff_summary_label(&self) -> String {
        format!("jj diff -r {} --summary", self.revision)
    }

    pub fn diff_summary_argv(&self) -> Vec<String> {
        vec![
            "diff".to_owned(),
            "-r".to_owned(),
            self.exact_change_id_revset(),
            "--summary".to_owned(),
        ]
    }

    pub fn run_preview(&self) -> Result<JjAbandonPreview> {
        let summary = run_direct_args_stdout(self.diff_summary_argv(), &self.diff_summary_label())?;
        let title = self.load_title().ok().flatten();

        Ok(JjAbandonPreview::new(self.revision.clone(), title, summary))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(self.command_argv(), &self.command_label(), "abandoned")
    }

    fn load_title(&self) -> Result<Option<String>> {
        let mut jj = base_command(ColorMode::Never);
        jj.args(self.title_argv());

        let output = jj.output()?;
        if !output.status.success() {
            return Err(eyre!(
                "{} title failed: {}",
                self.revision,
                summarize_output(&output.stdout, &output.stderr, "could not read title")
            ));
        }

        let title = String::from_utf8(output.stdout)?.trim().to_owned();
        Ok((!title.is_empty()).then_some(title))
    }

    fn title_argv(&self) -> Vec<String> {
        vec![
            "log".to_owned(),
            "-r".to_owned(),
            self.exact_change_id_revset(),
            "--no-graph".to_owned(),
            "-T".to_owned(),
            DESCRIPTION_FIRST_LINE_TEMPLATE.to_owned(),
        ]
    }

    fn exact_change_id_revset(&self) -> String {
        exact_change_id_revset(&self.revision)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPreview {
    revision: String,
    title: Option<String>,
    summary: String,
    change_state: AbandonChangeState,
}

impl JjAbandonPreview {
    pub fn new(revision: String, title: Option<String>, summary: String) -> Self {
        let change_state = if summary.trim().is_empty() {
            AbandonChangeState::Empty
        } else {
            AbandonChangeState::NonEmpty
        };

        Self {
            revision,
            title,
            summary,
            change_state,
        }
    }

    #[cfg(test)]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn is_empty_change(&self) -> bool {
        self.change_state == AbandonChangeState::Empty
    }

    pub fn preview_text(&self) -> String {
        let title = self.title.as_deref().unwrap_or("<no description>");
        let summary = if self.summary.trim().is_empty() {
            "empty change".to_owned()
        } else {
            self.summary.trim().to_owned()
        };
        let confirmation = if self.is_empty_change() {
            "press Enter to abandon this empty change".to_owned()
        } else {
            format!(
                "type exact revision '{}' before abandon runs",
                self.revision
            )
        };

        format!(
            "change: {}\ntitle: {}\ndiff summary:\n{}\n\neffect: abandon removes the selected change from the visible history; recovery stays available through jj undo\nconfirmation: {}\nundo path: jj undo",
            self.revision, title, summary, confirmation
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AbandonChangeState {
    Empty,
    NonEmpty,
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::*;

    #[test]
    fn new_plan_uses_positional_parent_revsets() {
        let plan = JjNewPlan::new(vec!["parent-a".to_owned()]);

        assert_eq!(plan.command_argv(), vec!["new", "parent-a"]);
        assert_eq!(plan.command_label(), "jj new parent-a");
        assert!(plan.preview_summary().contains("parent: parent-a"));
        assert!(plan.preview_summary().contains("undo path: jj undo"));
    }

    #[test]
    fn new_plan_preserves_multiple_parent_order() {
        let plan = JjNewPlan::new(vec![
            "parent-a".to_owned(),
            "parent-b".to_owned(),
            "parent-c".to_owned(),
        ]);

        assert_eq!(
            plan.command_argv(),
            vec!["new", "parent-a", "parent-b", "parent-c"]
        );
        assert_eq!(plan.command_label(), "jj new parent-a parent-b parent-c");
        assert_eq!(
            plan.preview_summary()
                .lines()
                .filter(|line| line.starts_with("parent: "))
                .collect::<Vec<_>>(),
            vec!["parent: parent-a", "parent: parent-b", "parent: parent-c"]
        );
    }

    #[test]
    fn duplicate_plan_uses_single_exact_change_revset() {
        let duplicate = JjDuplicatePlan::exact_change("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            duplicate.command_argv(),
            vec![
                "duplicate",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
            ]
        );
        assert_eq!(
            duplicate.command_label(),
            "jj duplicate exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
        );

        let preview = duplicate.preview_summary();
        assert!(preview.contains("source revision: tvykuurwpnwzzqulzrvwvmxxotnlywqw"));
        assert!(preview.contains("source count: 1 exact selected change"));
        assert!(preview.contains("multi-source duplicate is intentionally not exposed"));
        assert!(preview.contains("does not parse duplicate output for the new change id"));
        assert!(preview.contains("confirmation: press Enter to run jj duplicate"));
        assert!(preview.contains("recovery: jj undo"));
    }

    #[test]
    fn split_current_working_copy_uses_bare_command() {
        let split = JjSplitPlan::current_working_copy();

        assert_eq!(split.command_argv(), vec!["split"]);
        assert_eq!(split.command_label(), "jj split");
        assert_eq!(split.target().exact_change_id(), None);

        let preview = split.preview_summary();
        assert!(preview.contains("target: current working-copy change (@)"));
        assert!(preview.contains("jj's diff editor"));
        assert!(preview.contains("jk is not an in-app patch editor"));
        assert!(preview.contains("does not choose hunks or lines"));
        assert!(preview.contains("jj op show -p"));
    }

    #[test]
    fn split_exact_change_uses_exact_revision_option() {
        let split = JjSplitPlan::exact_change("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            split.command_argv(),
            vec![
                "split",
                "--revision",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
            ]
        );
        assert_eq!(
            split.command_label(),
            "jj split --revision exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
        );
        assert_eq!(
            split.target().exact_change_id(),
            Some("tvykuurwpnwzzqulzrvwvmxxotnlywqw")
        );
        assert!(
            split
                .preview_summary()
                .contains("target: exact selected graph revision tvykuurwpnwzzqulzrvwvmxxotnlywqw")
        );
    }

    #[test]
    fn split_interactive_command_inherits_stdio_and_keeps_no_pager() {
        let split = JjSplitPlan::exact_change("abc");
        let command = split.interactive_command();

        assert_eq!(command.program(), OsStr::new("jj"));
        assert_eq!(
            command.argv(),
            vec![
                OsStr::new("--no-pager"),
                OsStr::new("split"),
                OsStr::new("--revision"),
                OsStr::new("exactly(change_id(\"abc\"), 1)"),
            ]
        );
        assert_eq!(
            command.stdio_intent(),
            crate::interactive_process::StdioIntent::Inherit
        );
    }

    #[test]
    fn split_result_messages_do_not_claim_captured_stderr() {
        let split = JjSplitPlan::current_working_copy();

        let success = split.success_result_message("exit status: 0");
        assert!(success.contains("child exit status: exit status: 0"));
        assert!(success.contains("live while jk's terminal was suspended"));
        assert!(success.contains("did not capture that output"));

        let failure = split.failure_result_message("jj split failed with status exit status: 1");
        assert!(failure.contains("result: split command failed or did not complete"));
        assert!(failure.contains("runner status: jj split failed with status exit status: 1"));
        assert!(failure.contains("did not capture stderr"));
        assert!(failure.contains("if jj recorded an operation, use jj undo"));
    }

    #[test]
    fn describe_plan_targets_exact_change_before_message() {
        let plan = JjDescribePlan::new(
            JjDescribeTarget::exact_change("abcdefghijklmnopqrstuvwxzyabcdef"),
            "New description",
        );

        assert_eq!(
            plan.command_argv(),
            vec![
                "describe",
                "exactly(change_id(\"abcdefghijklmnopqrstuvwxzyabcdef\"), 1)",
                "--message",
                "New description"
            ]
        );
        assert_eq!(
            plan.command_label(),
            "jj describe abcdefghijklmnopqrstuvwxzyabcdef --message New description"
        );
        assert!(
            plan.preview_summary()
                .contains("target: exact selected revision")
        );
        assert!(plan.preview_summary().contains("without opening an editor"));
    }

    #[test]
    fn describe_plan_can_target_current_working_copy() {
        let plan = JjDescribePlan::new(JjDescribeTarget::current_working_copy(), "Describe @");

        assert_eq!(
            plan.command_argv(),
            vec!["describe", "@", "--message", "Describe @"]
        );
        assert_eq!(plan.command_label(), "jj describe @ --message Describe @");
        assert!(
            plan.preview_summary()
                .contains("current working-copy change (@)")
        );
    }

    #[test]
    fn commit_plan_uses_message_without_revision_argument() {
        let plan = JjCommitPlan::new("Commit working copy");

        assert_eq!(
            plan.command_argv(),
            vec!["commit", "--message", "Commit working copy"]
        );
        assert_eq!(
            plan.command_label(),
            "jj commit --message Commit working copy"
        );
        assert!(
            plan.preview_summary()
                .contains("target: current working-copy change (@)")
        );
        assert!(
            plan.preview_summary()
                .contains("selected graph rows are not arguments")
        );
    }

    #[test]
    fn edit_plan_uses_exact_change_id_revset() {
        let plan = JjWorkingCopyNavigationPlan::edit("change-a");

        assert_eq!(
            plan.command_argv(),
            vec!["edit", "exactly(change_id(\"change-a\"), 1)"]
        );
        assert_eq!(
            plan.command_label(),
            "jj edit exactly(change_id(\"change-a\"), 1)"
        );
        assert_eq!(plan.target_change_id(), Some("change-a"));
        assert!(
            plan.preview_summary()
                .contains("target: exact selected graph revision change-a")
        );
        assert!(
            plan.preview_summary()
                .contains("moves @ to edit that revision directly")
        );
    }

    #[test]
    fn next_plan_uses_explicit_edit_flag_and_ignores_selection() {
        let plan = JjWorkingCopyNavigationPlan::next();

        assert_eq!(plan.command_argv(), vec!["next", "--edit"]);
        assert_eq!(plan.command_label(), "jj next --edit");
        assert_eq!(plan.target_change_id(), None);
        assert!(
            plan.preview_summary()
                .contains("highlighted graph row is not an argument to jj next --edit")
        );
        assert!(
            plan.preview_summary()
                .contains("runs jj topology movement relative to @")
        );
    }

    #[test]
    fn prev_plan_uses_explicit_edit_flag_and_mentions_ambiguity() {
        let plan = JjWorkingCopyNavigationPlan::prev();

        assert_eq!(plan.command_argv(), vec!["prev", "--edit"]);
        assert_eq!(plan.command_label(), "jj prev --edit");
        assert_eq!(plan.target_change_id(), None);
        assert!(
            plan.preview_summary()
                .contains("highlighted graph row is not an argument to jj prev --edit")
        );
        assert!(
            plan.preview_summary()
                .contains("previous editable change is ambiguous or unavailable")
        );
    }

    #[test]
    fn restore_plan_uses_exact_change_revset_for_revision_restore() {
        let restore = JjRestorePlan::for_revision("change-a");

        assert_eq!(
            restore.command_argv(),
            vec![
                "restore",
                "--changes-in",
                "exactly(change_id(\"change-a\"), 1)"
            ]
        );
        assert_eq!(
            restore.preview_diff_argv(),
            vec!["diff", "-r", "exactly(change_id(\"change-a\"), 1)"]
        );
        assert_eq!(
            restore.command_label(),
            "jj restore --changes-in exactly(change_id(\"change-a\"), 1)"
        );

        let preview = restore.preview_summary("M src/main.rs\n");
        assert!(preview.contains("target revision: change-a"));
        assert!(preview.contains("effect: restore removes the selected revision's forward diff"));
        assert!(preview.contains("preview source: jj diff -r exactly(change_id(\"change-a\"), 1)"));
        assert!(preview.contains("jk is not simulating the final graph"));
        assert!(preview.contains("confirmation: press Enter to run jj restore"));
        assert!(preview.contains("undo path: jj undo"));
        assert!(preview.contains("forward diff:\nM src/main.rs"));
    }

    #[test]
    fn restore_plan_uses_root_file_fileset_for_exact_paths() {
        let restore = JjRestorePlan::for_path("change-a", "dir/with spaces/quo\"te\\[glob]?*");

        assert_eq!(
            restore.command_argv(),
            vec![
                "restore",
                "--changes-in",
                "exactly(change_id(\"change-a\"), 1)",
                "root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\""
            ]
        );
        assert_eq!(
            restore.preview_diff_argv(),
            vec![
                "diff",
                "-r",
                "exactly(change_id(\"change-a\"), 1)",
                "root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\""
            ]
        );

        let preview = restore.preview_summary("A dir/with spaces/quo\"te\\[glob]?*\n");
        assert!(preview.contains("selected path: dir/with spaces/quo\"te\\[glob]?*"));
        assert!(
            preview.contains("exact fileset: root-file:\"dir/with spaces/quo\\\"te\\\\[glob]?*\"")
        );
        assert!(preview.contains("effect: restore removes the selected path's forward diff"));
    }

    #[test]
    fn restore_plan_uses_default_working_copy_restore_for_status_paths() {
        let restore = JjRestorePlan::for_working_copy_path("src/status.rs");

        assert_eq!(
            restore.command_argv(),
            vec!["restore", "root-file:\"src/status.rs\""]
        );
        assert_eq!(
            restore.preview_diff_argv(),
            vec!["diff", "root-file:\"src/status.rs\""]
        );
        assert_eq!(
            restore.command_label(),
            "jj restore root-file:\"src/status.rs\""
        );

        let preview = restore.preview_summary("M src/status.rs\n");
        assert!(preview.contains("target revision: @"));
        assert!(preview.contains("selected path: src/status.rs"));
        assert!(preview.contains("working-copy diff from @"));
        assert!(preview.contains("preview source: jj diff root-file:\"src/status.rs\""));
    }

    #[test]
    fn revert_plan_uses_exact_change_revset_and_working_copy_destination() {
        let revert = JjRevertPlan::new("change-a");

        assert_eq!(
            revert.command_argv(),
            vec![
                "revert",
                "-r",
                "exactly(change_id(\"change-a\"), 1)",
                "-o",
                "@"
            ]
        );
        assert_eq!(
            revert.preview_diff_argv(),
            vec!["diff", "-r", "exactly(change_id(\"change-a\"), 1)"]
        );
        assert_eq!(
            revert.command_label(),
            "jj revert -r exactly(change_id(\"change-a\"), 1) -o @"
        );

        let preview = revert.preview_summary("M src/main.rs\n");
        assert!(preview.contains("target revision: change-a"));
        assert!(preview.contains("reverse-applies the selected revision's forward diff into @"));
        assert!(preview.contains("preview source: jj diff -r exactly(change_id(\"change-a\"), 1)"));
        assert!(preview.contains("jk is not simulating the final graph"));
        assert!(preview.contains("confirmation: press Enter to run jj revert"));
        assert!(preview.contains("undo path: jj undo"));
        assert!(preview.contains("forward diff:\nM src/main.rs"));
    }

    #[test]
    fn bookmark_create_and_set_target_exact_changes_or_current_working_copy() {
        let create = JjBookmarkMutationPlan::create(
            "feature/name",
            JjBookmarkTarget::exact_change("abcdefghijklmnopqrstuvwxzyabcdef"),
        );
        assert_eq!(
            create.command_argv(),
            vec![
                "bookmark",
                "create",
                "--revision",
                "exactly(change_id(\"abcdefghijklmnopqrstuvwxzyabcdef\"), 1)",
                "feature/name"
            ]
        );
        assert!(create.preview_summary().contains("exact selected revision"));
        assert!(create.preview_summary().contains("undo path: jj undo"));

        let set =
            JjBookmarkMutationPlan::set("feature/name", JjBookmarkTarget::current_working_copy());
        assert_eq!(
            set.command_argv(),
            vec!["bookmark", "set", "--revision", "@", "feature/name"]
        );
        assert!(
            set.preview_summary()
                .contains("current working-copy change (@)")
        );
    }

    #[test]
    fn bookmark_move_and_delete_use_exact_string_patterns() {
        let move_plan = JjBookmarkMutationPlan::move_to(
            "feature/\"quote\\tab",
            JjBookmarkTarget::exact_change("abcdefghijklmnopqrstuvwxzyabcdef"),
        );

        assert_eq!(
            move_plan.command_argv(),
            vec![
                "bookmark",
                "move",
                "--to",
                "exactly(change_id(\"abcdefghijklmnopqrstuvwxzyabcdef\"), 1)",
                "exact:\"feature/\\\"quote\\\\tab\""
            ]
        );
        assert!(
            move_plan
                .command_label()
                .contains("exact:\"feature/\\\"quote\\\\tab\"")
        );

        let delete = JjBookmarkMutationPlan::delete("feature/name");
        assert_eq!(
            delete.command_argv(),
            vec!["bookmark", "delete", "exact:\"feature/name\""]
        );
        assert!(delete.preview_summary().contains("delete, not forget"));
        assert!(
            delete
                .preview_summary()
                .contains("track/untrack stay disabled")
        );
    }

    #[test]
    fn bookmark_forget_uses_exact_local_or_include_remote_patterns() {
        let local = JjBookmarkMutationPlan::forget(
            "feature/name",
            JjBookmarkForgetTarget::local("tracked local bookmark"),
        );

        assert_eq!(
            local.command_argv(),
            vec!["bookmark", "forget", "exact:\"feature/name\""]
        );
        assert!(local.preview_summary().contains("tracked local bookmark"));
        assert!(local.preview_summary().contains("forget, not delete"));

        let remote_only = JjBookmarkMutationPlan::forget(
            "feature/name",
            JjBookmarkForgetTarget::remote_only("origin", "untracked remote bookmark"),
        );

        assert_eq!(
            remote_only.command_argv(),
            vec![
                "bookmark",
                "forget",
                "--include-remotes",
                "exact:\"feature/name\""
            ]
        );
        assert!(
            remote_only
                .preview_summary()
                .contains("remote-only bookmark on origin")
        );
    }

    #[test]
    fn bookmark_forget_exact_pattern_quotes_special_characters() {
        let forget = JjBookmarkMutationPlan::forget(
            "feature/\"quote\\tab",
            JjBookmarkForgetTarget::local("tracked local bookmark"),
        );

        assert_eq!(
            forget.command_argv(),
            vec!["bookmark", "forget", "exact:\"feature/\\\"quote\\\\tab\""]
        );
        assert!(
            forget
                .command_label()
                .contains("exact:\"feature/\\\"quote\\\\tab\"")
        );
    }

    #[test]
    fn bookmark_track_and_untrack_are_exact_remote_scoped() {
        let target = JjBookmarkTrackingTarget::local(
            "feature/name",
            "feature/name",
            "origin",
            "local bookmark with one remote peer",
        );
        let track = JjBookmarkMutationPlan::track("feature/name", target.clone());

        assert_eq!(
            track.command_argv(),
            vec![
                "bookmark",
                "track",
                "--remote",
                "exact:\"origin\"",
                "exact:\"feature/name\"",
            ]
        );
        assert_eq!(
            track.command_label(),
            "jj bookmark track --remote exact:\"origin\" exact:\"feature/name\""
        );
        let preview = track.preview_summary();
        assert!(preview.contains("local bookmark: feature/name"));
        assert!(preview.contains("remote bookmark: feature/name"));
        assert!(preview.contains("remote: origin"));
        assert!(preview.contains("confirmation: press Enter to run jj bookmark track"));
        assert!(preview.contains("recovery: jj undo; review: jj op show -p"));

        let untrack = JjBookmarkMutationPlan::untrack("feature/name", target);
        assert_eq!(
            untrack.command_argv(),
            vec![
                "bookmark",
                "untrack",
                "--remote",
                "exact:\"origin\"",
                "exact:\"feature/name\"",
            ]
        );
        assert!(
            untrack
                .preview_summary()
                .contains("does not delete the local or remote bookmark")
        );
    }

    #[test]
    fn bookmark_track_quotes_remote_and_bookmark_patterns() {
        let target = JjBookmarkTrackingTarget::remote_only(
            "feature/\"quote\\tab",
            "origin/\"remote",
            "remote-only bookmark",
        );
        let track = JjBookmarkMutationPlan::track("feature/\"quote\\tab", target);

        assert_eq!(
            track.command_argv(),
            vec![
                "bookmark",
                "track",
                "--remote",
                "exact:\"origin/\\\"remote\"",
                "exact:\"feature/\\\"quote\\\\tab\"",
            ]
        );
    }

    #[test]
    fn bookmark_rename_uses_old_and_new_names_as_argv() {
        let rename = JjBookmarkMutationPlan::rename("feature/\"old name\"", "feature/new'special");

        assert_eq!(
            rename.command_argv(),
            vec![
                "bookmark",
                "rename",
                "feature/\"old name\"",
                "feature/new'special"
            ]
        );
        assert_eq!(
            rename.command_label(),
            "jj bookmark rename feature/\"old name\" feature/new'special"
        );
        let preview = rename.preview_summary();
        assert!(preview.contains("old name: feature/\"old name\""));
        assert!(preview.contains("new name: feature/new'special"));
        assert!(preview.contains("without --overwrite-existing"));
        assert!(preview.contains("confirmation: press Enter to run jj bookmark rename"));
    }

    #[test]
    fn bookmark_rename_new_name_validation_rejects_obvious_invalid_inputs() {
        assert_eq!(
            validate_bookmark_rename_new_name("feature/name", "").unwrap_err(),
            "empty bookmark name"
        );
        assert_eq!(
            validate_bookmark_rename_new_name("feature/name", "feature/name").unwrap_err(),
            "new bookmark name is unchanged"
        );
        assert_eq!(
            validate_bookmark_rename_new_name("feature/name", "bad name").unwrap_err(),
            "bookmark name must not contain whitespace or control characters"
        );
        assert_eq!(
            validate_bookmark_rename_new_name("feature/name", " feature/renamed ").unwrap_err(),
            "bookmark name must not contain whitespace or control characters"
        );
        assert_eq!(
            validate_bookmark_rename_new_name("feature/name", "feature@origin").unwrap_err(),
            "bookmark name contains a reserved ref character"
        );
        assert_eq!(
            validate_bookmark_rename_new_name("feature/name", "feature//name").unwrap_err(),
            "bookmark name must not contain empty path components"
        );
        assert!(validate_bookmark_rename_new_name("feature/name", "feature/renamed").is_ok());
    }

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

    #[test]
    fn rebase_command_args_use_explicit_sources_and_destination() {
        let rebase = JjRebasePlan::new(
            vec!["source-a".to_owned(), "source-b".to_owned()],
            "dest".to_owned(),
        );

        assert_eq!(
            rebase.command_argv(false),
            vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
        );
        assert_eq!(
            rebase.command_label(false),
            "jj rebase -r source-a -r source-b -o dest"
        );
    }

    #[test]
    fn rebase_preview_summary_includes_command_effect_and_undo_path() {
        let rebase = JjRebasePlan::new(vec!["source-a".to_owned()], "dest".to_owned());

        let preview = rebase.preview_summary();

        assert!(preview.contains("command: jj rebase -r source-a -o dest"));
        assert!(preview.contains("source revision: source-a"));
        assert!(preview.contains("destination revision: dest"));
        assert!(preview.contains("source rows are selected in jk"));
        assert!(preview.contains("destination is the current row"));
        assert!(preview.contains("semantics: jj rebase --revision <source> --onto <destination>"));
        assert!(preview.contains("only listed source revisions are rebased"));
        assert!(preview.contains("dependencies among listed sources are preserved"));
        assert!(preview.contains("descendants outside the selected set may be rebased"));
        assert!(preview.contains("destination descendants are not inserted or rebased by -o"));
        assert!(preview.contains("not a graph preview"));
        assert!(preview.contains("jk has not run jj and is not simulating the final graph"));
        assert!(preview.contains("review after run: jj op show -p"));
        assert!(preview.contains("undo path: jj undo"));
        assert!(preview.contains("confirmation: press Enter to run jj rebase"));
    }

    #[test]
    fn squash_command_args_use_explicit_sources_destination_and_message_policy() {
        let squash = JjSquashPlan::new(
            vec!["source-a".to_owned(), "source-b".to_owned()],
            "dest".to_owned(),
        );

        assert_eq!(
            squash.command_argv(false),
            vec![
                "squash",
                "--from",
                "source-a",
                "--from",
                "source-b",
                "--into",
                "dest",
                "--use-destination-message"
            ]
        );
        assert_eq!(
            squash.command_label(false),
            "jj squash --from source-a --from source-b --into dest --use-destination-message"
        );
    }

    #[test]
    fn squash_preview_summary_includes_roles_effect_message_policy_and_undo_path() {
        let squash = JjSquashPlan::new(vec!["source-a".to_owned()], "dest".to_owned());

        let preview = squash.preview_summary();

        assert!(
            preview.contains(
                "command: jj squash --from source-a --into dest --use-destination-message"
            )
        );
        assert!(preview.contains("source: source-a"));
        assert!(preview.contains("destination: dest"));
        assert!(preview.contains("graph effect: moves the selected source changes"));
        assert!(preview.contains("--use-destination-message keeps the destination description"));
        assert!(preview.contains("confirmation: press Enter to run jj squash"));
        assert!(preview.contains("undo path: jj undo"));
    }

    #[test]
    fn absorb_command_args_use_exact_source_and_repeated_candidate_destinations() {
        let absorb = JjAbsorbPlan::new(
            "source-change",
            vec!["dest-a".to_owned(), "dest-b".to_owned()],
        );

        assert_eq!(
            absorb.command_argv(),
            vec![
                "absorb",
                "--from",
                "exactly(change_id(\"source-change\"), 1)",
                "--into",
                "exactly(change_id(\"dest-a\"), 1)",
                "--into",
                "exactly(change_id(\"dest-b\"), 1)",
            ]
        );
        assert_eq!(
            absorb.command_label(),
            "jj absorb --from exactly(change_id(\"source-change\"), 1) --into exactly(change_id(\"dest-a\"), 1) --into exactly(change_id(\"dest-b\"), 1)"
        );
    }

    #[test]
    fn absorb_preview_summary_names_bounded_opacity_and_recovery_paths() {
        let absorb = JjAbsorbPlan::new("source-change", vec!["dest-a".to_owned()]);

        let preview = absorb.preview_summary();

        assert!(preview.contains("source: source-change"));
        assert!(preview.contains("candidate destination: dest-a"));
        assert!(preview.contains("selected revisions are candidate destinations"));
        assert!(preview.contains("only considers selected revisions that are ancestors"));
        assert!(preview.contains("jk does not simulate line-level placement"));
        assert!(preview.contains("changes remain in the source"));
        assert!(preview.contains("source may become empty or abandoned"));
        assert!(preview.contains("recovery: jj undo"));
        assert!(preview.contains("review: jj op show -p"));
    }

    #[test]
    fn abandon_plan_uses_exact_revision_command_shape() {
        let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            abandon.command_argv(),
            vec![
                "abandon",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)"
            ]
        );
        assert_eq!(
            abandon.command_label(),
            "jj abandon tvykuurwpnwzzqulzrvwvmxxotnlywqw"
        );
    }

    #[test]
    fn abandon_diff_summary_probe_uses_revision_summary() {
        let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            abandon.diff_summary_argv(),
            vec![
                "diff",
                "-r",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)",
                "--summary"
            ]
        );
        assert_eq!(
            abandon.diff_summary_label(),
            "jj diff -r tvykuurwpnwzzqulzrvwvmxxotnlywqw --summary"
        );
    }

    #[test]
    fn abandon_title_probe_uses_exact_change_id_revset() {
        let abandon = JjAbandonPlan::new("tvykuurwpnwzzqulzrvwvmxxotnlywqw");

        assert_eq!(
            abandon.title_argv(),
            vec![
                "log",
                "-r",
                "exactly(change_id(\"tvykuurwpnwzzqulzrvwvmxxotnlywqw\"), 1)",
                "--no-graph",
                "-T",
                DESCRIPTION_FIRST_LINE_TEMPLATE,
            ]
        );
    }

    #[test]
    fn file_track_uses_root_file_fileset_after_double_dash() {
        let plan = JjFileMutationPlan::track("-leading dir/quo\"te\\[glob]?*.rs");

        assert_eq!(
            plan.command_argv(),
            vec![
                "file",
                "track",
                "--",
                "root-file:\"-leading dir/quo\\\"te\\\\[glob]?*.rs\""
            ]
        );
        let preview = plan.preview_summary();
        assert!(preview.contains("selected path: -leading dir/quo\"te\\[glob]?*.rs"));
        assert!(
            preview.contains("exact fileset: root-file:\"-leading dir/quo\\\"te\\\\[glob]?*.rs\"")
        );
        assert!(preview.contains("effect: starts tracking this exact untracked working-copy path"));
        assert!(preview.contains("output: jj stdout and stderr are preserved"));
        assert!(preview.contains("recovery: jj undo"));
        assert!(preview.contains("review: jj status; jj op show -p"));
    }

    #[test]
    fn file_untrack_uses_root_file_fileset_and_mentions_ignore_requirement() {
        let plan = JjFileMutationPlan::untrack("dir/file.rs");

        assert_eq!(
            plan.command_argv(),
            vec!["file", "untrack", "--", "root-file:\"dir/file.rs\""]
        );
        assert!(
            plan.preview_summary()
                .contains("jj requires the path to already be ignored")
        );
    }

    #[test]
    fn file_chmod_modes_use_installed_jj_mode_args() {
        let executable =
            JjFileMutationPlan::chmod_working_copy("src/main.rs", JjFileChmodMode::Executable);
        let normal = JjFileMutationPlan::chmod_working_copy("src/main.rs", JjFileChmodMode::Normal);

        assert_eq!(
            executable.command_argv(),
            vec!["file", "chmod", "x", "--", "root-file:\"src/main.rs\""]
        );
        assert_eq!(
            normal.command_argv(),
            vec!["file", "chmod", "n", "--", "root-file:\"src/main.rs\""]
        );
        assert!(
            executable
                .preview_summary()
                .contains("chmod mode: x (executable)")
        );
        assert!(normal.preview_summary().contains("chmod mode: n (normal)"));
    }

    #[test]
    fn exact_revision_file_chmod_uses_exact_change_revset_before_mode_and_fileset() {
        let plan = JjFileMutationPlan::chmod_exact_revision(
            "change-a",
            "dir/space file.rs",
            JjFileChmodMode::Executable,
        );

        assert_eq!(
            plan.command_argv(),
            vec![
                "file",
                "chmod",
                "-r",
                "exactly(change_id(\"change-a\"), 1)",
                "x",
                "--",
                "root-file:\"dir/space file.rs\""
            ]
        );
        assert_eq!(
            plan.command_label(),
            "jj file chmod -r exactly(change_id(\"change-a\"), 1) x -- root-file:\"dir/space file.rs\""
        );
        assert!(
            plan.preview_summary()
                .contains("target scope: exact selected revision change-a")
        );
    }

    #[test]
    fn abandon_preview_classifies_empty_summary_as_empty_change() {
        let preview = JjAbandonPreview::new(
            "change-id".to_owned(),
            Some("Start feature".to_owned()),
            "\n".to_owned(),
        );

        assert!(preview.is_empty_change());
        assert_eq!(preview.revision(), "change-id");
        assert!(preview.preview_text().contains("title: Start feature"));
        assert!(
            preview
                .preview_text()
                .contains("diff summary:\nempty change")
        );
        assert!(
            preview
                .preview_text()
                .contains("press Enter to abandon this empty change")
        );
        assert!(preview.preview_text().contains("undo path: jj undo"));
    }

    #[test]
    fn abandon_preview_classifies_non_empty_summary_as_strong_confirm() {
        let preview = JjAbandonPreview::new(
            "change-id".to_owned(),
            Some("Edit files".to_owned()),
            "M src/main.rs\nA README.md\n".to_owned(),
        );

        assert!(!preview.is_empty_change());
        let text = preview.preview_text();
        assert!(text.contains("change: change-id"));
        assert!(text.contains("title: Edit files"));
        assert!(text.contains("M src/main.rs\nA README.md"));
        assert!(text.contains("type exact revision 'change-id' before abandon runs"));
        assert!(text.contains("undo path: jj undo"));
    }

    #[test]
    fn rebase_plan_argv_includes_repeated_sources_and_destination() {
        let rebase = JjRebasePlan::new(
            vec![
                "source-a".to_owned(),
                "source-b".to_owned(),
                "source-c".to_owned(),
            ],
            "dest".to_owned(),
        );

        assert_eq!(
            rebase.command_argv(false),
            vec![
                "rebase", "-r", "source-a", "-r", "source-b", "-r", "source-c", "-o", "dest"
            ]
        );
    }

    #[test]
    fn rebase_plan_argv_and_label_do_not_change_for_preview_flag() {
        let rebase = JjRebasePlan::new(vec!["source-a".to_owned(), "source-b".to_owned()], "dest");

        assert_eq!(
            rebase.command_argv(true),
            vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
        );
        assert_eq!(
            rebase.command_label(false),
            "jj rebase -r source-a -r source-b -o dest"
        );
        assert_eq!(
            rebase.command_label(true),
            "jj rebase -r source-a -r source-b -o dest"
        );
        assert_eq!(
            rebase.command_argv(false),
            vec!["rebase", "-r", "source-a", "-r", "source-b", "-o", "dest"]
        );
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }
}
