use crate::config::KeybindConfig;
use crate::keys::KeyBinding;

use super::selection::{looks_like_graph_commit_row, strip_ansi};

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

pub(crate) fn render_status_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut section_lines: Vec<String> = Vec::new();
    let mut current_section: Option<String> = None;
    let mut has_working_copy_section = false;
    let mut has_conflicted_section = false;
    let mut working_copy_changes = 0usize;
    let mut conflicted_bookmarks = 0usize;

    for raw_line in lines {
        let display_line = raw_line.trim_end().to_string();
        let stripped = strip_ansi(&display_line);
        let trimmed = stripped.trim();
        let display_trimmed = display_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.ends_with(':') {
            if matches!(section_lines.last(), Some(previous) if !previous.is_empty()) {
                section_lines.push(String::new());
            }

            current_section = Some(trimmed.to_string());
            if trimmed == "Working copy changes:" {
                has_working_copy_section = true;
            }
            if trimmed == "Conflicted bookmarks:" {
                has_conflicted_section = true;
            }
            section_lines.push(display_trimmed.to_string());
            continue;
        }

        match current_section.as_deref() {
            Some("Working copy changes:") => {
                if is_working_copy_change_line(trimmed) {
                    section_lines.push(format!("  {display_trimmed}"));
                    working_copy_changes += 1;
                } else {
                    current_section = None;
                    section_lines.push(display_trimmed.to_string());
                }
            }
            Some("Conflicted bookmarks:") => {
                section_lines.push(display_trimmed.to_string());
                conflicted_bookmarks += 1;
            }
            _ => {
                section_lines.push(display_trimmed.to_string());
            }
        }
    }

    while matches!(section_lines.last(), Some(previous) if previous.is_empty()) {
        section_lines.pop();
    }

    let mut summary_parts = Vec::new();
    if has_working_copy_section {
        summary_parts.push(format!(
            "{working_copy_changes} working-copy change{}",
            plural_suffix(working_copy_changes)
        ));
    }
    if has_conflicted_section {
        summary_parts.push(format!(
            "{conflicted_bookmarks} conflicted bookmark{}",
            plural_suffix(conflicted_bookmarks)
        ));
    }

    let summary = if summary_parts.is_empty() {
        "Summary: status captured".to_string()
    } else {
        format!("Summary: {}", summary_parts.join(", "))
    };

    let mut rendered = vec![
        "Status Overview".to_string(),
        "===============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(section_lines);
    rendered.push(String::new());
    rendered.push("Shortcuts: s status, F fetch, P push, B rebase, :commands".to_string());
    rendered
}

pub(crate) fn render_show_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Show View".to_string(),
        "=========".to_string(),
        String::new(),
    ];
    rendered.extend(normalize_show_lines(lines));
    rendered.push(String::new());
    rendered.push("Shortcuts: Enter show selected, d diff selected, s status".to_string());
    rendered
}

pub(crate) fn render_diff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Diff View".to_string(),
        "=========".to_string(),
        String::new(),
    ];
    rendered.extend(normalize_diff_lines(lines));
    rendered.push(String::new());
    rendered.push("Shortcuts: d diff selected, Enter show selected, s status".to_string());
    rendered
}

pub(crate) fn render_interdiff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Interdiff View".to_string(),
        "==============".to_string(),
        String::new(),
    ];
    rendered.extend(normalize_diff_lines(lines));
    rendered.push(String::new());
    rendered.push("Tip: compare patch intent with `interdiff --from ... --to ...`".to_string());
    rendered
}

