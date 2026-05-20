//! General action completion and result-pane handling.
//!
//! This module owns post-command refresh, reveal, and result wording for non-rewrite-specific
//! mutation flows such as describe, commit, bookmarks, sync, duplicate, split, and operation
//! recovery.

use color_eyre::Result;
use ratatui::DefaultTerminal;

use crate::action_output::ActionOutput;
use crate::app_screen::InteractionMode;
use crate::app_status::StatusLine;
use crate::jj::{
    JjBookmarkMutationPlan, JjCommand, JjCommitPlan, JjDescribePlan, JjDuplicatePlan, JjGitFetch,
    JjGitPush, JjNewPlan, JjOperationRecovery, JjOperationTarget, JjSplitPlan, JjSplitTarget,
    LogViewMode,
};
use crate::view_state::ViewState;

use super::super::{App, current_viewport_width};
use super::shared::fetch_status_message;

impl App {
    pub(in crate::app) fn confirm_describe(
        &mut self,
        describe: JjDescribePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = describe.command_label();
        let reveal_change_id = describe.target().exact_change_id().map(str::to_owned);
        let result_message = match self.run_describe(&describe) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    let mut reveal_error = None;
                    let revealed_in_recent = match reveal_change_id.as_deref() {
                        Some(change_id) => {
                            match self.reveal_graph_change(change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height, current_viewport_width());
                                    Some(switched_modes)
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                    reveal_error = Some(format!(
                                        "{} | reveal failed: {} | jj undo",
                                        output.trim(),
                                        error
                                    ));
                                    None
                                }
                            }
                        }
                        None => None,
                    };

                    let message = match revealed_in_recent {
                        Some(switched_modes) => {
                            if switched_modes {
                                format!("{} | showing recent work | jj undo", output.trim())
                            } else {
                                format!("{} | jj undo", output.trim())
                            }
                        }
                        None => match reveal_error.as_deref() {
                            Some(message) => message.to_owned(),
                            None => format!("{} | jj undo", output.trim()),
                        },
                    };
                    if reveal_error.is_none() {
                        self.status = StatusLine::with_message(&self.view, message.as_str());
                    }
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!("{} | refresh failed: {error} | jj undo", output.trim())
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::DescribePreview {
            describe,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_commit(
        &mut self,
        commit: JjCommitPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = commit.command_label();
        let result_message = match self.run_commit(&commit) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    let message = format!(
                        "{} | new working-copy change created on top | jj undo",
                        output.trim()
                    );
                    self.status = StatusLine::with_message(&self.view, message.as_str());
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!(
                        "{} | refresh failed: {error} | new working-copy change created on top | jj undo",
                        output.trim()
                    )
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::CommitPreview {
            commit,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_bookmark_mutation(
        &mut self,
        mutation: JjBookmarkMutationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = mutation.command_label();
        let result_message = match self.run_bookmark_mutation(&mutation) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    let message = format!("{} | jj undo", output.trim());
                    self.status = StatusLine::with_message(&self.view, message.as_str());
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!("{} | refresh failed: {error} | jj undo", output.trim())
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::BookmarkMutationPreview {
            mutation,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_new_change(
        &mut self,
        new_change: JjNewPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = new_change.command_label();
        let result_message = match self.run_new_change(&new_change) {
            Ok(output) => {
                let new_change_id = match self.resolve_revision("@") {
                    Ok(change_id) => change_id,
                    Err(error) => {
                        let message =
                            format!("{} | resolve @ failed: {error} | jj undo", output.trim());
                        self.status = StatusLine::error(&self.view, error.to_string());
                        self.mode = InteractionMode::NewPreview {
                            new_change,
                            output: ActionOutput::finished(command_label, message, status_context),
                        };
                        return;
                    }
                };

                match self.refresh_view_state() {
                    Ok(()) => {
                        self.view.clamp(viewport_height, current_viewport_width());
                        let mut reveal_error = None;
                        let revealed_in_recent =
                            match self.reveal_graph_change(&new_change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height, current_viewport_width());
                                    Some(switched_modes)
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                    reveal_error = Some(format!(
                                        "{} | reveal failed: {} | jj undo",
                                        output.trim(),
                                        error
                                    ));
                                    None
                                }
                            };

                        let message = match revealed_in_recent {
                            Some(switched_modes) => {
                                if switched_modes {
                                    format!("{} | showing recent work | jj undo", output.trim())
                                } else {
                                    format!("{} | jj undo", output.trim())
                                }
                            }
                            None => match reveal_error.as_deref() {
                                Some(message) => message.to_owned(),
                                None => format!("{} | jj undo", output.trim()),
                            },
                        };
                        if reveal_error.is_none() {
                            self.status = StatusLine::with_message(&self.view, message.as_str());
                        }
                        message
                    }
                    Err(error) => {
                        self.status = StatusLine::error(&self.view, error.to_string());
                        format!("{} | refresh failed: {error} | jj undo", output.trim())
                    }
                }
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::NewPreview {
            new_change,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_duplicate(
        &mut self,
        duplicate: JjDuplicatePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = duplicate.command_label();
        let result_message = match self.run_duplicate(&duplicate) {
            Ok(output) => self.finish_successful_duplicate(&duplicate, output, viewport_height),
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::DuplicatePreview {
            duplicate,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

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
                    ViewState::Graph(_) => {
                        Some(self.reveal_graph_change(duplicate.source(), LogViewMode::Recent))
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
                            "{}\nrefresh: active view refreshed\nreveal: source fallback not attempted because the active view cannot reveal graph changes\nrecovery: jj undo\nreview: jj op show -p",
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

    pub(in crate::app) fn confirm_push(
        &mut self,
        push: JjGitPush,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = push.command_label(false);
        let result_message = match self.run_push(&push) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    self.status = StatusLine::with_message(&self.view, output.as_str());
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

        self.mode = InteractionMode::PushPreview {
            push,
            output: ActionOutput::finished(command_label, result_message, status_context),
        }
    }

    pub(in crate::app) fn confirm_fetch(
        &mut self,
        fetch: JjGitFetch,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = fetch.command_label();
        let result_message = match self.run_git_fetch(&fetch) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    self.status = StatusLine::with_message(
                        &self.view,
                        fetch_status_message(&fetch, output.as_str()),
                    );
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
        }
    }

    pub(in crate::app) fn confirm_operation_recovery(
        &mut self,
        recovery: JjOperationRecovery,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = recovery.command_label().to_owned();
        let result_message = match self.run_operation_recovery(&recovery) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    let message = format!("{} | {}", output.trim(), recovery.success_hint());
                    self.status = StatusLine::with_message(&self.view, message.as_str());
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!(
                        "{} | refresh failed: {error} | {}",
                        output.trim(),
                        recovery.success_hint()
                    )
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::OperationRecoveryPreview {
            recovery,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_split(
        &mut self,
        split: JjSplitPlan,
        status_context: Option<String>,
        viewport_height: u16,
        terminal: Option<&mut DefaultTerminal>,
    ) {
        let command_label = split.command_label();
        let result_message = match self.run_split(terminal, &split) {
            Ok(output) => self.finish_successful_split(&split, output, viewport_height),
            Err(error) => {
                let message = split.failure_result_message(&error.to_string());
                self.status = StatusLine::error(&self.view, message.clone());
                message
            }
        };

        self.mode = InteractionMode::SplitPreview {
            split,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    fn finish_successful_split(
        &mut self,
        split: &JjSplitPlan,
        output: String,
        viewport_height: u16,
    ) -> String {
        let reveal_change_id = match split.target() {
            JjSplitTarget::ExactChange(change_id) => Some(change_id.clone()),
            JjSplitTarget::CurrentWorkingCopy => match self.resolve_revision("@") {
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
                        match self.reveal_graph_change(change_id, LogViewMode::Recent) {
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

    pub(in crate::app) fn confirm_operation_target(
        &mut self,
        target: JjOperationTarget,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = target.command_label();
        let result_message = match self.run_operation_target(&target) {
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
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::OperationTargetPreview {
            target,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

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
