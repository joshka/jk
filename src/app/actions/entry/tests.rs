use super::*;

#[test]
fn push_remote_prompt_decision_reports_missing_remotes() {
    assert_eq!(
        decide_push_remote_prompt(Ok(Vec::new())),
        PushRemotePromptDecision::MissingRemotes {
            message: "no git remotes found; add a remote before pushing".to_owned(),
        },
    );
}

#[test]
fn push_remote_prompt_decision_opens_preview_for_single_remote() {
    assert_eq!(
        decide_push_remote_prompt(Ok(vec!["origin".to_owned()])),
        PushRemotePromptDecision::OpenPreview {
            remote: "origin".to_owned(),
        },
    );
}

#[test]
fn push_remote_prompt_decision_prompts_for_multiple_remotes() {
    assert_eq!(
        decide_push_remote_prompt(Ok(vec!["origin".to_owned(), "upstream".to_owned()])),
        PushRemotePromptDecision::Prompt {
            remotes: vec!["origin".to_owned(), "upstream".to_owned()],
        },
    );
}

#[test]
fn push_remote_prompt_decision_reports_remote_list_error() {
    assert_eq!(
        decide_push_remote_prompt(Err("jj git remote list failed: denied".to_owned())),
        PushRemotePromptDecision::RemoteListError {
            message: "jj git remote list failed: denied".to_owned(),
        },
    );
}

#[test]
fn fetch_remote_prompt_decision_reports_missing_remotes() {
    assert_eq!(
        decide_fetch_remote_prompt(Ok(Vec::new())),
        FetchRemotePromptDecision::MissingRemotes {
            message: "no git remotes found; run default fetch or add a remote before choosing one"
                .to_owned(),
            status_context: "fetch remote selection found no remotes".to_owned(),
        },
    );
}

#[test]
fn fetch_remote_prompt_decision_opens_preview_for_single_remote() {
    assert_eq!(
        decide_fetch_remote_prompt(Ok(vec!["origin".to_owned()])),
        FetchRemotePromptDecision::OpenPreview {
            remote: "origin".to_owned(),
        },
    );
}

#[test]
fn fetch_remote_prompt_decision_prompts_for_multiple_remotes() {
    assert_eq!(
        decide_fetch_remote_prompt(Ok(vec!["origin".to_owned(), "upstream".to_owned()])),
        FetchRemotePromptDecision::Prompt {
            remotes: vec!["origin".to_owned(), "upstream".to_owned()],
        },
    );
}

#[test]
fn fetch_remote_prompt_decision_reports_remote_list_error() {
    assert_eq!(
        decide_fetch_remote_prompt(Err("jj git remote list failed: denied".to_owned())),
        FetchRemotePromptDecision::RemoteListError {
            message: "jj git remote list failed: denied".to_owned(),
            status_context: "fetch remote selection failed to list remotes".to_owned(),
        },
    );
}
