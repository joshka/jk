//! Action lifecycle orchestration for app-owned mutation screens.
//!
//! Modal key dispatch stays in `mode_input`; this module owns what selected action-menu items,
//! prompt completion, preview opening, and confirmed jj action results do to app state.

use color_eyre::Result;

use crate::action_menu::{ActionKind, ActionMenuItem, FollowUp};
use crate::action_output::ActionOutput;
use crate::app_screen::InteractionMode;
use crate::app_status::StatusLine;
use crate::jj::{
    JjAbandonPlan, JjAbandonPreview, JjAbsorbPlan, JjBookmarkMutationKind, JjBookmarkMutationPlan,
    JjCommand, JjCommitPlan, JjDescribePlan, JjDescribeTarget, JjGitFetch, JjGitPush,
    JjGitPushTarget, JjNewPlan, JjOperationRecovery, JjOperationRecoveryKind, JjOperationTarget,
    JjRebasePlan, JjRestorePlan, JjRevertPlan, JjSquashPlan, JjWorkingCopyNavigationKind,
    JjWorkingCopyNavigationPlan, LogViewMode,
};
use crate::view_state::ViewState;

use super::App;

impl App {
    pub(super) fn apply_action_menu_item(&mut self, item: ActionMenuItem) {
        match item.follow_up() {
            FollowUp::StatusMessage(message) => {
                self.status = StatusLine::with_message(&self.view, message.as_str());
                self.mode = InteractionMode::Normal;
            }
            FollowUp::ExactRevision { revision } => {
                let action = item.action();
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                match action {
                    ActionKind::Abandon => {
                        self.open_abandon_preview(JjAbandonPlan::new(revision));
                    }
                    ActionKind::Edit
                    | ActionKind::New
                    | ActionKind::Split
                    | ActionKind::Restore
                    | ActionKind::Revert
                    | ActionKind::Rebase
                    | ActionKind::Squash
                    | ActionKind::Absorb => {
                        self.status =
                            StatusLine::with_message(&self.view, "preview not yet implemented");
                    }
                }
            }
            FollowUp::EditExactTarget { revision } => {
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                self.open_working_copy_navigation_preview(JjWorkingCopyNavigationPlan::edit(
                    revision,
                ));
            }
            FollowUp::RestoreExactTarget { revision, path } => {
                let revision = revision.clone();
                let path = path.clone();
                self.mode = InteractionMode::Normal;
                match path {
                    Some(path) => {
                        self.open_restore_preview(JjRestorePlan::for_path(revision, path));
                    }
                    None => {
                        self.open_restore_preview(JjRestorePlan::for_revision(revision));
                    }
                }
            }
            FollowUp::RestoreWorkingCopyPath { path } => {
                let path = path.clone();
                self.mode = InteractionMode::Normal;
                self.open_restore_preview(JjRestorePlan::for_working_copy_path(path));
            }
            FollowUp::RevertExactTarget { revision } => {
                let revision = revision.clone();
                self.mode = InteractionMode::Normal;
                self.open_revert_preview(JjRevertPlan::new(revision));
            }
            FollowUp::OperationRestoreExactTarget { operation_id } => {
                let operation_id = operation_id.clone();
                self.mode = InteractionMode::Normal;
                self.open_operation_target_preview(JjOperationTarget::restore(operation_id));
            }
            FollowUp::OperationRevertExactTarget { operation_id } => {
                let operation_id = operation_id.clone();
                self.mode = InteractionMode::Normal;
                self.open_operation_target_preview(JjOperationTarget::revert(operation_id));
            }
            FollowUp::NewParents { parents } => {
                let parents = parents.clone();
                self.mode = InteractionMode::Normal;
                self.open_new_preview(JjNewPlan::new(parents));
            }
            FollowUp::RolePrompt(prompt) => {
                self.mode = InteractionMode::RolePrompt {
                    action: item.action(),
                    prompt: prompt.clone(),
                    selected: 0,
                };
            }
            FollowUp::AbsorbCandidates {
                source,
                destinations,
            } => {
                let source = source.clone();
                let destinations = destinations.clone();
                self.mode = InteractionMode::Normal;
                self.open_absorb_preview(JjAbsorbPlan::new(source, destinations));
            }
        }
    }

