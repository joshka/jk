//! App-owned interpretation of view-reported effects.
//!
//! Feature views return `ViewEffect` as intent. This module owns the app-level
//! state transitions that follow: opening stack entries, updating shared
//! status, and choosing modal screens from copy/action results.

use color_eyre::Result;

use crate::app::status_line::StatusLine;
use crate::command::ViewEffect;
use crate::modes::InteractionMode;

use super::App;

impl App {
    /// Interpret a view effect and move the app-owned state it refers to.
    ///
    /// View code reports intent here; this dispatcher owns the resulting mode
    /// changes, status text, and follow-up navigation.
    pub fn apply_view_effect(&mut self, effect: ViewEffect, viewport_height: u16) -> Result<bool> {
        match effect {
            ViewEffect::Ignored | ViewEffect::Handled => Ok(true),
            ViewEffect::StatusMessage(message) => {
                self.status = StatusLine::with_message(&self.view, message);
                Ok(false)
            }
            ViewEffect::StatusError(message) => {
                self.status = StatusLine::error(&self.view, message);
                Ok(false)
            }
            ViewEffect::RunNewTrunk => {
                self.run_new_trunk(viewport_height);
                Ok(false)
            }
            ViewEffect::OpenDetail(command, revset) => {
                self.push_detail(command, revset)?;
                Ok(true)
            }
            ViewEffect::OpenView(spec) => {
                self.push_view(spec)?;
                Ok(true)
            }
            ViewEffect::SearchMoved => {
                if let Some(query) = &self.search {
                    self.status =
                        StatusLine::with_message(&self.view, format!("search: {}", query.text()));
                }
                Ok(false)
            }
            ViewEffect::SearchStarted { matches } => {
                self.status = StatusLine::with_message(&self.view, format!("{matches} matches"));
                Ok(false)
            }
            ViewEffect::OpenActionMenu(menu) => {
                self.mode = InteractionMode::ActionMenu { menu, selected: 0 };
                Ok(false)
            }
            ViewEffect::CopyOptions(options) => {
                if options.is_empty() {
                    self.status = StatusLine::with_message(&self.view, "nothing to copy");
                } else {
                    self.mode = InteractionMode::CopyMenu {
                        options,
                        selected: 0,
                    };
                }
                Ok(false)
            }
        }
    }
}
