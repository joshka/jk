//! Path-scoped action-menu policy for status rows and file/detail views.
//!
//! The parent action-menu module owns the shared action vocabulary. This module
//! owns which path actions are offered for working-copy paths versus exact
//! revision paths, including their ordering and follow-up payloads.

use crate::actions::JjFileChmodMode;

use super::{ActionKind, ActionMenu, ActionMenuItem, FollowUp, SafetyTier, short_id};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FileActionContext {
    /// Selected path whose menu entries are being built.
    path: String,
    /// Revision or working-copy scope that owns the selected path.
    scope: FileActionScope,
    /// Whether the surface may offer restore for this path.
    restore_allowed: bool,
    /// Whether the surface may offer tracking the selected path.
    track_allowed: bool,
    /// Whether the surface may offer untracking the selected path.
    untrack_allowed: bool,
    /// Whether the surface may offer chmod mutations for the selected path.
    chmod_allowed: bool,
}

/// Internal classification of where the selected path lives.
#[derive(Clone, Debug, Eq, PartialEq)]
enum FileActionScope {
    /// Path comes from the working copy, so no exact revision is attached.
    WorkingCopy,
    /// Path is rooted in one exact revision on a detail surface.
    ExactRevision(String),
}

impl FileActionContext {
    /// Build a working-copy path context for an untracked status row.
    pub(super) fn working_copy_untracked(path: String) -> Self {
        Self {
            path,
            scope: FileActionScope::WorkingCopy,
            restore_allowed: false,
            track_allowed: true,
            untrack_allowed: false,
            chmod_allowed: false,
        }
    }

    /// Build a working-copy path context for a tracked status or file row.
    pub(super) fn working_copy_tracked(
        path: String,
        restore_allowed: bool,
        chmod_allowed: bool,
    ) -> Self {
        Self {
            path,
            scope: FileActionScope::WorkingCopy,
            restore_allowed,
            track_allowed: false,
            untrack_allowed: true,
            chmod_allowed,
        }
    }

    /// Build an exact-revision path context for detail-surface file actions.
    pub(super) fn exact_revision_tracked(
        revision: String,
        path: String,
        chmod_allowed: bool,
    ) -> Self {
        Self {
            path,
            scope: FileActionScope::ExactRevision(revision),
            restore_allowed: true,
            track_allowed: false,
            untrack_allowed: false,
            chmod_allowed,
        }
    }

    /// Return the selected path string.
    fn path(&self) -> &str {
        &self.path
    }

    /// Return the owning exact revision, or `None` for working-copy paths.
    fn revision(&self) -> Option<&str> {
        match &self.scope {
            FileActionScope::WorkingCopy => None,
            FileActionScope::ExactRevision(revision) => Some(revision),
        }
    }
}

/// Build the status-surface path menu, including restore when the status row allows it.
pub(super) fn status_path_action_menu(file_action: &FileActionContext) -> ActionMenu {
    let mut items = Vec::new();
    if file_action.restore_allowed {
        let path = file_action.path();
        items.push(ActionMenuItem {
            action: ActionKind::Restore,
            shortcut: ActionKind::Restore.shortcut(),
            label: format!("restore selected status path {path}"),
            safety_tier: SafetyTier::PreviewFirst,
            follow_up: FollowUp::RestoreWorkingCopyPath {
                path: path.to_owned(),
            },
        });
    }
    items.extend(file_action_menu_items(file_action));
    ActionMenu::new(items)
}

/// Build the non-status path menu for detail or file surfaces.
pub(super) fn file_action_menu(file_action: &FileActionContext) -> ActionMenu {
    ActionMenu::new(file_action_menu_items(file_action))
}

