use crate::jj::command::option_value;
use crate::jj::view_spec::DiffFormat;

/// Infer the app-level diff-format modal state from direct startup args.
pub fn parse_diff_format(args: &[String]) -> DiffFormat {
    if args.iter().any(|arg| arg == "--git") {
        DiffFormat::Git
    } else {
        DiffFormat::Default
    }
}

/// Prepend the app-level diff-format flag when this spec should render with `jj --git`.
pub fn diff_format_args(
    diff_format: DiffFormat,
    args: impl IntoIterator<Item = String>,
) -> Vec<String> {
    diff_format
        .arg()
        .into_iter()
        .map(str::to_owned)
        .chain(args)
        .collect()
}

/// Parse the positional revset used by `jj show` while skipping option values safely.
pub fn show_revset_arg(args: &[String]) -> Option<&str> {
    let mut skip_next = false;

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if show_option_takes_value(arg) {
            skip_next = !arg.contains('=');
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        return Some(arg);
    }

    None
}

fn show_option_takes_value(arg: &str) -> bool {
    matches!(
        arg,
        "-T" | "--template"
            | "--tool"
            | "--context"
            | "-R"
            | "--repository"
            | "--at-operation"
            | "--at-op"
            | "--config"
            | "--config-file"
    ) || [
        "--template=",
        "--tool=",
        "--context=",
        "--repository=",
        "--at-operation=",
        "--at-op=",
        "--config=",
        "--config-file=",
    ]
    .iter()
    .any(|prefix| arg.starts_with(prefix))
}

/// Parse the revision context for `jj diff` startup args.
pub fn diff_revset_arg(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revisions"], &["--revisions="])
        .or_else(|| option_value(args, &["-t", "--to"], &["--to="]))
}

/// Parse a single `-r` / `--revision` value from startup args.
pub fn revision_arg(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revision"], &["--revision="])
}
