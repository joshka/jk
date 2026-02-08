//! Safety classification and read-only preview planning for confirmation mode.

use crate::commands::{SafetyTier, command_safety};

/// Return whether command tokens require confirmation gating.
pub(crate) fn is_dangerous(tokens: &[String]) -> bool {
    command_safety(tokens) == SafetyTier::C
}

/// Derive best-effort read-only preview tokens for dangerous commands.
///
/// Returning `None` means no safe preview strategy is known. Callers must still enforce explicit
/// confirmation before execution.
pub(crate) fn confirmation_preview_tokens(tokens: &[String]) -> Option<Vec<String>> {
    if matches!(
        (
            tokens.first().map(String::as_str),
            tokens.get(1).map(String::as_str)
        ),
        (Some("git"), Some("push"))
    ) && !tokens.iter().any(|token| token == "--dry-run")
    {
        let mut preview = tokens.to_vec();
        preview.push("--dry-run".to_string());
        return Some(preview);
    }

    if matches!(
        (
            tokens.first().map(String::as_str),
            tokens.get(1).map(String::as_str)
        ),
        (Some("operation"), Some("restore" | "revert"))
    ) {
        let operation = tokens
            .get(2)
            .filter(|value| !value.starts_with('-'))
            .cloned()
            .unwrap_or_else(|| "@".to_string());
        return Some(vec![
            "operation".to_string(),
            "show".to_string(),
            operation,
            "--no-op-diff".to_string(),
        ]);
    }

    match tokens.first().map(String::as_str) {
        Some("rebase") => {
            let source = find_flag_value(tokens, &["-r", "--revision", "-b", "--branch"])
                .unwrap_or_else(|| "@".to_string());
            let destination = find_flag_value(tokens, &["-d", "--destination", "--onto"])?;
            Some(log_preview_tokens(&format!("{source} | {destination}")))
        }
        Some("squash") => {
            let from = find_flag_value(tokens, &["--from"]).unwrap_or_else(|| "@".to_string());
            let into = find_flag_value(tokens, &["--into"]).unwrap_or_else(|| "@-".to_string());
            Some(log_preview_tokens(&format!("{from} | {into}")))
        }
        Some("split") => {
            let revision =
                find_flag_value(tokens, &["-r", "--revision"]).unwrap_or_else(|| "@".to_string());
            Some(vec!["show".to_string(), revision])
        }
        Some("abandon") => {
            let revision = tokens.get(1).cloned().unwrap_or_else(|| "@".to_string());
            Some(log_preview_tokens(&revision))
        }
        Some("restore") => {
            let from = find_flag_value(tokens, &["--from"]).unwrap_or_else(|| "@-".to_string());
            let to = find_flag_value(tokens, &["--to"]).unwrap_or_else(|| "@".to_string());
            Some(log_preview_tokens(&format!("{from} | {to}")))
        }
        Some("revert") => {
            let revisions =
                find_flag_value(tokens, &["-r", "--revisions"]).unwrap_or_else(|| "@".to_string());
            let onto =
                find_flag_value(tokens, &["-o", "--onto"]).unwrap_or_else(|| "@".to_string());
            Some(log_preview_tokens(&format!("{revisions} | {onto}")))
        }
        Some("bookmark")
            if matches!(
                tokens.get(1).map(String::as_str),
                Some("set" | "move" | "delete" | "forget" | "rename")
            ) =>
        {
            Some(vec![
                "bookmark".to_string(),
                "list".to_string(),
                "--all".to_string(),
            ])
        }
        Some("git") if matches!(tokens.get(1).map(String::as_str), Some("push")) => None,
        Some("undo" | "redo") => Some(operation_log_preview_tokens()),
        _ if is_dangerous(tokens) => Some(operation_log_preview_tokens()),
        _ => None,
    }
}

/// Find the first value associated with one of the supported flags.
pub(crate) fn find_flag_value(tokens: &[String], flags: &[&str]) -> Option<String> {
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        for flag in flags {
            if token == flag {
                if let Some(value) = tokens.get(index + 1) {
                    return Some(value.clone());
                }
            } else if let Some(value) = token.strip_prefix(&format!("{flag}=")) {
                return Some(value.to_string());
            }
        }
        index += 1;
    }

    None
}

/// Build a compact log preview command for a revset expression.
pub(crate) fn log_preview_tokens(revset: &str) -> Vec<String> {
    vec![
        "log".to_string(),
        "-r".to_string(),
        revset.to_string(),
        "-n".to_string(),
        "20".to_string(),
    ]
}

/// Toggle `--patch`/`-p` on command tokens by removing existing patch flags or appending one.
pub(crate) fn toggle_patch_flag(tokens: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(tokens.len() + 1);
    let mut has_patch = false;

    for token in tokens {
        if token == "-p" || token == "--patch" {
            has_patch = true;
            continue;
        }
        result.push(token.clone());
    }

    if !has_patch {
        result.push("--patch".to_string());
    }

    result
}

/// Build operation-log preview tokens used as a generic dangerous-command fallback.
pub(crate) fn operation_log_preview_tokens() -> Vec<String> {
    vec![
        "operation".to_string(),
        "log".to_string(),
        "-n".to_string(),
        "5".to_string(),
    ]
}