pub(crate) fn render_evolog_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut history_lines: Vec<String> = Vec::new();
    let mut entry_count = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if looks_like_graph_commit_row(&line) {
            entry_count += 1;
            if matches!(history_lines.last(), Some(previous) if !previous.is_empty()) {
                history_lines.push(String::new());
            }
        }

        history_lines.push(line);
    }

    while matches!(history_lines.last(), Some(previous) if previous.is_empty()) {
        history_lines.pop();
    }

    let summary = if entry_count == 0 {
        "Summary: change evolution shown".to_string()
    } else {
        format!(
            "Summary: {entry_count} evolution entr{} shown",
            if entry_count == 1 { "y" } else { "ies" }
        )
    };

    let mut rendered = vec![
        "Evolution Log".to_string(),
        "=============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(history_lines);
    rendered.push(String::new());
    rendered.push("Tip: use `evolog -p` to inspect patch-level evolution".to_string());
    rendered
}

pub(crate) fn normalize_show_lines(lines: Vec<String>) -> Vec<String> {
    let mut rendered: Vec<String> = Vec::new();

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.is_empty() {
            if matches!(rendered.last(), Some(previous) if !previous.is_empty()) {
                rendered.push(String::new());
            }
            continue;
        }

        if is_top_level_section_header(&line)
            && matches!(rendered.last(), Some(previous) if !previous.is_empty())
        {
            rendered.push(String::new());
        }

        rendered.push(line);
    }

    while matches!(rendered.last(), Some(previous) if previous.is_empty()) {
        rendered.pop();
    }

    rendered
}

pub(crate) fn normalize_diff_lines(lines: Vec<String>) -> Vec<String> {
    let mut rendered: Vec<String> = Vec::new();

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();

        if is_top_level_section_header(&line)
            && matches!(rendered.last(), Some(previous) if !previous.is_empty())
        {
            rendered.push(String::new());
        }

        rendered.push(line);
    }

    while matches!(rendered.last(), Some(previous) if previous.is_empty()) {
        rendered.pop();
    }

    rendered
}

pub(crate) fn is_top_level_section_header(line: &str) -> bool {
    let stripped = strip_ansi(line);
    !stripped.starts_with(' ') && stripped.ends_with(':')
}

pub(crate) fn keymap_overview_lines(config: &KeybindConfig, query: Option<&str>) -> Vec<String> {
    let filter = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    let entries: [(&str, &Vec<KeyBinding>); 48] = [
        ("normal.quit", &config.normal.quit),
        ("normal.refresh", &config.normal.refresh),
        ("normal.up", &config.normal.up),
        ("normal.down", &config.normal.down),
        ("normal.top", &config.normal.top),
        ("normal.bottom", &config.normal.bottom),
        ("normal.command_mode", &config.normal.command_mode),
        ("normal.help", &config.normal.help),
        ("normal.keymap", &config.normal.keymap),
        ("normal.aliases", &config.normal.aliases),
        ("normal.show", &config.normal.show),
        ("normal.diff", &config.normal.diff),
        ("normal.status", &config.normal.status),
        ("normal.log", &config.normal.log),
        ("normal.operation_log", &config.normal.operation_log),
        ("normal.bookmark_list", &config.normal.bookmark_list),
        ("normal.resolve_list", &config.normal.resolve_list),
        ("normal.file_list", &config.normal.file_list),
        ("normal.tag_list", &config.normal.tag_list),
        ("normal.root", &config.normal.root),
        ("normal.repeat_last", &config.normal.repeat_last),
        ("normal.toggle_patch", &config.normal.toggle_patch),
        ("normal.fetch", &config.normal.fetch),
        ("normal.push", &config.normal.push),
        ("normal.rebase_main", &config.normal.rebase_main),
        ("normal.rebase_trunk", &config.normal.rebase_trunk),
        ("normal.new", &config.normal.new),
        ("normal.next", &config.normal.next),
        ("normal.prev", &config.normal.prev),
        ("normal.edit", &config.normal.edit),
        ("normal.commit", &config.normal.commit),
        ("normal.describe", &config.normal.describe),
        ("normal.bookmark_set", &config.normal.bookmark_set),
        ("normal.abandon", &config.normal.abandon),
        ("normal.rebase", &config.normal.rebase),
        ("normal.squash", &config.normal.squash),
        ("normal.split", &config.normal.split),
        ("normal.restore", &config.normal.restore),
        ("normal.revert", &config.normal.revert),
        ("normal.undo", &config.normal.undo),
        ("normal.redo", &config.normal.redo),
        ("command.submit", &config.command.submit),
        ("command.cancel", &config.command.cancel),
        ("command.backspace", &config.command.backspace),
        ("command.history_prev", &config.command.history_prev),
        ("command.history_next", &config.command.history_next),
        ("confirm.accept", &config.confirm.accept),
        ("confirm.reject", &config.confirm.reject),
    ];

    let mut lines = vec![
        "jk keymap".to_string(),
        format!("{:<24} {}", "action", "keys"),
        "-".repeat(44),
    ];

    for (action, bindings) in entries {
        let labels = key_binding_labels(bindings);
        if let Some(filter) = &filter
            && !action.to_ascii_lowercase().contains(filter)
            && !labels.to_ascii_lowercase().contains(filter)
        {
            continue;
        }

        lines.push(format!("{:<24} {}", action, labels));
    }

    lines
}

