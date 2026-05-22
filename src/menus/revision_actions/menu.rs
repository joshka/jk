use super::super::path_actions::{
    file_action_menu, file_action_menu_items, status_path_action_menu,
};
use super::super::{
    ActionKind, ActionMenu, ActionMenuItem, FollowUp, PREVIEW_REQUIRED_MARKER, RolePrompt,
    RolePromptOption, SafetyTier, short_id,
};
use super::context::ExactActionContext;

/// Build the revision-scoped action menu for one exact selection context.
///
/// This boundary owns only action availability, menu row ordering, and the
/// follow-up payload attached to each row. Preview construction, command
/// execution, and refresh behavior stay in app dispatch and `actions`.
pub fn build_action_menu(context: &ExactActionContext) -> ActionMenu {
    if context.is_status_surface() {
        return context
            .file_action
            .as_ref()
            .map(status_path_action_menu)
            .unwrap_or_default();
    }

    let Some(current_revision) = context.current_revision() else {
        return context
            .file_action
            .as_ref()
            .map(file_action_menu)
            .unwrap_or_default();
    };

    let mut mutation_items = mutation_menu_items(current_revision, context.selected_path(), true);
    if context.is_detail_surface()
        && let Some(file_action) = &context.file_action
    {
        let file_items = file_action_menu_items(file_action);
        if context.selected_path().is_some() {
            mutation_items.splice(1..1, file_items);
        } else {
            mutation_items.extend(file_items);
        }
    }

    if context.is_detail_surface() {
        return ActionMenu::new(mutation_items);
    }

    let new_parents = if context.source_revisions().is_empty() {
        vec![current_revision.to_owned()]
    } else {
        context.source_revisions().to_vec()
    };

    if context.source_revisions().is_empty()
        || context
            .source_revisions()
            .iter()
            .all(|source| source == current_revision)
    {
        let edit = menu_item_for_edit(current_revision);
        let new = menu_item_for_new_parents(&new_parents);
        let split =
            menu_item_for_split(current_revision, context.current_is_visible_working_copy());
        let abandon = menu_item_for_single_revision(ActionKind::Abandon, current_revision);
        let mut items = vec![edit, new, split, abandon];
        items.extend(mutation_items);
        return ActionMenu::new(items);
    }

    let selected_revisions = context
        .source_revisions()
        .iter()
        .filter(|source| *source != current_revision)
        .cloned()
        .collect::<Vec<_>>();
    if selected_revisions.is_empty() {
        return ActionMenu::default();
    }

    let mut items = vec![
        menu_item_for_new_parents(&new_parents),
        menu_item_for_multirev_action(ActionKind::Rebase, &selected_revisions, current_revision),
        menu_item_for_multirev_action(ActionKind::Squash, &selected_revisions, current_revision),
        menu_item_for_absorb(current_revision, &selected_revisions),
    ];
    items.extend(mutation_menu_items(current_revision, None, false));
    ActionMenu::new(items)
}

fn menu_item_for_new_parents(parent_revisions: &[String]) -> ActionMenuItem {
    let label = if parent_revisions.len() == 1 {
        format!("new child of {}", short_id(&parent_revisions[0]))
    } else {
        format!("new merge child of {} parents", parent_revisions.len())
    };
    ActionMenuItem {
        action: ActionKind::New,
        shortcut: ActionKind::New.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::NewParents {
            parents: parent_revisions.to_vec(),
        },
    }
}

fn menu_item_for_single_revision(action: ActionKind, revision: &str) -> ActionMenuItem {
    let label = format!(
        "{} selected revision {}",
        action.label(),
        short_id(revision)
    );
    let follow_up = match action {
        ActionKind::Abandon => FollowUp::ExactRevision {
            revision: revision.to_owned(),
        },
        ActionKind::Edit
        | ActionKind::New
        | ActionKind::Duplicate
        | ActionKind::Restore
        | ActionKind::Revert
        | ActionKind::Rebase
        | ActionKind::Squash
        | ActionKind::Absorb
        | ActionKind::FileTrack
        | ActionKind::FileUntrack
        | ActionKind::FileChmodExecutable
        | ActionKind::FileChmodNormal
        | ActionKind::Split => {
            let message = format!("{} {}", label, PREVIEW_REQUIRED_MARKER);
            FollowUp::StatusMessage(message)
        }
    };
    ActionMenuItem {
        action,
        shortcut: action.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up,
    }
}

