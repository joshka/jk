//! Command construction and output loading for the `jj` CLI.
//!
//! `jk` intentionally treats `jj`'s rendered terminal output as the product
//! contract. Shelling out keeps user config, templates, graph symbols, colors,
//! and future jj formatting behavior aligned with the CLI instead of rebuilding
//! a parallel view from repository data.
//!
//! The root module intentionally re-exports four kinds of boundary surface:
//! command vocabulary (`Command`, `LogViewMode`), `ViewSpec` state that
//! shapes one CLI invocation, syntax helpers for quoting and labels, and
//! process helpers that actually run `jj`.

mod command;
mod process;
mod syntax;
mod view_spec;

#[cfg(test)]
pub use command::{
    ALL_REPO_REVSET, CHANGE_ID_TEMPLATE, NEW_TRUNK_ARGS, OPERATION_LOG_LIMIT, RECENT_WORK_REVSET,
    TRUNK_WORK_REVSET, jj_command_args, jj_command_args_with_template,
    jj_command_args_with_template_no_graph, resolve_exact_change_id_command_argv,
    workspace_root_command_args,
};
/// `jj` command families and named log-mode presets used across app and view code.
pub use command::{Command, LogViewMode};
/// Process-level helpers used by higher layers once they have already chosen a `ViewSpec` or
/// argv.
pub use process::{
    ColorMode, base_command, interactive_jj_command, load_workspace_root, run_direct_args,
    run_direct_args_stdout, run_jj, run_jj_template_lines, run_jj_template_lines_no_graph,
    summarize_output,
};
/// Read-only helpers for direct repository queries that are not tied to one `ViewSpec`.
pub use process::{git_remotes, new_trunk, resolve_exact_change_id};
#[cfg(test)]
pub use process::{parse_exact_change_id, parse_git_remotes};
/// String-shaping helpers for revsets, filesets, and human-readable command labels.
pub use syntax::{
    command_label_from_argv, exact_change_id_revset, exact_string_pattern, root_file_fileset,
};
/// Stateful description of one rendered `jj` surface.
pub use view_spec::{DiffFormat, ViewSpec};

#[cfg(test)]
mod tests;
