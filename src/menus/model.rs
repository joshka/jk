//! Shared action-menu presentation models.
//!
//! This module owns the stable action vocabulary, safety marker text,
//! role-prompt presentation state, and follow-up payloads handed back after a
//! selection. Feature roots and their builders decide which actions are
//! available for the current row or path context.

use crate::actions::JjFileChmodMode;

pub(in crate::menus) const PREVIEW_REQUIRED_MARKER: &str = "Preview required before execution.";

/// Safety policy shown before a menu action can mutate repository state.
///
/// The menu only advertises the requirement. Preview construction, command execution, and
/// post-command refresh stay in the app action lifecycle and `actions`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyTier {
    /// The action must show a preview before any repository mutation can run.
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
    /// Edit the selected revision into the working copy.
    Edit,
    /// Create a new child or merge child from the selected parent set.
    New,
    /// Split the selected revision or current working-copy change.
    Split,
    /// Duplicate the selected revision.
    Duplicate,
    /// Abandon the selected revision.
    Abandon,
    /// Restore a selected revision or path.
    Restore,
    /// Revert the selected revision or operation target.
    Revert,
    /// Rebase selected source revisions into a destination revision.
    Rebase,
    /// Squash selected source revisions into a destination revision.
    Squash,
    /// Absorb selected source revisions into destination descendants.
    Absorb,
    /// Start tracking the selected path.
    FileTrack,
    /// Stop tracking the selected path.
    FileUntrack,
    /// Mark the selected path executable.
    FileChmodExecutable,
    /// Mark the selected path non-executable.
    FileChmodNormal,
}

impl ActionKind {
    pub(super) fn label(self) -> &'static str {
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

    pub(super) fn shortcut(self) -> char {
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
    /// Presentation label naming how the selected revision will be used.
    role: &'static str,
    /// Exact revision string preserved for later preview-plan construction.
    value: String,
}

impl RolePromptOption {
    /// Build one immutable role/value row for a rewrite prompt.
    pub fn new(role: &'static str, value: impl Into<String>) -> Self {
        Self {
            role,
            value: value.into(),
        }
    }

    /// Return the presentation role shown beside the selected revision.
    pub fn role(&self) -> &'static str {
        self.role
    }

    /// Return the exact revision string that the builder selected.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Render the role/value pair for status text or list rows.
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
    /// Prompt title describing the pending rewrite action.
    title: &'static str,
    /// Immutable list of role assignments that the user can inspect or accept.
    options: Vec<RolePromptOption>,
    /// User-facing safety reminder appended beneath the role list.
    preview_required_message: &'static str,
}

impl RolePrompt {
    /// Build the immutable prompt model carried by `InteractionMode::RolePrompt`.
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

    /// Return the user-facing action title for the prompt.
    pub fn title(&self) -> &str {
        self.title
    }

    /// Return the ordered role rows shown in the prompt.
    pub fn options(&self) -> &[RolePromptOption] {
        &self.options
    }

    /// Return the safety reminder shown below the role rows.
    pub fn preview_required_message(&self) -> &str {
        self.preview_required_message
    }

    /// Render the prompt rows and preview reminder into a status-text block.
    pub fn status_message(&self) -> String {
        let mut lines = self
            .options
            .iter()
            .map(RolePromptOption::label)
            .collect::<Vec<_>>();
        lines.push(self.preview_required_message.to_owned());
        lines.join("\n")
    }

    /// Return every selected revision whose role is `"source"`.
    pub fn source_revisions(&self) -> Vec<&str> {
        self.options
            .iter()
            .filter(|option| option.role() == "source")
            .map(RolePromptOption::value)
            .collect()
    }

