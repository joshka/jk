use super::*;

#[test]
fn single_exact_revision_builds_log_menu_with_duplicate_restore_and_revert() {
    let context = ExactActionContext::with_current("0000000011111111222222223333333344444444");
    let menu = build_action_menu(&context);

    assert_eq!(menu.items().len(), 7);
    assert_eq!(menu.items()[0].action(), ActionKind::Edit);
    assert_eq!(menu.items()[0].shortcut(), 'e');
    assert_eq!(menu.items()[1].action(), ActionKind::New);
    assert_eq!(menu.items()[1].shortcut(), 'n');
    assert_eq!(menu.items()[2].action(), ActionKind::Split);
    assert_eq!(menu.items()[2].shortcut(), 's');
    assert_eq!(menu.items()[3].action(), ActionKind::Abandon);
    assert_eq!(menu.items()[3].shortcut(), 'x');
    assert_eq!(menu.items()[4].action(), ActionKind::Duplicate);
    assert_eq!(menu.items()[4].shortcut(), 'd');
    assert_eq!(menu.items()[5].action(), ActionKind::Restore);
    assert_eq!(menu.items()[5].shortcut(), 'r');
    assert_eq!(menu.items()[6].action(), ActionKind::Revert);
    assert_eq!(menu.items()[6].shortcut(), 'v');
    assert!(menu.items()[0].safety_tier().is_preview_first());
    assert!(menu.items()[1].safety_tier().is_preview_first());
    assert!(menu.items()[2].safety_tier().is_preview_first());
    assert!(menu.items()[3].safety_tier().is_preview_first());
    assert!(menu.items()[4].safety_tier().is_preview_first());
    assert!(menu.items()[5].safety_tier().is_preview_first());
    assert!(menu.items()[6].safety_tier().is_preview_first());
    assert!(matches!(
        menu.items()[0].follow_up(),
        FollowUp::EditExactTarget { revision }
            if revision == "0000000011111111222222223333333344444444"
    ));
    assert!(matches!(
        menu.items()[1].follow_up(),
        FollowUp::NewParents { parents }
            if parents == &vec!["0000000011111111222222223333333344444444".to_owned()]
    ));
    assert!(matches!(
        menu.items()[2].follow_up(),
        FollowUp::SplitExactTarget { revision }
            if revision == "0000000011111111222222223333333344444444"
    ));
    assert!(matches!(
        menu.items()[3].follow_up(),
        FollowUp::ExactRevision { revision }
            if revision == "0000000011111111222222223333333344444444"
    ));
    assert!(matches!(
        menu.items()[4].follow_up(),
        FollowUp::DuplicateExactTarget { revision }
            if revision == "0000000011111111222222223333333344444444"
    ));
    assert!(matches!(
        menu.items()[5].follow_up(),
        FollowUp::RestoreExactTarget { revision, path }
            if revision == "0000000011111111222222223333333344444444" && path.is_none()
    ));
    assert!(matches!(
        menu.items()[6].follow_up(),
        FollowUp::RevertExactTarget { revision }
            if revision == "0000000011111111222222223333333344444444"
    ));
}

