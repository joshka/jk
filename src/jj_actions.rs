//! Preview-first action and mutation plans for `jj` commands.
//!
//! These value types own argv construction, labels, preview summaries, exact
//! revset/fileset quoting, and direct execution for user-confirmed mutation
//! flows. Rendered row loading and view-spec command construction stay in
//! [`crate::jj`].

use color_eyre::Result;
use color_eyre::eyre::eyre;

use crate::jj::{
    ColorMode, base_command, run_direct_args, run_direct_args_stdout, summarize_output,
};

const DESCRIPTION_FIRST_LINE_TEMPLATE: &str = "description.first_line() ++ \"\\n\"";

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

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjGitPushTarget {
    Bookmark(String),
    Revision(String),
    Status,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjGitPush {
    target: JjGitPushTarget,
    remote: Option<String>,
}

#[allow(dead_code)]
impl JjGitPush {
    pub fn for_bookmark(name: String) -> Self {
        Self {
            target: JjGitPushTarget::Bookmark(name),
            remote: None,
        }
    }

    pub fn for_revision(revset: String) -> Self {
        Self {
            target: JjGitPushTarget::Revision(revset),
            remote: None,
        }
    }

    pub fn for_status() -> Self {
        Self {
            target: JjGitPushTarget::Status,
            remote: None,
        }
    }

    pub fn with_remote(mut self, remote: impl Into<String>) -> Self {
        self.remote = Some(remote.into());
        self
    }

    pub fn remote(&self) -> Option<&str> {
        self.remote.as_deref()
    }

    pub fn command_label(&self, dry_run: bool) -> String {
        let label_args = self
            .command_argv(dry_run)
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self, dry_run: bool) -> Vec<String> {
        let mut argv = vec!["git".to_owned(), "push".to_owned()];

        if dry_run {
            argv.push("--dry-run".to_owned());
        }
        if let Some(remote) = &self.remote {
            argv.push("--remote".to_owned());
            argv.push(remote.clone());
        }

        match &self.target {
            JjGitPushTarget::Bookmark(name) => {
                argv.push("--bookmark".to_owned());
                argv.push(name.clone());
            }
            JjGitPushTarget::Revision(revset) => {
                argv.push("--revision".to_owned());
                argv.push(revset.clone());
            }
            JjGitPushTarget::Status => {}
        }

        argv
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(true),
            &self.command_label(true),
            "preview complete",
        )
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(false),
            &self.command_label(false),
            "pushed",
        )
    }
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjNewPlan {
    parents: Vec<String>,
}

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRestorePlan {
    revision: String,
    path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjRevertPlan {
    revision: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjBookmarkMutationKind {
    Create,
    Set,
    Move,
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjBookmarkTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjBookmarkMutationPlan {
    kind: JjBookmarkMutationKind,
    name: String,
    target: Option<JjBookmarkTarget>,
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

impl JjRestorePlan {
    pub fn for_revision(revision: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
            path: None,
        }
    }

    pub fn for_path(revision: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            revision: revision.into(),
            path: Some(path.into()),
        }
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn command_label(&self) -> String {
        let label_args = self.command_argv().join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        let mut argv = vec![
            "restore".to_owned(),
            "--changes-in".to_owned(),
            exact_change_id_revset(&self.revision),
        ];
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
        let mut argv = vec![
            "diff".to_owned(),
            "-r".to_owned(),
            exact_change_id_revset(&self.revision),
        ];
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
            format!("target revision: {}", self.revision),
            format!("command: {}", self.command_label()),
        ];

        match &self.path {
            Some(path) => {
                lines.push(format!("selected path: {path}"));
                lines.push(format!("exact fileset: {}", root_file_fileset(path)));
                lines.push(
                    "effect: restore removes the selected path's forward diff from that exact revision"
                        .to_owned(),
                );
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

impl JjBookmarkMutationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Set => "set",
            Self::Move => "move",
            Self::Delete => "delete",
        }
    }

    fn success_fallback(self) -> &'static str {
        match self {
            Self::Create => "created bookmark",
            Self::Set => "set bookmark",
            Self::Move => "moved bookmark",
            Self::Delete => "deleted bookmark",
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
            target: Some(target),
        }
    }

    pub fn set(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Set,
            name: name.into(),
            target: Some(target),
        }
    }

    pub fn move_to(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Move,
            name: name.into(),
            target: Some(target),
        }
    }

    pub fn delete(name: impl Into<String>) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Delete,
            name: name.into(),
            target: None,
        }
    }

    pub fn kind(&self) -> JjBookmarkMutationKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
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
            JjBookmarkMutationKind::Delete => vec![
                "bookmark".to_owned(),
                "delete".to_owned(),
                exact_string_pattern(&self.name),
            ],
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
        }

        lines.join("\n")
    }

    fn required_target(&self) -> &JjBookmarkTarget {
        self.target
            .as_ref()
            .expect("bookmark mutation kind requires target")
    }
}

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