fn menu_item_for_split(revision: &str, current_is_visible_working_copy: bool) -> ActionMenuItem {
    let label = if current_is_visible_working_copy {
        "split current working-copy change @".to_owned()
    } else {
        format!("split selected revision {}", short_id(revision))
    };
    let follow_up = if current_is_visible_working_copy {
        FollowUp::SplitCurrentWorkingCopy
    } else {
        FollowUp::SplitExactTarget {
            revision: revision.to_owned(),
        }
    };

    ActionMenuItem {
        action: ActionKind::Split,
        shortcut: ActionKind::Split.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up,
    }
}

fn menu_item_for_edit(revision: &str) -> ActionMenuItem {
    ActionMenuItem {
        action: ActionKind::Edit,
        shortcut: ActionKind::Edit.shortcut(),
        label: format!("edit selected revision {}", short_id(revision)),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::EditExactTarget {
            revision: revision.to_owned(),
        },
    }
}

fn menu_item_for_multirev_action(
    action: ActionKind,
    source_revisions: &[String],
    destination_revision: &str,
) -> ActionMenuItem {
    let label = format!(
        "{} {} source revision{} into destination {}",
        action.label(),
        source_revisions.len(),
        if source_revisions.len() == 1 { "" } else { "s" },
        short_id(destination_revision),
    );
    let options = source_revisions
        .iter()
        .map(|source| RolePromptOption::new("source", source))
        .chain(std::iter::once(RolePromptOption::new(
            "destination",
            destination_revision.to_owned(),
        )))
        .collect();
    let role_prompt = RolePrompt::new(
        "confirm role assignment",
        options,
        SafetyTier::PreviewFirst.preview_marker(),
    );
    ActionMenuItem {
        action,
        shortcut: action.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RolePrompt(role_prompt),
    }
}

fn menu_item_for_absorb(source_revision: &str, destination_revisions: &[String]) -> ActionMenuItem {
    let label = format!(
        "absorb current revision {} into {} candidate destination{}",
        short_id(source_revision),
        destination_revisions.len(),
        if destination_revisions.len() == 1 {
            ""
        } else {
            "s"
        },
    );
    ActionMenuItem {
        action: ActionKind::Absorb,
        shortcut: ActionKind::Absorb.shortcut(),
        label,
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::AbsorbCandidates {
            source: source_revision.to_owned(),
            destinations: destination_revisions.to_vec(),
        },
    }
}

fn mutation_menu_items(
    current_revision: &str,
    selected_path: Option<&str>,
    include_duplicate: bool,
) -> Vec<ActionMenuItem> {
    let mut items = Vec::new();
    if let Some(path) = selected_path {
        items.push(ActionMenuItem {
            action: ActionKind::Restore,
            shortcut: 'p',
            label: format!("restore selected path from {}", short_id(current_revision)),
            safety_tier: SafetyTier::PreviewFirst,
            follow_up: FollowUp::RestoreExactTarget {
                revision: current_revision.to_owned(),
                path: Some(path.to_owned()),
            },
        });
    }
    if include_duplicate {
        items.push(ActionMenuItem {
            action: ActionKind::Duplicate,
            shortcut: ActionKind::Duplicate.shortcut(),
            label: format!("duplicate selected revision {}", short_id(current_revision)),
            safety_tier: SafetyTier::PreviewFirst,
            follow_up: FollowUp::DuplicateExactTarget {
                revision: current_revision.to_owned(),
            },
        });
    }
    items.push(ActionMenuItem {
        action: ActionKind::Restore,
        shortcut: ActionKind::Restore.shortcut(),
        label: format!("restore selected revision {}", short_id(current_revision)),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RestoreExactTarget {
            revision: current_revision.to_owned(),
            path: None,
        },
    });
    items.push(ActionMenuItem {
        action: ActionKind::Revert,
        shortcut: ActionKind::Revert.shortcut(),
        label: format!(
            "revert selected revision {} into @",
            short_id(current_revision)
        ),
        safety_tier: SafetyTier::PreviewFirst,
        follow_up: FollowUp::RevertExactTarget {
            revision: current_revision.to_owned(),
        },
    });
    items
}