/// Build the ordered path-specific mutation rows for the selected path context.
pub(super) fn file_action_menu_items(file_action: &FileActionContext) -> Vec<ActionMenuItem> {
    let mut items = Vec::new();
    let path = file_action.path();
    if file_action.track_allowed {
        items.push(ActionMenuItem {
            action: ActionKind::FileTrack,
            shortcut: ActionKind::FileTrack.shortcut(),
            label: format!("track selected path {path}"),
            safety_tier: SafetyTier::PreviewFirst,
            follow_up: FollowUp::FileTrack {
                path: path.to_owned(),
            },
        });
    }
    if file_action.untrack_allowed {
        items.push(ActionMenuItem {
            action: ActionKind::FileUntrack,
            shortcut: ActionKind::FileUntrack.shortcut(),
            label: format!("untrack selected path {path}"),
            safety_tier: SafetyTier::PreviewFirst,
            follow_up: FollowUp::FileUntrack {
                path: path.to_owned(),
            },
        });
    }
    if file_action.chmod_allowed {
        items.push(file_chmod_item(
            ActionKind::FileChmodExecutable,
            path,
            file_action.revision(),
            JjFileChmodMode::Executable,
        ));
        items.push(file_chmod_item(
            ActionKind::FileChmodNormal,
            path,
            file_action.revision(),
            JjFileChmodMode::Normal,
        ));
    }
    items
}

/// Build one chmod row for the selected path and revision scope.
fn file_chmod_item(
    action: ActionKind,
    path: &str,
    revision: Option<&str>,
    mode: JjFileChmodMode,
) -> ActionMenuItem {
    let scope = revision
        .map(|revision| format!(" in {}", short_id(revision)))
        .unwrap_or_default();
    ActionMenuItem {
        action,
        shortcut: action.shortcut(),
        label: format!("{} selected path {}{}", action.label(), path, scope),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::FileChmod {
            path: path.to_owned(),
            revision: revision.map(str::to_owned),
            mode,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ExactActionContext, build_action_menu};
    use super::*;

    #[test]
    fn detail_context_with_selected_path_offers_path_restore_first() {
        let menu = build_action_menu(
            &ExactActionContext::detail("ccccdddd1111111111111111111111111111111111")
                .with_selected_path("src/quoted path.rs"),
        );

        let actions = menu
            .items()
            .iter()
            .map(ActionMenuItem::action)
            .collect::<Vec<_>>();

        assert_eq!(
            actions,
            vec![
                ActionKind::Restore,
                ActionKind::FileChmodExecutable,
                ActionKind::FileChmodNormal,
                ActionKind::Duplicate,
                ActionKind::Restore,
                ActionKind::Revert
            ]
        );
        assert_eq!(menu.items()[0].shortcut(), 'p');
        assert_eq!(menu.items()[1].shortcut(), 'x');
        assert_eq!(menu.items()[2].shortcut(), 'n');
        assert_eq!(menu.items()[3].shortcut(), 'd');
        assert_eq!(menu.items()[4].shortcut(), 'r');
        assert_eq!(menu.items()[5].shortcut(), 'v');
        assert_eq!(
            menu.item_for_shortcut('p').map(ActionMenuItem::label),
            Some("restore selected path from ccccdddd")
        );
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::RestoreExactTarget { revision, path }
                if revision == "ccccdddd1111111111111111111111111111111111"
                    && path.as_deref() == Some("src/quoted path.rs")
        ));
    }

    #[test]
    fn status_path_context_offers_working_copy_restore_and_hygiene_actions() {
        let menu = build_action_menu(&ExactActionContext::status_path("src/status.rs"));

        assert_eq!(menu.items().len(), 4);
        assert_eq!(menu.items()[0].action(), ActionKind::Restore);
        assert_eq!(menu.items()[0].shortcut(), 'r');
        assert_eq!(menu.items()[1].action(), ActionKind::FileUntrack);
        assert_eq!(menu.items()[2].action(), ActionKind::FileChmodExecutable);
        assert_eq!(menu.items()[3].action(), ActionKind::FileChmodNormal);
        assert_eq!(
            menu.items()[0].label(),
            "restore selected status path src/status.rs"
        );
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::RestoreWorkingCopyPath { path } if path == "src/status.rs"
        ));
    }

    #[test]
    fn status_untracked_context_offers_only_file_track() {
        let menu = build_action_menu(&ExactActionContext::status_untracked_path("scratch.txt"));

        assert_eq!(menu.items().len(), 1);
        assert_eq!(menu.items()[0].action(), ActionKind::FileTrack);
        assert_eq!(menu.items()[0].shortcut(), 't');
        assert!(matches!(
            menu.items()[0].follow_up(),
            FollowUp::FileTrack { path } if path == "scratch.txt"
        ));
    }
}
