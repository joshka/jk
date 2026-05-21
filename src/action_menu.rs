//! Action-menu presentation models for graph, status, file, bookmark, sync,
//! and operation surfaces.
//!
//! This module owns only shared menu contracts: the stable action vocabulary,
//! safety marker text, role-prompt presentation state, and follow-up payloads
//! handed back after a selection. Feature roots and their builders decide which
//! actions are available for the current row or path context. The app action
//! lifecycle and `jj_actions` own preview construction, process execution, and
//! any refresh or reveal behavior after a command completes.

mod path_actions;
mod revision_actions;

use crate::jj_actions::JjFileChmodMode;
pub use revision_actions::ExactActionContext;

const PREVIEW_REQUIRED_MARKER: &str = "Preview required before execution.";

/// Safety policy shown before a menu action can mutate repository state.
///
/// The menu only advertises the requirement. Preview construction, command execution, and
/// post-command refresh stay in the app action lifecycle and `jj_actions`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyTier {
    PreviewFirst,
}

impl SafetyTier {
    #[cfg(test)]
    pub fn is_preview_first(&self) -> bool {
        matches!(self, Self::PreviewFirst)
    }

    /// User-facing marker appended to action menus and role prompts.
    pub fn preview_marker(&self) -> &'static str {
        PREVIEW_REQUIRED_MARKER
    }
}

/// Stable action vocabulary shared by menus, prompts, and follow-up dispatch.
///
/// This enum names user-visible verbs only. Feature-specific availability rules belong in the
/// feature or action-menu builder that knows the selected row context. Labels and shortcuts are part
/// of the shared presentation contract, but they are not command construction policy.
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

/// One role/value pair in an action prompt that needs an explicit source or destination choice.
///
/// Roles are presentation labels and dispatcher cues, not parsed revsets. The follow-up action plan
/// is responsible for quoting selected values before passing them to `jj`. Values are the exact
/// revision strings selected by the builder, so callers should not normalize them while the prompt is
/// open.
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

/// Prompt model for actions that need a role choice before preview.
///
/// The prompt is immutable UI state owned by `InteractionMode`; choosing an option only creates the
/// next follow-up, and never executes `jj` directly. The role names currently consumed by app
/// reducers are `"source"` and `"destination"`; additional role semantics belong with the reducer
/// that turns a chosen prompt into a preview plan.
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

/// Deferred action payload produced by a selected menu item.
///
/// Follow-ups intentionally carry exact strings from rendered row metadata or selected paths. The
/// app turns them into preview-first `jj_actions` plans before any process side effects occur. Keep
/// payloads to the metadata needed to construct that plan: exact revision strings, operation ids,
/// selected paths, role prompts, candidate lists, and chmod modes. UI selection state, command
/// preview text, post-command status, refresh policy, and reveal targets belong in the app lifecycle
/// or `jj_actions`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FollowUp {
    /// User-visible terminal payload when a builder cannot form a safe mutation target.
    StatusMessage(String),
    /// Revision-oriented payloads carry the exact target selected by a feature row.
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
    /// New-change payloads keep the exact parent list selected upstream.
    NewParents {
        parents: Vec<String>,
    },
    /// Multi-target rewrite payloads preserve candidate ordering for the next app prompt or plan.
    RolePrompt(RolePrompt),
    /// Absorb payloads preserve the chosen source and destination ordering.
    AbsorbCandidates {
        source: String,
        destinations: Vec<String>,
    },
    /// Path payloads carry the selected fileset string and, when needed, its revision context.
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

/// One selectable row in an action menu.
///
/// Items are pure presentation and dispatch data: label, shortcut, safety marker, and follow-up.
/// They do not know whether the selected action is valid after a later refresh. Builders should
/// attach only the metadata needed by `FollowUp`; any validation that depends on current repository
/// state happens when the app constructs or executes the preview plan.
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

/// Immutable action menu for the currently selected view item.
///
/// Builders own action availability. The shared menu type only preserves item order and shortcut
/// lookup for modal input. Rendering, selected-index clamping, and accepted-selection behavior are
/// app and TUI responsibilities.
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

/// Build the shared revision action menu for an exact graph/detail context.
///
/// This facade exists so callers do not depend on the current staging module. New
/// feature-specific action policy should move toward that feature owner instead of growing this
/// shared wrapper.
pub fn build_action_menu(context: &ExactActionContext) -> ActionMenu {
    revision_actions::build_action_menu(context)
}

fn short_id(id: &str) -> &str {
    id.get(..8).unwrap_or(id)
}
