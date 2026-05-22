//! General action completion and result-pane handling.
//!
//! This module owns post-command refresh, reveal, and result wording for non-rewrite-specific
//! mutation flows such as describe, commit, bookmarks, sync, duplicate, split, and operation
//! recovery. It calls `AppServices` directly for the completed command output and keeps the
//! app-owned refresh/reveal/status policy local to this file.

use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::action_pane::ActionPane;
use crate::actions::{
    JjBookmarkMutationPlan, JjCommitPlan, JjDescribePlan, JjDuplicatePlan, JjFileMutationPlan,
    JjGitFetch, JjGitPush, JjNewPlan, JjOperationRecovery, JjOperationTarget, JjSplitPlan,
    JjSplitTarget,
};
use crate::jj::{JjCommand, LogViewMode};
use crate::modes::InteractionMode;
use crate::status_line::StatusLine;
use crate::view_state::ViewState;

use super::super::{App, current_viewport_width};
use super::shared::fetch_status_message;

impl App {
    /// Run the describe command and leave its finished output on the describe pane.
    pub(in crate::app) fn confirm_describe(
        &mut self,
        describe: JjDescribePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = describe.command_label();
        let reveal_change_id = describe.target().exact_change_id().map(str::to_owned);
        let result_message = match self.services.run_describe(&describe) {
            Ok(output) => self.finish_successful_action_revealing_change(
                output,
                reveal_change_id.as_deref(),
                viewport_height,
                " | jj undo",
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::DescribePreview {
            describe,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the commit command and leave its finished output on the commit pane.
    pub(in crate::app) fn confirm_commit(
        &mut self,
        commit: JjCommitPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = commit.command_label();
        let result_message = match self.services.run_commit(&commit) {
            Ok(output) => self.finish_successful_action(
                output,
                viewport_height,
                " | new working-copy change created on top | jj undo",
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::CommitPreview {
            commit,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the bookmark mutation and leave its finished output on the bookmark pane.
    pub(in crate::app) fn confirm_bookmark_mutation(
        &mut self,
        mutation: JjBookmarkMutationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = mutation.command_label();
        let result_message = match self.services.run_bookmark_mutation(&mutation) {
            Ok(output) => self.finish_successful_action(output, viewport_height, " | jj undo"),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::BookmarkMutationPreview {
            mutation,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the file mutation and leave its finished output on the file-mutation pane.
    pub(in crate::app) fn confirm_file_mutation(
        &mut self,
        mutation: JjFileMutationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = mutation.command_label();
        let result_message = match self.services.run_file_mutation(&mutation) {
            Ok(output) => {
                self.finish_successful_action(output, viewport_height, " | jj undo | jj op show -p")
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::FileMutationPreview {
            mutation,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the new-change command, resolve the new working copy, and leave the result on the pane.
    pub(in crate::app) fn confirm_new_change(
        &mut self,
        new_change: JjNewPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = new_change.command_label();
        let result_message = match self.services.run_new_change(&new_change) {
            Ok(output) => {
                let new_change_id = match self.services.resolve_revision("@") {
                    Ok(change_id) => change_id,
                    Err(error) => {
                        let message =
                            format!("{} | resolve @ failed: {error} | jj undo", output.trim());
                        self.status = StatusLine::error(&self.view, error.to_string());
                        self.mode = InteractionMode::NewPreview {
                            new_change,
                            output: ActionPane::finished(command_label, message, status_context),
                        };
                        return;
                    }
                };

                self.finish_successful_action_revealing_change(
                    output,
                    Some(new_change_id.as_str()),
                    viewport_height,
                    " | jj undo",
                )
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::NewPreview {
            new_change,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the duplicate command and leave the refresh/reveal result on the duplicate pane.
    pub(in crate::app) fn confirm_duplicate(
        &mut self,
        duplicate: JjDuplicatePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = duplicate.command_label();
        let result_message = match self.services.run_duplicate(&duplicate) {
            Ok(output) => self.finish_successful_duplicate(&duplicate, output, viewport_height),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::DuplicatePreview {
            duplicate,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Refresh after duplicate and fall back to revealing the original source change when possible.
    fn finish_successful_duplicate(
        &mut self,
        duplicate: &JjDuplicatePlan,
        output: String,
        viewport_height: u16,
    ) -> String {
        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                let reveal_result = match &self.view {
                    ViewState::Log(_) => {
                        Some(self.reveal_log_change(duplicate.source(), LogViewMode::Recent))
                    }
                    _ => None,
                };

                match reveal_result {
                    Some(Ok(switched_modes)) => {
                        self.view.clamp(viewport_height, current_viewport_width());
                        let message = if switched_modes {
                            "duplicate completed | showing recent work fallback for source | jj undo | jj op show -p"
                        } else {
                            "duplicate completed | source selected as fallback | jj undo | jj op show -p"
                        };
                        self.status = StatusLine::with_message(&self.view, message);
                        format!(
                            "{}\nrefresh: active view refreshed\nreveal: selected original source {} because jk does not parse duplicate output for the new change id\nrecovery: jj undo\nreview: jj op show -p",
                            output.trim(),
                            duplicate.source()
                        )
                    }
                    Some(Err(error)) => {
                        self.status = StatusLine::error(&self.view, error.to_string());
                        format!(
                            "{}\nrefresh: active view refreshed\nreveal: source fallback failed: {error}\nrecovery: jj undo\nreview: jj op show -p",
                            output.trim()
                        )
                    }
                    None => {
                        let message = "duplicate completed | active view refreshed | source reveal unavailable | jj undo | jj op show -p";
                        self.status = StatusLine::with_message(&self.view, message);
                        format!(
                            "{}\nrefresh: active view refreshed\nreveal: source fallback not attempted because the active view cannot reveal log changes\nrecovery: jj undo\nreview: jj op show -p",
                            output.trim()
                        )
                    }
                }
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                format!(
                    "{}\nrefresh: failed: {error}\nrecovery: jj undo\nreview: jj op show -p",
                    output.trim()
                )
            }
        }
    }

    /// Run the push command and leave its finished output on the push pane.
    pub(in crate::app) fn confirm_push(
        &mut self,
        push: JjGitPush,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = push.command_label(false);
        let result_message = match self.services.run_push(&push) {
            Ok(output) => {
                self.finish_successful_sync_action(output, viewport_height, str::to_owned)
            }
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::PushPreview {
            push,
            output: ActionPane::finished(command_label, result_message, status_context),
        }
    }

    /// Run the fetch command and leave its finished output on the fetch pane.
    pub(in crate::app) fn confirm_fetch(
        &mut self,
        fetch: JjGitFetch,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = fetch.command_label();
        let result_message = match self.services.run_git_fetch(&fetch) {
            Ok(output) => self.finish_successful_sync_action(output, viewport_height, |output| {
                fetch_status_message(&fetch, output)
            }),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::FetchPreview {
            fetch,
            output: ActionPane::finished(command_label, result_message, status_context),
        }
    }

    /// Run the undo/redo operation and leave its finished output on the recovery pane.
    pub(in crate::app) fn confirm_operation_recovery(
        &mut self,
        recovery: JjOperationRecovery,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = recovery.command_label().to_owned();
        let result_message = match self.services.run_operation_recovery(&recovery) {
            Ok(output) => self.finish_successful_action(
                output,
                viewport_height,
                &format!(" | {}", recovery.success_hint()),
            ),
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::OperationRecoveryPreview {
            recovery,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Run the split command, including inherited-stdio interactive mode when needed.
    pub(in crate::app) fn confirm_split(
        &mut self,
        split: JjSplitPlan,
        status_context: Option<String>,
        viewport_height: u16,
        terminal: Option<&mut DefaultTerminal>,
    ) {
        let command_label = split.command_label();
        let result_message = match self.services.run_split(terminal, &split) {
            Ok(output) => self.finish_successful_split(&split, output, viewport_height),
            Err(error) => {
                let message = split.failure_result_message(&error.to_string());
                self.status = StatusLine::error(&self.view, message.clone());
                message
            }
        };

        self.mode = InteractionMode::SplitPreview {
            split,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Refresh after split and reveal the exact or current working-copy target when possible.
    fn finish_successful_split(
        &mut self,
        split: &JjSplitPlan,
        output: String,
        viewport_height: u16,
    ) -> String {
        let reveal_change_id = match split.target() {
            JjSplitTarget::ExactChange(change_id) => Some(change_id.clone()),
            JjSplitTarget::CurrentWorkingCopy => match self.services.resolve_revision("@") {
                Ok(change_id) => Some(change_id),
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    return format!(
                        "{output}\nrefresh: skipped because resolving @ failed: {error}\nrecovery: jj undo\nreview: jj op show -p"
                    );
                }
            },
        };

        match self.refresh_view_state() {
            Ok(()) => {
                self.view.clamp(viewport_height, current_viewport_width());
                let mut reveal_error = None;
                let revealed_in_recent = match reveal_change_id.as_deref() {
                    Some(change_id) => {
                        match self.reveal_log_change(change_id, LogViewMode::Recent) {
                            Ok(switched_modes) => {
                                self.view.clamp(viewport_height, current_viewport_width());
                                Some(switched_modes)
                            }
                            Err(error) => {
                                self.status = StatusLine::error(&self.view, error.to_string());
                                reveal_error = Some(format!(
                                    "{output}\nrefresh: active view refreshed\nreveal: failed: {error}\nrecovery: jj undo\nreview: jj op show -p"
                                ));
                                None
                            }
                        }
                    }
                    None => None,
                };

                let message = match revealed_in_recent {
                    Some(true) => "split completed | showing recent work | jj undo | jj op show -p",
                    Some(false) => "split completed | jj undo | jj op show -p",
                    None => match reveal_error.as_deref() {
                        Some(message) => return message.to_owned(),
                        None => "split completed | jj undo | jj op show -p",
                    },
                };
                self.status = StatusLine::with_message(&self.view, message);
                format!("{output}\nrefresh: {message}")
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                format!(
                    "{output}\nrefresh: failed: {error}\nrecovery: jj undo\nreview: jj op show -p"
                )
            }
        }
    }

    /// Run the operation restore/revert command and refresh current plus stacked repo views.
    pub(in crate::app) fn confirm_operation_target(
        &mut self,
        target: JjOperationTarget,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = target.command_label();
        let result_message = match self.services.run_operation_target(&target) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    match self.refresh_stacked_repo_views(viewport_height) {
                        Ok(()) => {
                            let message = format!("{} | jj undo", output.trim());
                            self.status = StatusLine::with_message(&self.view, message.as_str());
                            message
                        }
                        Err(error) => {
                            self.status = StatusLine::error(&self.view, error.to_string());
                            format!(
                                "{} | stacked view refresh failed: {error} | jj undo",
                                output.trim()
                            )
                        }
                    }
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!("{} | refresh failed: {error} | jj undo", output.trim())
                }
            },
            Err(error) => self.finish_failed_action(error),
        };

        self.mode = InteractionMode::OperationTargetPreview {
            target,
            output: ActionPane::finished(command_label, result_message, status_context),
        };
    }

    /// Refresh any repo-backed views stored on the app stack after an operation-level mutation.
    pub(in crate::app) fn refresh_stacked_repo_views(
        &mut self,
        viewport_height: u16,
    ) -> Result<()> {
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
