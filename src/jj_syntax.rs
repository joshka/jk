//! Pure helpers for building `jj` syntax fragments and display labels.
//!
//! Action builders use these helpers to keep revset/fileset quoting and
//! command-label formatting in one place without pulling in execution logic.

pub(crate) fn command_label_from_argv(command_argv: &[String]) -> String {
    if command_argv.is_empty() {
        "jj".to_owned()
    } else {
        format!("jj {}", command_argv.join(" "))
    }
}

pub(crate) fn exact_change_id_revset(change_id: &str) -> String {
    format!(
        "exactly(change_id({}), 1)",
        revset_string_literal(change_id)
    )
}

pub(crate) fn root_file_fileset(path: &str) -> String {
    format!("root-file:{}", revset_string_literal(path))
}

pub(crate) fn exact_string_pattern(value: &str) -> String {
    format!("exact:{}", revset_string_literal(value))
}

fn revset_string_literal(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_label_from_argv_formats_display_label_without_shell_quoting() {
        let label = command_label_from_argv(&[
            "git".to_owned(),
            "fetch".to_owned(),
            "--remote".to_owned(),
            "exact:\"origin/\\\"remote\"".to_owned(),
        ]);

        assert_eq!(label, "jj git fetch --remote exact:\"origin/\\\"remote\"");
    }

    #[test]
    fn command_label_from_argv_handles_empty_argv() {
        let label = command_label_from_argv(&[]);

        assert_eq!(label, "jj");
    }

    #[test]
    fn exact_change_id_revset_quotes_literal_prefix() {
        assert_eq!(
            exact_change_id_revset("abc\"\\"),
            "exactly(change_id(\"abc\\\"\\\\\"), 1)"
        );
    }

    #[test]
    fn root_file_fileset_quotes_spaces_quotes_backslashes_and_metacharacters() {
        assert_eq!(
            root_file_fileset("a b/\"c\"/d\\e[f]{g}(h)|i?*"),
            "root-file:\"a b/\\\"c\\\"/d\\\\e[f]{g}(h)|i?*\""
        );
    }

    #[test]
    fn exact_string_pattern_quotes_special_characters() {
        assert_eq!(
            exact_string_pattern("origin/\"remote\\name"),
            "exact:\"origin/\\\"remote\\\\name\""
        );
    }
}