pub(crate) fn key_binding_labels(bindings: &[KeyBinding]) -> String {
    bindings
        .iter()
        .map(key_binding_label)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn key_binding_label(binding: &KeyBinding) -> String {
    match binding {
        KeyBinding::Char(value) => value.to_string(),
        KeyBinding::Enter => "Enter".to_string(),
        KeyBinding::Esc => "Esc".to_string(),
        KeyBinding::Backspace => "Backspace".to_string(),
        KeyBinding::Up => "Up".to_string(),
        KeyBinding::Down => "Down".to_string(),
        KeyBinding::Left => "Left".to_string(),
        KeyBinding::Right => "Right".to_string(),
        KeyBinding::Home => "Home".to_string(),
        KeyBinding::End => "End".to_string(),
    }
}

pub(crate) fn render_root_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Workspace Root".to_string(),
        "==============".to_string(),
        String::new(),
    ];

    for line in lines {
        let display_line = line.trim_end().to_string();
        let stripped = strip_ansi(&display_line);
        let trimmed = stripped.trim();
        let display_trimmed = display_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        rendered.push(display_trimmed.to_string());
    }

    rendered.push(String::new());
    rendered.push("Tip: use jjrt/jk root to inspect current workspace path".to_string());
    rendered
}

pub(crate) fn render_version_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = if let Some(line) = detail_lines.first() {
        format!("Summary: {}", line.trim())
    } else {
        "Summary: version command completed with no output".to_string()
    };

    let mut rendered = vec![
        "Version".to_string(),
        "=======".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `version` after upgrades to confirm toolchain state".to_string());
    rendered
}

