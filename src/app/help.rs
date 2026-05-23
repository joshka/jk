use std::time::Instant;

use color_eyre::Result;
use crossterm::event::KeyEvent;

use super::keyboard::{COMMAND_PREFIX_TIMEOUT, prefix_status_message};
use super::reducers::{is_help_close_key, is_help_scroll_key};
use super::{APP_BINDINGS, App, PendingCommand};
use crate::app::status_line::StatusLine;
use crate::command::{
    Binding, BindingMatch, help_binding_prefix_next_labels, match_help_binding_sequence,
};
use crate::modes::InteractionMode;

impl App {
    /// Handle keys while the help overlay is active, including help-specific prefix matching.
    pub fn handle_help_key(&mut self, key: KeyEvent, viewport_height: u16) -> Result<bool> {
        if is_help_close_key(key) {
            self.pending_command = None;
            self.mode = InteractionMode::Normal;
            return Ok(true);
        }
        if is_help_scroll_key(key) {
            return Ok(true);
        }

        if self.pending_command.is_some() {
            return self.handle_pending_help_key(key, viewport_height, Instant::now());
        }

        let keys = [key];
        let context = self.view.help_context();
        let Some(binding_match) =
            match_help_binding_sequence(&[APP_BINDINGS, self.view.bindings()], &keys, context)
        else {
            self.status = StatusLine::with_message(&self.view, "not available from help menu");
            return Ok(true);
        };

        match binding_match {
            BindingMatch::Exact(binding) => {
                self.execute_help_binding(binding, viewport_height)?;
                Ok(true)
            }
            BindingMatch::Prefix { fallback } => {
                self.pending_command = Some(PendingCommand {
                    keys: keys.to_vec(),
                    fallback,
                    deadline: Instant::now() + COMMAND_PREFIX_TIMEOUT,
                });
                self.status = StatusLine::with_message(
                    &self.view,
                    prefix_status_message(
                        "help",
                        &keys,
                        &help_binding_prefix_next_labels(
                            &[APP_BINDINGS, self.view.bindings()],
                            &keys,
                            context,
                        ),
                    ),
                );
                Ok(true)
            }
        }
    }

    /// Continue a multi-key help binding after the help overlay has already claimed the first key.
    fn handle_pending_help_key(
        &mut self,
        key: KeyEvent,
        viewport_height: u16,
        now: Instant,
    ) -> Result<bool> {
        if self
            .pending_command
            .as_ref()
            .is_some_and(|pending| now >= pending.deadline)
        {
            self.run_pending_fallback(viewport_height)?;
            return self.handle_key_after_prefix_fallback(key, viewport_height, now);
        }

        let Some(mut pending) = self.pending_command.take() else {
            return Ok(true);
        };
        pending.keys.push(key);

        match match_help_binding_sequence(
            &[APP_BINDINGS, self.view.bindings()],
            &pending.keys,
            self.view.help_context(),
        ) {
            Some(BindingMatch::Exact(binding)) => {
                self.execute_help_binding(binding, viewport_height)?;
            }
            Some(BindingMatch::Prefix { fallback }) => {
                self.status = StatusLine::with_message(
                    &self.view,
                    prefix_status_message(
                        "help",
                        &pending.keys,
                        &help_binding_prefix_next_labels(
                            &[APP_BINDINGS, self.view.bindings()],
                            &pending.keys,
                            self.view.help_context(),
                        ),
                    ),
                );
                self.pending_command = Some(PendingCommand {
                    keys: pending.keys,
                    fallback,
                    deadline: now + COMMAND_PREFIX_TIMEOUT,
                });
            }
            None => {
                let Some(fallback) = pending.fallback else {
                    self.status =
                        StatusLine::with_message(&self.view, "unknown help command prefix");
                    return Ok(true);
                };

                self.execute_help_binding(fallback, viewport_height)?;
                return self.handle_key_after_prefix_fallback(key, viewport_height, now);
            }
        }

        Ok(true)
    }

    /// Leave help mode and execute the chosen binding through the normal app root path.
    pub fn execute_help_binding(&mut self, binding: Binding, viewport_height: u16) -> Result<()> {
        self.pending_command = None;
        self.mode = InteractionMode::Normal;
        self.run_binding_with_status_refresh(binding, viewport_height)
    }
}
