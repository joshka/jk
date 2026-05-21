//! Action-menu presentation models for graph, status, file, bookmark, sync,
//! and operation surfaces.
//!
//! This module owns the user-visible action vocabulary, prompts, and follow-up
//! context used to present those actions. Execution still happens elsewhere.

mod path_actions;
mod revision_actions;

use crate::jj_actions::JjFileChmodMode;
pub use revision_actions::ExactActionContext;

const PREVIEW_REQUIRED_MARKER: &str = "Preview required before execution.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyTier {
    PreviewFirst,
}

impl SafetyTier {
    #[cfg(test)]
    pub fn is_preview_first(&self) -> bool {
        matches!(self, Self::PreviewFirst)
    }

    pub fn preview_marker(&self) -> &'static str {
        PREVIEW_REQUIRED_MARKER
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionKind {
    Edit,
    New,
    Split,
    Duplicate,
    Abandon,
    Restore,
    Revert,
    Rebase,
    Squash,
    Absorb,
    FileTrack,
    FileUntrack,
    FileChmodExecutable,
    FileChmodNormal,
}

impl ActionKind {
    fn label(self) -> &'static str {
        match self {
            Self::Edit => "edit",
            Self::New => "new",
            Self::Split => "split",
            Self::Duplicate => "duplicate",
            Self::Abandon => "abandon",
            Self::Restore => "restore",
            Self::Revert => "revert",
            Self::Rebase => "rebase",
            Self::Squash => "squash",
            Self::Absorb => "absorb",
            Self::FileTrack => "track",
            Self::FileUntrack => "untrack",
            Self::FileChmodExecutable => "chmod x",
            Self::FileChmodNormal => "chmod n",
        }
    }

    fn shortcut(self) -> char {
        match self {
            Self::Edit => 'e',
            Self::New => 'n',
            Self::Split => 's',
            Self::Duplicate => 'd',
            Self::Abandon => 'x',
            Self::Restore => 'r',
            Self::Revert => 'v',
            Self::Rebase => 'b',
            Self::Squash => 'u',
            Self::Absorb => 'a',
            Self::FileTrack => 't',
            Self::FileUntrack => 'u',
            Self::FileChmodExecutable => 'x',
            Self::FileChmodNormal => 'n',
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePromptOption {
    role: &'static str,
    value: String,
}

impl RolePromptOption {
    pub fn new(role: &'static str, value: impl Into<String>) -> Self {
        Self {
            role,
            value: value.into(),
        }
    }

    pub fn role(&self) -> &'static str {
        self.role
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.role, self.value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePrompt {
    title: &'static str,
    options: Vec<RolePromptOption>,
    preview_required_message: &'static str,
}

impl RolePrompt {
    pub fn new(
        title: &'static str,
        options: Vec<RolePromptOption>,
        preview_required_message: &'static str,
    ) -> Self {
        Self {
            title,
            options,
            preview_required_message,
        }
    }

    pub fn title(&self) -> &str {
        self.title
    }

    pub fn options(&self) -> &[RolePromptOption] {
        &self.options
    }

    pub fn preview_required_message(&self) -> &str {
        self.preview_required_message
    }

    pub fn status_message(&self) -> String {
        let mut lines = self
            .options
            .iter()
            .map(RolePromptOption::label)
            .collect::<Vec<_>>();
        lines.push(self.preview_required_message.to_owned());
        lines.join("\n")
    }

    pub fn source_revisions(&self) -> Vec<&str> {
        self.options
            .iter()
            .filter(|option| option.role() == "source")
            .map(RolePromptOption::value)
            .collect()
    }

    pub fn destination_revision(&self) -> Option<&str> {
        self.options
            .iter()
            .find(|option| option.role() == "destination")
            .map(RolePromptOption::value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FollowUp {
    StatusMessage(String),
    ExactRevision {
        revision: String,
    },
    SplitExactTarget {
        revision: String,
    },
    SplitCurrentWorkingCopy,
    DuplicateExactTarget {
        revision: String,
    },
    EditExactTarget {
        revision: String,
    },
    RestoreExactTarget {
        revision: String,
        path: Option<String>,
    },
    RestoreWorkingCopyPath {
        path: String,
    },
    RevertExactTarget {
        revision: String,
    },
    OperationRestoreExactTarget {
        operation_id: String,
    },
    OperationRevertExactTarget {
        operation_id: String,
    },
    NewParents {
        parents: Vec<String>,
    },
    RolePrompt(RolePrompt),
    AbsorbCandidates {
        source: String,
        destinations: Vec<String>,
    },
    FileTrack {
        path: String,
    },
    FileUntrack {
        path: String,
    },
    FileChmod {
        path: String,
        revision: Option<String>,
        mode: JjFileChmodMode,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionMenuItem {
    action: ActionKind,
    shortcut: char,
    label: String,
    safety_tier: SafetyTier,
    follow_up: FollowUp,
}

impl ActionMenuItem {
    pub fn new(
        action: ActionKind,
        label: impl Into<String>,
        safety_tier: SafetyTier,
        follow_up: FollowUp,
    ) -> Self {
        Self {
            action,
            shortcut: action.shortcut(),
            label: label.into(),
            safety_tier,
            follow_up,
        }
    }

    pub fn action(&self) -> ActionKind {
        self.action
    }

    pub fn shortcut(&self) -> char {
        self.shortcut
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn safety_tier(&self) -> SafetyTier {
        self.safety_tier
    }

    pub fn follow_up(&self) -> &FollowUp {
        &self.follow_up
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionMenu {
    items: Vec<ActionMenuItem>,
}

impl ActionMenu {
    pub fn new(items: Vec<ActionMenuItem>) -> Self {
        Self { items }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &[ActionMenuItem] {
        &self.items
    }

    pub fn item_for_shortcut(&self, shortcut: char) -> Option<&ActionMenuItem> {
        self.items.iter().find(|item| item.shortcut() == shortcut)
    }
}

pub fn build_action_menu(context: &ExactActionContext) -> ActionMenu {
    revision_actions::build_action_menu(context)
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}