pub(crate) fn render_resolve_list_view(lines: Vec<String>) -> Vec<String> {
    let mut body_lines = Vec::new();
    let mut conflict_count = 0usize;
    let mut saw_no_conflicts = false;

    for line in lines {
        let display_line = line.trim_end().to_string();
        let stripped = strip_ansi(&display_line);
        let trimmed = stripped.trim();
        let display_trimmed = display_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "(no output)" {
            continue;
        }

        if trimmed.contains("No conflicts found") {
            saw_no_conflicts = true;
        } else if !trimmed.starts_with("Error:") && !trimmed.starts_with("Hint:") {
            conflict_count += 1;
        }

        body_lines.push(display_trimmed.to_string());
    }

    let summary = if saw_no_conflicts || conflict_count == 0 {
        "Summary: no conflicts listed".to_string()
    } else {
        format!(
            "Summary: {conflict_count} conflicted path{} listed",
            plural_suffix(conflict_count)
        )
    };

    let mut rendered = vec![
        "Resolve List".to_string(),
        "============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if body_lines.is_empty() {
        rendered.push("(no conflicts found)".to_string());
    } else {
        rendered.extend(body_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: run `resolve <path>` to open a merge tool for specific conflicts".to_string());
    rendered
}

pub(crate) fn render_resolve_action_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let detail_count = detail_lines.len();

    let summary = if let Some(signal) = detail_lines.iter().find(|line| {
        let stripped = strip_ansi(line);
        stripped.trim().starts_with("Resolved ")
    }) {
        format!("Summary: {}", strip_ansi(signal).trim())
    } else if detail_count == 0 {
        "Summary: resolve command completed with no output".to_string()
    } else {
        format!(
            "Summary: {detail_count} output line{}",
            plural_suffix(detail_count)
        )
    };

    let mut rendered = vec![
        "Resolve".to_string(),
        "=======".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: use `resolve -l` to verify remaining conflicts after resolution".to_string());
    rendered
}

pub(crate) fn has_resolve_list_flag(command: &[String]) -> bool {
    command
        .iter()
        .any(|token| token == "-l" || token == "--list")
}

pub(crate) fn render_file_list_view(lines: Vec<String>) -> Vec<String> {
    let file_lines: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed != "(no output)"
        })
        .collect();
    let file_count = file_lines.len();

    let mut rendered = vec![
        "File List".to_string(),
        "=========".to_string(),
        String::new(),
        format!(
            "Summary: {file_count} file{} listed",
            plural_suffix(file_count)
        ),
        String::new(),
    ];

    if file_count == 0 {
        rendered.push("(no files listed)".to_string());
    } else {
        rendered.extend(file_lines);
    }

    rendered.push(String::new());
    rendered.push(
        "Tip: use `show`/`diff` with selection to inspect file-affecting revisions".to_string(),
    );
    rendered
}

pub(crate) fn render_file_show_view(lines: Vec<String>) -> Vec<String> {
    let mut content_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .collect();
    if content_lines.len() == 1 && content_lines[0].trim() == "(no output)" {
        content_lines.clear();
    }

    let line_count = content_lines.len();
    let mut rendered = vec![
        "File Show".to_string(),
        "=========".to_string(),
        String::new(),
        format!(
            "Summary: {line_count} content line{}",
            plural_suffix(line_count)
        ),
        String::new(),
    ];

    if content_lines.is_empty() {
        rendered.push("(no file content shown)".to_string());
    } else {
        rendered.extend(content_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: use `show`/`diff -r <rev>` to inspect surrounding change context".to_string());
    rendered
}

pub(crate) fn render_file_search_view(lines: Vec<String>) -> Vec<String> {
    let match_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let match_count = match_lines.len();

    let mut rendered = vec![
        "File Search".to_string(),
        "===========".to_string(),
        String::new(),
        format!(
            "Summary: {match_count} match line{}",
            plural_suffix(match_count)
        ),
        String::new(),
    ];

    if match_lines.is_empty() {
        rendered.push("(no matches found)".to_string());
    } else {
        rendered.extend(match_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: refine search patterns with additional terms or regex options".to_string());
    rendered
}

pub(crate) fn render_file_annotate_view(lines: Vec<String>) -> Vec<String> {
    let annotation_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let annotation_count = annotation_lines.len();

    let mut rendered = vec![
        "File Annotate".to_string(),
        "=============".to_string(),
        String::new(),
        format!(
            "Summary: {annotation_count} annotated line{}",
            plural_suffix(annotation_count)
        ),
        String::new(),
    ];

    if annotation_lines.is_empty() {
        rendered.push("(no annotation output)".to_string());
    } else {
        rendered.extend(annotation_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: pair with `show <rev>` to inspect the source revision details".to_string());
    rendered
}

pub(crate) fn render_file_track_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Track",
        "==========",
        lines,
        "Tip: review tracked paths with `file list` and verify with `status`",
    )
}

pub(crate) fn render_file_untrack_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Untrack",
        "============",
        lines,
        "Tip: ensure paths are ignored before untracking and confirm with `status`",
    )
}

pub(crate) fn render_file_chmod_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Chmod",
        "==========",
        lines,
        "Tip: run `file show` or `diff` to verify executable-bit updates",
    )
}

