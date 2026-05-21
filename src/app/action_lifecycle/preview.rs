//! Preview opening and immediate action execution for app-owned action flows.
//!
//! This module builds the preview/result panes before a command is confirmed. It owns preview
//! status context wording, while `jj_actions` remains the owner of argv construction.

use color_eyre::Result;

use crate::action_output::ActionOutput;
use crate::app_screen::InteractionMode;
use crate::app_status::StatusLine;
use crate::jj::LogViewMode;
use crate::jj_actions::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationPlan, JjCommitPlan,
    JjDescribePlan, JjDuplicatePlan, JjFileMutationPlan, JjGitFetch, JjGitPush, JjGitPushTarget,
    JjNewPlan, JjOperationRecovery, JjOperationRecoveryKind, JjOperationTarget, JjRebasePlan,
    JjRestorePlan, JjRevertPlan, JjSplitPlan, JjSquashPlan, JjWorkingCopyNavigationKind,
    JjWorkingCopyNavigationPlan,
};

use super::super::{App, current_viewport_width};
use super::shared::{
    bookmark_status_context, fetch_status_context, fetch_status_message, push_status_context,
    short_id,
};

impl App {
    pub(in crate::app) fn open_fetch_preview(&mut self, remote: String) {
        let fetch = JjGitFetch::for_remote(remote);
        let status_context = Some(fetch_status_context(&fetch));

        let command_label = fetch.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            fetch.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::FetchPreview { fetch, output };
    }

