//! Wrappers for `status`, `show`, `diff`, and keymap views.

use crate::config::KeybindConfig;
use crate::keys::KeyBinding;

use super::common::{is_working_copy_change_line, plural_suffix};
use super::{looks_like_graph_commit_row, strip_ansi};

/// Render a structured status view with section spacing and summary line.
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
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(section_lines);
    rendered.push(String::new());
    rendered.push("Shortcuts: s status, F fetch, P push, B rebase, :commands".to_string());
    rendered
}

/// Render `show` output with section spacing and shortcut hints.
pub(crate) fn render_show_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec!["Show View".to_string(), String::new()];
    rendered.extend(normalize_show_lines(lines));
    rendered.push(String::new());
    rendered.push("Shortcuts: Enter show selected, d diff selected, s status".to_string());
    rendered
}

/// Render `diff` output with normalized spacing and shortcut hints.
pub(crate) fn render_diff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec!["Diff View".to_string(), String::new()];
    rendered.extend(normalize_diff_lines(lines));
    rendered.push(String::new());
    rendered.push("Shortcuts: d diff selected, Enter show selected, s status".to_string());
    rendered
}

/// Render `interdiff` output with normalized spacing.
pub(crate) fn render_interdiff_view(lines: Vec<String>) -> Vec<String> {
    if lines.is_empty() || lines == ["(no output)"] {
        return lines;
    }

    let mut rendered = vec!["Interdiff View".to_string(), String::new()];
    rendered.extend(normalize_diff_lines(lines));
    rendered.push(String::new());
    rendered.push("Tip: compare patch intent with `interdiff --from ... --to ...`".to_string());
    rendered
}

/// Render `evolog` output with entry counting and readable spacing.
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
        String::new(),
        summary,
        String::new(),
    ];
    rendered.extend(history_lines);
    rendered.push(String::new());
    rendered.push("Tip: use `evolog -p` to inspect patch-level evolution".to_string());
    rendered
}

/// Normalize `show` output by collapsing repeated blank lines and spacing top-level sections.
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

/// Normalize diff-like output by spacing top-level section headers.
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

/// Return whether a line is a top-level section header in `jj` output.
pub(crate) fn is_top_level_section_header(line: &str) -> bool {
    let stripped = strip_ansi(line);
    !stripped.starts_with(' ') && stripped.ends_with(':')
}

