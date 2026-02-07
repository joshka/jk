pub fn normalize_alias(tokens: &[String]) -> Vec<String> {
    if tokens.is_empty() {
        return vec!["log".to_string()];
    }

    let first = tokens[0].as_str();
    let Some(prefix) = alias_prefix(first) else {
        return tokens.to_vec();
    };

    let mut result = prefix.iter().map(ToString::to_string).collect::<Vec<_>>();
    result.extend_from_slice(&tokens[1..]);
    result
}

fn alias_prefix(alias: &str) -> Option<&'static [&'static str]> {
    match alias {
        "desc" | "jjds" => Some(&["describe"]),
        "jjdmsg" => Some(&["describe", "--message"]),
        "st" | "jjst" => Some(&["status"]),
        "gf" | "jjgf" => Some(&["git", "fetch"]),
        "gfa" | "jjgfa" => Some(&["git", "fetch", "--all-remotes"]),
        "gp" | "jjgp" => Some(&["git", "push"]),
        "gpt" | "jjgpt" => Some(&["git", "push", "--tracked"]),
        "gpa" | "jjgpa" => Some(&["git", "push", "--all"]),
        "gpd" | "jjgpd" => Some(&["git", "push", "--deleted"]),
        "rbm" => Some(&["rebase", "-d", "main"]),
        "rbt" | "jjrbm" => Some(&["rebase", "-d", "trunk()"]),
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
    fn maps_oh_my_zsh_aliases() {
        assert_eq!(
            normalize_alias(&to_vec(&["jjgf"])),
            to_vec(&["git", "fetch"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjgpt"])),
            to_vec(&["git", "push", "--tracked"])
        );
        assert_eq!(
            normalize_alias(&to_vec(&["jjrbm"])),
            to_vec(&["rebase", "-d", "trunk()"])
        );
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
