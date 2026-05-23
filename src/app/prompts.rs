use color_eyre::Result;
use crossterm::event::KeyCode;

use super::App;
use super::reducers::{
    PromptAcceptDecision, TextPromptKey, reduce_bookmark_name_prompt_accept,
    reduce_bookmark_rename_prompt_accept, reduce_commit_prompt_accept,
    reduce_describe_prompt_accept, reduce_text_prompt_key,
};
use crate::app::status_line::StatusLine;
use crate::modes::InteractionMode;
use crate::search::SearchQuery;

impl App {
    pub fn handle_search_prompt_key(
        &mut self,
        code: KeyCode,
        _viewport_height: u16,
    ) -> Result<bool> {
        let InteractionMode::SearchPrompt(input) = &mut self.mode else {
            unreachable!("search prompt key handler requires search prompt mode");
        };

        match reduce_text_prompt_key(input, code) {
            TextPromptKey::Cancel => self.mode = InteractionMode::Normal,
            TextPromptKey::Accept => {
                self.search = SearchQuery::new(input.clone());
                self.mode = InteractionMode::Normal;
                self.status = if self.search.is_some() {
                    match self.execute_view(crate::command::ViewCommand::StartSearch) {
                        crate::command::ViewEffect::SearchStarted { matches } => {
                            StatusLine::with_message(&self.view, format!("{matches} matches"))
                        }
                        _ => StatusLine::ready(&self.view),
                    }
                } else {
                    StatusLine::ready(&self.view)
                };
            }
            TextPromptKey::Edited | TextPromptKey::Ignored => {}
        }
        Ok(true)
    }

    pub fn handle_log_revset_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::LogRevsetPrompt(input) = &mut self.mode else {
            unreachable!("log revset prompt key handler requires log revset prompt mode");
        };

        match reduce_text_prompt_key(input, code) {
            TextPromptKey::Cancel => self.mode = InteractionMode::Normal,
            TextPromptKey::Accept => {
                let revset = std::mem::take(input);
                self.mode = InteractionMode::Normal;
                self.apply_custom_log_revset(revset);
            }
            TextPromptKey::Edited | TextPromptKey::Ignored => {}
        }
        Ok(true)
    }

    pub fn handle_describe_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::DescribePrompt { target, input } = &mut self.mode else {
            unreachable!("describe prompt key handler requires describe prompt mode");
        };

        match reduce_text_prompt_key(input, code) {
            TextPromptKey::Cancel => {
                self.mode = InteractionMode::Normal;
                self.status = StatusLine::with_message(&self.view, "describe cancelled".to_owned());
            }
            TextPromptKey::Accept => {
                let decision = reduce_describe_prompt_accept(target, input);
                self.apply_text_prompt_accept_decision(decision, Self::open_describe_preview);
            }
            TextPromptKey::Edited | TextPromptKey::Ignored => {}
        }
        Ok(true)
    }

    pub fn handle_commit_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::CommitPrompt(input) = &mut self.mode else {
            unreachable!("commit prompt key handler requires commit prompt mode");
        };

        match reduce_text_prompt_key(input, code) {
            TextPromptKey::Cancel => {
                self.mode = InteractionMode::Normal;
                self.status = StatusLine::with_message(&self.view, "commit cancelled".to_owned());
            }
            TextPromptKey::Accept => {
                let decision = reduce_commit_prompt_accept(input);
                self.apply_text_prompt_accept_decision(decision, Self::open_commit_preview);
            }
            TextPromptKey::Edited | TextPromptKey::Ignored => {}
        }
        Ok(true)
    }

    pub fn handle_bookmark_name_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::BookmarkNamePrompt {
            kind,
            target,
            input,
        } = &mut self.mode
        else {
            unreachable!("bookmark name prompt key handler requires bookmark name prompt mode");
        };

        match reduce_text_prompt_key(input, code) {
            TextPromptKey::Cancel => {
                let kind = *kind;
                self.mode = InteractionMode::Normal;
                self.status = StatusLine::with_message(
                    &self.view,
                    format!("bookmark {} cancelled", kind.label()),
                );
            }
            TextPromptKey::Accept => {
                let decision = reduce_bookmark_name_prompt_accept(*kind, target, input);
                self.apply_text_prompt_accept_decision(
                    decision,
                    Self::open_bookmark_mutation_preview,
                );
            }
            TextPromptKey::Edited | TextPromptKey::Ignored => {}
        }
        Ok(true)
    }

    pub fn handle_bookmark_rename_prompt_key(&mut self, code: KeyCode) -> Result<bool> {
        let InteractionMode::BookmarkRenamePrompt { old_name, input } = &mut self.mode else {
            unreachable!("bookmark rename prompt key handler requires bookmark rename prompt mode");
        };

        match reduce_text_prompt_key(input, code) {
            TextPromptKey::Cancel => {
                self.mode = InteractionMode::Normal;
                self.status = StatusLine::with_message(&self.view, "bookmark rename cancelled");
            }
            TextPromptKey::Accept => {
                let decision = reduce_bookmark_rename_prompt_accept(old_name, input);
                self.apply_text_prompt_accept_decision(
                    decision,
                    Self::open_bookmark_mutation_preview,
                );
            }
            TextPromptKey::Edited | TextPromptKey::Ignored => {}
        }
        Ok(true)
    }

    fn apply_text_prompt_accept_decision<T>(
        &mut self,
        decision: PromptAcceptDecision<T>,
        open_preview: impl FnOnce(&mut Self, T),
    ) {
        self.mode = InteractionMode::Normal;

        match decision {
            PromptAcceptDecision::Preview(plan) => open_preview(self, plan),
            PromptAcceptDecision::StatusMessage(message) => {
                self.status = StatusLine::with_message(&self.view, message);
            }
        }
    }
}