    pub(super) fn graph_selected_revision(&self) -> Option<String> {
        match &self.view {
            ViewState::Graph(view) => view.selected_revision().map(str::to_owned),
            ViewState::Show(_)
            | ViewState::Diff(_)
            | ViewState::Status(_)
            | ViewState::Resolve(_)
            | ViewState::FileList(_)
            | ViewState::FileShow(_)
            | ViewState::Bookmarks(_)
            | ViewState::OperationLog(_)
            | ViewState::OperationDetail(_) => None,
        }
    }

    pub(super) fn open_describe_prompt(&mut self) {
        let target = match self.view.command() {
            JjCommand::Default | JjCommand::Log => match self.view.push_target() {
                Ok(Some(JjGitPushTarget::Revision(revision))) => {
                    JjDescribeTarget::exact_change(revision)
                }
                Ok(_) | Err(_) => {
                    self.status = StatusLine::error(
                        &self.view,
                        "describe from graph requires a selected row with an exact revision"
                            .to_owned(),
                    );
                    return;
                }
            },
            JjCommand::Status => JjDescribeTarget::current_working_copy(),
            JjCommand::Show
            | JjCommand::Diff
            | JjCommand::Resolve
            | JjCommand::FileList
            | JjCommand::FileShow
            | JjCommand::Bookmarks
            | JjCommand::OperationLog
            | JjCommand::OperationShow
            | JjCommand::OperationDiff => {
                self.status = StatusLine::error(
                    &self.view,
                    "describe is only available from graph or status views".to_owned(),
                );
                return;
            }
        };

        self.mode = InteractionMode::DescribePrompt {
            target,
            input: String::new(),
        };
    }

    pub(super) fn open_commit_prompt(&mut self) {
        if matches!(
            self.view.command(),
            JjCommand::Default | JjCommand::Log | JjCommand::Status
        ) {
            self.mode = InteractionMode::CommitPrompt(String::new());
        } else {
            self.status = StatusLine::error(
                &self.view,
                "commit is only available from graph or status because jj commit always acts on @"
                    .to_owned(),
            );
        }
    }

