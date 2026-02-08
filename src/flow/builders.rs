//! Prompt input builders for guided commands.
//!
//! Builders enforce minimal argument contracts and normalize shorthand prompt formats before
//! converting to concrete command tokens.

/// Build bookmark mutation commands that target a specific revision.
pub(super) fn build_bookmark_target_command(
    subcommand: &str,
    input: &str,
    target_revision: &str,
    empty_message: &str,
    target_flag: &str,
) -> Result<Vec<String>, String> {
    if input.is_empty() {
        return Err(empty_message.to_string());
    }

    Ok(vec![
        "bookmark".to_string(),
        subcommand.to_string(),
        input.to_string(),
        target_flag.to_string(),
        target_revision.to_string(),
    ])
}

/// Build `bookmark track/untrack` with optional remote input.
///
/// Input must be `<bookmark> [remote]`.
///
/// # Examples
///
/// ```text
/// subcommand="track", input="feature origin"
/// => ["bookmark", "track", "feature", "--remote", "origin"]
/// ```
pub(super) fn build_track_command(subcommand: &str, input: &str) -> Result<Vec<String>, String> {
    if input.is_empty() {
        return Err("bookmark name is required".to_string());
    }

    let segments: Vec<&str> = input.split_whitespace().collect();
    if segments.len() > 2 {
        return Err("use format: <bookmark> [remote]".to_string());
    }

    let mut tokens = vec![
        "bookmark".to_string(),
        subcommand.to_string(),
        segments[0].to_string(),
    ];

    if let Some(remote) = segments.get(1) {
        tokens.push("--remote".to_string());
        tokens.push((*remote).to_string());
    }

    Ok(tokens)
}

/// Build bookmark commands that accept one or more bookmark names.
pub(super) fn build_bookmark_names_command(
    subcommand: &str,
    input: &str,
) -> Result<Vec<String>, String> {
    let names: Vec<&str> = input.split_whitespace().collect();
    if names.is_empty() {
        return Err("at least one bookmark name is required".to_string());
    }

    let mut tokens = vec!["bookmark".to_string(), subcommand.to_string()];
    tokens.extend(names.into_iter().map(ToString::to_string));
    Ok(tokens)
}

/// Build `bookmark rename` requiring exactly `<old> <new>`.
pub(super) fn build_bookmark_rename_command(input: &str) -> Result<Vec<String>, String> {
    let names: Vec<&str> = input.split_whitespace().collect();
    if names.len() != 2 {
        return Err("use format: <old> <new>".to_string());
    }

    Ok(vec![
        "bookmark".to_string(),
        "rename".to_string(),
        names[0].to_string(),
        names[1].to_string(),
    ])
}

/// Return whether tokens already specify a revision-like destination flag.
fn has_revision_flag(tokens: &[String]) -> bool {
    tokens.iter().any(|token| {
        matches!(token.as_str(), "-r" | "--revision" | "--to")
            || token.starts_with("-r=")
            || token.starts_with("--revision=")
            || token.starts_with("--to=")
    })
}

/// Build `tag set` and inject default revision when none is supplied.
///
/// # Examples
///
/// ```text
/// input="v1", default_revision="@"
/// => ["tag", "set", "v1", "--revision", "@"]
/// ```
///
/// ```text
/// input="v1 release", default_revision="@"
/// => ["tag", "set", "v1", "--revision", "release"]
/// ```
pub(super) fn build_tag_set_command(
    input: &str,
    default_revision: &str,
) -> Result<Vec<String>, String> {
    let segments: Vec<String> = input.split_whitespace().map(ToString::to_string).collect();
    if segments.is_empty() {
        return Err("at least one tag name is required".to_string());
    }

    let mut tokens = vec!["tag".to_string(), "set".to_string()];
    if segments.len() >= 2 && !segments[1].starts_with('-') && !has_revision_flag(&segments) {
        tokens.push(segments[0].clone());
        tokens.push("--revision".to_string());
        tokens.push(segments[1].clone());
        tokens.extend(segments.into_iter().skip(2));
        return Ok(tokens);
    }

    tokens.extend(segments.clone());
    if !has_revision_flag(&segments) {
        tokens.push("--revision".to_string());
        tokens.push(default_revision.to_string());
    }
    Ok(tokens)
}

/// Build `tag delete` for one or more tag names.
pub(super) fn build_tag_delete_command(input: &str) -> Result<Vec<String>, String> {
    let names: Vec<&str> = input.split_whitespace().collect();
    if names.is_empty() {
        return Err("at least one tag name is required".to_string());
    }

    let mut tokens = vec!["tag".to_string(), "delete".to_string()];
    tokens.extend(names.into_iter().map(ToString::to_string));
    Ok(tokens)
}

/// Build `workspace add` from `<destination> [name]`.
///
/// # Examples
///
/// ```text
/// input="../repo-copy feature-workspace"
/// => ["workspace", "add", "--name", "feature-workspace", "../repo-copy"]
/// ```
pub(super) fn build_workspace_add_command(input: &str) -> Result<Vec<String>, String> {
    let segments: Vec<&str> = input.split_whitespace().collect();
    match segments.as_slice() {
        [destination] => Ok(vec![
            "workspace".to_string(),
            "add".to_string(),
            (*destination).to_string(),
        ]),
        [destination, name] => Ok(vec![
            "workspace".to_string(),
            "add".to_string(),
            "--name".to_string(),
            (*name).to_string(),
            (*destination).to_string(),
        ]),
        _ => Err("use format: <destination> [name]".to_string()),
    }
}

/// Build `workspace forget` from optional whitespace-separated workspace names.
pub(super) fn build_workspace_forget_command(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = vec!["workspace".to_string(), "forget".to_string()];
    tokens.extend(
        input
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
    );
    Ok(tokens)
}

/// Build `file track` requiring at least one path/fileset.
pub(super) fn build_file_track_command(input: &str) -> Result<Vec<String>, String> {
    let paths: Vec<&str> = input.split_whitespace().collect();
    if paths.is_empty() {
        return Err("at least one file/fileset is required".to_string());
    }

    let mut tokens = vec!["file".to_string(), "track".to_string()];
    tokens.extend(paths.into_iter().map(ToString::to_string));
    Ok(tokens)
}

/// Build `file untrack` requiring at least one path/fileset.
pub(super) fn build_file_untrack_command(input: &str) -> Result<Vec<String>, String> {
    let paths: Vec<&str> = input.split_whitespace().collect();
    if paths.is_empty() {
        return Err("at least one file/fileset is required".to_string());
    }

    let mut tokens = vec!["file".to_string(), "untrack".to_string()];
    tokens.extend(paths.into_iter().map(ToString::to_string));
    Ok(tokens)
}

/// Build `file chmod` and inject default revision when unspecified.
pub(super) fn build_file_chmod_command(
    input: &str,
    default_revision: &str,
) -> Result<Vec<String>, String> {
    let parts: Vec<String> = input.split_whitespace().map(ToString::to_string).collect();
    if parts.len() < 2 {
        return Err("use format: <mode> <path...> [--revision REVSET]".to_string());
    }

    let mut tokens = vec!["file".to_string(), "chmod".to_string()];
    tokens.extend(parts.clone());
    if !has_revision_flag(&parts) {
        tokens.push("--revision".to_string());
        tokens.push(default_revision.to_string());
    }
    Ok(tokens)
}
