//! Alias normalization from user-entered tokens to canonical `jj` command tokens.

/// Normalize startup/command-mode tokens into canonical planner input.
///
/// Empty input defaults to `log` so runtime always starts from a stable home view.
pub fn normalize_alias(tokens: &[String]) -> Vec<String> {
    if tokens.is_empty() {
        return vec!["log".to_string()];
    }

    let first = tokens[0].as_str();
    if let Some(tokens) = normalize_destination_alias(first, tokens) {
        return tokens;
    }

    let Some(prefix) = alias_prefix(first) else {
        return tokens.to_vec();
    };

    let mut result = prefix.iter().map(ToString::to_string).collect::<Vec<_>>();
    result.extend_from_slice(&tokens[1..]);
    result
}

/// Resolve destination-aware rebase aliases with explicit-flag precedence.
///
/// If caller already passes destination flags, this preserves them instead of appending defaults.
fn normalize_destination_alias(alias: &str, tokens: &[String]) -> Option<Vec<String>> {
    let default_destination = match alias {
        "rbm" => "main",
        "rbt" | "jjrbm" => "trunk()",
        _ => return None,
    };

    let remainder = &tokens[1..];
    if has_destination_flag(remainder) {
        let mut result = vec!["rebase".to_string()];
        result.extend(remainder.iter().cloned());
        return Some(result);
    }

    let (destination, tail) = match remainder.first() {
        Some(value) if !value.starts_with('-') => (value.clone(), &remainder[1..]),
        _ => (default_destination.to_string(), remainder),
    };

    let mut result = vec!["rebase".to_string(), "-d".to_string(), destination];
    result.extend(tail.iter().cloned());
    Some(result)
}

/// Return whether tokens already include an explicit destination selector.
fn has_destination_flag(tokens: &[String]) -> bool {
    tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "-d" | "-t" | "--destination" | "--to" | "--into"
        ) || token.starts_with("-d=")
            || token.starts_with("-t=")
            || token.starts_with("--destination=")
            || token.starts_with("--to=")
            || token.starts_with("--into=")
    })
}

