const ALIAS_CATALOG: [(&str, &str); 44] = [
    ("b", "bookmark"),
    ("ci", "commit"),
    ("desc", "describe"),
    ("gf", "git fetch"),
    ("gp", "git push"),
    ("op", "operation"),
    ("rbm", "rebase -d main"),
    ("rbt", "rebase -d trunk()"),
    ("st", "status"),
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

#[cfg(test)]
mod tests {
    use super::{alias_overview_lines, alias_overview_lines_with_query};

    #[test]
    fn renders_alias_catalog_with_expected_entries() {
        let lines = alias_overview_lines();
        assert_eq!(lines.first(), Some(&"jk alias catalog".to_string()));
        assert!(lines.iter().any(|line| line.contains("desc")));
        assert!(lines.iter().any(|line| line.contains("rbm")));
        assert!(lines.iter().any(|line| line.contains("jjrt")));
    }

    #[test]
    fn catalog_includes_core_jj_default_aliases() {
        let lines = alias_overview_lines();
        for alias in ["b", "ci", "desc", "op", "st"] {
            assert!(
                lines.iter().any(|line| line.contains(alias)),
                "expected alias {alias} in catalog"
            );
        }
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