pub(crate) fn render_file_mutation_view(
    title: &str,
    underline: &str,
    lines: Vec<String>,
    tip: &str,
) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let detail_count = detail_lines.len();

    let summary = if detail_count == 0 {
        "Summary: command completed with no output".to_string()
    } else {
        format!(
            "Summary: {detail_count} output line{}",
            plural_suffix(detail_count)
        )
    };

    let mut rendered = vec![
        title.to_string(),
        underline.to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push(tip.to_string());
    rendered
}

pub(crate) fn render_tag_list_view(lines: Vec<String>) -> Vec<String> {
    let tag_lines: Vec<String> = lines
        .into_iter()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed != "(no output)"
        })
        .collect();
    let tag_count = tag_lines.len();

    let mut rendered = vec![
        "Tag List".to_string(),
        "========".to_string(),
        String::new(),
        format!(
            "Summary: {tag_count} tag{} listed",
            plural_suffix(tag_count)
        ),
        String::new(),
    ];

    if tag_count == 0 {
        rendered.push("(no tags listed)".to_string());
    } else {
        rendered.extend(tag_lines);
    }

    rendered.push(String::new());
    rendered.push(
        "Tip: use `tag create` and `tag forget` from command mode for tag updates".to_string(),
    );
    rendered
}

pub(crate) fn render_tag_set_view(lines: Vec<String>) -> Vec<String> {
    render_tag_mutation_view(
        "Tag Set",
        "=======",
        lines,
        "Tip: run `tag list` to confirm updated tag targets",
    )
}

pub(crate) fn render_tag_delete_view(lines: Vec<String>) -> Vec<String> {
    render_tag_mutation_view(
        "Tag Delete",
        "==========",
        lines,
        "Tip: run `tag list` to confirm removed tags",
    )
}

pub(crate) fn render_tag_mutation_view(
    title: &str,
    underline: &str,
    lines: Vec<String>,
    tip: &str,
) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let detail_count = detail_lines.len();

    let summary = if detail_count == 0 {
        "Summary: command completed with no output".to_string()
    } else {
        format!(
            "Summary: {detail_count} output line{}",
            plural_suffix(detail_count)
        )
    };

    let mut rendered = vec![
        title.to_string(),
        underline.to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push(tip.to_string());
    rendered
}

pub(crate) fn render_workspace_list_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut workspace_lines = Vec::new();
    let mut workspace_count = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        if line.contains(':') {
            workspace_count += 1;
        }
        workspace_lines.push(line);
    }

    let mut rendered = vec![
        "Workspace List".to_string(),
        "==============".to_string(),
        String::new(),
        format!(
            "Summary: {workspace_count} workspace{} listed",
            plural_suffix(workspace_count)
        ),
        String::new(),
    ];
    rendered.extend(workspace_lines);
    rendered.push(String::new());
    rendered
        .push("Tip: use `workspace add/forget/rename` flows from normal mode or `:`".to_string());
    rendered
}

pub(crate) fn render_git_fetch_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = git_fetch_summary(&detail_lines);

    let mut rendered = vec![
        "Git Fetch".to_string(),
        "=========".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `log` or `status` to inspect fetched changes".to_string());
    rendered
}

