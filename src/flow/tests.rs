use super::{FlowAction, PromptKind, plan_command};

fn selected() -> Option<String> {
    Some("abc12345".to_string())
}

fn assert_prompt_kind(action: FlowAction, expected: PromptKind) {
    match action {
        FlowAction::Prompt(request) => assert_eq!(request.kind, expected),
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn plans_show_diff_edit_and_abandon_with_selected_revision() {
    assert_eq!(
        plan_command("show", selected()),
        FlowAction::Execute(vec!["show".to_string(), "abc12345".to_string()])
    );
    assert_eq!(
        plan_command("diff", selected()),
        FlowAction::Execute(vec![
            "diff".to_string(),
            "-r".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_eq!(
        plan_command("edit", selected()),
        FlowAction::Execute(vec!["edit".to_string(), "abc12345".to_string()])
    );
    assert_eq!(
        plan_command("abandon", selected()),
        FlowAction::Execute(vec!["abandon".to_string(), "abc12345".to_string()])
    );
}

#[test]
fn plans_prompt_for_describe_commit_and_rewrite_flows() {
    let describe = plan_command("describe", selected());
    match describe {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::DescribeMessage {
                    revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    let commit = plan_command("commit", selected());
    match commit {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::CommitMessage);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    let rebase = plan_command("rebase", selected());
    match rebase {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::RebaseDestination {
                    source_revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    let squash = plan_command("jjsq", selected());
    match squash {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::SquashInto {
                    from_revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    let split = plan_command("split", selected());
    match split {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::SplitFileset {
                    revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn maps_alias_and_prompts_fetch_push() {
    let fetch = plan_command("gf", selected());
    match fetch {
        FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::GitFetchRemote),
        other => panic!("expected prompt, got {other:?}"),
    }

    let push = plan_command("jjgp", selected());
    match push {
        FlowAction::Prompt(request) => assert_eq!(request.kind, PromptKind::GitPushBookmark),
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn maps_fetch_push_variant_aliases() {
    assert_eq!(
        plan_command("jjgfa", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "fetch".to_string(),
            "--all-remotes".to_string()
        ])
    );
    assert_eq!(
        plan_command("jjgpt", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "push".to_string(),
            "--tracked".to_string()
        ])
    );
    assert_eq!(
        plan_command("jjgpa", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "push".to_string(),
            "--all".to_string()
        ])
    );
    assert_eq!(
        plan_command("jjgpd", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "push".to_string(),
            "--deleted".to_string()
        ])
    );
}

#[test]
fn maps_log_and_status_aliases() {
    assert_eq!(
        plan_command("jjst", selected()),
        FlowAction::Execute(vec!["status".to_string()])
    );
    assert_eq!(
        plan_command("jjl", selected()),
        FlowAction::Execute(vec!["log".to_string()])
    );
}

#[test]
fn maps_core_jj_default_aliases() {
    assert_eq!(
        plan_command("st", selected()),
        FlowAction::Execute(vec!["status".to_string()])
    );
    assert_prompt_kind(
        plan_command("desc", selected()),
        PromptKind::DescribeMessage {
            revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(plan_command("ci", selected()), PromptKind::CommitMessage);
    assert_eq!(
        plan_command("b", selected()),
        FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
    );
    assert_eq!(
        plan_command("op", selected()),
        FlowAction::Execute(vec!["operation".to_string(), "log".to_string()])
    );
}

#[test]
fn covers_oh_my_zsh_gold_alias_flow_contract() {
    assert_prompt_kind(plan_command("jjgf", selected()), PromptKind::GitFetchRemote);
    assert_prompt_kind(
        plan_command("jjgp", selected()),
        PromptKind::GitPushBookmark,
    );
    assert_eq!(
        plan_command("jjgfa", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "fetch".to_string(),
            "--all-remotes".to_string()
        ])
    );
    assert_eq!(
        plan_command("jjgpt", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "push".to_string(),
            "--tracked".to_string()
        ])
    );
    assert_eq!(
        plan_command("jjgpa", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "push".to_string(),
            "--all".to_string()
        ])
    );
    assert_eq!(
        plan_command("jjgpd", selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "push".to_string(),
            "--deleted".to_string()
        ])
    );

    assert_prompt_kind(
        plan_command("jjrb", selected()),
        PromptKind::RebaseDestination {
            source_revision: "abc12345".to_string(),
        },
    );
    assert_eq!(
        plan_command("jjrbm", selected()),
        FlowAction::Execute(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "trunk()".to_string()
        ])
    );
    assert_eq!(
        plan_command("jjst", selected()),
        FlowAction::Execute(vec!["status".to_string()])
    );
    assert_eq!(
        plan_command("jjl", selected()),
        FlowAction::Execute(vec!["log".to_string()])
    );
    assert_eq!(
        plan_command("jjd", selected()),
        FlowAction::Execute(vec![
            "diff".to_string(),
            "-r".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_prompt_kind(plan_command("jjc", selected()), PromptKind::CommitMessage);
    assert_prompt_kind(
        plan_command("jjds", selected()),
        PromptKind::DescribeMessage {
            revision: "abc12345".to_string(),
        },
    );
    assert_eq!(
        plan_command("jje", selected()),
        FlowAction::Execute(vec!["edit".to_string(), "abc12345".to_string()])
    );
    assert_prompt_kind(plan_command("jjn", selected()), PromptKind::NewMessage);
    assert_eq!(
        plan_command("jjnt", selected()),
        FlowAction::Execute(vec!["new".to_string(), "trunk()".to_string()])
    );
    assert_prompt_kind(
        plan_command("jjsp", selected()),
        PromptKind::SplitFileset {
            revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("jjsq", selected()),
        PromptKind::SquashInto {
            from_revision: "abc12345".to_string(),
        },
    );
    assert_eq!(
        plan_command("jjb", selected()),
        FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
    );
    assert_eq!(
        plan_command("jjbl", selected()),
        FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
    );
    assert_prompt_kind(
        plan_command("jjbs", selected()),
        PromptKind::BookmarkSet {
            target_revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("jjbm", selected()),
        PromptKind::BookmarkMove {
            target_revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(plan_command("jjbt", selected()), PromptKind::BookmarkTrack);
    assert_prompt_kind(
        plan_command("jjbu", selected()),
        PromptKind::BookmarkUntrack,
    );
    assert_prompt_kind(
        plan_command("jjrs", selected()),
        PromptKind::RestoreFrom {
            target_revision: "abc12345".to_string(),
        },
    );
    assert_eq!(
        plan_command("jja", selected()),
        FlowAction::Execute(vec!["abandon".to_string(), "abc12345".to_string()])
    );
    assert_eq!(
        plan_command("jjrt", selected()),
        FlowAction::Execute(vec!["root".to_string()])
    );
}

#[test]
fn maps_rebase_aliases_to_expected_destinations() {
    assert_eq!(
        plan_command("rbm", selected()),
        FlowAction::Execute(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "main".to_string()
        ])
    );

    assert_eq!(
        plan_command("rbt", selected()),
        FlowAction::Execute(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "trunk()".to_string()
        ])
    );

    assert_eq!(
        plan_command("rbm release", selected()),
        FlowAction::Execute(vec![
            "rebase".to_string(),
            "-d".to_string(),
            "release".to_string()
        ])
    );
}

#[test]
fn covers_gold_command_flow_contract() {
    assert_eq!(
        plan_command("log", selected()),
        FlowAction::Execute(vec!["log".to_string()])
    );
    assert_eq!(
        plan_command("status", selected()),
        FlowAction::Execute(vec!["status".to_string()])
    );
    assert_eq!(
        plan_command("show", selected()),
        FlowAction::Execute(vec!["show".to_string(), "abc12345".to_string()])
    );
    assert_eq!(
        plan_command("diff", selected()),
        FlowAction::Execute(vec![
            "diff".to_string(),
            "-r".to_string(),
            "abc12345".to_string()
        ])
    );

    assert_prompt_kind(plan_command("new", selected()), PromptKind::NewMessage);
    assert_prompt_kind(
        plan_command("describe", selected()),
        PromptKind::DescribeMessage {
            revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("commit", selected()),
        PromptKind::CommitMessage,
    );
    assert_eq!(
        plan_command("next", selected()),
        FlowAction::Execute(vec!["next".to_string()])
    );
    assert_eq!(
        plan_command("prev", selected()),
        FlowAction::Execute(vec!["prev".to_string()])
    );
    assert_eq!(
        plan_command("edit", selected()),
        FlowAction::Execute(vec!["edit".to_string(), "abc12345".to_string()])
    );

    assert_prompt_kind(
        plan_command("rebase", selected()),
        PromptKind::RebaseDestination {
            source_revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("squash", selected()),
        PromptKind::SquashInto {
            from_revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("split", selected()),
        PromptKind::SplitFileset {
            revision: "abc12345".to_string(),
        },
    );
    assert_eq!(
        plan_command("abandon", selected()),
        FlowAction::Execute(vec!["abandon".to_string(), "abc12345".to_string()])
    );
    assert_eq!(
        plan_command("undo", selected()),
        FlowAction::Execute(vec!["undo".to_string()])
    );
    assert_eq!(
        plan_command("redo", selected()),
        FlowAction::Execute(vec!["redo".to_string()])
    );

    assert_eq!(
        plan_command("bookmark", selected()),
        FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
    );
    assert_prompt_kind(
        plan_command("bookmark create", selected()),
        PromptKind::BookmarkCreate {
            target_revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("bookmark set", selected()),
        PromptKind::BookmarkSet {
            target_revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("bookmark move", selected()),
        PromptKind::BookmarkMove {
            target_revision: "abc12345".to_string(),
        },
    );
    assert_prompt_kind(
        plan_command("bookmark track", selected()),
        PromptKind::BookmarkTrack,
    );
    assert_prompt_kind(
        plan_command("bookmark untrack", selected()),
        PromptKind::BookmarkUntrack,
    );

    assert_prompt_kind(
        plan_command("git fetch", selected()),
        PromptKind::GitFetchRemote,
    );
    assert_prompt_kind(
        plan_command("git push", selected()),
        PromptKind::GitPushBookmark,
    );
}

#[test]
fn adds_selection_aware_absorb_duplicate_and_parallelize_flows() {
    assert_eq!(
        plan_command("absorb", selected()),
        FlowAction::Execute(vec![
            "absorb".to_string(),
            "--from".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_eq!(
        plan_command("duplicate", selected()),
        FlowAction::Execute(vec!["duplicate".to_string(), "abc12345".to_string()])
    );
    assert_eq!(
        plan_command("parallelize", selected()),
        FlowAction::Execute(vec!["parallelize".to_string(), "abc12345".to_string()])
    );
}

#[test]
fn adds_selection_aware_evolog_interdiff_and_metaedit_flows() {
    assert_eq!(
        plan_command("evolog", selected()),
        FlowAction::Execute(vec![
            "evolog".to_string(),
            "-r".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_eq!(
        plan_command("interdiff", selected()),
        FlowAction::Execute(vec![
            "interdiff".to_string(),
            "--from".to_string(),
            "@-".to_string(),
            "--to".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_prompt_kind(
        plan_command("metaedit", selected()),
        PromptKind::MetaeditMessage {
            revision: "abc12345".to_string(),
        },
    );
    assert_eq!(
        plan_command("simplify-parents", selected()),
        FlowAction::Execute(vec!["simplify-parents".to_string(), "abc12345".to_string()])
    );
    assert_eq!(
        plan_command("fix", selected()),
        FlowAction::Execute(vec![
            "fix".to_string(),
            "-s".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_eq!(
        plan_command("diffedit", selected()),
        FlowAction::Execute(vec![
            "diffedit".to_string(),
            "-r".to_string(),
            "abc12345".to_string()
        ])
    );
}

#[test]
fn bookmark_without_subcommand_lists() {
    assert_eq!(
        plan_command("bookmark", selected()),
        FlowAction::Execute(vec!["bookmark".to_string(), "list".to_string()])
    );
}

#[test]
fn bookmark_short_subcommand_is_canonicalized() {
    match plan_command("bookmark s", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::BookmarkSet {
                    target_revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn adds_guided_restore_and_revert_flows() {
    match plan_command("restore", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::RestoreFrom {
                    target_revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("revert", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::RevertRevisions {
                    default_revisions: "abc12345".to_string(),
                    onto_revision: "@".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn adds_guided_delete_forget_and_rename_flows() {
    match plan_command("bookmark delete", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::BookmarkDelete);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("bookmark forget", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::BookmarkForget);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("bookmark rename", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::BookmarkRename);
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn adds_guided_operation_subcommand_flows() {
    match plan_command("operation show", selected()) {
        FlowAction::Execute(tokens) => {
            assert_eq!(tokens, vec!["operation".to_string(), "show".to_string()]);
        }
        other => panic!("expected execute, got {other:?}"),
    }

    match plan_command("operation diff", selected()) {
        FlowAction::Execute(tokens) => {
            assert_eq!(tokens, vec!["operation".to_string(), "diff".to_string()]);
        }
        other => panic!("expected execute, got {other:?}"),
    }

    match plan_command("operation restore", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::OperationRestore);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("operation revert", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::OperationRevert {
                    default_operation: "@".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn adds_guided_workspace_subcommand_flows() {
    match plan_command("workspace rename", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::WorkspaceRename);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("workspace add", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::WorkspaceAdd);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("workspace root", selected()) {
        FlowAction::Execute(tokens) => {
            assert_eq!(tokens, vec!["workspace".to_string(), "root".to_string()])
        }
        other => panic!("expected execute, got {other:?}"),
    }
}

#[test]
fn prompt_kind_builds_tokens() {
    let bookmark = PromptKind::BookmarkSet {
        target_revision: "abc12345".to_string(),
    };
    assert_eq!(
        bookmark.to_tokens("feature"),
        Ok(vec![
            "bookmark".to_string(),
            "set".to_string(),
            "feature".to_string(),
            "-r".to_string(),
            "abc12345".to_string()
        ])
    );

    let track = PromptKind::BookmarkTrack;
    assert_eq!(
        track.to_tokens("topic origin"),
        Ok(vec![
            "bookmark".to_string(),
            "track".to_string(),
            "topic".to_string(),
            "--remote".to_string(),
            "origin".to_string()
        ])
    );

    let rebase = PromptKind::RebaseDestination {
        source_revision: "abc12345".to_string(),
    };
    assert_eq!(
        rebase.to_tokens("main"),
        Ok(vec![
            "rebase".to_string(),
            "-r".to_string(),
            "abc12345".to_string(),
            "-d".to_string(),
            "main".to_string()
        ])
    );

    let squash = PromptKind::SquashInto {
        from_revision: "abc12345".to_string(),
    };
    assert_eq!(
        squash.to_tokens(""),
        Ok(vec![
            "squash".to_string(),
            "--from".to_string(),
            "abc12345".to_string(),
            "--into".to_string(),
            "@-".to_string()
        ])
    );

    let bookmark_delete = PromptKind::BookmarkDelete;
    assert_eq!(
        bookmark_delete.to_tokens("feature hotfix"),
        Ok(vec![
            "bookmark".to_string(),
            "delete".to_string(),
            "feature".to_string(),
            "hotfix".to_string()
        ])
    );

    let bookmark_rename = PromptKind::BookmarkRename;
    assert_eq!(
        bookmark_rename.to_tokens("old-name new-name"),
        Ok(vec![
            "bookmark".to_string(),
            "rename".to_string(),
            "old-name".to_string(),
            "new-name".to_string()
        ])
    );

    let restore = PromptKind::RestoreFrom {
        target_revision: "@".to_string(),
    };
    assert_eq!(
        restore.to_tokens(""),
        Ok(vec![
            "restore".to_string(),
            "--from".to_string(),
            "@-".to_string(),
            "--to".to_string(),
            "@".to_string()
        ])
    );

    let revert = PromptKind::RevertRevisions {
        default_revisions: "abc12345".to_string(),
        onto_revision: "@".to_string(),
    };
    assert_eq!(
        revert.to_tokens(""),
        Ok(vec![
            "revert".to_string(),
            "-r".to_string(),
            "abc12345".to_string(),
            "-o".to_string(),
            "@".to_string()
        ])
    );

    let operation_restore = PromptKind::OperationRestore;
    assert_eq!(
        operation_restore.to_tokens("abc123"),
        Ok(vec![
            "operation".to_string(),
            "restore".to_string(),
            "abc123".to_string()
        ])
    );

    let operation_revert = PromptKind::OperationRevert {
        default_operation: "@".to_string(),
    };
    assert_eq!(
        operation_revert.to_tokens(""),
        Ok(vec!["operation".to_string(), "revert".to_string()])
    );

    let workspace_add = PromptKind::WorkspaceAdd;
    assert_eq!(
        workspace_add.to_tokens("tmp/ws demo"),
        Ok(vec![
            "workspace".to_string(),
            "add".to_string(),
            "--name".to_string(),
            "demo".to_string(),
            "tmp/ws".to_string()
        ])
    );

    let metaedit = PromptKind::MetaeditMessage {
        revision: "abc12345".to_string(),
    };
    assert_eq!(
        metaedit.to_tokens("rewrite metadata"),
        Ok(vec![
            "metaedit".to_string(),
            "-m".to_string(),
            "rewrite metadata".to_string(),
            "abc12345".to_string()
        ])
    );

    let tag_set = PromptKind::TagSet {
        default_revision: "abc12345".to_string(),
    };
    assert_eq!(
        tag_set.to_tokens("v0.2.0"),
        Ok(vec![
            "tag".to_string(),
            "set".to_string(),
            "v0.2.0".to_string(),
            "--revision".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_eq!(
        tag_set.to_tokens("v0.2.0 main"),
        Ok(vec![
            "tag".to_string(),
            "set".to_string(),
            "v0.2.0".to_string(),
            "--revision".to_string(),
            "main".to_string()
        ])
    );
    assert_eq!(
        tag_set.to_tokens("v0.2.0 --revision release"),
        Ok(vec![
            "tag".to_string(),
            "set".to_string(),
            "v0.2.0".to_string(),
            "--revision".to_string(),
            "release".to_string()
        ])
    );

    let tag_delete = PromptKind::TagDelete;
    assert_eq!(
        tag_delete.to_tokens("v0.1.0 v0.2.0"),
        Ok(vec![
            "tag".to_string(),
            "delete".to_string(),
            "v0.1.0".to_string(),
            "v0.2.0".to_string()
        ])
    );

    let file_track = PromptKind::FileTrack;
    assert_eq!(
        file_track.to_tokens("src/app.rs src/flows.rs"),
        Ok(vec![
            "file".to_string(),
            "track".to_string(),
            "src/app.rs".to_string(),
            "src/flows.rs".to_string()
        ])
    );

    let file_untrack = PromptKind::FileUntrack;
    assert_eq!(
        file_untrack.to_tokens("target/generated.txt"),
        Ok(vec![
            "file".to_string(),
            "untrack".to_string(),
            "target/generated.txt".to_string()
        ])
    );

    let file_chmod = PromptKind::FileChmod {
        default_revision: "abc12345".to_string(),
    };
    assert_eq!(
        file_chmod.to_tokens("x scripts/deploy.sh"),
        Ok(vec![
            "file".to_string(),
            "chmod".to_string(),
            "x".to_string(),
            "scripts/deploy.sh".to_string(),
            "--revision".to_string(),
            "abc12345".to_string()
        ])
    );
    assert_eq!(
        file_chmod.to_tokens("n scripts/deploy.sh --revision @-"),
        Ok(vec![
            "file".to_string(),
            "chmod".to_string(),
            "n".to_string(),
            "scripts/deploy.sh".to_string(),
            "--revision".to_string(),
            "@-".to_string()
        ])
    );

    let workspace_forget = PromptKind::WorkspaceForget;
    assert_eq!(
        workspace_forget.to_tokens(""),
        Ok(vec!["workspace".to_string(), "forget".to_string()])
    );

    let workspace_rename = PromptKind::WorkspaceRename;
    assert_eq!(
        workspace_rename.to_tokens("main"),
        Ok(vec![
            "workspace".to_string(),
            "rename".to_string(),
            "main".to_string()
        ])
    );
}

#[test]
fn undo_and_redo_execute_directly() {
    assert_eq!(
        plan_command("undo", selected()),
        FlowAction::Execute(vec!["undo".to_string()])
    );
    assert_eq!(
        plan_command("redo", selected()),
        FlowAction::Execute(vec!["redo".to_string()])
    );
}

#[test]
fn next_and_prev_execute_directly() {
    assert_eq!(
        plan_command("next", selected()),
        FlowAction::Execute(vec!["next".to_string()])
    );
    assert_eq!(
        plan_command("prev", selected()),
        FlowAction::Execute(vec!["prev".to_string()])
    );
}

#[test]
fn renders_command_registry_in_app() {
    match plan_command("commands", selected()) {
        FlowAction::Render { lines, status } => {
            assert_eq!(status, "Showing command registry".to_string());
            assert!(
                lines
                    .iter()
                    .any(|line| line.contains("jj top-level coverage"))
            );
        }
        other => panic!("expected render action, got {other:?}"),
    }
}

#[test]
fn defaults_operation_workspace_resolve_file_and_tag_to_list_views() {
    assert_eq!(
        plan_command("op", selected()),
        FlowAction::Execute(vec!["operation".to_string(), "log".to_string()])
    );
    assert_eq!(
        plan_command("workspace", selected()),
        FlowAction::Execute(vec!["workspace".to_string(), "list".to_string()])
    );
    assert_eq!(
        plan_command("resolve", selected()),
        FlowAction::Execute(vec!["resolve".to_string(), "-l".to_string()])
    );
    assert_eq!(
        plan_command("file", selected()),
        FlowAction::Execute(vec!["file".to_string(), "list".to_string()])
    );
    assert_eq!(
        plan_command("tag", selected()),
        FlowAction::Execute(vec!["tag".to_string(), "list".to_string()])
    );
}

#[test]
fn canonicalizes_tag_short_subcommands() {
    assert_eq!(
        plan_command("tag l", selected()),
        FlowAction::Execute(vec!["tag".to_string(), "list".to_string()])
    );
    assert_eq!(
        plan_command("tag s v0.1.0", selected()),
        FlowAction::Execute(vec![
            "tag".to_string(),
            "set".to_string(),
            "v0.1.0".to_string()
        ])
    );
    assert_eq!(
        plan_command("tag d v0.1.0", selected()),
        FlowAction::Execute(vec![
            "tag".to_string(),
            "delete".to_string(),
            "v0.1.0".to_string()
        ])
    );
}

#[test]
fn adds_guided_tag_set_and_delete_flows() {
    match plan_command("tag set", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::TagSet {
                    default_revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("tag delete", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::TagDelete);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("tag s", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::TagSet {
                    default_revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("tag d", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::TagDelete);
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn adds_guided_file_track_and_untrack_flows() {
    match plan_command("file track", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::FileTrack);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("file untrack", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(request.kind, PromptKind::FileUntrack);
        }
        other => panic!("expected prompt, got {other:?}"),
    }

    match plan_command("file chmod", selected()) {
        FlowAction::Prompt(request) => {
            assert_eq!(
                request.kind,
                PromptKind::FileChmod {
                    default_revision: "abc12345".to_string()
                }
            );
        }
        other => panic!("expected prompt, got {other:?}"),
    }
}

#[test]
fn filters_command_registry_with_query() {
    match plan_command("commands work", selected()) {
        FlowAction::Render { lines, status } => {
            assert_eq!(status, "Showing command registry for `work`".to_string());
            assert!(lines.iter().any(|line| line.starts_with("workspace")));
            assert!(!lines.iter().any(|line| line.starts_with("rebase")));
        }
        other => panic!("expected render action, got {other:?}"),
    }
}

#[test]
fn renders_alias_catalog_in_app() {
    match plan_command("aliases", selected()) {
        FlowAction::Render { lines, status } => {
            assert_eq!(status, "Showing alias catalog".to_string());
            assert!(lines.iter().any(|line| line.contains("jjrbm")));
        }
        other => panic!("expected render action, got {other:?}"),
    }
}

#[test]
fn filters_alias_catalog_with_query() {
    match plan_command("aliases push", selected()) {
        FlowAction::Render { lines, status } => {
            assert_eq!(status, "Showing alias catalog for `push`".to_string());
            assert!(lines.iter().any(|line| line.contains("jjgp")));
            assert!(!lines.iter().any(|line| line.contains("jjrbm")));
        }
        other => panic!("expected render action, got {other:?}"),
    }
}

#[test]
fn parses_quoted_arguments_in_command_mode() {
    assert_eq!(
        plan_command(r#"git push --bookmark "team feature""#, selected()),
        FlowAction::Execute(vec![
            "git".to_string(),
            "push".to_string(),
            "--bookmark".to_string(),
            "team feature".to_string()
        ])
    );
}

#[test]
fn reports_invalid_quoted_command_input() {
    assert_eq!(
        plan_command(r#"git push --bookmark "unterminated"#, selected()),
        FlowAction::Status("Invalid command quoting".to_string())
    );
}