    pub(in crate::app) fn fetch(&mut self, viewport_height: u16) {
        let fetch = JjGitFetch::default_remotes();
        let command_label = fetch.command_label();
        let status_context = Some(fetch_status_context(&fetch));
        let result_message = match self.run_git_fetch(&fetch) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    self.status =
                        StatusLine::with_message(&self.view, fetch_status_message(&fetch, &output));
                    output
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    if output.is_empty() {
                        format!("refresh failed: {error}")
                    } else {
                        format!("{output}\nrefresh failed: {error}")
                    }
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::FetchPreview {
            fetch,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn open_push_preview(&mut self, target: JjGitPushTarget, remote: String) {
        let status_context = Some(push_status_context(&target, remote.as_str()));
        let push = match target {
            JjGitPushTarget::Bookmark(name) => JjGitPush::for_bookmark(name).with_remote(remote),
            JjGitPushTarget::Revision(name) => JjGitPush::for_revision(name).with_remote(remote),
            JjGitPushTarget::Status => JjGitPush::for_status().with_remote(remote),
        };

        let command_label = push.command_label(true);
        let output = self.preview_output_with_error_status(
            command_label,
            self.load_push_preview(&push),
            std::convert::identity,
            status_context,
        );
        self.mode = InteractionMode::PushPreview { push, output };
    }

    pub(in crate::app) fn open_operation_recovery_preview(
        &mut self,
        kind: JjOperationRecoveryKind,
    ) {
        let recovery = JjOperationRecovery::new(kind);
        let status_context = Some(format!(
            "global current-repo {} from {}",
            recovery.status_action(),
            self.view.spec().app_label()
        ));
        self.mode = InteractionMode::OperationRecoveryPreview {
            output: ActionOutput::pending(
                recovery.command_label().to_owned(),
                recovery.preview_text().to_owned(),
                status_context,
            ),
            recovery,
        };
    }

    pub(in crate::app) fn open_operation_target_preview(&mut self, target: JjOperationTarget) {
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

    pub(in crate::app) fn open_graph_working_copy_navigation_preview(
        &mut self,
        kind: JjWorkingCopyNavigationKind,
    ) {
        let navigation = match kind {
            JjWorkingCopyNavigationKind::Edit => {
                let Some(revision) = self.graph_selected_revision() else {
                    self.status = StatusLine::error(
                        &self.view,
                        "edit from graph requires a selected row with an exact revision".to_owned(),
                    );
                    return;
                };
                JjWorkingCopyNavigationPlan::edit(revision)
            }
            JjWorkingCopyNavigationKind::Next => JjWorkingCopyNavigationPlan::next(),
            JjWorkingCopyNavigationKind::Prev => JjWorkingCopyNavigationPlan::prev(),
        };

        self.open_working_copy_navigation_preview(navigation);
    }

    pub(in crate::app) fn open_working_copy_navigation_preview(
        &mut self,
        navigation: JjWorkingCopyNavigationPlan,
    ) {
        let status_context = Some(match navigation.kind() {
            JjWorkingCopyNavigationKind::Edit => format!(
                "edit exact graph revision {} from {}",
                navigation
                    .target_change_id()
                    .expect("edit preview requires exact change id"),
                self.view.spec().app_label()
            ),
            JjWorkingCopyNavigationKind::Next => format!(
                "move @ with jj next --edit from {}",
                self.view.spec().app_label()
            ),
            JjWorkingCopyNavigationKind::Prev => format!(
                "move @ with jj prev --edit from {}",
                self.view.spec().app_label()
            ),
        });

        let command_label = navigation.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            navigation.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::WorkingCopyNavigationPreview { navigation, output };
    }

    pub(in crate::app) fn open_describe_preview(&mut self, describe: JjDescribePlan) {
        let status_context = Some(format!(
            "describe {} from {}",
            describe.target().label(),
            self.view.spec().app_label()
        ));

        let command_label = describe.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            describe.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::DescribePreview { describe, output };
    }

    pub(in crate::app) fn open_commit_preview(&mut self, commit: JjCommitPlan) {
        let status_context = Some(format!(
            "commit current working-copy change (@) from {}",
            self.view.spec().app_label()
        ));

        let command_label = commit.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            commit.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::CommitPreview { commit, output };
    }

    pub(in crate::app) fn open_bookmark_mutation_preview(
        &mut self,
        mutation: JjBookmarkMutationPlan,
    ) {
        let status_context = Some(bookmark_status_context(
            &mutation,
            self.view.spec().app_label().as_str(),
        ));

        let command_label = mutation.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            mutation.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::BookmarkMutationPreview { mutation, output };
    }

    pub(in crate::app) fn open_file_mutation_preview(&mut self, mutation: JjFileMutationPlan) {
        let target = mutation
            .revision()
            .map(|revision| format!("{} at {}", mutation.path(), revision))
            .unwrap_or_else(|| format!("{} at @", mutation.path()));
        let status_context = Some(format!(
            "file {} {} from {}",
            mutation.kind().label(),
            target,
            self.view.spec().app_label()
        ));

        let command_label = mutation.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            mutation.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::FileMutationPreview { mutation, output };
    }

    pub(in crate::app) fn open_new_preview(&mut self, new_change: JjNewPlan) {
        let parent_labels = new_change
            .parents()
            .iter()
            .map(|parent| short_id(parent))
            .collect::<Vec<_>>()
            .join(", ");
        let status_context = Some(format!(
            "new from {} parent(s) from {} | parent(s): {}",
            new_change.parents().len(),
            self.view.spec().app_label(),
            parent_labels
        ));

        let command_label = new_change.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            new_change.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::NewPreview { new_change, output };
    }

    pub(in crate::app) fn open_duplicate_preview(&mut self, duplicate: JjDuplicatePlan) {
        let status_context = Some(format!(
            "duplicate exact source {} from {}",
            duplicate.source(),
            self.view.spec().app_label()
        ));

        let command_label = duplicate.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            duplicate.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::DuplicatePreview { duplicate, output };
    }

    pub(in crate::app) fn run_new_trunk(&mut self, viewport_height: u16) {
        if let Err(error) = self.resolve_revision("trunk()") {
            self.status = StatusLine::error(&self.view, error.to_string());
            return;
        }

        match self.services.run_new_trunk() {
            Ok(_) => {
                let new_change_id = match self.resolve_revision("@") {
                    Ok(change_id) => change_id,
                    Err(error) => {
                        self.status = StatusLine::error(&self.view, error.to_string());
                        return;
                    }
                };
                match self.refresh_view_state() {
                    Ok(()) => {
                        self.view.clamp(viewport_height, current_viewport_width());
                        let revealed_in_recent =
                            match self.reveal_graph_change(&new_change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height, current_viewport_width());
                                    switched_modes
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                    return;
                                }
                            };
                        let message = if revealed_in_recent {
                            "created new change from trunk | showing recent work | jj undo"
                        } else {
                            "created new change from trunk | jj undo"
                        };
                        self.status = StatusLine::with_message(&self.view, message);
                    }
                    Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
                }
            }
            Err(error) => self.status = StatusLine::error(&self.view, error.to_string()),
        }
    }

    pub(in crate::app) fn open_rebase_preview(&mut self, rebase: JjRebasePlan) {
        let status_context = Some(format!(
            "rebase from {} source(s) into {} from {}",
            rebase.sources().len(),
            rebase.destination(),
            self.view.spec().app_label()
        ));
        let source_labels = rebase
            .sources()
            .iter()
            .map(|source| short_id(source))
            .collect::<Vec<_>>()
            .join(", ");
        let status_context = if source_labels.is_empty() {
            status_context
        } else {
            status_context
                .map(|status_context| format!("{status_context} | source(s): {source_labels}"))
        };

        let command_label = rebase.command_label(true);
        let output = self.preview_output_with_error_status(
            command_label,
            rebase.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::RebasePreview { rebase, output };
    }

    pub(in crate::app) fn open_split_preview(&mut self, split: JjSplitPlan) {
        let status_context = Some(format!(
            "{} from {}",
            split.status_context(),
            self.view.spec().app_label()
        ));

        let command_label = split.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            split.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::SplitPreview { split, output };
    }

    pub(in crate::app) fn open_restore_preview(&mut self, restore: JjRestorePlan) {
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
            self.load_restore_preview(&restore),
            std::convert::identity,
            status_context,
        );
        self.mode = InteractionMode::RestorePreview { restore, output };
    }

    pub(in crate::app) fn open_revert_preview(&mut self, revert: JjRevertPlan) {
        let status_context = Some(format!(
            "revert revision {} into @ from {}",
            revert.revision(),
            self.view.spec().app_label()
        ));

        let command_label = revert.command_label();
        let output = self.preview_output_with_error_status(
            command_label,
            self.load_revert_preview(&revert),
            std::convert::identity,
            status_context,
        );
        self.mode = InteractionMode::RevertPreview { revert, output };
    }

    pub(in crate::app) fn open_squash_preview(&mut self, squash: JjSquashPlan) {
        let status_context = Some(format!(
            "squash from {} source(s) into {} from {}",
            squash.sources().len(),
            squash.destination(),
            self.view.spec().app_label()
        ));
        let source_labels = squash
            .sources()
            .iter()
            .map(|source| short_id(source))
            .collect::<Vec<_>>()
            .join(", ");
        let status_context = if source_labels.is_empty() {
            status_context
        } else {
            status_context
                .map(|status_context| format!("{status_context} | source(s): {source_labels}"))
        };

        let command_label = squash.command_label(true);
        let output = self.preview_output_with_error_status(
            command_label,
            squash.run_preview(),
            |output| output.message().to_owned(),
            status_context,
        );
        self.mode = InteractionMode::SquashPreview { squash, output };
    }

    pub(in crate::app) fn open_absorb_preview(&mut self, absorb: JjAbsorbPlan) {
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

    pub(in crate::app) fn open_abandon_preview(&mut self, abandon: JjAbandonPlan) {
        let status_context = Some(format!(
            "abandon exact revision {} from {}",
            abandon.revision(),
            self.view.spec().app_label()
        ));

        match self.load_abandon_preview(&abandon) {
            Ok(preview) => {
                let command_label = abandon.command_label();
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    output: ActionOutput::pending(
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
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    // Keep mode construction at each caller; this helper owns only pane state and error status.
    fn preview_output_with_error_status<T>(
        &mut self,
        command_label: String,
        preview_result: Result<T>,
        preview_text: impl FnOnce(T) -> String,
        status_context: Option<String>,
    ) -> ActionOutput {
        match preview_result {
            Ok(output) => {
                ActionOutput::pending(command_label, preview_text(output), status_context)
            }
            Err(error) => {
                let message = error.to_string();
                self.status = StatusLine::error(&self.view, message.clone());
                ActionOutput::finished(command_label, message, status_context)
            }
        }
    }
}
