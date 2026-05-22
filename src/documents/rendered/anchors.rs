use ratatui::text::{Line, Span};

use crate::documents::FileAnchor;

pub fn file_anchor(line_index: usize, line: &Line<'static>) -> Option<FileAnchor> {
    let heading = file_heading(line)?;
    Some(FileAnchor {
        line_index,
        heading: heading.heading,
        label: heading.label,
    })
}

struct FileHeading {
    heading: Line<'static>,
    label: String,
}

fn file_heading(line: &Line<'static>) -> Option<FileHeading> {
    let text = super::projection::line_text(line);
    git_file_heading(line, &text).or_else(|| default_file_heading(line, &text))
}

fn default_file_heading(line: &Line<'static>, text: &str) -> Option<FileHeading> {
    let trimmed = text.trim_end();
    let label = default_file_label(trimmed)?;
    Some(FileHeading {
        heading: styled_subline(line, 0, trimmed.len()),
        label,
    })
}

pub fn default_file_label(heading: &str) -> Option<String> {
    let body = heading.strip_suffix(':')?;
    let file = [
        "Added ",
        "Modified ",
        "Removed ",
        "Deleted ",
        "Renamed ",
        "Copied ",
    ]
    .into_iter()
    .find_map(|prefix| body.strip_prefix(prefix))?;
    let file = file
        .strip_prefix("regular file ")
        .or_else(|| file.strip_prefix("executable file "))
        .or_else(|| file.strip_prefix("symlink "))
        .unwrap_or(file);
    Some(
        file.rsplit_once(" => ")
            .map(|(_, destination)| destination)
            .unwrap_or(file)
            .to_owned(),
    )
}

fn git_file_heading(line: &Line<'static>, text: &str) -> Option<FileHeading> {
    let prefix = "diff --git ";
    let rest = text.strip_prefix(prefix)?;
    let space_index = rest.find(' ')?;
    let b_path_start = prefix.len() + space_index + 1;
    let b_path = &text[b_path_start..];
    let (label, start, end) = clean_git_path_range(b_path, b_path_start)?;
    Some(FileHeading {
        heading: styled_subline(line, start, end),
        label: label.to_owned(),
    })
}

fn clean_git_path_range(path: &str, path_start: usize) -> Option<(&str, usize, usize)> {
    let trimmed = path.trim();
    let trim_start = path.find(trimmed).unwrap_or(0);
    let clean = trimmed
        .strip_prefix("b/")
        .or_else(|| trimmed.strip_prefix("a/"))?;
    let clean_start = path_start + trim_start + trimmed.len().saturating_sub(clean.len());
    Some((clean, clean_start, clean_start + clean.len()))
}

fn styled_subline(line: &Line<'static>, start: usize, end: usize) -> Line<'static> {
    let mut spans = Vec::new();
    let mut span_start = 0;

    for source_span in &line.spans {
        let content = source_span.content.as_ref();
        let span_end = span_start + content.len();
        let overlap_start = start.max(span_start);
        let overlap_end = end.min(span_end);
        if overlap_start < overlap_end
            && let Some(content) =
                content.get((overlap_start - span_start)..(overlap_end - span_start))
        {
            // Sticky file headers should look like the original jj line,
            // including colors from default jj output or `--git` output.
            let mut span = Span::from(content.to_owned());
            span.style = source_span.style;
            spans.push(span);
        }
        span_start = span_end;
    }

    Line::from(spans)
}
