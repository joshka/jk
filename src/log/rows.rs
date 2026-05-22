//! Rendered log revision row loading and metadata pairing.
//!
//! This module owns the revision-specific row contract: rendered `jj log` rows
//! are preserved as styled lines, while change and commit ids come from a
//! narrow metadata template and are paired only when row counts still match.

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;

use crate::jj::{ColorMode, JjCommand, ViewSpec, run_jj, run_jj_template_lines};

use crate::rendered_rows::{RowMetadata, first_content_char, is_standalone_graph_line, line_text};

/// One selectable item parsed from rendered graph output.
///
/// A visible graph item can span multiple terminal lines. When jj prints a real
/// revision, metadata stores the full change id and commit id used for
/// navigation and copying.
#[derive(Clone, Debug)]
pub struct LogItem {
    /// Rendered terminal lines that make up one selectable graph item.
    lines: Vec<Line<'static>>,
    /// Exact change id paired back from metadata when this item represents a revision row.
    change_id: Option<String>,
    /// Exact commit id paired back from metadata for copy actions that need commit identity.
    commit_id: Option<String>,
}

impl LogItem {
    pub fn new(
        lines: Vec<Line<'static>>,
        change_id: Option<String>,
        commit_id: Option<String>,
    ) -> Self {
        Self {
            lines,
            change_id,
            commit_id,
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn action_id(&self) -> Option<&str> {
        self.change_id()
    }

    pub fn change_id(&self) -> Option<&str> {
        self.change_id.as_deref()
    }

    pub fn commit_id(&self) -> Option<&str> {
        self.commit_id.as_deref()
    }

    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn is_visible_working_copy(&self) -> bool {
        self.lines
            .first()
            .is_some_and(|line| first_content_char(&line_text(line)) == Some('@'))
    }
}

/// Load selectable log items from the rendered `jj` output for one `ViewSpec`.
///
/// The rendered rows remain the presentation source of truth. When the command groups log items,
/// this loader also fetches narrow metadata so each visible item can still carry exact change and
/// commit ids for navigation and copy behavior.
pub fn load_entries(spec: &ViewSpec) -> Result<Vec<LogItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    if spec.command().groups_log_items() {
        let metadata = run_jj_with_template(spec, r#"change_id ++ " " ++ commit_id ++ "\n""#)?;
        Ok(group_lines(lines, metadata))
    } else {
        Ok(lines
            .into_iter()
            .map(|line| LogItem::new(vec![line], spec.target().map(str::to_owned), None))
            .collect())
    }
}

pub fn load_compact_log_context(revset: &str) -> Result<Vec<Line<'static>>> {
    let spec = ViewSpec::new(JjCommand::Log, vec!["-r".to_owned(), revset.to_owned()]);
    let output = run_jj(&spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    Ok(group_lines(lines, RowMetadata::Valid(Vec::new()))
        .into_iter()
        .next()
        .map(|item| item.lines().into_iter().take(2).collect())
        .unwrap_or_default())
}

/// Load row metadata using a narrow template that can be paired back to rendered rows.
fn run_jj_with_template(spec: &ViewSpec, template: &str) -> Result<RowMetadata<RevisionMetadata>> {
    Ok(parse_revision_metadata_lines(run_jj_template_lines(
        spec, template, false,
    )?))
}

/// Group rendered terminal lines into selectable log items and pair optional metadata by row.
fn group_lines(lines: Vec<Line<'static>>, metadata: RowMetadata<RevisionMetadata>) -> Vec<LogItem> {
    let revision_row_count = lines.iter().filter(|line| starts_log_item(line)).count();
    let mut metadata = metadata
        .into_rows_matching(revision_row_count)
        .map(Vec::into_iter);

    let mut items = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_metadata: Option<RevisionMetadata> = None;

    for line in lines {
        let starts_item = starts_log_item(&line);
        let standalone_graph_line = is_standalone_graph_line(&line);

        if (starts_item || standalone_graph_line) && !current_lines.is_empty() {
            items.push(LogItem::new(
                current_lines,
                current_metadata
                    .as_ref()
                    .map(|metadata| metadata.change_id.clone()),
                current_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.commit_id.clone()),
            ));
            current_lines = Vec::new();
            current_metadata = None;
        }

        if starts_item {
            current_metadata = metadata.as_mut().and_then(Iterator::next);
        }
        current_lines.push(line);
    }

    if !current_lines.is_empty() {
        items.push(LogItem::new(
            current_lines,
            current_metadata
                .as_ref()
                .map(|metadata| metadata.change_id.clone()),
            current_metadata.and_then(|metadata| metadata.commit_id),
        ));
    }

    items
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RevisionMetadata {
    /// Full change id emitted by the metadata template for one rendered revision row.
    change_id: String,
    /// Full commit id emitted alongside the change id when metadata pairing remains aligned.
    commit_id: Option<String>,
}

fn parse_metadata_line(line: &str) -> Option<RevisionMetadata> {
    let line = line
        .char_indices()
        .find(|(_, character)| !matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮'))
        .map(|(index, _)| &line[index..])?;

    let line = line
        .strip_prefix("@  ")
        .or_else(|| line.strip_prefix("○  "))
        .or_else(|| line.strip_prefix("◆  "))?;

    let mut tokens = line.split_whitespace();
    let change_id = tokens.next()?;
    let commit_id = tokens.next()?;

    if tokens.next().is_some() || line != format!("{change_id} {commit_id}") {
        return None;
    }

    if !is_full_change_id(change_id) || !is_full_commit_id(commit_id) {
        return None;
    }

    Some(RevisionMetadata {
        change_id: change_id.to_owned(),
        commit_id: Some(commit_id.to_owned()),
    })
}

fn parse_revision_metadata_lines(lines: Vec<String>) -> RowMetadata<RevisionMetadata> {
    let mut metadata = Vec::new();
    for line in lines {
        if is_graph_only_revision_metadata_line(&line) {
            continue;
        }
        let Some(row) = parse_metadata_line(&line) else {
            return RowMetadata::Drifted;
        };
        metadata.push(row);
    }
    RowMetadata::Valid(metadata)
}

fn is_graph_only_revision_metadata_line(line: &str) -> bool {
    let text = line.trim_start_matches(|character| {
        matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮')
    });

    text.is_empty() || text == "~" || text == "~  (elided revisions)"
}

fn is_full_commit_id(token: &str) -> bool {
    token.len() == 40 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_full_change_id(token: &str) -> bool {
    token.len() == 32 && token.bytes().all(|byte| byte.is_ascii_lowercase())
}

fn starts_log_item(line: &Line<'_>) -> bool {
    starts_log_item_text(&line_text(line))
}

fn starts_log_item_text(text: &str) -> bool {
    first_content_char(text).is_some_and(|character| matches!(character, '@' | '○' | '◆'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_ansi_output_to_selectable_items() {
        let text =
            b"\x1b[1m@\x1b[0m  change\n\xE2\x94\x82  description\n\xE2\x97\x8B  parent\n".to_vec();
        let lines = text.into_text().unwrap().lines;
        let metadata = vec![metadata("abc", "123"), metadata("def", "456")];
        let items = group_lines(lines, metadata_rows(metadata));

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].lines.len(), 2);
        assert_eq!(items[0].change_id(), Some("abc"));
        assert_eq!(items[0].commit_id(), Some("123"));
        assert_eq!(items[0].lines[0].spans[0].content, "@");
    }

    #[test]
    fn does_not_split_on_description_mentions() {
        let lines = b"@  change\n\xE2\x94\x82  email me@example.com\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![metadata("abc", "123")];

        assert_eq!(group_lines(lines, metadata_rows(metadata)).len(), 1);
    }

    #[test]
    fn pairs_one_metadata_line_with_multi_line_display_items() {
        let lines = b"@  current\n\xE2\x94\x82  current description\n\xE2\x97\x8B  parent\n\xE2\x94\x82  parent description\n\xE2\x97\x86  root\n"
                .to_vec()
                .into_text()
                .unwrap()
                .lines;
        let metadata = vec![
            metadata("current", "111"),
            metadata("parent", "222"),
            metadata("root", "333"),
        ];
        let items = group_lines(lines, metadata_rows(metadata));

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].lines.len(), 2);
        assert_eq!(items[0].change_id(), Some("current"));
        assert_eq!(items[1].lines.len(), 2);
        assert_eq!(items[1].change_id(), Some("parent"));
        assert_eq!(items[2].lines.len(), 1);
        assert_eq!(items[2].change_id(), Some("root"));
    }

    #[test]
    fn keeps_elided_log_rows_separate() {
        let lines = b"@  change\n\xE2\x94\x82  desc\n\xE2\x94\x82 ~  (elided revisions)\n\xE2\x94\x9C\xE2\x94\x80\xE2\x95\xAF\n\xE2\x97\x8B  parent\n"
                .to_vec()
                .into_text()
                .unwrap()
                .lines;
        let metadata = vec![metadata("abc", "123"), metadata("def", "456")];
        let items = group_lines(lines, metadata_rows(metadata));

        assert_eq!(items.len(), 4);
        assert_eq!(items[0].change_id(), Some("abc"));
        assert_eq!(items[1].change_id(), None);
        assert_eq!(items[2].change_id(), None);
        assert_eq!(items[3].change_id(), Some("def"));
    }

    #[test]
    fn log_rows_ignore_graph_only_revision_metadata_lines() {
        let lines = b"@  current\n\xE2\x97\x8B  parent\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let current_change_id = "a".repeat(32);
        let current_commit_id = "1".repeat(40);
        let parent_change_id = "b".repeat(32);
        let parent_commit_id = "2".repeat(40);
        let metadata = parse_revision_metadata_lines(vec![
            log_revision_metadata_text('@', 'a', '1'),
            "│ ~  (elided revisions)".to_owned(),
            "│ ├─╯".to_owned(),
            log_revision_metadata_text('○', 'b', '2'),
        ]);

        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].change_id(), Some(current_change_id.as_str()));
        assert_eq!(items[0].commit_id(), Some(current_commit_id.as_str()));
        assert_eq!(items[1].change_id(), Some(parent_change_id.as_str()));
        assert_eq!(items[1].commit_id(), Some(parent_commit_id.as_str()));
    }

