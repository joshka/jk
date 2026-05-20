//! Action lifecycle orchestration for app-owned mutation screens.
//!
//! Modal key dispatch stays in `mode_input`. The lifecycle modules own selected action-menu
//! items, prompt-to-preview setup, immediate actions, and confirmed jj action result handling.

mod completion;
mod entry;
mod preview;
mod rewrite_completion;
mod shared;
