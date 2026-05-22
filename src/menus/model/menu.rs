use crate::actions::JjFileChmodMode;

use super::{ActionKind, RolePrompt, SafetyTier};

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
    pub action: ActionKind,
    /// Single-key accelerator accepted while the menu is open.
    pub shortcut: char,
    /// User-facing row text shown in the menu.
    pub label: String,
    /// Safety requirement that the renderer surfaces for this row.
    pub safety_tier: SafetyTier,
    /// Deferred payload handed back when the row is accepted.
    pub follow_up: FollowUp,
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