pub(crate) fn render_git_push_view(lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();

    let summary = git_push_summary(&detail_lines);

    let mut rendered = vec![
        "Git Push".to_string(),
        "========".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered
        .push("Tip: push stays confirm-gated with a dry-run preview when available".to_string());
    rendered
}

pub(crate) fn git_fetch_summary(detail_lines: &[String]) -> String {
    if detail_lines
        .iter()
        .any(|line| strip_ansi(line).contains("Nothing changed"))
    {
        return "Summary: no remote updates fetched".to_string();
    }
    if detail_lines.is_empty() {
        return "Summary: fetch completed with no output".to_string();
    }
    if let Some(signal) = detail_lines.iter().find(|line| is_git_fetch_signal(line)) {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_git_fetch_signal(line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    trimmed.starts_with("Fetched ")
        || trimmed.starts_with("From ")
        || trimmed.starts_with("Updated bookmark ")
}

pub(crate) fn git_push_summary(detail_lines: &[String]) -> String {
    if detail_lines
        .iter()
        .any(|line| strip_ansi(line).contains("Nothing changed"))
    {
        return "Summary: no bookmark updates pushed".to_string();
    }
    if detail_lines.is_empty() {
        return "Summary: push completed with no output".to_string();
    }
    if let Some(signal) = detail_lines.iter().find(|line| is_git_push_signal(line)) {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_git_push_signal(line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    trimmed.starts_with("Pushed bookmark ") || trimmed.starts_with("Pushed ")
}

pub(crate) fn render_top_level_mutation_view(
    command_name: &str,
    lines: Vec<String>,
) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = top_level_mutation_summary(command_name, &detail_lines);
    let title = format!("{} Result", capitalize_word(command_name));
    let mut rendered = vec![
        title.clone(),
        "=".repeat(title.len()),
        String::new(),
        summary,
    ];
    rendered.push(String::new());

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push(top_level_mutation_tip(command_name).to_string());
    rendered
}

pub(crate) fn top_level_mutation_summary(command_name: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: `{command_name}` completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_top_level_mutation_signal(command_name, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_top_level_mutation_signal(command_name: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match command_name {
        "new" | "describe" | "commit" | "metaedit" | "edit" | "next" | "prev" => {
            trimmed.starts_with("Working copy now at:")
                || trimmed.starts_with("Working copy  (@) :")
                || trimmed.starts_with("Rebased ")
        }
        "undo" => trimmed.starts_with("Undid operation"),
        "redo" => trimmed.starts_with("Redid operation"),
        "rebase" | "squash" | "split" | "simplify-parents" | "diffedit" => {
            trimmed.starts_with("Rebased ")
        }
        "fix" => trimmed.starts_with("Fixed ") || trimmed.starts_with("Rebased "),
        "abandon" => trimmed.starts_with("Abandoned "),
        "restore" => trimmed.starts_with("Restored "),
        "revert" => trimmed.starts_with("Reverted "),
        "absorb" => trimmed.starts_with("Absorbed "),
        "duplicate" => trimmed.starts_with("Duplicated "),
        "parallelize" => trimmed.starts_with("Parallelized "),
        _ => false,
    }
}

pub(crate) fn top_level_mutation_tip(command_name: &str) -> &'static str {
    match command_name {
        "new" | "describe" | "commit" | "metaedit" => {
            "Tip: run `show` or `log` to inspect the updated revision"
        }
        "edit" | "next" | "prev" => "Tip: run `show` or `diff` to inspect the selected revision",
        "undo" | "redo" => "Tip: run `operation log` to inspect operation history",
        "rebase" | "squash" | "split" | "simplify-parents" | "diffedit" | "fix" | "abandon"
        | "restore" | "revert" | "absorb" | "duplicate" | "parallelize" => {
            "Tip: run `log`, `status`, or `diff` to verify the resulting history"
        }
        _ => "Tip: run `log` and `status` to verify repository state",
    }
}

pub(crate) fn render_bookmark_mutation_view(
    subcommand: Option<&str>,
    lines: Vec<String>,
) -> Vec<String> {
    let subcommand = subcommand.unwrap_or("update");
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = bookmark_mutation_summary(subcommand, &detail_lines);

    let mut rendered = vec![
        format!("Bookmark {}", capitalize_word(subcommand)),
        "================".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `bookmark list` to verify bookmark state".to_string());
    rendered
}

pub(crate) fn bookmark_mutation_summary(subcommand: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: bookmark {subcommand} completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_bookmark_mutation_signal(subcommand, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_bookmark_mutation_signal(subcommand: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match subcommand {
        "create" => trimmed.starts_with("Created bookmark "),
        "set" | "move" => trimmed.starts_with("Moved bookmark "),
        "track" => trimmed.starts_with("Started tracking bookmark "),
        "untrack" => trimmed.starts_with("Stopped tracking bookmark "),
        "delete" => trimmed.starts_with("Deleted bookmark "),
        "forget" => trimmed.starts_with("Forgot bookmark "),
        "rename" => trimmed.starts_with("Renamed bookmark "),
        _ => {
            trimmed.starts_with("Created bookmark ")
                || trimmed.starts_with("Moved bookmark ")
                || trimmed.starts_with("Started tracking bookmark ")
                || trimmed.starts_with("Stopped tracking bookmark ")
                || trimmed.starts_with("Deleted bookmark ")
                || trimmed.starts_with("Forgot bookmark ")
                || trimmed.starts_with("Renamed bookmark ")
        }
    }
}

pub(crate) fn render_bookmark_list_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec![
        "Bookmark List".to_string(),
        "=============".to_string(),
        String::new(),
    ];
    rendered.extend(lines);
    rendered.push(String::new());
    rendered.push("Tip: use `bookmark set/move/track` flows from normal mode or `:`".to_string());
    rendered
}

pub(crate) fn render_workspace_mutation_view(
    subcommand: Option<&str>,
    lines: Vec<String>,
) -> Vec<String> {
    let subcommand = subcommand.unwrap_or("update");
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = workspace_mutation_summary(subcommand, &detail_lines);

    let mut rendered = vec![
        format!("Workspace {}", capitalize_word(subcommand)),
        "=================".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `workspace list` to inspect workspace state".to_string());
    rendered
}

pub(crate) fn workspace_mutation_summary(subcommand: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: workspace {subcommand} completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_workspace_mutation_signal(subcommand, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_workspace_mutation_signal(subcommand: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match subcommand {
        "add" => trimmed.starts_with("Created workspace "),
        "forget" => trimmed.starts_with("Forgot workspace "),
        "rename" => trimmed.starts_with("Renamed workspace "),
        "update-stale" => trimmed.starts_with("Updated ") && trimmed.contains("stale workspace"),
        _ => {
            trimmed.starts_with("Created workspace ")
                || trimmed.starts_with("Forgot workspace ")
                || trimmed.starts_with("Renamed workspace ")
                || (trimmed.starts_with("Updated ") && trimmed.contains("stale workspace"))
        }
    }
}

pub(crate) fn render_operation_show_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut detail_lines: Vec<String> = Vec::new();
    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if line.ends_with(':')
            && matches!(detail_lines.last(), Some(previous) if !previous.is_empty())
        {
            detail_lines.push(String::new());
        }

        detail_lines.push(line);
    }

    while matches!(detail_lines.last(), Some(previous) if previous.is_empty()) {
        detail_lines.pop();
    }

    let operation_id = detail_lines
        .first()
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or("@");

    let mut rendered = vec![
        "Operation Details".to_string(),
        "=================".to_string(),
        String::new(),
        format!("Summary: operation {operation_id}"),
        String::new(),
    ];
    rendered.extend(detail_lines);
    rendered.push(String::new());
    rendered.push("Tip: operation restore/revert remain confirm-gated with previews".to_string());
    rendered
}

pub(crate) fn render_operation_mutation_view(subcommand: &str, lines: Vec<String>) -> Vec<String> {
    let detail_lines: Vec<String> = lines
        .into_iter()
        .map(|line| line.trim_end().to_string())
        .filter(|line| !line.trim().is_empty() && line.trim() != "(no output)")
        .collect();
    let summary = operation_mutation_summary(subcommand, &detail_lines);

    let mut rendered = vec![
        format!("Operation {}", capitalize_word(subcommand)),
        "=================".to_string(),
        String::new(),
        summary,
        String::new(),
    ];

    if detail_lines.is_empty() {
        rendered.push("(no output)".to_string());
    } else {
        rendered.extend(detail_lines);
    }

    rendered.push(String::new());
    rendered.push("Tip: run `operation log` and `status` to validate repository state".to_string());
    rendered
}

pub(crate) fn operation_mutation_summary(subcommand: &str, detail_lines: &[String]) -> String {
    if detail_lines.is_empty() {
        return format!("Summary: operation {subcommand} completed with no output");
    }

    if let Some(signal) = detail_lines
        .iter()
        .find(|line| is_operation_mutation_signal(subcommand, line))
    {
        return format!("Summary: {}", strip_ansi(signal).trim());
    }

    format!(
        "Summary: {} output line{}",
        detail_lines.len(),
        plural_suffix(detail_lines.len())
    )
}

pub(crate) fn is_operation_mutation_signal(subcommand: &str, line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim();
    match subcommand {
        "restore" => trimmed.starts_with("Restored to operation "),
        "revert" => trimmed.starts_with("Reverted operation "),
        _ => {
            trimmed.starts_with("Restored to operation ")
                || trimmed.starts_with("Reverted operation ")
        }
    }
}

pub(crate) fn render_operation_diff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut detail_lines: Vec<String> = Vec::new();
    let mut commit_count = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if is_operation_entry_header(&line) {
            commit_count += 1;
        }

        if line.ends_with(':')
            && matches!(detail_lines.last(), Some(previous) if !previous.is_empty())
        {
            detail_lines.push(String::new());
        }

        detail_lines.push(line);
    }

    while matches!(detail_lines.last(), Some(previous) if previous.is_empty()) {
        detail_lines.pop();
    }

    let summary = if commit_count == 0 {
        "Summary: operation delta shown".to_string()
    } else {
        format!(
            "Summary: {commit_count} changed commit entr{} shown",
            if commit_count == 1 { "y" } else { "ies" }
        )
    };

    let mut rendered = vec![
        "Operation Diff".to_string(),
        "==============".to_string(),
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(detail_lines);
    rendered.push(String::new());
    rendered
        .push("Tip: use operation show/restore/revert for deeper operation workflows".to_string());
    rendered
}

pub(crate) fn render_operation_log_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut operation_lines: Vec<String> = Vec::new();
    let mut operation_count = 0usize;

    for raw_line in lines {
        let line = raw_line.trim_end().to_string();
        if line.trim().is_empty() {
            continue;
        }

        if is_operation_entry_header(&line) {
            if matches!(operation_lines.last(), Some(previous) if !previous.is_empty()) {
                operation_lines.push(String::new());
            }
            operation_count += 1;
        }

        operation_lines.push(line);
    }

    while matches!(operation_lines.last(), Some(previous) if previous.is_empty()) {
        operation_lines.pop();
    }

    let mut rendered = vec![
        "Operation Log".to_string(),
        "=============".to_string(),
        String::new(),
        format!(
            "Summary: {operation_count} operation entr{} shown",
            if operation_count == 1 { "y" } else { "ies" }
        ),
        String::new(),
    ];
    rendered.extend(operation_lines);
    rendered.push(String::new());
    rendered.push("Tip: restore/revert operations stay confirm-gated with previews".to_string());
    rendered
}

pub(crate) fn is_operation_entry_header(line: &str) -> bool {
    let stripped = strip_ansi(line);
    let trimmed = stripped.trim_start();
    trimmed.starts_with('@') || trimmed.starts_with('○')
}

pub(crate) fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

pub(crate) fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

pub(crate) fn is_working_copy_change_line(line: &str) -> bool {
    let stripped = strip_ansi(line);
    let mut chars = stripped.chars();
    match (chars.next(), chars.next()) {
        (Some(status), Some(' ')) => matches!(status, 'M' | 'A' | 'D' | 'R' | 'C' | '?' | 'U'),
        _ => false,
    }
}
