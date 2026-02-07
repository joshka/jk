pub fn normalize_alias(tokens: &[String]) -> Vec<String> {
    if tokens.is_empty() {
        return vec!["log".to_string()];
    }

    let mut result = Vec::new();
    let first = tokens[0].as_str();

    match first {
        "gf" | "jjgf" => {
            result.extend(["git", "fetch"].into_iter().map(ToString::to_string));
            result.extend_from_slice(&tokens[1..]);
        }
        "gfa" | "jjgfa" => {
            result.extend(
                ["git", "fetch", "--all-remotes"]
                    .into_iter()
                    .map(ToString::to_string),
            );
            result.extend_from_slice(&tokens[1..]);
        }
        "gp" | "jjgp" => {
            result.extend(["git", "push"].into_iter().map(ToString::to_string));
            result.extend_from_slice(&tokens[1..]);
        }
        "gpt" | "jjgpt" => {
            result.extend(
                ["git", "push", "--tracked"]
                    .into_iter()
                    .map(ToString::to_string),
            );
            result.extend_from_slice(&tokens[1..]);
        }
        "gpa" | "jjgpa" => {
            result.extend(
                ["git", "push", "--all"]
                    .into_iter()
                    .map(ToString::to_string),
            );
            result.extend_from_slice(&tokens[1..]);
        }
        "gpd" | "jjgpd" => {
            result.extend(
                ["git", "push", "--deleted"]
                    .into_iter()
                    .map(ToString::to_string),
            );
            result.extend_from_slice(&tokens[1..]);
        }
        "rbm" => {
            result.extend(
                ["rebase", "-d", "main"]
                    .into_iter()
                    .map(ToString::to_string),
            );
            result.extend_from_slice(&tokens[1..]);
        }
        "rbt" | "jjrbm" => {
            result.extend(
                ["rebase", "-d", "trunk()"]
                    .into_iter()
                    .map(ToString::to_string),
            );
            result.extend_from_slice(&tokens[1..]);
        }
        "jjrb" => {
            result.push("rebase".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        "jjst" => {
            result.push("status".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        "jjl" => {
            result.push("log".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        "jjd" => {
            result.push("diff".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        "jjc" => {
            result.push("commit".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        "jjrs" => {
            result.push("restore".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        "jja" => {
            result.push("abandon".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        "jjrt" => {
            result.push("root".to_string());
            result.extend_from_slice(&tokens[1..]);
        }
        _ => return tokens.to_vec(),
    }

    result
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
    }

    #[test]
    fn leaves_regular_commands_unchanged() {
        let input = to_vec(&["log", "-n", "10"]);
        assert_eq!(normalize_alias(&input), input);
    }
}