#[test]
fn selected_sources_and_destination_prompt_with_explicit_roles() {
    let context = ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
        .with_sources([
            "aaaabbbb1111111111111111111111111111111111",
            "eeeeffff2222222222222222222222222222222222",
        ]);
    let menu = build_action_menu(&context);

    assert_eq!(menu.items().len(), 6);
    assert!(menu.items()[0].label().contains("new merge child"));
    assert!(
        menu.items()[1]
            .label()
            .contains("source revisions into destination ccccdddd")
    );
    assert!(
        menu.items()[2]
            .label()
            .contains("source revisions into destination ccccdddd")
    );
    assert!(matches!(
        menu.items()[0].follow_up(),
        FollowUp::NewParents { parents }
            if parents == &vec![
                "aaaabbbb1111111111111111111111111111111111".to_owned(),
                "eeeeffff2222222222222222222222222222222222".to_owned()
            ]
    ));
    assert!(menu.items()[1].label().contains("rebase"));
    assert!(matches!(
        menu.items()[1].follow_up(),
        FollowUp::RolePrompt(prompt)
            if prompt.title() == "confirm role assignment"
    ));
    if let FollowUp::RolePrompt(prompt) = menu.items()[1].follow_up() {
        assert_eq!(menu.items()[1].action(), ActionKind::Rebase);
        assert_eq!(prompt.options()[0].role(), "source");
        assert_eq!(
            prompt.options()[0].value(),
            "aaaabbbb1111111111111111111111111111111111"
        );
        assert_eq!(
            prompt.source_revisions(),
            vec![
                "aaaabbbb1111111111111111111111111111111111",
                "eeeeffff2222222222222222222222222222222222"
            ]
        );
        assert_eq!(
            prompt.destination_revision(),
            Some("ccccdddd1111111111111111111111111111111111")
        );
        let labels = prompt
            .options()
            .iter()
            .map(|option| option.label())
            .collect::<Vec<_>>();
        assert_eq!(
            labels[0],
            "source: aaaabbbb1111111111111111111111111111111111"
        );
        assert_eq!(
            labels[1],
            "source: eeeeffff2222222222222222222222222222222222"
        );
        assert_eq!(
            labels[2],
            "destination: ccccdddd1111111111111111111111111111111111"
        );
        assert!(prompt.status_message().ends_with(PREVIEW_REQUIRED_MARKER));
    } else {
        panic!("expected role prompt follow-up");
    }
    if let FollowUp::RolePrompt(prompt) = menu.items()[2].follow_up() {
        assert_eq!(menu.items()[2].action(), ActionKind::Squash);
        assert_eq!(
            prompt.source_revisions(),
            vec![
                "aaaabbbb1111111111111111111111111111111111",
                "eeeeffff2222222222222222222222222222222222"
            ]
        );
        assert_eq!(
            prompt.destination_revision(),
            Some("ccccdddd1111111111111111111111111111111111")
        );
    } else {
        panic!("expected squash role prompt follow-up");
    }
    assert!(
        menu.items()[3]
            .label()
            .contains("absorb current revision ccccdddd into 2 candidate destinations")
    );
    assert!(matches!(
        menu.items()[3].follow_up(),
        FollowUp::AbsorbCandidates {
            source,
            destinations,
        } if source == "ccccdddd1111111111111111111111111111111111"
            && destinations == &vec![
                "aaaabbbb1111111111111111111111111111111111".to_owned(),
                "eeeeffff2222222222222222222222222222222222".to_owned()
            ]
    ));
    assert!(matches!(
        menu.items()[4].follow_up(),
        FollowUp::RestoreExactTarget { revision, path }
            if revision == "ccccdddd1111111111111111111111111111111111" && path.is_none()
    ));
    assert!(matches!(
        menu.items()[5].follow_up(),
        FollowUp::RevertExactTarget { revision }
            if revision == "ccccdddd1111111111111111111111111111111111"
    ));
}

#[test]
fn no_exact_actionable_ids_returns_empty_menu() {
    let context = ExactActionContext::none();
    let menu = build_action_menu(&context);

    assert!(menu.is_empty());
}

#[test]
fn multi_source_menu_excludes_abandon() {
    let context = ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
        .with_sources(["aaaabbbb1111111111111111111111111111111111"]);
    let menu = build_action_menu(&context);

    let actions = menu
        .items()
        .iter()
        .map(ActionMenuItem::action)
        .collect::<Vec<_>>();

    assert_eq!(
        actions,
        vec![
            ActionKind::New,
            ActionKind::Rebase,
            ActionKind::Squash,
            ActionKind::Absorb,
            ActionKind::Restore,
            ActionKind::Revert
        ]
    );
}

#[test]
fn self_selection_keeps_new_parent_and_single_revision_actions() {
    let context = ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
        .with_sources(["ccccdddd1111111111111111111111111111111111"]);
    let menu = build_action_menu(&context);

    let actions = menu
        .items()
        .iter()
        .map(ActionMenuItem::action)
        .collect::<Vec<_>>();

    assert_eq!(
        actions,
        vec![
            ActionKind::Edit,
            ActionKind::New,
            ActionKind::Split,
            ActionKind::Abandon,
            ActionKind::Duplicate,
            ActionKind::Restore,
            ActionKind::Revert
        ]
    );
    assert!(matches!(
        menu.items()[1].follow_up(),
        FollowUp::NewParents { parents }
            if parents == &vec!["ccccdddd1111111111111111111111111111111111".to_owned()]
    ));
    assert!(matches!(
        menu.items()[2].follow_up(),
        FollowUp::SplitExactTarget { revision }
            if revision == "ccccdddd1111111111111111111111111111111111"
    ));
}

#[test]
fn visible_working_copy_split_uses_current_follow_up() {
    let context = ExactActionContext::with_current("ccccdddd1111111111111111111111111111111111")
        .with_visible_working_copy();
    let menu = build_action_menu(&context);

    assert_eq!(menu.items()[2].action(), ActionKind::Split);
    assert_eq!(
        menu.items()[2].label(),
        "split current working-copy change @"
    );
    assert!(matches!(
        menu.items()[2].follow_up(),
        FollowUp::SplitCurrentWorkingCopy
    ));
}

#[test]
fn detail_context_offers_duplicate_restore_and_revert() {
    let menu = build_action_menu(&ExactActionContext::detail(
        "ccccdddd1111111111111111111111111111111111",
    ));

    let actions = menu
        .items()
        .iter()
        .map(ActionMenuItem::action)
        .collect::<Vec<_>>();

    assert_eq!(
        actions,
        vec![
            ActionKind::Duplicate,
            ActionKind::Restore,
            ActionKind::Revert
        ]
    );
}