/// Resolve single-token aliases to prefix command segments.
///
/// The remainder of input arguments is appended unchanged by the caller.
fn alias_prefix(alias: &str) -> Option<&'static [&'static str]> {
    match alias {
        "b" => Some(&["bookmark"]),
        "ci" => Some(&["commit"]),
        "desc" | "jjds" => Some(&["describe"]),
        "jjdmsg" => Some(&["describe", "--message"]),
        "op" => Some(&["operation"]),
        "st" | "jjst" => Some(&["status"]),
        "gf" | "jjgf" => Some(&["git", "fetch"]),
        "gfa" | "jjgfa" => Some(&["git", "fetch", "--all-remotes"]),
        "gp" | "jjgp" => Some(&["git", "push"]),
        "gpt" | "jjgpt" => Some(&["git", "push", "--tracked"]),
        "gpa" | "jjgpa" => Some(&["git", "push", "--all"]),
        "gpd" | "jjgpd" => Some(&["git", "push", "--deleted"]),
        "jjrb" => Some(&["rebase"]),
        "jjl" => Some(&["log"]),
        "jjla" => Some(&["log", "-r", "all()"]),
        "jjd" => Some(&["diff"]),
        "jjc" => Some(&["commit"]),
        "jjcmsg" => Some(&["commit", "--message"]),
        "jjn" => Some(&["new"]),
        "jjnt" => Some(&["new", "trunk()"]),
        "jje" => Some(&["edit"]),
        "jjsp" => Some(&["split"]),
        "jjsq" => Some(&["squash"]),
        "jjrs" => Some(&["restore"]),
        "jja" => Some(&["abandon"]),
        "jjgcl" => Some(&["git", "clone"]),
        "jjb" => Some(&["bookmark"]),
        "jjbc" => Some(&["bookmark", "create"]),
        "jjbd" => Some(&["bookmark", "delete"]),
        "jjbf" => Some(&["bookmark", "forget"]),
        "jjbl" => Some(&["bookmark", "list"]),
        "jjbm" => Some(&["bookmark", "move"]),
        "jjbr" => Some(&["bookmark", "rename"]),
        "jjbs" => Some(&["bookmark", "set"]),
        "jjbt" => Some(&["bookmark", "track"]),
        "jjbu" => Some(&["bookmark", "untrack"]),
        "jjrt" => Some(&["root"]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_alias;

    fn to_vec(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn defaults_to_log_when_empty() {
        assert_eq!(normalize_alias(&[]), to_vec(&["log"]));
    }

    #[test]
    fn maps_core_short_aliases() {
        assert_eq!(normalize_alias(&to_vec(&["b"])), to_vec(&["bookmark"]));
        assert_eq!(normalize_alias(&to_vec(&["ci"])), to_vec(&["commit"]));
        assert_eq!(normalize_alias(&to_vec(&["desc"])), to_vec(&["describe"]));
        assert_eq!(normalize_alias(&to_vec(&["op"])), to_vec(&["operation"]));
        assert_eq!(normalize_alias(&to_vec(&["st"])), to_vec(&["status"]));
        assert_eq!(normalize_alias(&to_vec(&["gf"])), to_vec(&["git", "fetch"]));
        assert_eq!(normalize_alias(&to_vec(&["gp"])), to_vec(&["git", "push"]));
        assert_eq!(
            normalize_alias(&to_vec(&["rbm"])),
            to_vec(&["rebase", "-d", "main"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["rbt"])),
            to_vec(&["rebase", "-d", "trunk()"])
        );
    }

    #[test]
    fn supports_destination_overrides_for_rebase_shortcuts() {
        assert_eq!(
            normalize_alias(&to_vec(&["rbm", "release"])),
            to_vec(&["rebase", "-d", "release"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["rbt", "main"])),
            to_vec(&["rebase", "-d", "main"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjrbm", "main"])),
            to_vec(&["rebase", "-d", "main"])
        );
    }

    #[test]
    fn keeps_flag_arguments_for_rebase_shortcuts() {
        assert_eq!(
            normalize_alias(&to_vec(&["rbm", "-r", "abc123"])),
            to_vec(&["rebase", "-d", "main", "-r", "abc123"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["rbm", "release", "-r", "abc123"])),
            to_vec(&["rebase", "-d", "release", "-r", "abc123"])
        );
    }

    #[test]
    fn respects_explicit_destination_flags_for_rebase_shortcuts() {
        assert_eq!(
            normalize_alias(&to_vec(&["rbm", "-d", "release"])),
            to_vec(&["rebase", "-d", "release"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["rbm", "--destination", "release"])),
            to_vec(&["rebase", "--destination", "release"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["rbt", "--to", "main"])),
            to_vec(&["rebase", "--to", "main"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjrbm", "--into=release"])),
            to_vec(&["rebase", "--into=release"])
        );
    }

    #[test]
    fn maps_oh_my_zsh_aliases() {
        assert_eq!(
            normalize_alias(&to_vec(&["jjgf"])),
            to_vec(&["git", "fetch"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjgfa"])),
            to_vec(&["git", "fetch", "--all-remotes"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjgpt"])),
            to_vec(&["git", "push", "--tracked"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjgpa"])),
            to_vec(&["git", "push", "--all"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjgpd"])),
            to_vec(&["git", "push", "--deleted"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjrbm"])),
            to_vec(&["rebase", "-d", "trunk()"])
        );
        assert_eq!(normalize_alias(&to_vec(&["jjst"])), to_vec(&["status"]));
        assert_eq!(normalize_alias(&to_vec(&["jjl"])), to_vec(&["log"]));
        assert_eq!(normalize_alias(&to_vec(&["jjrt"])), to_vec(&["root"]));
        assert_eq!(
            normalize_alias(&to_vec(&["jjbl"])),
            to_vec(&["bookmark", "list"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjbt"])),
            to_vec(&["bookmark", "track"])
        );
        assert_eq!(normalize_alias(&to_vec(&["jjds"])), to_vec(&["describe"]));
        assert_eq!(
            normalize_alias(&to_vec(&["jjdmsg", "fix", "msg"])),
            to_vec(&["describe", "--message", "fix", "msg"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjcmsg", "ship", "it"])),
            to_vec(&["commit", "--message", "ship", "it"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjla"])),
            to_vec(&["log", "-r", "all()"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjnt"])),
            to_vec(&["new", "trunk()"])
        );
        assert_eq!(normalize_alias(&to_vec(&["jjsq"])), to_vec(&["squash"]));
        assert_eq!(
            normalize_alias(&to_vec(&["jjbr"])),
            to_vec(&["bookmark", "rename"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjgcl"])),
            to_vec(&["git", "clone"])
        );
    }

    #[test]
    fn leaves_regular_commands_unchanged() {
        let input = to_vec(&["log", "-n", "10"]);
        assert_eq!(normalize_alias(&input), input);
    }
}
