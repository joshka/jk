//! Lightweight structure over rendered `jj` output.
//!
//! This module does not try to model repository data. It recognizes just enough
//! of jj's default and git diff text to pin file context while preserving the
//! original spans/styles produced by the CLI.

use ratatui::text::{Line, Span};

/// Lines emitted by `jj`, with their terminal styling preserved.
#[derive(Clone, Debug)]
pub struct DocumentLines {
    lines: Vec<Line<'static>>,
}

impl DocumentLines {
    pub fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }

    pub fn lines(&self) -> &[Line<'static>] {
        &self.lines
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn file_anchors(&self) -> Vec<FileAnchor> {
        self.lines
            .iter()
            .enumerate()
            .filter_map(|(line_index, line)| file_anchor(line_index, line))
            .collect()
    }

    pub fn line_is_blank(&self, line_index: usize) -> bool {
        self.lines
            .get(line_index)
            .is_some_and(|line| line_text(line).trim().is_empty())
    }
}

/// A file heading detected in rendered jj output.
///
/// `heading` is the styled text shown in the sticky header. `label` is the plain
/// file name used for copy actions and file navigation state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileAnchor {
    line_index: usize,
    heading: Line<'static>,
    label: String,
}

impl FileAnchor {
    pub fn line_index(&self) -> usize {
        self.line_index
    }

    pub fn heading(&self) -> Line<'static> {
        self.heading.clone()
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// A document split into fixed context and a scrollable body.
///
/// The fixed lines are derived from rendered jj output rather than regenerated,
/// so colors and wording stay aligned with user config and jj defaults.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedDocument {
    fixed_lines: Vec<Line<'static>>,
    body_lines: Vec<Line<'static>>,
    body_scroll_offset: usize,
}

impl PinnedDocument {
    pub fn fixed_lines(&self) -> &[Line<'static>] {
        &self.fixed_lines
    }

    pub fn body_lines(&self) -> &[Line<'static>] {
        &self.body_lines
    }

    pub fn body_scroll_offset(&self) -> usize {
        self.body_scroll_offset
    }

    pub fn sticky_height(&self) -> u16 {
        self.fixed_lines.len().min(u16::MAX as usize) as u16
    }
}

pub fn project_with_active_file(
    document: &DocumentLines,
    anchors: &[FileAnchor],
    scroll_offset: usize,
    prefix: impl IntoIterator<Item = Line<'static>>,
) -> PinnedDocument {
    // Show views pass compact log context as a prefix; diff views pass no
    // prefix. When context exists, keep a blank line before the file heading so
    // the sticky header looks like jj output rather than a fused paragraph.
    let Some(anchor) = active_file(anchors, scroll_offset)
        .or_else(|| file_after_separator(document, anchors, scroll_offset))
    else {
        return PinnedDocument {
            fixed_lines: Vec::new(),
            body_lines: document.lines().to_vec(),
            body_scroll_offset: scroll_offset,
        };
    };

    let fixed_lines = fixed_lines(prefix, anchor);
    PinnedDocument {
        fixed_lines,
        body_lines: lines_from_active_file(document, anchor.line_index()),
        body_scroll_offset: scroll_offset.saturating_sub(anchor.line_index().saturating_add(1)),
    }
}

fn fixed_lines(
    prefix: impl IntoIterator<Item = Line<'static>>,
    anchor: &FileAnchor,
) -> Vec<Line<'static>> {
    let mut lines = prefix.into_iter().collect::<Vec<_>>();
    if !lines.is_empty() {
        lines.push(Line::default());
    }
    lines.push(anchor.heading());
    lines
}

pub fn active_file(anchors: &[FileAnchor], scroll_offset: usize) -> Option<&FileAnchor> {
    anchors
        .iter()
        .take_while(|anchor| anchor.line_index() <= scroll_offset)
        .last()
}

fn file_after_separator<'a>(
    document: &DocumentLines,
    anchors: &'a [FileAnchor],
    scroll_offset: usize,
) -> Option<&'a FileAnchor> {
    // jj show commonly separates commit metadata from the first file with a
    // blank line. Activating the file on that separator avoids a dead scroll
    // press where only hidden state changes.
    anchors.first().filter(|anchor| {
        anchor.line_index() == scroll_offset.saturating_add(1)
            && document
                .lines()
                .get(scroll_offset)
                .is_some_and(|line| line_text(line).trim().is_empty())
    })
}

fn lines_from_active_file(
    document: &DocumentLines,
    file_heading_index: usize,
) -> Vec<Line<'static>> {
    document
        .lines()
        .iter()
        .skip(file_heading_index.saturating_add(1))
        .cloned()
        .collect()
}

