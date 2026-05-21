//! App orchestration tests split by behavior group.
//!
//! Test support lives in `support`; child modules own focused app behavior contracts instead of
//! collecting every lifecycle assertion in one grab bag.

mod abandon_actions;
mod bookmark_actions;
mod command_navigation;
mod describe_commit_actions;
mod detail_restore_actions;
mod file_actions;
mod operation_actions;
mod rewrite_actions;
mod support;
mod sync_actions;
mod working_copy_actions;
