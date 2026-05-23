//! Action lifecycle orchestration for app-owned mutation screens.
//!
//! Modal key dispatch stays in `input`. The lifecycle modules own selected action-menu
//! items, prompt-to-preview setup, immediate actions, and confirmed jj action result handling.
//! They call `AppServices` directly for jj/view effects and leave only current-view coupling on
//! `App` itself. Treat this root as a table of contents for the action subtree: `entry` accepts
//! chosen actions, `pane` owns preview/result overlay state, `preview` opens panes, `input`
//! reduces shared preview keys, `completion` runs confirmed commands, and `shared` keeps only
//! wording plus refresh/reveal helpers used by multiple lifecycle stages.

mod completion;
mod entry;
mod input;
mod pane;
mod preview;
mod shared;

pub use pane::{ActionPane, ActionPaneKey, action_pane_visible_lines, handle_action_pane_key};