struct FileHeading {
    heading: Line<'static>,
    label: String,
}

fn file_anchor(line_index: usize, line: &Line<'static>) -> Option<FileAnchor> {
    let heading = file_heading(line)?;
    Some(FileAnchor {
        line_index,
        heading: heading.heading,
        label: heading.label,
    })
}

fn file_heading(line: &Line<'static>) -> Option<FileHeading> {
    let text = line_text(line);
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

fn default_file_label(heading: &str) -> Option<String> {
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

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};
    use ratatui_macros::line;

    use super::*;

    #[test]
    fn extracts_file_headings() {
        let document = DocumentLines::new(vec![
            line!("Commit ID: abc"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
            line!("Modified regular file src/main.rs:"),
        ]);

        let anchors = document.file_anchors();

        assert_eq!(anchors.len(), 2);
        assert_eq!(anchors[0].line_index(), 1);
        assert_eq!(
            line_text(&anchors[0].heading()),
            "Added regular file Cargo.toml:"
        );
        assert_eq!(anchors[0].label(), "Cargo.toml");
        assert_eq!(anchors[1].line_index(), 3);
        assert_eq!(anchors[1].label(), "src/main.rs");
    }

    #[test]
    fn extracts_git_file_headings_as_clean_paths() {
        let document = DocumentLines::new(vec![
            line!("diff --git a/src/main.rs b/src/main.rs"),
            line!("index 0000000..1111111"),
            line!("--- a/src/main.rs"),
            line!("+++ b/src/main.rs"),
            line!("@@ -1 +1 @@"),
        ]);

        let anchors = document.file_anchors();

        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].line_index(), 0);
        assert_eq!(anchors[0].label(), "src/main.rs");
        assert_eq!(line_text(&anchors[0].heading()), "src/main.rs");
    }

    #[test]
    fn default_file_heading_labels_remove_status_and_file_kind() {
        assert_eq!(
            default_file_label("Added regular file Cargo.toml:").as_deref(),
            Some("Cargo.toml")
        );
        assert_eq!(
            default_file_label("Modified executable file scripts/run:").as_deref(),
            Some("scripts/run")
        );
        assert_eq!(
            default_file_label("Removed symlink docs/current:").as_deref(),
            Some("docs/current")
        );
        assert_eq!(
            default_file_label("Renamed regular file src/old.rs => src/new.rs:").as_deref(),
            Some("src/new.rs")
        );
    }

    #[test]
    fn default_file_heading_retains_source_style_when_pinned() {
        let style = Style::default().fg(Color::Green);
        let document = DocumentLines::new(vec![
            Line::from(Span::styled("Added regular file Cargo.toml:", style)),
            line!("        1: [package]"),
        ]);
        let anchors = document.file_anchors();

        assert_eq!(
            line_text(&anchors[0].heading()),
            "Added regular file Cargo.toml:"
        );
        assert_eq!(anchors[0].heading().spans[0].style, style);
    }

    #[test]
    fn git_file_heading_retains_source_path_style_when_pinned() {
        let style = Style::default().fg(Color::Yellow);
        let document = DocumentLines::new(vec![Line::from(vec![
            Span::raw("diff --git a/src/main.rs "),
            Span::styled("b/src/main.rs", style),
        ])]);
        let anchors = document.file_anchors();

        assert_eq!(line_text(&anchors[0].heading()), "src/main.rs");
        assert_eq!(anchors[0].heading().spans[0].style, style);
    }

    #[test]
    fn active_file_is_nearest_heading_at_or_before_scroll() {
        let anchors = vec![
            FileAnchor {
                line_index: 2,
                heading: line!("Added regular file Cargo.toml:"),
                label: "Added regular file Cargo.toml:".to_owned(),
            },
            FileAnchor {
                line_index: 5,
                heading: line!("Modified regular file src/main.rs:"),
                label: "Modified regular file src/main.rs:".to_owned(),
            },
        ];

        assert!(active_file(&anchors, 1).is_none());
        assert_eq!(active_file(&anchors, 2).unwrap().line_index(), 2);
        assert_eq!(active_file(&anchors, 4).unwrap().line_index(), 2);
        assert_eq!(active_file(&anchors, 5).unwrap().line_index(), 5);
    }

    #[test]
    fn projection_pins_active_file_without_duplication() {
        let document = DocumentLines::new(vec![
            line!("Commit ID: abc"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
        ]);
        let anchors = document.file_anchors();

        let projection = project_with_active_file(&document, &anchors, 1, [line!("@  abc")]);

        assert_eq!(projection.fixed_lines().len(), 3);
        assert_eq!(projection.body_scroll_offset(), 0);
        assert_eq!(projection.body_lines().len(), 1);
        assert!(
            projection
                .body_lines()
                .iter()
                .all(|line| line_text(line) != "Added regular file Cargo.toml:")
        );
    }

    #[test]
    fn projection_scroll_transitions_do_not_reattach_prior_context() {
        let document = DocumentLines::new(vec![
            line!("Commit ID: abc"),
            line!("Change ID: def"),
            line!(""),
            line!("    subject"),
            line!(""),
            line!("Added regular file .gitignore:"),
            line!("        1: /target"),
            line!("Added regular file Cargo.toml:"),
            line!("        1: [package]"),
            line!("        2: name = \"jk\""),
        ]);
        let anchors = document.file_anchors();

        insta::assert_snapshot!(scroll_transitions(&document, &anchors, 0..9), @r#"
        == scroll 0 ==
        [body @0]
        Commit ID: abc
        Change ID: def
        
            subject
        
        Added regular file .gitignore:
                1: /target
        Added regular file Cargo.toml:
                1: [package]
                2: name = "jk"

        == scroll 1 ==
        [body @1]
        Commit ID: abc
        Change ID: def
        
            subject
        
        Added regular file .gitignore:
                1: /target
        Added regular file Cargo.toml:
                1: [package]
                2: name = "jk"

        == scroll 2 ==
        [body @2]
        Commit ID: abc
        Change ID: def
        
            subject
        
        Added regular file .gitignore:
                1: /target
        Added regular file Cargo.toml:
                1: [package]
                2: name = "jk"

        == scroll 3 ==
        [body @3]
        Commit ID: abc
        Change ID: def
        
            subject
        
        Added regular file .gitignore:
                1: /target
        Added regular file Cargo.toml:
                1: [package]
                2: name = "jk"

        == scroll 4 ==
        [fixed]
        @  abc
        │  subject
        
        Added regular file .gitignore:
        [body @0]
                1: /target
        Added regular file Cargo.toml:
                1: [package]
                2: name = "jk"

        == scroll 5 ==
        [fixed]
        @  abc
        │  subject
        
        Added regular file .gitignore:
        [body @0]
                1: /target
        Added regular file Cargo.toml:
                1: [package]
                2: name = "jk"

        == scroll 6 ==
        [fixed]
        @  abc
        │  subject
        
        Added regular file .gitignore:
        [body @0]
                1: /target
        Added regular file Cargo.toml:
                1: [package]
                2: name = "jk"

        == scroll 7 ==
        [fixed]
        @  abc
        │  subject
        
        Added regular file Cargo.toml:
        [body @0]
                1: [package]
                2: name = "jk"

        == scroll 8 ==
        [fixed]
        @  abc
        │  subject
        
        Added regular file Cargo.toml:
        [body @0]
                1: [package]
                2: name = "jk"
        "#);
    }

    #[test]
    fn projection_is_plain_document_before_first_file() {
        let document = DocumentLines::new(vec![
            line!("Commit ID: abc"),
            line!("Added regular file Cargo.toml:"),
        ]);
        let anchors = document.file_anchors();

        let projection = project_with_active_file(&document, &anchors, 0, [line!("@  abc")]);

        assert!(projection.fixed_lines().is_empty());
        assert_eq!(projection.body_lines().len(), 2);
        assert_eq!(projection.body_scroll_offset(), 0);
    }

    fn scroll_transitions(
        document: &DocumentLines,
        anchors: &[FileAnchor],
        offsets: std::ops::Range<usize>,
    ) -> String {
        offsets
            .map(|offset| {
                let projection = project_with_active_file(
                    document,
                    anchors,
                    offset,
                    [line!("@  abc"), line!("│  subject")],
                );
                format_projection(offset, projection)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_projection(offset: usize, projection: PinnedDocument) -> String {
        let mut output = format!("== scroll {offset} ==\n");
        if !projection.fixed_lines().is_empty() {
            output.push_str("[fixed]\n");
            output.push_str(&format_lines(projection.fixed_lines()));
        }
        output.push_str(&format!("[body @{}]\n", projection.body_scroll_offset()));
        output.push_str(&format_lines(projection.body_lines()));
        output
    }

    fn format_lines(lines: &[Line<'_>]) -> String {
        let mut output = lines.iter().map(line_text).collect::<Vec<_>>().join("\n");
        output.push('\n');
        output
    }
}