    /// Return the selected revision whose role is `"destination"`, if present.
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
/// app turns them into preview-first `actions` plans before any process side effects occur. Keep
/// payloads to the metadata needed to construct that plan: exact revision strings, operation ids,
/// selected paths, role prompts, candidate lists, and chmod modes. UI selection state, command
/// preview text, post-command status, refresh policy, and reveal targets belong in the app lifecycle
/// or `actions`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FollowUp {
    /// User-visible terminal payload when a builder cannot form a safe mutation target.
    StatusMessage(String),
    /// Revision-oriented payloads carry the exact target selected by a feature row.
    ExactRevision {
        /// Exact revision string selected upstream.
        revision: String,
    },
    SplitExactTarget {
        /// Exact revision string that should be split.
        revision: String,
    },
    /// Marker for splitting the visible working-copy change instead of an exact revision.
    SplitCurrentWorkingCopy,
    DuplicateExactTarget {
        /// Exact revision string that should be duplicated.
        revision: String,
    },
    EditExactTarget {
        /// Exact revision string that should become the working copy.
        revision: String,
    },
    RestoreExactTarget {
        /// Exact revision string that owns the path or tree being restored from.
        revision: String,
        /// Optional selected path; `None` means restore the whole revision target.
        path: Option<String>,
    },
    RestoreWorkingCopyPath {
        /// Working-copy path selected on the status surface.
        path: String,
    },
    RevertExactTarget {
        /// Exact revision string that should be reverted.
        revision: String,
    },
    OperationRestoreExactTarget {
        /// Exact operation id whose tree should be restored.
        operation_id: String,
    },
    OperationRevertExactTarget {
        /// Exact operation id that should be reverted.
        operation_id: String,
    },
    /// New-change payloads keep the exact parent list selected upstream.
    NewParents {
        /// Ordered parent revisions chosen for the new change.
        parents: Vec<String>,
    },
    /// Multi-target rewrite payloads preserve candidate ordering for the next app prompt or plan.
    RolePrompt(RolePrompt),
    /// Absorb payloads preserve the chosen source and destination ordering.
    AbsorbCandidates {
        /// Exact source revision selected for absorb.
        source: String,
        /// Ordered destination revisions offered by the builder.
        destinations: Vec<String>,
    },
    /// Path payloads carry the selected fileset string and, when needed, its revision context.
    FileTrack {
        /// Selected path that should become tracked.
        path: String,
    },
    FileUntrack {
        /// Selected path that should become untracked.
        path: String,
    },
    FileChmod {
        /// Selected path whose mode should change.
        path: String,
        /// Exact revision context for detail/file surfaces; `None` means working copy.
        revision: Option<String>,
        /// Requested target chmod mode.
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
    /// Stable action vocabulary entry represented by this row.
    pub(in crate::menus) action: ActionKind,
    /// Single-key accelerator accepted while the menu is open.
    pub(in crate::menus) shortcut: char,
    /// User-facing row text shown in the menu.
    pub(in crate::menus) label: String,
    /// Safety requirement that the renderer surfaces for this row.
    pub(in crate::menus) safety_tier: SafetyTier,
    /// Deferred payload handed back when the row is accepted.
    pub(in crate::menus) follow_up: FollowUp,
}

impl ActionMenuItem {
    /// Build one immutable action-menu row with the default shortcut for its action kind.
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

    /// Return the stable action vocabulary entry for this row.
    pub fn action(&self) -> ActionKind {
        self.action
    }

    /// Return the single-key accelerator accepted by the menu reducer.
    pub fn shortcut(&self) -> char {
        self.shortcut
    }

    /// Return the user-facing row label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Return the safety tier the renderer should surface for this row.
    pub fn safety_tier(&self) -> SafetyTier {
        self.safety_tier
    }

    /// Return the deferred payload that app dispatch should turn into the next step.
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
    /// Ordered rows offered for the currently selected context.
    items: Vec<ActionMenuItem>,
}

impl ActionMenu {
    /// Build the immutable menu carried by `InteractionMode::ActionMenu`.
    pub fn new(items: Vec<ActionMenuItem>) -> Self {
        Self { items }
    }

    /// Return whether the current context exposes any menu rows.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Return the ordered rows shown to the user.
    pub fn items(&self) -> &[ActionMenuItem] {
        &self.items
    }

    /// Return the first row whose accelerator matches the typed shortcut.
    pub fn item_for_shortcut(&self, shortcut: char) -> Option<&ActionMenuItem> {
        self.items.iter().find(|item| item.shortcut() == shortcut)
    }
}