    #[test]
    fn log_rows_ignore_bare_tilde_graph_noise_without_losing_ids() {
        let lines = b"@  current\n\xE2\x94\x82 ~\n\xE2\x97\x8B  parent\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let current_change_id = "a".repeat(32);
        let current_commit_id = "1".repeat(40);
        let parent_change_id = "b".repeat(32);
        let parent_commit_id = "2".repeat(40);
        let metadata = parse_revision_metadata_lines(vec![
            log_revision_metadata_text('@', 'a', '1'),
            "│ ~".to_owned(),
            log_revision_metadata_text('○', 'b', '2'),
        ]);

        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].change_id(), Some(current_change_id.as_str()));
        assert_eq!(items[0].commit_id(), Some(current_commit_id.as_str()));
        assert_eq!(items[1].change_id(), None);
        assert_eq!(items[2].change_id(), Some(parent_change_id.as_str()));
        assert_eq!(items[2].commit_id(), Some(parent_commit_id.as_str()));
    }

    #[test]
    fn log_rows_fail_closed_on_malformed_metadata_line() {
        let lines = b"@  current\n\xE2\x97\x8B  parent\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = parse_revision_metadata_lines(vec![
            log_revision_metadata_text('@', 'a', '1'),
            format!("junk {}", log_revision_metadata_text('○', 'b', '2')),
        ]);

        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.change_id().is_none()));
        assert_eq!(items[0].row_text(), "@  current");
        assert_eq!(items[1].row_text(), "○  parent");
    }

    #[test]
    fn log_rows_fail_closed_when_a_metadata_line_is_missing_ids() {
        let lines = b"@  current\n\xE2\x97\x8B  parent\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = parse_revision_metadata_lines(vec![
            log_revision_metadata_text('@', 'a', '1'),
            "○  ".to_owned(),
        ]);

        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.change_id().is_none()));
        assert_eq!(items[0].row_text(), "@  current");
        assert_eq!(items[1].row_text(), "○  parent");
    }

    #[test]
    fn log_rows_fail_closed_on_extra_metadata() {
        let lines = b"@  current\n".to_vec().into_text().unwrap().lines;
        let metadata = metadata_rows(vec![metadata("current", "111"), metadata("extra", "222")]);

        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].change_id(), None);
        assert_eq!(items[0].commit_id(), None);
        assert_eq!(items[0].row_text(), "@  current");
    }

    #[test]
    fn log_rows_fail_closed_on_row_count_mismatch() {
        let lines = b"@  current\n\xE2\x97\x8B  parent\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = metadata_rows(vec![metadata("current", "111")]);

        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.change_id().is_none()));
        assert_eq!(items[0].row_text(), "@  current");
        assert_eq!(items[1].row_text(), "○  parent");
    }

    #[test]
    fn parses_revision_metadata_lines_graph_shape() {
        let change_id = "t".repeat(32);
        let commit_id = "6".repeat(40);

        assert_eq!(
            parse_metadata_line(&log_revision_metadata_text('@', 't', '6')),
            Some(RevisionMetadata {
                change_id: change_id.to_owned(),
                commit_id: Some(commit_id.to_owned()),
            })
        );
        assert_eq!(
            parse_metadata_line(&log_revision_metadata_text('○', 't', '6')),
            Some(RevisionMetadata {
                change_id: change_id.to_owned(),
                commit_id: Some(commit_id.to_owned()),
            })
        );
        assert_eq!(
            parse_metadata_line(&format!("│ ○  {} {}", change_id, commit_id)),
            Some(RevisionMetadata {
                change_id: change_id.to_owned(),
                commit_id: Some(commit_id.to_owned()),
            })
        );
        assert_eq!(
            parse_metadata_line(&format!("│ ◆  {} {}", change_id, commit_id)),
            Some(RevisionMetadata {
                change_id: change_id.to_owned(),
                commit_id: Some(commit_id.to_owned()),
            })
        );
        assert_eq!(
            parse_metadata_line(&format!(
                "junk {}",
                log_revision_metadata_text('@', 't', '6')
            )),
            None
        );
        assert_eq!(
            parse_metadata_line(&format!(
                "{} junk",
                log_revision_metadata_text('@', 't', '6')
            )),
            None
        );
        assert_eq!(
            parse_metadata_line(&format!("│ junk @  {} {}", change_id, commit_id)),
            None
        );
    }

    fn metadata(change_id: &str, commit_id: &str) -> RevisionMetadata {
        RevisionMetadata {
            change_id: change_id.to_owned(),
            commit_id: Some(commit_id.to_owned()),
        }
    }

    fn metadata_rows(rows: Vec<RevisionMetadata>) -> RowMetadata<RevisionMetadata> {
        RowMetadata::Valid(rows)
    }

    fn log_revision_metadata_text(marker: char, change_digit: char, commit_digit: char) -> String {
        format!(
            "{marker}  {} {}",
            std::iter::repeat_n(change_digit, 32).collect::<String>(),
            std::iter::repeat_n(commit_digit, 40).collect::<String>()
        )
    }
}
