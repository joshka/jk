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

const ALIAS_CATALOG: [(&str, &str); 39] = [
    ("gf", "git fetch"),
    ("gp", "git push"),
    ("rbm", "rebase -d main"),
    ("rbt", "rebase -d trunk()"),
    ("jja", "abandon"),
    ("jjb", "bookmark (defaults to list in jk)"),
    ("jjbc", "bookmark create"),
    ("jjbd", "bookmark delete"),
    ("jjbf", "bookmark forget"),
    ("jjbl", "bookmark list"),
    ("jjbm", "bookmark move"),
    ("jjbr", "bookmark rename"),
    ("jjbs", "bookmark set"),
    ("jjbt", "bookmark track"),
    ("jjbu", "bookmark untrack"),
    ("jjc", "commit"),
    ("jjcmsg", "commit --message"),
    ("jjd", "diff"),
    ("jjdmsg", "describe --message"),
    ("jjds", "describe"),
    ("jje", "edit"),
    ("jjgcl", "git clone"),
    ("jjgf", "git fetch"),
    ("jjgfa", "git fetch --all-remotes"),
    ("jjgp", "git push"),
    ("jjgpa", "git push --all"),
    ("jjgpd", "git push --deleted"),
    ("jjgpt", "git push --tracked"),
    ("jjl", "log"),
    ("jjla", "log -r all()"),
    ("jjn", "new"),
    ("jjnt", "new trunk()"),
    ("jjrb", "rebase"),
    ("jjrbm", "rebase -d trunk()"),
    ("jjrs", "restore"),
    ("jjrt", "root (in-app equivalent of plugin cd alias)"),
    ("jjsp", "split"),
    ("jjsq", "squash"),
    ("jjst", "status"),
];

pub fn alias_overview_lines() -> Vec<String> {
    alias_overview_lines_with_query(None)
}

pub fn alias_overview_lines_with_query(query: Option<&str>) -> Vec<String> {
    let filter = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    let mut lines = vec![
        "jk alias catalog".to_string(),
        format!("{:<8} {}", "alias", "expands to"),
        "-".repeat(40),
    ];

    for (alias, expansion) in ALIAS_CATALOG {
        if let Some(filter) = &filter
            && !alias.to_ascii_lowercase().contains(filter)
            && !expansion.to_ascii_lowercase().contains(filter)
        {
            continue;
        }
        lines.push(format!("{:<8} {}", alias, expansion));
    }

    lines
}

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
    use super::{alias_overview_lines, alias_overview_lines_with_query, normalize_alias};

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
        assert_eq!(normalize_alias(&to_vec(&["op"])), to_vec(&["operation"]));
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

    #[test]
    fn renders_alias_catalog_with_expected_entries() {
        let lines = alias_overview_lines();
        assert_eq!(lines.first(), Some(&"jk alias catalog".to_string()));
        assert!(lines.iter().any(|line| line.contains("rbm")));
        assert!(lines.iter().any(|line| line.contains("jjrt")));
    }

    #[test]
    fn filters_alias_catalog_by_query() {
        let lines = alias_overview_lines_with_query(Some("push"));
        assert!(lines.iter().any(|line| line.contains("jjgp")));
        assert!(!lines.iter().any(|line| line.contains("jjrbm")));
    }

    #[test]
    fn catalog_includes_oh_my_zsh_plugin_aliases() {
        let lines = alias_overview_lines();
        for alias in [
            "jja", "jjb", "jjbc", "jjbd", "jjbf", "jjbl", "jjbm", "jjbr", "jjbs", "jjbt", "jjbu",
            "jjc", "jjcmsg", "jjd", "jjdmsg", "jjds", "jje", "jjgcl", "jjgf", "jjgfa", "jjgp",
            "jjgpa", "jjgpd", "jjgpt", "jjl", "jjla", "jjn", "jjnt", "jjrb", "jjrbm", "jjrs",
            "jjrt", "jjsp", "jjsq", "jjst",
        ] {
            assert!(
                lines.iter().any(|line| line.contains(alias)),
                "expected alias {alias} in catalog"
            );
        }
    }
}