    pub(super) fn open_bookmark_name_prompt(&mut self, kind: JjBookmarkMutationKind) {
        let target = match self.view.bookmark_target() {
            Ok(Some(target)) => target,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    format!(
                        "bookmark {} is only available from graph or status views",
                        kind.label()
                    ),
                );
                return;
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                return;
            }
        };

        self.mode = InteractionMode::BookmarkNamePrompt {
            kind,
            target,
            input: String::new(),
        };
    }

    pub(super) fn open_bookmark_delete_preview(&mut self) {
        let name = match self.view.selected_local_bookmark_name() {
            Ok(Some(name)) => name.to_owned(),
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "bookmark delete is only available from bookmarks view".to_owned(),
                );
                return;
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                return;
            }
        };

        self.open_bookmark_mutation_preview(JjBookmarkMutationPlan::delete(name));
    }

    pub(super) fn open_push_prompt(&mut self) -> Result<bool> {
        let target = match self.view.push_target() {
            Ok(Some(target)) => target,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "push is only available from graph, status, or bookmarks views".to_owned(),
                );
                return Ok(false);
            }
            Err(message) => {
                self.status = StatusLine::error(&self.view, message.to_string());
                return Ok(false);
            }
        };

        match self.load_git_remotes() {
            Ok(remotes) => {
                match remotes.as_slice() {
                    [] => {
                        self.status = StatusLine::error(
                            &self.view,
                            "no git remotes found; add a remote before pushing".to_owned(),
                        );
                    }
                    [remote] => self.open_push_preview(target, remote.to_owned()),
                    _ => {
                        self.mode = InteractionMode::PushRemotePrompt {
                            target,
                            remotes,
                            selected: 0,
                        };
                    }
                }
                Ok(false)
            }
            Err(error) => {
                self.status = StatusLine::error(&self.view, error.to_string());
                Ok(false)
            }
        }
    }

    pub(super) fn open_fetch_remote_prompt(&mut self) {
        match self.load_git_remotes() {
            Ok(remotes) => match remotes.as_slice() {
                [] => {
                    let message = "no git remotes found; run default fetch or add a remote before choosing one"
                        .to_owned();
                    self.status = StatusLine::error(&self.view, message.clone());
                    self.mode = InteractionMode::FetchPreview {
                        fetch: JjGitFetch::default_remotes(),
                        output: ActionOutput::finished(
                            "jj git remote list".to_owned(),
                            message,
                            Some("fetch remote selection found no remotes".to_owned()),
                        ),
                    };
                }
                [remote] => self.open_fetch_preview(remote.to_owned()),
                _ => {
                    self.mode = InteractionMode::FetchRemotePrompt {
                        remotes,
                        selected: 0,
                    };
                }
            },
            Err(error) => {
                let message = error.to_string();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::FetchPreview {
                    fetch: JjGitFetch::default_remotes(),
                    output: ActionOutput::finished(
                        "jj git remote list".to_owned(),
                        message,
                        Some("fetch remote selection failed to list remotes".to_owned()),
                    ),
                };
            }
        }
    }

    pub(super) fn open_fetch_preview(&mut self, remote: String) {
        let fetch = JjGitFetch::for_remote(remote);
        let status_context = Some(fetch_status_context(&fetch));

        match fetch.run_preview() {
            Ok(output) => {
                let command_label = fetch.command_label();
                self.mode = InteractionMode::FetchPreview {
                    fetch,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = fetch.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::FetchPreview {
                    fetch,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_push_preview(&mut self, target: JjGitPushTarget, remote: String) {
        let status_context = Some(push_status_context(&target, remote.as_str()));
        let push = match target {
            JjGitPushTarget::Bookmark(name) => JjGitPush::for_bookmark(name).with_remote(remote),
            JjGitPushTarget::Revision(name) => JjGitPush::for_revision(name).with_remote(remote),
            JjGitPushTarget::Status => JjGitPush::for_status().with_remote(remote),
        };

        match self.load_push_preview(&push) {
            Ok(output) => {
                let command_label = push.command_label(true);
                self.mode = InteractionMode::PushPreview {
                    push,
                    output: ActionOutput::pending(command_label, output, status_context),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = push.command_label(true);
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::PushPreview {
                    push,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_operation_recovery_preview(&mut self, kind: JjOperationRecoveryKind) {
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

    pub(super) fn open_operation_target_preview(&mut self, target: JjOperationTarget) {
        let status_context = Some(format!(
            "operation {} exact id {} from {}",
            target.status_action(),
            target.operation_id(),
            self.view.spec().app_label()
        ));

        match target.run_preview() {
            Ok(output) => {
                let command_label = target.command_label();
                self.mode = InteractionMode::OperationTargetPreview {
                    target,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = target.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::OperationTargetPreview {
                    target,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_graph_working_copy_navigation_preview(
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

    pub(super) fn open_working_copy_navigation_preview(
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

        match navigation.run_preview() {
            Ok(output) => {
                let command_label = navigation.command_label();
                self.mode = InteractionMode::WorkingCopyNavigationPreview {
                    navigation,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = navigation.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::WorkingCopyNavigationPreview {
                    navigation,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_describe_preview(&mut self, describe: JjDescribePlan) {
        let status_context = Some(format!(
            "describe {} from {}",
            describe.target().label(),
            self.view.spec().app_label()
        ));

        match describe.run_preview() {
            Ok(output) => {
                let command_label = describe.command_label();
                self.mode = InteractionMode::DescribePreview {
                    describe,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = describe.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::DescribePreview {
                    describe,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_commit_preview(&mut self, commit: JjCommitPlan) {
        let status_context = Some(format!(
            "commit current working-copy change (@) from {}",
            self.view.spec().app_label()
        ));

        match commit.run_preview() {
            Ok(output) => {
                let command_label = commit.command_label();
                self.mode = InteractionMode::CommitPreview {
                    commit,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = commit.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::CommitPreview {
                    commit,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_bookmark_mutation_preview(&mut self, mutation: JjBookmarkMutationPlan) {
        let status_context = Some(bookmark_status_context(
            &mutation,
            self.view.spec().app_label().as_str(),
        ));

        match mutation.run_preview() {
            Ok(output) => {
                let command_label = mutation.command_label();
                self.mode = InteractionMode::BookmarkMutationPreview {
                    mutation,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = mutation.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::BookmarkMutationPreview {
                    mutation,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_new_preview(&mut self, new_change: JjNewPlan) {
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

        match new_change.run_preview() {
            Ok(output) => {
                let command_label = new_change.command_label();
                self.mode = InteractionMode::NewPreview {
                    new_change,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = new_change.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::NewPreview {
                    new_change,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_rebase_preview(&mut self, rebase: JjRebasePlan) {
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

        match rebase.run_preview() {
            Ok(output) => {
                let command_label = rebase.command_label(true);
                self.mode = InteractionMode::RebasePreview {
                    rebase,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = rebase.command_label(true);
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::RebasePreview {
                    rebase,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_restore_preview(&mut self, restore: JjRestorePlan) {
        let target = restore
            .path()
            .map(|path| format!("path {path} from {}", restore.revision()))
            .unwrap_or_else(|| format!("revision {}", restore.revision()));
        let status_context = Some(format!(
            "restore {target} from {}",
            self.view.spec().app_label()
        ));

        match self.load_restore_preview(&restore) {
            Ok(output) => {
                let command_label = restore.command_label();
                self.mode = InteractionMode::RestorePreview {
                    restore,
                    output: ActionOutput::pending(command_label, output, status_context),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = restore.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::RestorePreview {
                    restore,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_revert_preview(&mut self, revert: JjRevertPlan) {
        let status_context = Some(format!(
            "revert revision {} into @ from {}",
            revert.revision(),
            self.view.spec().app_label()
        ));

        match self.load_revert_preview(&revert) {
            Ok(output) => {
                let command_label = revert.command_label();
                self.mode = InteractionMode::RevertPreview {
                    revert,
                    output: ActionOutput::pending(command_label, output, status_context),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = revert.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::RevertPreview {
                    revert,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_squash_preview(&mut self, squash: JjSquashPlan) {
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

        match squash.run_preview() {
            Ok(output) => {
                let command_label = squash.command_label(true);
                self.mode = InteractionMode::SquashPreview {
                    squash,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = squash.command_label(true);
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::SquashPreview {
                    squash,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_absorb_preview(&mut self, absorb: JjAbsorbPlan) {
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

        match absorb.run_preview() {
            Ok(output) => {
                let command_label = absorb.command_label();
                self.mode = InteractionMode::AbsorbPreview {
                    absorb,
                    output: ActionOutput::pending(
                        command_label,
                        output.message().to_owned(),
                        status_context,
                    ),
                };
            }
            Err(error) => {
                let message = error.to_string();
                let command_label = absorb.command_label();
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::AbsorbPreview {
                    absorb,
                    output: ActionOutput::finished(command_label, message, status_context),
                };
            }
        }
    }

    pub(super) fn open_abandon_preview(&mut self, abandon: JjAbandonPlan) {
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

    pub(super) fn confirm_describe(
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
                    self.view.clamp(viewport_height);
                    let mut reveal_error = None;
                    let revealed_in_recent = match reveal_change_id.as_deref() {
                        Some(change_id) => {
                            match self.reveal_graph_change(change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_commit(
        &mut self,
        commit: JjCommitPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = commit.command_label();
        let result_message = match self.run_commit(&commit) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_bookmark_mutation(
        &mut self,
        mutation: JjBookmarkMutationPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = mutation.command_label();
        let result_message = match self.run_bookmark_mutation(&mutation) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_new_change(
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
                        self.view.clamp(viewport_height);
                        let mut reveal_error = None;
                        let revealed_in_recent =
                            match self.reveal_graph_change(&new_change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_push(
        &mut self,
        push: JjGitPush,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = push.command_label(false);
        let result_message = match self.run_push(&push) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_fetch(
        &mut self,
        fetch: JjGitFetch,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = fetch.command_label();
        let result_message = match self.run_git_fetch(&fetch) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_operation_recovery(
        &mut self,
        recovery: JjOperationRecovery,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = recovery.command_label().to_owned();
        let result_message = match self.run_operation_recovery(&recovery) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_operation_target(
        &mut self,
        target: JjOperationTarget,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = target.command_label();
        let result_message = match self.run_operation_target(&target) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn refresh_stacked_repo_views(&mut self, viewport_height: u16) -> Result<()> {
        for view in &mut self.stack {
            match view.command() {
                JjCommand::Default
                | JjCommand::Log
                | JjCommand::Status
                | JjCommand::Bookmarks
                | JjCommand::OperationLog => {
                    self.services.refresh_view(view)?;
                    view.clamp(viewport_height);
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

    pub(super) fn confirm_working_copy_navigation(
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
                        self.view.clamp(viewport_height);
                        let mut reveal_error = None;
                        let revealed_in_recent = match self
                            .reveal_graph_change(&reveal_change_id, LogViewMode::Recent)
                        {
                            Ok(switched_modes) => {
                                self.view.clamp(viewport_height);
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

    pub(super) fn confirm_abandon(
        &mut self,
        abandon: JjAbandonPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = abandon.command_label();
        let result_message = match self.run_abandon(&abandon) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_empty_abandon_after_recheck(
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

    pub(super) fn confirm_restore(
        &mut self,
        restore: JjRestorePlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = restore.command_label();
        let result_message = match self.run_restore(&restore) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_revert(
        &mut self,
        revert: JjRevertPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = revert.command_label();
        let result_message = match self.run_revert(&revert) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_rebase(
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
                    self.view.clamp(viewport_height);
                    let mut reveal_error = None;
                    let revealed_in_recent = match primary_source.as_deref() {
                        Some(change_id) => {
                            match self.reveal_graph_change(change_id, LogViewMode::Recent) {
                                Ok(switched_modes) => {
                                    self.view.clamp(viewport_height);
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

    pub(super) fn confirm_squash(
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
                    self.view.clamp(viewport_height);
                    let mut reveal_error = None;
                    let revealed_in_recent =
                        match self.reveal_graph_change(&destination, LogViewMode::Recent) {
                            Ok(switched_modes) => {
                                self.view.clamp(viewport_height);
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

    pub(super) fn confirm_absorb(
        &mut self,
        absorb: JjAbsorbPlan,
        status_context: Option<String>,
        viewport_height: u16,
    ) {
        let command_label = absorb.command_label();
        let result_message = match self.run_absorb(&absorb) {
            Ok(output) => match self.refresh_view_state() {
                Ok(()) => {
                    self.view.clamp(viewport_height);
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

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}

fn push_status_context(target: &JjGitPushTarget, remote: &str) -> String {
    match target {
        JjGitPushTarget::Bookmark(name) => {
            format!("bookmark push targets exact bookmark '{name}' on remote {remote}")
        }
        JjGitPushTarget::Revision(revision) => {
            format!("graph push targets exact selected revision '{revision}' on remote {remote}")
        }
        JjGitPushTarget::Status => {
            format!("status push uses jj default target resolution for remote {remote}")
        }
    }
}

pub(super) fn fetch_status_context(fetch: &JjGitFetch) -> String {
    match fetch.remote() {
        Some(remote) => {
            let pattern = fetch
                .exact_remote_pattern()
                .expect("remote-specific fetch has a remote pattern");
            format!("fetch targets exact remote '{remote}' with pattern {pattern}")
        }
        None => "default fetch uses jj git fetch remote resolution".to_owned(),
    }
}

pub(super) fn fetch_status_message(fetch: &JjGitFetch, output: &str) -> String {
    match fetch.remote() {
        Some(remote) => format!("fetch {remote}: {output}"),
        None => format!("fetch: {output}"),
    }
}

fn bookmark_status_context(mutation: &JjBookmarkMutationPlan, view_label: &str) -> String {
    match mutation.target() {
        Some(target) => format!(
            "bookmark {} '{}' targets {} from {}",
            mutation.kind().label(),
            mutation.name(),
            target.label(),
            view_label
        ),
        None => format!(
            "bookmark {} '{}' from {}",
            mutation.kind().label(),
            mutation.name(),
            view_label
        ),
    }
}
