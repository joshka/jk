use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::jj::{exact_string_pattern, run_direct_args};

use super::targets::{JjBookmarkForgetTarget, JjBookmarkTarget, JjBookmarkTrackingTarget};

// Bookmark mutation plans keep local-name changes, remote tracking metadata,
// and forget/delete semantics in one exact-pattern command family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JjBookmarkMutationKind {
    /// Create one local bookmark at the target revision.
    Create,
    /// Set one local bookmark to the target revision.
    Set,
    /// Move one exactly named local bookmark to the target revision.
    Move,
    /// Rename one local bookmark.
    Rename,
    /// Delete one exactly named local bookmark.
    Delete,
    /// Forget one bookmark, optionally including the exact remote peer.
    Forget,
    /// Track one exact remote bookmark.
    Track,
    /// Untrack one exact remote bookmark.
    Untrack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjBookmarkMutationPlan {
    /// Bookmark mutation subcommand owned by this plan.
    kind: JjBookmarkMutationKind,
    /// Primary bookmark name supplied by the caller.
    name: String,
    /// New bookmark name for rename plans.
    new_name: Option<String>,
    /// Revision target for create, set, and move plans.
    target: Option<JjBookmarkTarget>,
    /// Forget-target policy for forget plans.
    forget_target: Option<JjBookmarkForgetTarget>,
    /// Exact remote tracking target for track and untrack plans.
    tracking_target: Option<Box<JjBookmarkTrackingTarget>>,
}

impl JjBookmarkMutationKind {
    /// Returns the user-facing action label for this mutation kind.
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

impl JjBookmarkMutationPlan {
    /// Builds a create plan for one bookmark name and revision target.
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

    /// Builds a set plan for one bookmark name and revision target.
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

    /// Builds a move plan for one bookmark name and revision target.
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

    /// Builds a rename plan for one local bookmark.
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

    /// Builds a delete plan for one exactly named local bookmark.
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

    /// Builds a forget plan from a selected bookmark row and resolved forget target.
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

    /// Builds a track plan from a resolved remote bookmark target.
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

    /// Builds an untrack plan from a resolved remote bookmark target.
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
                    target.remote_pattern(),
                    target.bookmark_pattern(),
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
                    format!("remote pattern: {}", target.remote_pattern()),
                    format!("bookmark pattern: {}", target.bookmark_pattern()),
                    format!("visible state: {}", target.visible_state()),
                    tracking_effect(self.kind),
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

fn tracking_effect(kind: JjBookmarkMutationKind) -> String {
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
