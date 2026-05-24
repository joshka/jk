use color_eyre::Result;

use crate::actions::CommandOutput;
use crate::bookmarks::actions::targets::{
    JjBookmarkForgetTarget, JjBookmarkTarget, JjBookmarkTrackingTarget,
};
use crate::jj::run_direct_args;

mod argv;
mod preview;

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

    pub fn success_fallback(self) -> &'static str {
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

    pub fn required_target(&self) -> &JjBookmarkTarget {
        self.target
            .as_ref()
            .expect("bookmark mutation kind requires target")
    }

    pub fn required_new_name(&self) -> &str {
        self.new_name
            .as_deref()
            .expect("bookmark rename requires new name")
    }

    pub fn required_forget_target(&self) -> &JjBookmarkForgetTarget {
        self.forget_target
            .as_ref()
            .expect("bookmark forget requires a forget target")
    }

    pub fn required_tracking_target(&self) -> &JjBookmarkTrackingTarget {
        self.tracking_target
            .as_deref()
            .expect("bookmark track/untrack requires a tracking target")
    }
}
