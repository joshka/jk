//! Rendered `jj` row loading and narrow metadata pairing.
//!
//! This module owns conversion from rendered jj terminal output and narrow
//! machine-oriented metadata templates into the selectable row models consumed
//! by the app's list views. Command identity, navigation provenance, and the
//! process boundary stay in `jj.rs`.

mod bookmarks;
mod operations;
mod workspaces;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;
use serde_json::Value;

use crate::jj::{ColorMode, JjCommand, ViewSpec, run_jj, run_jj_template_lines};

pub use self::bookmarks::{
    BookmarkItem, BookmarkLocalPeerState, BookmarkRowState, LocalBookmarkRemoteState,
    RemoteBookmarkTrackingState, load_bookmark_entries,
};
#[cfg(test)]
pub(crate) use self::operations::OPERATION_ID_TEMPLATE;
pub use self::operations::{OperationLogItem, load_operation_log_entries};
#[cfg(test)]
pub(crate) use self::workspaces::WORKSPACE_METADATA_TEMPLATE;
pub use self::workspaces::{WorkspaceContext, WorkspaceItem, load_workspace_context};
pub(crate) const RESOLVE_CONFLICT_TEMPLATE: &str = r#"self.conflicted_files().map(|entry| "{\"path\":" ++ json(entry.path()) ++ ",\"file_type\":" ++ json(entry.file_type()) ++ ",\"side_count\":" ++ json(entry.conflict_side_count()) ++ "}\n").join("")"#;

/// One selectable item parsed from rendered graph output.
///
/// A visible graph item can span multiple terminal lines. When jj prints a real
/// revision, metadata stores the full change id and commit id used for
/// navigation and copying.
#[derive(Clone, Debug)]
pub struct LogItem {
    lines: Vec<Line<'static>>,
    change_id: Option<String>,
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

/// One selectable file item parsed from rendered file-list output.
#[derive(Clone, Debug)]
pub struct FileListItem {
    lines: Vec<Line<'static>>,
    path: String,
}

impl FileListItem {
    pub fn new(lines: Vec<Line<'static>>, path: String) -> Self {
        Self { lines, path }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    #[cfg(test)]
    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// One conflicted path reported by the resolve template contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolveEntry {
    path: Option<String>,
    file_type: Option<String>,
    side_count: Option<usize>,
    raw_line: Option<String>,
}

impl ResolveEntry {
    pub fn parsed(
        path: Option<String>,
        file_type: Option<String>,
        side_count: Option<usize>,
    ) -> Self {
        Self {
            path,
            file_type,
            side_count,
            raw_line: None,
        }
    }

