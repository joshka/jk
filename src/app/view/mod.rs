//! Command-output wrapper rendering.
//!
//! Wrapper functions add concise headings, summaries, and follow-up tips while preserving original
//! command output lines for detailed inspection.

mod common;
mod file_tag;
mod operation;
mod root_resolve;
mod status_diff;
mod workspace_git_top;

use super::selection::{looks_like_graph_commit_row, strip_ansi};

#[allow(unused_imports)]
pub(crate) use file_tag::{
    render_file_annotate_view, render_file_chmod_view, render_file_list_view,
    render_file_search_view, render_file_show_view, render_file_track_view,
    render_file_untrack_view, render_tag_delete_view, render_tag_list_view, render_tag_set_view,
};
#[allow(unused_imports)]
pub(crate) use operation::{
    operation_mutation_summary, render_operation_diff_view, render_operation_log_view,
    render_operation_mutation_view, render_operation_show_view,
};
#[allow(unused_imports)]
pub(crate) use root_resolve::{
    has_resolve_list_flag, render_resolve_action_view, render_resolve_list_view, render_root_view,
    render_version_view,
};
#[allow(unused_imports)]
pub(crate) use status_diff::{
    key_binding_label, key_binding_labels, keymap_overview_lines, normalize_diff_lines,
    normalize_show_lines, render_diff_view, render_evolog_view, render_interdiff_view,
    render_show_view, render_status_view,
};
#[allow(unused_imports)]
pub(crate) use workspace_git_top::{
    bookmark_mutation_summary, git_fetch_summary, git_push_summary, render_bookmark_list_view,
    render_bookmark_mutation_view, render_git_fetch_view, render_git_push_view,
    render_top_level_mutation_view, render_workspace_list_view, render_workspace_mutation_view,
    top_level_mutation_summary, workspace_mutation_summary,
};

/// Decorate raw command output with command-specific wrapper views when available.
pub(crate) fn decorate_command_output(command: &[String], output: Vec<String>) -> Vec<String> {
    match command.first().map(String::as_str) {
        Some("status") => render_status_view(output),
        Some("show") => render_show_view(output),
        Some("diff") => render_diff_view(output),
        Some("diffedit") => render_top_level_mutation_view("diffedit", output),
        Some("interdiff") => render_interdiff_view(output),
        Some("evolog") => render_evolog_view(output),
        Some("new") => render_top_level_mutation_view("new", output),
        Some("describe") => render_top_level_mutation_view("describe", output),
        Some("commit") => render_top_level_mutation_view("commit", output),
        Some("metaedit") => render_top_level_mutation_view("metaedit", output),
        Some("edit") => render_top_level_mutation_view("edit", output),
        Some("next") => render_top_level_mutation_view("next", output),
        Some("prev") => render_top_level_mutation_view("prev", output),
        Some("rebase") => render_top_level_mutation_view("rebase", output),
        Some("squash") => render_top_level_mutation_view("squash", output),
        Some("split") => render_top_level_mutation_view("split", output),
        Some("simplify-parents") => render_top_level_mutation_view("simplify-parents", output),
        Some("fix") => render_top_level_mutation_view("fix", output),
        Some("abandon") => render_top_level_mutation_view("abandon", output),
        Some("undo") => render_top_level_mutation_view("undo", output),
        Some("redo") => render_top_level_mutation_view("redo", output),
        Some("restore") => render_top_level_mutation_view("restore", output),
        Some("revert") => render_top_level_mutation_view("revert", output),
        Some("absorb") => render_top_level_mutation_view("absorb", output),
        Some("duplicate") => render_top_level_mutation_view("duplicate", output),
        Some("parallelize") => render_top_level_mutation_view("parallelize", output),
        Some("root") => render_root_view(output),
        Some("version") => render_version_view(output),
        Some("resolve") if has_resolve_list_flag(command) => render_resolve_list_view(output),
        Some("resolve") => render_resolve_action_view(output),
        Some("file") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_file_list_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("show")) => {
            render_file_show_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("search")) => {
            render_file_search_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("annotate")) => {
            render_file_annotate_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("track")) => {
            render_file_track_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("untrack")) => {
            render_file_untrack_view(output)
        }
        Some("file") if matches!(command.get(1).map(String::as_str), Some("chmod")) => {
            render_file_chmod_view(output)
        }
        Some("tag") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_tag_list_view(output)
        }
        Some("tag") if matches!(command.get(1).map(String::as_str), Some("set")) => {
            render_tag_set_view(output)
        }
        Some("tag") if matches!(command.get(1).map(String::as_str), Some("delete")) => {
            render_tag_delete_view(output)
        }
        Some("workspace") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_workspace_list_view(output)
        }
        Some("workspace") if matches!(command.get(1).map(String::as_str), Some("root")) => {
            render_root_view(output)
        }
        Some("git") if matches!(command.get(1).map(String::as_str), Some("fetch")) => {
            render_git_fetch_view(output)
        }
        Some("git") if matches!(command.get(1).map(String::as_str), Some("push")) => {
            render_git_push_view(output)
        }
        Some("bookmark") if matches!(command.get(1).map(String::as_str), Some("list")) => {
            render_bookmark_list_view(output)
        }
        Some("bookmark")
            if matches!(
                command.get(1).map(String::as_str),
                Some(
                    "create"
                        | "set"
                        | "move"
                        | "track"
                        | "untrack"
                        | "delete"
                        | "forget"
                        | "rename"
                )
            ) =>
        {
            render_bookmark_mutation_view(command.get(1).map(String::as_str), output)
        }
        Some("workspace")
            if matches!(
                command.get(1).map(String::as_str),
                Some("add" | "forget" | "rename" | "update-stale")
            ) =>
        {
            render_workspace_mutation_view(command.get(1).map(String::as_str), output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("diff")) => {
            render_operation_diff_view(output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("show")) => {
            render_operation_show_view(output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("restore")) => {
            render_operation_mutation_view("restore", output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("revert")) => {
            render_operation_mutation_view("revert", output)
        }
        Some("operation") if matches!(command.get(1).map(String::as_str), Some("log")) => {
            render_operation_log_view(output)
        }
        _ => output,
    }
}
