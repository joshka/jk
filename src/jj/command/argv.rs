use crate::jj::ViewSpec;

use super::command_words;

pub const TRUNK_WORK_REVSET: &str = "trunk().. | trunk()";
pub const RECENT_WORK_REVSET: &str = "latest(mutable(), 20) | @ | trunk()";
pub const ALL_REPO_REVSET: &str = "all()";
pub const JJ_GIT_REMOTE_ARGS: [&str; 3] = ["git", "remote", "list"];
pub const NEW_TRUNK_ARGS: [&str; 2] = ["new", "trunk()"];
pub const CHANGE_ID_TEMPLATE: &str = "change_id ++ \"\\n\"";
pub const OPERATION_LOG_LIMIT: &str = "100";

pub fn workspace_root_command_args() -> Vec<String> {
    vec!["root".to_owned()]
}

pub fn resolve_exact_change_id_command_argv(revset: &str) -> Vec<String> {
    vec![
        "log".to_owned(),
        "--no-graph".to_owned(),
        "-r".to_owned(),
        revset.to_owned(),
        "-T".to_owned(),
        CHANGE_ID_TEMPLATE.to_owned(),
    ]
}

/// Build the `jj` argv for one `ViewSpec`, optionally overriding the template or graph mode.
pub fn jj_command_args(spec: &ViewSpec, template: Option<&str>, no_graph: bool) -> Vec<String> {
    let mut args = command_words(spec)
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect::<Vec<_>>();
    args.extend(
        spec.command()
            .prefix_args()
            .iter()
            .map(|arg| (*arg).to_owned()),
    );
    if no_graph {
        args.push("--no-graph".to_owned());
    }
    if let Some(template) = template {
        args.push("-T".to_owned());
        args.push(template.to_owned());
    }
    args.extend(spec.args().iter().cloned());
    args
}

/// Find the value associated with a flag that may use either `--flag value` or `--flag=value`.
pub fn option_value<'a>(
    args: &'a [String],
    value_options: &[&str],
    value_prefixes: &[&str],
) -> Option<&'a str> {
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        if value_options.contains(&arg.as_str()) {
            return args.next().map(String::as_str);
        }
        if let Some(value) = value_prefixes
            .iter()
            .find_map(|prefix| arg.strip_prefix(prefix))
        {
            return Some(value);
        }
    }

    None
}
