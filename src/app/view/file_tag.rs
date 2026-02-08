//! Wrapper views for `file` and `tag` command outputs.

use super::common::plural_suffix;

/// Render `file list` output with file-count summary.
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

/// Render `file show` output with content-line summary.
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

/// Render `file search` output with match-count summary.
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

/// Render `file annotate` output with annotation-count summary.
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

/// Render `file track` mutation output.
pub(crate) fn render_file_track_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Track",
        "==========",
        lines,
        "Tip: review tracked paths with `file list` and verify with `status`",
    )
}

/// Render `file untrack` mutation output.
pub(crate) fn render_file_untrack_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Untrack",
        "============",
        lines,
        "Tip: ensure paths are ignored before untracking and confirm with `status`",
    )
}

/// Render `file chmod` mutation output.
pub(crate) fn render_file_chmod_view(lines: Vec<String>) -> Vec<String> {
    render_file_mutation_view(
        "File Chmod",
        "==========",
        lines,
        "Tip: run `file show` or `diff` to verify executable-bit updates",
    )
}

/// Shared renderer for file mutation wrappers.
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

/// Render `tag list` output with count summary.
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

/// Render `tag set` mutation output.
pub(crate) fn render_tag_set_view(lines: Vec<String>) -> Vec<String> {
    render_tag_mutation_view(
        "Tag Set",
        "=======",
        lines,
        "Tip: run `tag list` to confirm updated tag targets",
    )
}

/// Render `tag delete` mutation output.
pub(crate) fn render_tag_delete_view(lines: Vec<String>) -> Vec<String> {
    render_tag_mutation_view(
        "Tag Delete",
        "==========",
        lines,
        "Tip: run `tag list` to confirm removed tags",
    )
}

/// Shared renderer for tag mutation wrappers.
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
