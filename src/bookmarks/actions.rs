//! Bookmark action plans and validation.
//!
//! This module owns bookmark mutation argv construction, exact-name quoting,
//! preview summaries, and rename validation for the bookmarks feature. App
//! prompt policy stays in the action lifecycle; selected-row target resolution
//! and bookmark row metadata stay in sibling bookmark modules.

use color_eyre::Result;

use crate::jj::run_direct_args;
use crate::jj_actions::CommandOutput;
use crate::jj_syntax::{exact_change_id_revset, exact_string_pattern};

// Bookmark mutation plans keep local-name changes, remote tracking metadata,
// and forget/delete semantics in one exact-pattern command family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjBookmarkMutationKind {
    Create,
    Set,
    Move,
    Rename,
    Delete,
    Forget,
    Track,
    Untrack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjBookmarkTarget {
    ExactChange(String),
    CurrentWorkingCopy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjBookmarkForgetTarget {
    Local { tracking: String },
    RemoteOnly { remote: String, tracking: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjBookmarkTrackingTarget {
    local_bookmark: Option<String>,
    remote_bookmark: String,
    remote: String,
    visible_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjBookmarkMutationPlan {
    kind: JjBookmarkMutationKind,
    name: String,
    new_name: Option<String>,
    target: Option<JjBookmarkTarget>,
    forget_target: Option<JjBookmarkForgetTarget>,
    tracking_target: Option<Box<JjBookmarkTrackingTarget>>,
}

// Bookmark mutation owns all bookmark subcommand argv so exact-name matching,
// tracking metadata, and preview wording stay consistent.
impl JjBookmarkMutationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Set => "set",
            Self::Move => "move",
            Self::Rename => "rename",
            Self::Delete => "delete",
            Self::Forget => "forget",
            Self::Track => "track",
            Self::Untrack => "untrack",
        }
    }

    fn success_fallback(self) -> &'static str {
        match self {
            Self::Create => "created bookmark",
            Self::Set => "set bookmark",
            Self::Move => "moved bookmark",
            Self::Rename => "renamed bookmark",
            Self::Delete => "deleted bookmark",
            Self::Forget => "forgot bookmark",
            Self::Track => "tracked bookmark",
            Self::Untrack => "untracked bookmark",
        }
    }
}

impl JjBookmarkTarget {
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    fn command_arg(&self) -> String {
        match self {
            Self::ExactChange(change_id) => exact_change_id_revset(change_id),
            Self::CurrentWorkingCopy => "@".to_owned(),
        }
    }

    fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => format!("exact selected revision {change_id}"),
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }
}

impl JjBookmarkMutationPlan {
    pub fn create(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Create,
            name: name.into(),
            new_name: None,
            target: Some(target),
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn set(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Set,
            name: name.into(),
            new_name: None,
            target: Some(target),
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn move_to(name: impl Into<String>, target: JjBookmarkTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Move,
            name: name.into(),
            new_name: None,
            target: Some(target),
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn rename(old_name: impl Into<String>, new_name: impl Into<String>) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Rename,
            name: old_name.into(),
            new_name: Some(new_name.into()),
            target: None,
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn delete(name: impl Into<String>) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Delete,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: None,
            tracking_target: None,
        }
    }

    pub fn forget(name: impl Into<String>, target: JjBookmarkForgetTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Forget,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: Some(target),
            tracking_target: None,
        }
    }

