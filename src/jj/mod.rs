//! Command construction and output loading for the `jj` CLI.
//!
//! `jk` intentionally treats `jj`'s rendered terminal output as the product
//! contract. Shelling out keeps user config, templates, graph symbols, colors,
//! and future jj formatting behavior aligned with the CLI instead of rebuilding
//! a parallel view from repository data.

mod command;
mod process;
mod syntax;
mod view_spec;

pub use command::{JjCommand, LogViewMode};
pub use process::{git_remotes, new_trunk, resolve_exact_change_id};
pub use syntax::{
    command_label_from_argv, exact_change_id_revset, exact_string_pattern, root_file_fileset,
};
pub use view_spec::{DiffFormat, ViewSpec};

pub use process::{
    ColorMode, base_command, interactive_jj_command, load_workspace_root, run_direct_args,
    run_direct_args_stdout, run_jj, run_jj_template_lines, summarize_output,
};

#[cfg(test)]
pub use command::{
    ALL_REPO_REVSET, CHANGE_ID_TEMPLATE, NEW_TRUNK_ARGS, OPERATION_LOG_LIMIT, RECENT_WORK_REVSET,
    TRUNK_WORK_REVSET, jj_command_args, resolve_exact_change_id_command_argv,
    workspace_root_command_args,
};
#[cfg(test)]
pub use process::{parse_exact_change_id, parse_git_remotes};

#[cfg(test)]
mod tests;