/// Render keymap catalog lines, optionally filtered by action/key label query.
pub(crate) fn keymap_overview_lines(config: &KeybindConfig, query: Option<&str>) -> Vec<String> {
    let filter = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);

    let mut lines = vec![
        "jk keymap".to_string(),
        String::new(),
        "grouped bindings (common first)".to_string(),
        String::new(),
    ];

    let mut matched_any = false;
    matched_any |= push_keymap_group(
        &mut lines,
        "Navigation",
        &[
            ("quit", &config.normal.quit),
            ("refresh", &config.normal.refresh),
            ("up", &config.normal.up),
            ("down", &config.normal.down),
            ("page up", &config.normal.page_up),
            ("page down", &config.normal.page_down),
            ("back", &config.normal.back),
            ("forward", &config.normal.forward),
            ("top", &config.normal.top),
            ("bottom", &config.normal.bottom),
            ("repeat", &config.normal.repeat_last),
        ],
        filter.as_deref(),
    );
    matched_any |= push_keymap_group(
        &mut lines,
        "Views",
        &[
            ("command mode", &config.normal.command_mode),
            ("help", &config.normal.help),
            ("keymap", &config.normal.keymap),
            ("aliases", &config.normal.aliases),
            ("show", &config.normal.show),
            ("diff", &config.normal.diff),
            ("status", &config.normal.status),
            ("log", &config.normal.log),
            ("operation log", &config.normal.operation_log),
            ("bookmark list", &config.normal.bookmark_list),
            ("resolve list", &config.normal.resolve_list),
            ("file list", &config.normal.file_list),
            ("tag list", &config.normal.tag_list),
            ("workspace root", &config.normal.root),
        ],
        filter.as_deref(),
    );
    matched_any |= push_keymap_group(
        &mut lines,
        "Actions",
        &[
            ("toggle patch", &config.normal.toggle_patch),
            ("fetch", &config.normal.fetch),
            ("push", &config.normal.push),
            ("rebase main", &config.normal.rebase_main),
            ("rebase trunk", &config.normal.rebase_trunk),
            ("new", &config.normal.new),
            ("next", &config.normal.next),
            ("prev", &config.normal.prev),
            ("edit", &config.normal.edit),
            ("commit", &config.normal.commit),
            ("describe", &config.normal.describe),
            ("bookmark set", &config.normal.bookmark_set),
            ("abandon", &config.normal.abandon),
            ("rebase", &config.normal.rebase),
            ("squash", &config.normal.squash),
            ("split", &config.normal.split),
            ("restore", &config.normal.restore),
            ("revert", &config.normal.revert),
            ("undo", &config.normal.undo),
            ("redo", &config.normal.redo),
            ("cmd submit", &config.command.submit),
            ("cmd backspace", &config.command.backspace),
            ("cmd history prev", &config.command.history_prev),
            ("cmd history next", &config.command.history_next),
        ],
        filter.as_deref(),
    );
    matched_any |= push_keymap_group(
        &mut lines,
        "Safety",
        &[
            ("cmd cancel", &config.command.cancel),
            ("confirm yes", &config.confirm.accept),
            ("confirm no", &config.confirm.reject),
        ],
        filter.as_deref(),
    );

    if !matched_any {
        lines.push("(no matching keybinds)".to_string());
        return lines;
    }

    lines.push("tip: use :keys <query> to narrow the list (for example :keys rebase)".to_string());
    lines
}

/// Append one named keymap group and return whether any entry matched.
fn push_keymap_group(
    lines: &mut Vec<String>,
    title: &str,
    entries: &[(&str, &Vec<KeyBinding>)],
    filter: Option<&str>,
) -> bool {
    let mut compact = Vec::new();
    for (action, bindings) in entries {
        let labels = key_binding_labels(bindings);
        if let Some(filter) = filter
            && !action.to_ascii_lowercase().contains(filter)
            && !labels.to_ascii_lowercase().contains(filter)
            && !title.to_ascii_lowercase().contains(filter)
        {
            continue;
        }

        compact.push(format!("{:<16} {}", action, labels));
    }

    if compact.is_empty() {
        return false;
    }

    lines.push(format!("{title}:"));
    lines.extend(compact_two_column(&compact, 42));
    lines.push(String::new());
    true
}

/// Render comma-separated labels for a set of bindings.
pub(crate) fn key_binding_labels(bindings: &[KeyBinding]) -> String {
    bindings
        .iter()
        .map(key_binding_label)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Render human-readable label for one keybinding.
pub(crate) fn key_binding_label(binding: &KeyBinding) -> String {
    match binding {
        KeyBinding::Char(value) => value.to_string(),
        KeyBinding::Enter => "Enter".to_string(),
        KeyBinding::Esc => "Esc".to_string(),
        KeyBinding::Backspace => "Backspace".to_string(),
        KeyBinding::Up => "Up".to_string(),
        KeyBinding::Down => "Down".to_string(),
        KeyBinding::PageUp => "PageUp".to_string(),
        KeyBinding::PageDown => "PageDown".to_string(),
        KeyBinding::Left => "Left".to_string(),
        KeyBinding::Right => "Right".to_string(),
        KeyBinding::Home => "Home".to_string(),
        KeyBinding::End => "End".to_string(),
    }
}

/// Pack one-column entry lines into compact two-column output.
fn compact_two_column(entries: &[String], width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    for pair in entries.chunks(2) {
        if pair.len() == 1 {
            lines.push(pair[0].clone());
            continue;
        }

        lines.push(format!("{:<width$}  {}", pair[0], pair[1], width = width));
    }

    lines
}