    pub fn track(name: impl Into<String>, target: JjBookmarkTrackingTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Track,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: None,
            tracking_target: Some(Box::new(target)),
        }
    }

    pub fn untrack(name: impl Into<String>, target: JjBookmarkTrackingTarget) -> Self {
        Self {
            kind: JjBookmarkMutationKind::Untrack,
            name: name.into(),
            new_name: None,
            target: None,
            forget_target: None,
            tracking_target: Some(Box::new(target)),
        }
    }

    pub fn kind(&self) -> JjBookmarkMutationKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new_name(&self) -> Option<&str> {
        self.new_name.as_deref()
    }

    pub fn target(&self) -> Option<&JjBookmarkTarget> {
        self.target.as_ref()
    }

    pub fn command_label(&self) -> String {
        let label_args = self
            .command_argv()
            .iter()
            .map(|arg| arg.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        format!("jj {label_args}")
    }

    pub fn command_argv(&self) -> Vec<String> {
        match self.kind {
            JjBookmarkMutationKind::Create => vec![
                "bookmark".to_owned(),
                "create".to_owned(),
                "--revision".to_owned(),
                self.required_target().command_arg(),
                self.name.clone(),
            ],
            JjBookmarkMutationKind::Set => vec![
                "bookmark".to_owned(),
                "set".to_owned(),
                "--revision".to_owned(),
                self.required_target().command_arg(),
                self.name.clone(),
            ],
            JjBookmarkMutationKind::Move => vec![
                "bookmark".to_owned(),
                "move".to_owned(),
                "--to".to_owned(),
                self.required_target().command_arg(),
                exact_string_pattern(&self.name),
            ],
            JjBookmarkMutationKind::Rename => vec![
                "bookmark".to_owned(),
                "rename".to_owned(),
                self.name.clone(),
                self.required_new_name().to_owned(),
            ],
            JjBookmarkMutationKind::Delete => vec![
                "bookmark".to_owned(),
                "delete".to_owned(),
                exact_string_pattern(&self.name),
            ],
            JjBookmarkMutationKind::Forget => {
                let mut argv = vec!["bookmark".to_owned(), "forget".to_owned()];
                if self.required_forget_target().include_remotes() {
                    argv.push("--include-remotes".to_owned());
                }
                argv.push(exact_string_pattern(&self.name));
                argv
            }
            JjBookmarkMutationKind::Track | JjBookmarkMutationKind::Untrack => {
                let target = self.required_tracking_target();
                vec![
                    "bookmark".to_owned(),
                    self.kind.label().to_owned(),
                    "--remote".to_owned(),
                    exact_string_pattern(target.remote()),
                    exact_string_pattern(target.remote_bookmark()),
                ]
            }
        }
    }

    pub fn run_preview(&self) -> Result<CommandOutput> {
        Ok(CommandOutput::new(self.preview_summary()))
    }

    pub fn run(&self) -> Result<CommandOutput> {
        run_direct_args(
            self.command_argv(),
            &self.command_label(),
            self.kind.success_fallback(),
        )
    }

    pub fn preview_summary(&self) -> String {
        let mut lines = vec![
            format!("command: {}", self.command_label()),
            String::new(),
            format!("bookmark: {}", self.name),
        ];

        match self.kind {
            JjBookmarkMutationKind::Create => {
                lines.extend([
                    "source/current: new local bookmark name".to_owned(),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: creates one local bookmark at the exact destination target".to_owned(),
                    "confirmation: press Enter to run jj bookmark create".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Set => {
                lines.extend([
                    "source/current: jj resolves the literal local bookmark name".to_owned(),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: sets one local bookmark to the exact destination target".to_owned(),
                    "confirmation: press Enter to run jj bookmark set".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Move => {
                lines.extend([
                    format!(
                        "source/current: exact pattern {}",
                        exact_string_pattern(&self.name)
                    ),
                    format!("destination: {}", self.required_target().preview_target()),
                    "effect: moves one exactly named local bookmark to the destination target"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark move".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Rename => {
                lines.extend([
                    format!("old name: {}", self.name),
                    format!("new name: {}", self.required_new_name()),
                    "target: exact selected local bookmark row; rendered labels are not parsed"
                        .to_owned(),
                    "effect: renames one local bookmark without --overwrite-existing".to_owned(),
                    "duplicate name: jj failure output is preserved if the new name already exists"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark rename".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Delete => {
                lines.extend([
                    format!(
                        "source/current: exact pattern {}",
                        exact_string_pattern(&self.name)
                    ),
                    "destination: none".to_owned(),
                    "effect: deletes one local bookmark; this is delete, not forget".to_owned(),
                    "tracking: track/untrack stay disabled until exact tracking metadata exists"
                        .to_owned(),
                    "confirmation: press Enter to run jj bookmark delete".to_owned(),
                    "undo path: jj undo".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Forget => {
                let target = self.required_forget_target();
                lines.extend([
                    format!(
                        "target: exact bookmark {}",
                        exact_string_pattern(&self.name)
                    ),
                    format!("visible state: {}", target.visible_state()),
                    format!("scope: {}", target.scope_summary()),
                    "effect: forgets tracking relationship metadata; this is forget, not delete"
                        .to_owned(),
                    "output: full jj failure output remains inspectable in this pane".to_owned(),
                    "confirmation: press Enter to run jj bookmark forget".to_owned(),
                    "recovery: jj undo; review: jj op show -p".to_owned(),
                ]);
            }
            JjBookmarkMutationKind::Track | JjBookmarkMutationKind::Untrack => {
                let target = self.required_tracking_target();
                lines.extend([
                    format!("local bookmark: {}", target.local_bookmark_label()),
                    format!("remote bookmark: {}", target.remote_bookmark()),
                    format!("remote: {}", target.remote()),
                    format!("remote pattern: {}", exact_string_pattern(target.remote())),
                    format!(
                        "bookmark pattern: {}",
                        exact_string_pattern(target.remote_bookmark())
                    ),
                    format!("visible state: {}", target.visible_state()),
                    target.effect(self.kind),
                    "output: full jj result or failure output remains inspectable in this pane"
                        .to_owned(),
                    format!(
                        "confirmation: press Enter to run jj bookmark {}",
                        self.kind.label()
                    ),
                    "recovery: jj undo; review: jj op show -p".to_owned(),
                ]);
            }
        }

        lines.join("\n")
    }

    fn required_target(&self) -> &JjBookmarkTarget {
        self.target
            .as_ref()
            .expect("bookmark mutation kind requires target")
    }

    fn required_new_name(&self) -> &str {
        self.new_name
            .as_deref()
            .expect("bookmark rename requires new name")
    }

    fn required_forget_target(&self) -> &JjBookmarkForgetTarget {
        self.forget_target
            .as_ref()
            .expect("bookmark forget requires a forget target")
    }

    fn required_tracking_target(&self) -> &JjBookmarkTrackingTarget {
        self.tracking_target
            .as_deref()
            .expect("bookmark track/untrack requires a tracking target")
    }
}

impl JjBookmarkForgetTarget {
    pub fn local(tracking: impl Into<String>) -> Self {
        Self::Local {
            tracking: tracking.into(),
        }
    }

    pub fn remote_only(remote: impl Into<String>, tracking: impl Into<String>) -> Self {
        Self::RemoteOnly {
            remote: remote.into(),
            tracking: tracking.into(),
        }
    }

    fn include_remotes(&self) -> bool {
        matches!(self, Self::RemoteOnly { .. })
    }

    fn visible_state(&self) -> String {
        match self {
            Self::Local { tracking } => format!("local bookmark; {tracking}"),
            Self::RemoteOnly { remote, tracking } => {
                format!("remote-only bookmark on {remote}; {tracking}")
            }
        }
    }

    fn scope_summary(&self) -> &'static str {
        match self {
            Self::Local { .. } => "local tracked bookmark or local bookmark with remote peer",
            Self::RemoteOnly { .. } => "one remote peer and no local peer; includes remotes",
        }
    }
}

impl JjBookmarkTrackingTarget {
    pub fn new(
        local_bookmark: Option<String>,
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self {
            local_bookmark,
            remote_bookmark: remote_bookmark.into(),
            remote: remote.into(),
            visible_state: visible_state.into(),
        }
    }

    pub fn local(
        local_bookmark: impl Into<String>,
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self::new(
            Some(local_bookmark.into()),
            remote_bookmark,
            remote,
            visible_state,
        )
    }

    pub fn remote_only(
        remote_bookmark: impl Into<String>,
        remote: impl Into<String>,
        visible_state: impl Into<String>,
    ) -> Self {
        Self::new(None, remote_bookmark, remote, visible_state)
    }

    pub fn remote_bookmark(&self) -> &str {
        &self.remote_bookmark
    }

    pub fn remote(&self) -> &str {
        &self.remote
    }

    pub fn visible_state(&self) -> &str {
        &self.visible_state
    }

    fn local_bookmark_label(&self) -> &str {
        self.local_bookmark.as_deref().unwrap_or("absent")
    }

    fn effect(&self, kind: JjBookmarkMutationKind) -> String {
        match kind {
            JjBookmarkMutationKind::Track => {
                "effect: tracks the exact remote bookmark for the exact local bookmark; this does not fetch, push, delete, or rename".to_owned()
            }
            JjBookmarkMutationKind::Untrack => {
                "effect: untracks the exact remote bookmark relationship; this does not delete the local or remote bookmark".to_owned()
            }
            JjBookmarkMutationKind::Create
            | JjBookmarkMutationKind::Set
            | JjBookmarkMutationKind::Move
            | JjBookmarkMutationKind::Rename
            | JjBookmarkMutationKind::Delete
            | JjBookmarkMutationKind::Forget => {
                unreachable!("tracking target effects only apply to track/untrack")
            }
        }
    }
}

pub fn validate_bookmark_rename_new_name(
    old_name: &str,
    new_name: &str,
) -> std::result::Result<(), String> {
    if new_name.is_empty() {
        return Err("empty bookmark name".to_owned());
    }
    if new_name == old_name {
        return Err("new bookmark name is unchanged".to_owned());
    }
    if new_name == "@" {
        return Err("bookmark name must not be @".to_owned());
    }
    if new_name.starts_with('-') {
        return Err("bookmark name must not start with '-'".to_owned());
    }
    if new_name.starts_with('/') || new_name.ends_with('/') || new_name.contains("//") {
        return Err("bookmark name must not contain empty path components".to_owned());
    }
    if new_name.starts_with('.') || new_name.contains("/.") {
        return Err("bookmark name components must not start with '.'".to_owned());
    }
    if new_name.ends_with('.') || new_name.ends_with(".lock") {
        return Err("bookmark name must not end with '.' or '.lock'".to_owned());
    }
    if new_name.contains("..") {
        return Err("bookmark name must not contain '..'".to_owned());
    }
    if new_name
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err("bookmark name must not contain whitespace or control characters".to_owned());
    }
    if new_name
        .chars()
        .any(|character| matches!(character, '@' | ':' | '?' | '*' | '[' | '\\' | '^' | '~'))
    {
        return Err("bookmark name contains a reserved ref character".to_owned());
    }

    Ok(())
}

#[cfg(test)]
mod tests;