    pub fn unparsed(raw_line: String) -> Self {
        Self {
            path: None,
            file_type: None,
            side_count: None,
            raw_line: Some(raw_line),
        }
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    pub fn file_type(&self) -> Option<&str> {
        self.file_type.as_deref()
    }

    pub fn side_count(&self) -> Option<usize> {
        self.side_count
    }

    pub fn raw_line(&self) -> Option<&str> {
        self.raw_line.as_deref()
    }
}

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

pub fn load_resolve_entries(spec: &ViewSpec) -> Result<Vec<ResolveEntry>> {
    Ok(
        run_jj_template_lines(spec, RESOLVE_CONFLICT_TEMPLATE, true)?
            .into_iter()
            .map(|line| parse_resolve_entry_line(&line))
            .collect(),
    )
}

pub fn load_file_list_entries(spec: &ViewSpec) -> Result<Vec<FileListItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    Ok(lines
        .into_iter()
        .filter_map(|line| {
            let path = parse_file_list_path(&line_text(&line))?;
            Some(FileListItem::new(vec![line], path))
        })
        .collect())
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

pub fn document_plain_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

fn run_jj_with_template(spec: &ViewSpec, template: &str) -> Result<RowMetadata<RevisionMetadata>> {
    Ok(parse_revision_metadata_lines(run_jj_template_lines(
        spec, template, false,
    )?))
}

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
    change_id: String,
    commit_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RowMetadata<T> {
    Valid(Vec<T>),
    Drifted,
}

impl<T> RowMetadata<T> {
    fn into_rows_matching(self, rendered_row_count: usize) -> Option<Vec<T>> {
        match self {
            Self::Valid(rows) if rows.len() == rendered_row_count => Some(rows),
            Self::Valid(_) | Self::Drifted => None,
        }
    }
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

fn parse_resolve_entry_line(line: &str) -> ResolveEntry {
    let raw_line = line.to_owned();
    let Ok(Value::Object(fields)) = serde_json::from_str::<Value>(line) else {
        return ResolveEntry::unparsed(raw_line);
    };

    ResolveEntry::parsed(
        string_field(&fields, "path"),
        string_field(&fields, "file_type"),
        integer_field(&fields, "side_count"),
    )
}

fn parse_file_list_path(line: &str) -> Option<String> {
    (!line.is_empty()).then(|| line.to_owned())
}

fn string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    fields.get(name).and_then(Value::as_str).map(str::to_owned)
}

fn non_empty_string_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<String> {
    string_field(fields, name).filter(|field| !field.is_empty())
}

fn optional_string_field(
    fields: &serde_json::Map<String, Value>,
    name: &str,
) -> Option<Option<String>> {
    match fields.get(name) {
        Some(Value::Null) | None => Some(None),
        Some(Value::String(value)) if value.is_empty() => Some(None),
        Some(Value::String(value)) => Some(Some(value.clone())),
        _ => None,
    }
}

fn boolean_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<bool> {
    fields.get(name).and_then(Value::as_bool)
}

fn integer_field(fields: &serde_json::Map<String, Value>, name: &str) -> Option<usize> {
    fields
        .get(name)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
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

fn is_standalone_graph_line(line: &Line<'_>) -> bool {
    let text = line_text(line);
    first_content_char(&text).is_none_or(|character| character == '~')
}

fn first_content_char(text: &str) -> Option<char> {
    text.chars()
        .find(|character| !matches!(character, ' ' | '│' | '├' | '─' | '╯' | '╰' | '╭' | '╮'))
}

fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
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
    fn keeps_elided_graph_rows_separate() {
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
            graph_revision_metadata_text('@', 'a', '1'),
            "│ ~  (elided revisions)".to_owned(),
            "│ ├─╯".to_owned(),
            graph_revision_metadata_text('○', 'b', '2'),
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
            graph_revision_metadata_text('@', 'a', '1'),
            "│ ~".to_owned(),
            graph_revision_metadata_text('○', 'b', '2'),
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
            graph_revision_metadata_text('@', 'a', '1'),
            format!("junk {}", graph_revision_metadata_text('○', 'b', '2')),
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
            graph_revision_metadata_text('@', 'a', '1'),
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
            parse_metadata_line(&graph_revision_metadata_text('@', 't', '6')),
            Some(RevisionMetadata {
                change_id: change_id.to_owned(),
                commit_id: Some(commit_id.to_owned()),
            })
        );
        assert_eq!(
            parse_metadata_line(&graph_revision_metadata_text('○', 't', '6')),
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
                graph_revision_metadata_text('@', 't', '6')
            )),
            None
        );
        assert_eq!(
            parse_metadata_line(&format!(
                "{} junk",
                graph_revision_metadata_text('@', 't', '6')
            )),
            None
        );
        assert_eq!(
            parse_metadata_line(&format!("│ junk @  {} {}", change_id, commit_id)),
            None
        );
    }

    #[test]
    fn resolve_entry_parser_keeps_exact_fields() {
        let entry = parse_resolve_entry_line(
            r#"{"path":"dir/space file.txt","file_type":"file","side_count":3}"#,
        );

        assert_eq!(
            entry,
            ResolveEntry::parsed(
                Some("dir/space file.txt".to_owned()),
                Some("file".to_owned()),
                Some(3),
            )
        );
    }

    #[test]
    fn resolve_entry_parser_degrades_invalid_json_to_unparsed_row() {
        let entry = parse_resolve_entry_line("{not json");

        assert_eq!(entry, ResolveEntry::unparsed("{not json".to_owned()));
    }

    #[test]
    fn resolve_entry_parser_allows_missing_exact_path() {
        let entry =
            parse_resolve_entry_line(r#"{"path":null,"file_type":"symlink","side_count":2}"#);

        assert_eq!(
            entry,
            ResolveEntry::parsed(None, Some("symlink".to_owned()), Some(2))
        );
    }

    #[test]
    fn file_list_path_parser_preserves_exact_text() {
        assert_eq!(
            parse_file_list_path("src/path with spaces"),
            Some("src/path with spaces".to_owned())
        );
        assert_eq!(parse_file_list_path(""), None);
    }

    #[test]
    fn file_list_item_preserves_row_lines_and_path() {
        let lines = b"src/path with spaces\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let item = FileListItem::new(lines, "src/path with spaces".to_owned());

        assert_eq!(item.line_count(), 1);
        assert_eq!(item.path(), "src/path with spaces");
        assert_eq!(item.row_text(), "src/path with spaces");
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

    fn graph_revision_metadata_text(
        marker: char,
        change_digit: char,
        commit_digit: char,
    ) -> String {
        format!(
            "{marker}  {} {}",
            std::iter::repeat_n(change_digit, 32).collect::<String>(),
            std::iter::repeat_n(commit_digit, 40).collect::<String>()
        )
    }
}
