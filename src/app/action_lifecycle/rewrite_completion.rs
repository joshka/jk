//! Rewrite action completion and result-pane handling.
//!
//! This module owns post-command refresh, reveal, and result wording for commands that move or
//! rewrite changes after preview confirmation.

use crate::action_output::ActionOutput;
use crate::app_screen::InteractionMode;
use crate::app_status::StatusLine;
use crate::jj::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjRebasePlan, JjRestorePlan, JjRevertPlan,
    JjSquashPlan, JjWorkingCopyNavigationKind, JjWorkingCopyNavigationPlan, LogViewMode,
};

use super::super::{App, current_viewport_width};

impl App {
    pub(in crate::app) fn confirm_working_copy_navigation(
        &mut self,
        navigation: JjWorkingCopyNavigationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = navigation.command_label();
        let result_message = match self.run_working_copy_navigation(&navigation) {
            Ok(output) => {
                let reveal_change_id = match navigation.kind() {
                    JjWorkingCopyNavigationKind::Edit => navigation
                        .target_change_id()
                        .expect("edit navigation requires exact target change id")
                        .to_owned(),
                    JjWorkingCopyNavigationKind::Next | JjWorkingCopyNavigationKind::Prev => {
                        match self.resolve_revision("@") {
                            Ok(change_id) => change_id,
                            Err(error) => {
                                let message = format!(
                                    "{} | resolve @ failed: {error} | jj undo",
                                    output.trim()
                                );
                                self.status = StatusLine::error(&self.view, error.to_string());
                                self.mode = InteractionMode::WorkingCopyNavigationPreview {
                                    navigation,
                                    output: ActionOutput::finished(
                                        command_label,
                                        message,
                                        status_context,
                                    ),
                                };
                                return;
                            }
                        }
                    }
                };

                match self.refresh_view_state() {
                    Ok(()) => {
                        self.view.clamp(viewport_height, current_viewport_width());
                        let mut reveal_error = None;
                        let revealed_in_recent = match self
                            .reveal_graph_change(&reveal_change_id, LogViewMode::Recent)
                        {
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

        self.mode = InteractionMode::WorkingCopyNavigationPreview {
            navigation,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_abandon(
        &mut self,
        abandon: JjAbandonPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = abandon.command_label();
        let result_message = match self.run_abandon(&abandon) {
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

        self.mode = InteractionMode::AbandonPreview {
            abandon,
            preview: JjAbandonPreview::new(String::new(), None, String::new()),
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_empty_abandon_after_recheck(
        &mut self,
        abandon: JjAbandonPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        match self.load_abandon_preview(&abandon) {
            Ok(preview) if preview.is_empty_change() => {
                self.confirm_abandon(abandon, status_context, viewport_height);
            }
            Ok(preview) => {
                let message = "change is no longer empty; type exact revision to confirm abandon";
                self.status = StatusLine::error(&self.view, message.to_owned());
                let command_label = abandon.command_label();
                let output = format!("{message}\n\n{}", preview.preview_text());
                self.mode = InteractionMode::AbandonConfirm {
                    abandon,
                    input: String::new(),
                    output: ActionOutput::pending(command_label, output, status_context),
                };
            }
            Err(error) => {
                let message = error.to_string();
                self.status = StatusLine::error(&self.view, message.clone());
                let command_label = abandon.diff_summary_label();
                self.mode = InteractionMode::AbandonPreview {
                    abandon,
                    preview: JjAbandonPreview::new(String::new(), None, String::new()),
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(in crate::app) fn confirm_restore(
        &mut self,
        restore: JjRestorePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = restore.command_label();
        let result_message = match self.run_restore(&restore) {
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

        self.mode = InteractionMode::RestorePreview {
            restore,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_revert(
        &mut self,
        revert: JjRevertPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = revert.command_label();
        let result_message = match self.run_revert(&revert) {
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

        self.mode = InteractionMode::RevertPreview {
            revert,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_rebase(
        &mut self,
        rebase: JjRebasePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = rebase.command_label(false);
        let primary_source = rebase.sources().first().cloned();
        let result_message = match self.run_rebase(&rebase) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    let mut reveal_error = None;
                    let revealed_in_recent = match primary_source.as_deref() {
                        Some(change_id) => {
                            match self.reveal_graph_change(change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height, current_viewport_width());
                                    Some(switched_modes)
                                }
                                Err(error) => {
                                    self.status = StatusLine::error(&self.view, error.to_string());
                                    reveal_error = Some(format!(
                                        "{} | reveal failed: {} | jj undo | jj op show -p",
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
                                format!(
                                    "{} | showing recent work | jj undo | jj op show -p",
                                    output.trim()
                                )
                            } else {
                                format!("{} | jj undo | jj op show -p", output.trim())
                            }
                        }
                        None => match reveal_error.as_deref() {
                            Some(message) => message.to_owned(),
                            None => format!("{} | jj undo | jj op show -p", output.trim()),
                        },
                    };
                    if reveal_error.is_none() {
                        self.status = StatusLine::with_message(&self.view, message.as_str());
                    }
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!(
                        "{} | refresh failed: {error} | jj undo | jj op show -p",
                        output.trim()
                    )
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::RebasePreview {
            rebase,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_squash(
        &mut self,
        squash: JjSquashPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = squash.command_label(false);
        let destination = squash.destination().to_owned();
        let result_message = match self.run_squash(&squash) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    let mut reveal_error = None;
                    let revealed_in_recent =
                        match self.reveal_graph_change(&destination, LogViewMode::Recent) {
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
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::SquashPreview {
            squash,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }

    pub(in crate::app) fn confirm_absorb(
        &mut self,
        absorb: JjAbsorbPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = absorb.command_label();
        let result_message = match self.run_absorb(&absorb) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height, current_viewport_width());
                    let message = format!("{} | jj undo | jj op show -p", output.trim());
                    self.status = StatusLine::with_message(&self.view, message.as_str());
                    message
                }
                Err(error) => {
                    self.status = StatusLine::error(&self.view, error.to_string());
                    format!(
                        "{} | refresh failed: {error} | jj undo | jj op show -p",
                        output.trim()
                    )
                }
            },
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                error.to_string()
            }
        };

        self.mode = InteractionMode::AbsorbPreview {
            absorb,
            output: ActionOutput::finished(command_label, result_message, status_context),
        };
    }
}