fn exact_change_id_revset(change_id: &str) -> String {
    format!(
        "exactly(change_id({}), 1)",
        revset_string_literal(change_id)
    )
}

fn root_file_fileset(path: &str) -> String {
    format!("root-file:{}", revset_string_literal(path))
}

fn revset_string_literal(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

fn exact_string_pattern(value: &str) -> String {
    format!("exact:{}", revset_string_literal(value))
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
    fn exact_change_id_revset_quotes_literal_prefix() {
        assert_eq!(
            exact_change_id_revset("abc\"\\"),
            "exactly(change_id(\"abc\\\"\\\\\"), 1)"
        );
    }

    #[test]
    fn root_file_fileset_quotes_spaces_quotes_backslashes_and_metacharacters() {
        assert_eq!(
            root_file_fileset("a b/\"c\"/d\\e[f]{g}(h)|i?*"),
            "root-file:\"a b/\\\"c\\\"/d\\\\e[f]{g}(h)|i?*\""
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
    fn git_push_bookmark_args_include_dry_run_when_previewing() {
        let push = JjGitPush::for_bookmark("main".to_owned()).with_remote("origin".to_owned());

        assert_eq!(
            push.command_argv(true),
            vec![
                "git",
                "push",
                "--dry-run",
                "--remote",
                "origin",
                "--bookmark",
                "main"
            ]
        );
        assert_eq!(
            push.command_label(false),
            "jj git push --remote origin --bookmark main"
        );
        assert_eq!(
            push.command_label(true),
            "jj git push --dry-run --remote origin --bookmark main"
        );
        assert_eq!(
            push.command_argv(false),
            vec!["git", "push", "--remote", "origin", "--bookmark", "main"]
        );
    }

    #[test]
    fn git_push_revision_args_follow_revision_target() {
        let push = JjGitPush::for_revision("main".to_owned()).with_remote("origin".to_owned());

        assert_eq!(
            push.command_argv(true),
            vec![
                "git",
                "push",
                "--dry-run",
                "--remote",
                "origin",
                "--revision",
                "main"
            ]
        );
    }

    #[test]
    fn git_push_revision_can_use_jj_default_remote_resolution() {
        let push = JjGitPush::for_revision("main".to_owned());

        assert_eq!(
            push.command_argv(false),
            vec!["git", "push", "--revision", "main"]
        );
        assert_eq!(
            push.command_label(true),
            "jj git push --dry-run --revision main"
        );
    }

    #[test]
    fn git_push_status_default_uses_remote_only_target() {
        let push = JjGitPush::for_status().with_remote("origin".to_owned());

        assert_eq!(
            push.command_argv(false),
            vec!["git", "push", "--remote", "origin"]
        );
        assert_eq!(
            push.command_label(true),
            "jj git push --dry-run --remote origin"
        );
    }

    #[test]
    fn git_push_bookmark_can_use_jj_default_remote_resolution() {
        let push = JjGitPush::for_bookmark("main".to_owned());

        assert_eq!(
            push.command_argv(true),
            vec!["git", "push", "--dry-run", "--bookmark", "main"]
        );
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

    #[test]
    fn git_push_keeps_status_target_with_no_remote_optional() {
        assert_eq!(
            JjGitPush::for_status().command_argv(true),
            vec!["git", "push", "--dry-run"]
        );
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }
}
