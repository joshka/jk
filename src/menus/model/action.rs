pub const PREVIEW_REQUIRED_MARKER: &str = "Preview required before execution.";

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
/// feature or action-menu builder that knows the selected row context. Labels and shortcuts are
/// part of the shared presentation contract, but they are not command construction policy.
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
    pub fn label(self) -> &'static str {
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

    pub fn shortcut(self) -> char {
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
