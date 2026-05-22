//! Process execution helpers for the `jj` CLI boundary.
//!
//! Higher layers decide which `ViewSpec` or direct argv to run. This root keeps
//! the stable process-facing export surface while child modules separate
//! `ViewSpec` execution from direct repository queries and command launching.

mod direct;
mod run;

#[cfg(test)]
pub use self::direct::parse_git_remotes;
pub use self::direct::{
    git_remotes, interactive_jj_command, load_workspace_root, new_trunk, resolve_exact_change_id,
    run_direct_args, run_direct_args_stdout,
};
#[cfg(test)]
pub use self::run::parse_exact_change_id;
pub use self::run::{ColorMode, base_command, run_jj, run_jj_template_lines, summarize_output};
