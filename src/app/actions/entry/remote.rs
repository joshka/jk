use color_eyre::Result;

use crate::actions::JjGitFetch;
use crate::app::actions::ActionPane;
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;

use super::super::super::App;

const PUSH_NO_REMOTES_MESSAGE: &str = "no git remotes found; add a remote before pushing";
const FETCH_NO_REMOTES_MESSAGE: &str =
    "no git remotes found; run default fetch or add a remote before choosing one";
const FETCH_REMOTE_LIST_COMMAND_LABEL: &str = "jj git remote list";
const FETCH_NO_REMOTES_CONTEXT: &str = "fetch remote selection found no remotes";
const FETCH_REMOTE_LIST_ERROR_CONTEXT: &str = "fetch remote selection failed to list remotes";

#[derive(Debug, Eq, PartialEq)]
pub enum PushRemotePromptDecision {
    MissingRemotes {
        /// Status text shown when no remote can be chosen.
        message: String,
    },
    OpenPreview {
        /// Sole remote that can be used immediately without prompting.
        remote: String,
    },
    Prompt {
        /// Ordered remotes offered in the selection prompt.
        remotes: Vec<String>,
    },
    RemoteListError {
        /// Status text shown when remote discovery fails.
        message: String,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FetchRemotePromptDecision {
    MissingRemotes {
        /// Status text shown when no remote can be chosen.
        message: String,
        /// Command/output context captured for the action pane.
        status_context: String,
    },
    OpenPreview {
        /// Sole remote that can be used immediately without prompting.
        remote: String,
    },
    Prompt {
        /// Ordered remotes offered in the selection prompt.
        remotes: Vec<String>,
    },
    RemoteListError {
        /// Status text shown when remote discovery fails.
        message: String,
        /// Command/output context captured for the action pane.
        status_context: String,
    },
}

impl App {
    /// Open push preview immediately, or prompt for a remote when multiple choices exist.
    pub fn open_push_prompt(&mut self) -> Result<bool> {
        let target = match self.view.push_target() {
            Ok(Some(target)) => target,
            Ok(None) => {
                self.status = StatusLine::error(
                    &self.view,
                    "push is only available from log, status, or bookmarks views".to_owned(),
                );
                return Ok(false);
            }
            Err(message) => {
                self.status = StatusLine::error(&self.view, message.to_string());
                return Ok(false);
            }
        };

        let decision = decide_push_remote_prompt(
            self.services
                .load_git_remotes()
                .map_err(|error| error.to_string()),
        );
        match decision {
            PushRemotePromptDecision::MissingRemotes { message }
            | PushRemotePromptDecision::RemoteListError { message } => {
                self.status = StatusLine::error(&self.view, message);
            }
            PushRemotePromptDecision::OpenPreview { remote } => {
                self.open_push_preview(target, remote);
            }
            PushRemotePromptDecision::Prompt { remotes } => {
                self.mode = InteractionMode::PushRemotePrompt {
                    target,
                    remotes,
                    selected: 0,
                };
            }
        }
        Ok(false)
    }

    /// Open fetch preview immediately, or prompt for a remote when multiple choices exist.
    pub fn open_fetch_remote_prompt(&mut self) {
        let decision = decide_fetch_remote_prompt(
            self.services
                .load_git_remotes()
                .map_err(|error| error.to_string()),
        );
        match decision {
            FetchRemotePromptDecision::MissingRemotes {
                message,
                status_context,
            }
            | FetchRemotePromptDecision::RemoteListError {
                message,
                status_context,
            } => {
                self.status = StatusLine::error(&self.view, message.clone());
                self.mode = InteractionMode::FetchPreview {
                    fetch: JjGitFetch::default_remotes(),
                    output: ActionPane::finished(
                        FETCH_REMOTE_LIST_COMMAND_LABEL.to_owned(),
                        message,
                        Some(status_context),
                    ),
                };
            }
            FetchRemotePromptDecision::OpenPreview { remote } => {
                self.open_fetch_preview(remote);
            }
            FetchRemotePromptDecision::Prompt { remotes } => {
                self.mode = InteractionMode::FetchRemotePrompt {
                    remotes,
                    selected: 0,
                };
            }
        }
    }
}

/// Decide whether push should fail, prompt, or preview based on discovered remotes.
pub fn decide_push_remote_prompt(
    remotes: std::result::Result<Vec<String>, String>,
) -> PushRemotePromptDecision {
    match remotes {
        Ok(remotes) => match remotes.as_slice() {
            [] => PushRemotePromptDecision::MissingRemotes {
                message: PUSH_NO_REMOTES_MESSAGE.to_owned(),
            },
            [remote] => PushRemotePromptDecision::OpenPreview {
                remote: remote.to_owned(),
            },
            _ => PushRemotePromptDecision::Prompt { remotes },
        },
        Err(message) => PushRemotePromptDecision::RemoteListError { message },
    }
}

/// Decide whether fetch should fail, prompt, or preview based on discovered remotes.
pub fn decide_fetch_remote_prompt(
    remotes: std::result::Result<Vec<String>, String>,
) -> FetchRemotePromptDecision {
    match remotes {
        Ok(remotes) => match remotes.as_slice() {
            [] => FetchRemotePromptDecision::MissingRemotes {
                message: FETCH_NO_REMOTES_MESSAGE.to_owned(),
                status_context: FETCH_NO_REMOTES_CONTEXT.to_owned(),
            },
            [remote] => FetchRemotePromptDecision::OpenPreview {
                remote: remote.to_owned(),
            },
            _ => FetchRemotePromptDecision::Prompt { remotes },
        },
        Err(message) => FetchRemotePromptDecision::RemoteListError {
            message,
            status_context: FETCH_REMOTE_LIST_ERROR_CONTEXT.to_owned(),
        },
    }
}
