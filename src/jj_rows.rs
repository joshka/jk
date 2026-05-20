//! Rendered `jj` row loading and narrow metadata pairing.
//!
//! This module owns conversion from rendered jj terminal output and narrow
//! machine-oriented metadata templates into the selectable row models consumed
//! by the app's list views. Command identity, navigation provenance, and the
//! process boundary stay in `jj.rs`.

use std::collections::HashSet;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;
use serde_json::Value;

use crate::jj::{ColorMode, JjCommand, ViewSpec, run_jj, run_jj_template_lines};

pub(crate) const BOOKMARK_METADATA_TEMPLATE: &str = concat!(
    r#""{\"name\":" ++ json(name)"#,
    r#" ++ ",\"remote\":" ++ json(remote)"#,
    r#" ++ ",\"tracked\":" ++ json(tracked)"#,
    r#" ++ ",\"tracking_present\":" ++ json(tracking_present)"#,
    r#" ++ ",\"synced\":" ++ json(synced)"#,
    r#" ++ ",\"target_change_id\":" ++ json(if(normal_target, normal_target.change_id(), ""))"#,
    r#" ++ ",\"target_commit_id\":" ++ json(if(normal_target, normal_target.commit_id(), ""))"#,
    r#" ++ "}\n""#,
);
pub(crate) const OPERATION_ID_TEMPLATE: &str = "self.id() ++ \"\\n\"";
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

/// One selectable bookmark item parsed from rendered bookmark output.
#[derive(Clone, Debug)]
pub struct BookmarkItem {
    lines: Vec<Line<'static>>,
    name: String,
    target_change_id: Option<String>,
    target_commit_id: Option<String>,
    state: BookmarkRowState,
}

impl BookmarkItem {
    pub fn new(
        lines: Vec<Line<'static>>,
        name: String,
        target_change_id: Option<String>,
        target_commit_id: Option<String>,
    ) -> Self {
        Self {
            lines,
            name,
            target_change_id,
            target_commit_id,
            state: BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Ambiguous,
            },
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn bookmark_name(&self) -> &str {
        &self.name
    }

    pub fn target_change_id(&self) -> Option<&str> {
        self.target_change_id.as_deref()
    }

    pub fn target_commit_id(&self) -> Option<&str> {
        self.target_commit_id.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn is_local(&self) -> bool {
        matches!(self.state, BookmarkRowState::Local { .. })
    }

    pub fn state(&self) -> &BookmarkRowState {
        &self.state
    }

    #[cfg(test)]
    pub(crate) fn with_local(mut self, local: bool) -> Self {
        self.state = if local {
            BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Ambiguous,
            }
        } else {
            BookmarkRowState::Unknown
        };
        self
    }

    #[cfg(test)]
    pub(crate) fn with_state(mut self, state: BookmarkRowState) -> Self {
        self.state = state;
        self
    }

    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BookmarkRowState {
    Local {
        tracking: LocalBookmarkRemoteState,
    },
    Remote {
        remote: String,
        tracking: RemoteBookmarkTrackingState,
        local_peer: BookmarkLocalPeerState,
    },
    Unknown,
}

impl BookmarkRowState {
    fn local(tracking: LocalBookmarkRemoteState) -> Self {
        Self::Local { tracking }
    }

    fn remote(
        remote: String,
        tracking: RemoteBookmarkTrackingState,
        local_peer: BookmarkLocalPeerState,
    ) -> Self {
        Self::Remote {
            remote,
            tracking,
            local_peer,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LocalBookmarkRemoteState {
    LocalOnly,
    Tracked { untracked_remote_present: bool },
    UntrackedRemotePresent,
    Ambiguous,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteBookmarkTrackingState {
    Tracked { local_present: bool, synced: bool },
    Untracked { synced: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BookmarkLocalPeerState {
    Present,
    Absent,
    Unknown,
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

/// One selectable operation item parsed from rendered operation-log output.
#[derive(Clone, Debug)]
pub struct OperationLogItem {
    lines: Vec<Line<'static>>,
    operation_id: Option<String>,
}

impl OperationLogItem {
    pub fn new(lines: Vec<Line<'static>>, operation_id: Option<String>) -> Self {
        Self {
            lines,
            operation_id,
        }
    }

    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn operation_id(&self) -> Option<&str> {
        self.operation_id.as_deref()
    }

    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
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

pub fn load_bookmark_entries(spec: &ViewSpec) -> Result<Vec<BookmarkItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;
    let metadata = run_jj_bookmark_metadata(spec)?;
    Ok(pair_bookmark_lines(
        lines,
        metadata,
        bookmark_metadata_coverage(spec),
    ))
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

pub fn load_operation_log_entries(spec: &ViewSpec) -> Result<Vec<OperationLogItem>> {
    let output = run_jj(spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;
    let operation_ids = run_operation_log_ids(spec)?;
    Ok(group_operation_log_lines(lines, operation_ids))
}

pub fn load_compact_log_context(revset: &str) -> Result<Vec<Line<'static>>> {
    let spec = ViewSpec::new(JjCommand::Log, vec!["-r".to_owned(), revset.to_owned()]);
    let output = run_jj(&spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    Ok(group_lines(lines, Vec::new())
        .into_iter()
        .next()
        .map(|item| item.lines().into_iter().take(2).collect())
        .unwrap_or_default())
}

pub fn document_plain_text(lines: &[Line<'static>]) -> String {
    lines.iter().map(line_text).collect::<Vec<_>>().join("\n")
}

fn run_jj_with_template(spec: &ViewSpec, template: &str) -> Result<Vec<RevisionMetadata>> {
    Ok(run_jj_template_lines(spec, template, false)?
        .into_iter()
        .filter_map(|line| parse_metadata_line(&line))
        .collect())
}

fn run_jj_bookmark_metadata(spec: &ViewSpec) -> Result<Vec<BookmarkMetadata>> {
    Ok(
        run_jj_template_lines(spec, BOOKMARK_METADATA_TEMPLATE, false)?
            .into_iter()
            .filter_map(|line| parse_bookmark_metadata_line(&line))
            .collect(),
    )
}

fn run_operation_log_ids(spec: &ViewSpec) -> Result<Vec<Option<String>>> {
    Ok(run_jj_template_lines(spec, OPERATION_ID_TEMPLATE, true)?
        .into_iter()
        .map(|line| parse_operation_id_line(&line))
        .collect())
}

fn group_lines(lines: Vec<Line<'static>>, metadata: Vec<RevisionMetadata>) -> Vec<LogItem> {
    let mut items = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_metadata: Option<RevisionMetadata> = None;
    let mut metadata = metadata.into_iter();

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
            current_metadata = metadata.next();
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

fn pair_bookmark_lines(
    lines: Vec<Line<'static>>,
    metadata: Vec<BookmarkMetadata>,
    coverage: BookmarkMetadataCoverage,
) -> Vec<BookmarkItem> {
    if lines.len() != metadata.len() {
        return lines
            .into_iter()
            .map(|line| {
                let text = line_text(&line);
                BookmarkItem {
                    lines: vec![line],
                    name: bookmark_name_from_rendered_row(&text),
                    target_change_id: None,
                    target_commit_id: None,
                    state: BookmarkRowState::Unknown,
                }
            })
            .collect();
    }

    let local_names = metadata
        .iter()
        .filter(|metadata| metadata.remote.is_none())
        .map(|metadata| metadata.name.clone())
        .collect::<HashSet<_>>();
    let tracked_remote_names = metadata
        .iter()
        .filter(|metadata| metadata.remote.is_some() && metadata.tracked)
        .map(|metadata| metadata.name.clone())
        .collect::<HashSet<_>>();
    let untracked_remote_names = metadata
        .iter()
        .filter(|metadata| metadata.remote.is_some() && !metadata.tracked)
        .map(|metadata| metadata.name.clone())
        .collect::<HashSet<_>>();

    let mut items = Vec::new();
    let mut metadata = metadata.into_iter();

    for line in lines {
        let text = line_text(&line);
        let metadata = metadata.next();
        let bookmark_name = metadata
            .as_ref()
            .map(|metadata| metadata.name.clone())
            .unwrap_or_else(|| bookmark_name_from_rendered_row(&text));
        let mut item = BookmarkItem::new(
            vec![line],
            bookmark_name,
            metadata
                .as_ref()
                .and_then(|metadata| metadata.target_change_id.clone()),
            metadata
                .as_ref()
                .and_then(|metadata| metadata.target_commit_id.clone()),
        );
        item.state = metadata
            .as_ref()
            .map_or(BookmarkRowState::Unknown, |metadata| {
                metadata.row_state(
                    coverage,
                    &local_names,
                    &tracked_remote_names,
                    &untracked_remote_names,
                )
            });
        items.push(item);
    }

    items
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BookmarkMetadataCoverage {
    VisibleRowsOnly,
    AllRemotes,
}

fn bookmark_metadata_coverage(spec: &ViewSpec) -> BookmarkMetadataCoverage {
    if spec
        .args()
        .iter()
        .any(|arg| matches!(arg.as_str(), "--all-remotes" | "-a"))
    {
        BookmarkMetadataCoverage::AllRemotes
    } else {
        BookmarkMetadataCoverage::VisibleRowsOnly
    }
}

fn group_operation_log_lines(
    lines: Vec<Line<'static>>,
    operation_ids: Vec<Option<String>>,
) -> Vec<OperationLogItem> {
    let mut items = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_operation_id = None;
    let mut operation_ids = operation_ids.into_iter();

    for line in lines {
        let starts_item = starts_operation_log_item(&line);
        let standalone_graph_line = is_standalone_graph_line(&line);

        if (starts_item || standalone_graph_line) && !current_lines.is_empty() {
            items.push(OperationLogItem::new(current_lines, current_operation_id));
            current_lines = Vec::new();
            current_operation_id = None;
        }

        if starts_item {
            current_operation_id = operation_ids.next().flatten();
        }
        current_lines.push(line);
    }

    if !current_lines.is_empty() {
        items.push(OperationLogItem::new(current_lines, current_operation_id));
    }

    items
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RevisionMetadata {
    change_id: String,
    commit_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BookmarkMetadata {
    name: String,
    remote: Option<String>,
    tracked: bool,
    tracking_present: bool,
    synced: bool,
    target_change_id: Option<String>,
    target_commit_id: Option<String>,
}

impl BookmarkMetadata {
    fn row_state(
        &self,
        coverage: BookmarkMetadataCoverage,
        local_names: &HashSet<String>,
        tracked_remote_names: &HashSet<String>,
        untracked_remote_names: &HashSet<String>,
    ) -> BookmarkRowState {
        match &self.remote {
            None => BookmarkRowState::local(local_bookmark_remote_state(
                self,
                coverage,
                tracked_remote_names,
                untracked_remote_names,
            )),
            Some(remote) => BookmarkRowState::remote(
                remote.clone(),
                remote_bookmark_tracking_state(self),
                local_peer_state(self, coverage, local_names),
            ),
        }
    }
}

fn parse_metadata_line(line: &str) -> Option<RevisionMetadata> {
    let mut change_id = None;
    let mut commit_id = None;

    for token in line.split_whitespace() {
        if change_id.is_none() && is_full_change_id(token) {
            change_id = Some(token.to_owned());
        } else if commit_id.is_none() && is_full_commit_id(token) {
            commit_id = Some(token.to_owned());
        }
    }

    change_id.map(|change_id| RevisionMetadata {
        change_id,
        commit_id,
    })
}

fn parse_bookmark_metadata_line(line: &str) -> Option<BookmarkMetadata> {
    if line.is_empty() {
        return None;
    }

    let Value::Object(fields) = serde_json::from_str::<Value>(line).ok()? else {
        return None;
    };

    let name = string_field(&fields, "name")?;
    if name.is_empty() {
        return None;
    }

    Some(BookmarkMetadata {
        name,
        remote: optional_string_field(&fields, "remote")?,
        tracked: boolean_field(&fields, "tracked")?,
        tracking_present: boolean_field(&fields, "tracking_present")?,
        synced: boolean_field(&fields, "synced")?,
        target_change_id: non_empty_string_field(&fields, "target_change_id"),
        target_commit_id: non_empty_string_field(&fields, "target_commit_id"),
    })
}

fn local_bookmark_remote_state(
    metadata: &BookmarkMetadata,
    coverage: BookmarkMetadataCoverage,
    tracked_remote_names: &HashSet<String>,
    untracked_remote_names: &HashSet<String>,
) -> LocalBookmarkRemoteState {
    let tracked_remote_present = tracked_remote_names.contains(metadata.name.as_str());
    let untracked_remote_present = untracked_remote_names.contains(metadata.name.as_str());

    if tracked_remote_present {
        LocalBookmarkRemoteState::Tracked {
            untracked_remote_present,
        }
    } else if untracked_remote_present {
        LocalBookmarkRemoteState::UntrackedRemotePresent
    } else if coverage == BookmarkMetadataCoverage::AllRemotes {
        LocalBookmarkRemoteState::LocalOnly
    } else {
        LocalBookmarkRemoteState::Ambiguous
    }
}

fn remote_bookmark_tracking_state(metadata: &BookmarkMetadata) -> RemoteBookmarkTrackingState {
    if metadata.tracked {
        RemoteBookmarkTrackingState::Tracked {
            local_present: metadata.tracking_present,
            synced: metadata.synced,
        }
    } else {
        RemoteBookmarkTrackingState::Untracked {
            synced: metadata.synced,
        }
    }
}

fn local_peer_state(
    metadata: &BookmarkMetadata,
    coverage: BookmarkMetadataCoverage,
    local_names: &HashSet<String>,
) -> BookmarkLocalPeerState {
    if local_names.contains(metadata.name.as_str()) {
        BookmarkLocalPeerState::Present
    } else if coverage == BookmarkMetadataCoverage::AllRemotes {
        BookmarkLocalPeerState::Absent
    } else {
        BookmarkLocalPeerState::Unknown
    }
}

fn parse_operation_id_line(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|token| is_operation_id(token))
        .map(str::to_owned)
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

fn is_operation_id(token: &str) -> bool {
    token.len() == 128 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn starts_log_item(line: &Line<'_>) -> bool {
    starts_log_item_text(&line_text(line))
}

fn starts_log_item_text(text: &str) -> bool {
    first_content_char(text).is_some_and(|character| matches!(character, '@' | '○' | '◆'))
}

fn starts_operation_log_item(line: &Line<'_>) -> bool {
    first_content_char(&line_text(line)).is_some_and(|character| matches!(character, '@' | '○'))
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

fn bookmark_name_from_rendered_row(text: &str) -> String {
    text.split_once(':')
        .map(|(name, _)| name.trim())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| text.trim())
        .to_owned()
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
        let items = group_lines(lines, metadata);

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

        assert_eq!(group_lines(lines, metadata).len(), 1);
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
        let items = group_lines(lines, metadata);

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
        let items = group_lines(lines, metadata);

        assert_eq!(items.len(), 4);
        assert_eq!(items[0].change_id(), Some("abc"));
        assert_eq!(items[1].change_id(), None);
        assert_eq!(items[2].change_id(), None);
        assert_eq!(items[3].change_id(), Some("def"));
    }

    #[test]
    fn groups_operation_log_rows_and_preserves_styles() {
        let text =
            b"\x1b[1m@\x1b[0m  current\n\xE2\x94\x82  args: jj describe\n\xE2\x97\x8B  previous\n"
                .to_vec();
        let lines = text.into_text().unwrap().lines;
        let operation_ids = vec![Some(operation_id('a')), Some(operation_id('b'))];

        let items = group_operation_log_lines(lines, operation_ids);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].line_count(), 2);
        assert_eq!(items[0].operation_id(), Some(operation_id('a').as_str()));
        assert_eq!(items[0].lines[0].spans[0].content, "@");
        assert_eq!(items[1].operation_id(), Some(operation_id('b').as_str()));
    }

    #[test]
    fn operation_log_rows_allow_missing_metadata() {
        let lines = b"@  current\n\xE2\x94\x82  args: jj describe\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;

        let items = group_operation_log_lines(lines, vec![None]);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].operation_id(), None);
    }

    #[test]
    fn parses_revision_metadata_lines() {
        assert_eq!(
            parse_metadata_line(
                "@  tvykuurwpnwzzqulzrvwvmxxotnlywqw 64d399917e441072c228d7811743550753c9f6cf"
            ),
            Some(RevisionMetadata {
                change_id: "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
                commit_id: Some("64d399917e441072c228d7811743550753c9f6cf".to_owned()),
            })
        );
        assert_eq!(
            parse_metadata_line("@  tvykuurwpnwzzqulzrvwvmxxotnlywqw"),
            Some(RevisionMetadata {
                change_id: "tvykuurwpnwzzqulzrvwvmxxotnlywqw".to_owned(),
                commit_id: None,
            })
        );
        assert_eq!(parse_metadata_line("│ ~  (elided revisions)"), None);
    }

    #[test]
    fn parses_operation_id_lines() {
        let operation_id = operation_id('a');

        assert_eq!(
            parse_operation_id_line(&("@  ".to_owned() + &operation_id + "\n")),
            Some(operation_id)
        );
        assert_eq!(parse_operation_id_line("not-an-operation"), None);
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

    #[test]
    fn parses_bookmark_metadata_lines() {
        assert_eq!(
            parse_bookmark_metadata_line(
                r#"{"name":"main","remote":null,"tracked":false,"tracking_present":false,"synced":true,"target_change_id":"wuqolszplkmommqzmxpmmwtwrpuuwkmo","target_commit_id":"2f81d8af4234fef19b84d1495383a55999bb37fa"}"#
            ),
            Some(BookmarkMetadata {
                name: "main".to_owned(),
                remote: None,
                tracked: false,
                tracking_present: false,
                synced: true,
                target_change_id: Some("wuqolszplkmommqzmxpmmwtwrpuuwkmo".to_owned()),
                target_commit_id: Some("2f81d8af4234fef19b84d1495383a55999bb37fa".to_owned()),
            })
        );
        assert_eq!(
            parse_bookmark_metadata_line(
                r#"{"name":"main","remote":"origin","tracked":true,"tracking_present":true,"synced":false,"target_change_id":"","target_commit_id":"","future_field":"ignored"}"#
            ),
            Some(BookmarkMetadata {
                name: "main".to_owned(),
                remote: Some("origin".to_owned()),
                tracked: true,
                tracking_present: true,
                synced: false,
                target_change_id: None,
                target_commit_id: None,
            })
        );
        assert_eq!(parse_bookmark_metadata_line(r#"{"name":"main"}"#), None);
        assert_eq!(parse_bookmark_metadata_line("main\torigin\t\t"), None);
        assert_eq!(parse_bookmark_metadata_line(""), None);
    }

    #[test]
    fn pairs_bookmark_rows_in_render_order() {
        let lines = b"main: okrnpmzv d10e26b6 Update agent repository guidance\nprototype: nqvrkyps f65c4354 docs: add explicit unsupported warning\n"
                .to_vec()
                .into_text()
                .unwrap()
                .lines;
        let metadata = vec![
            bookmark_metadata(
                "main",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            bookmark_metadata(
                "prototype",
                Some("nqvrkypsnqvrkypsnqvrkypsnqvrkyps"),
                Some("f65c4354f65c4354f65c4354f65c4354f65c4354"),
            ),
        ];

        let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::VisibleRowsOnly);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].line_count(), 1);
        assert_eq!(items[0].bookmark_name(), "main");
        assert_eq!(
            items[0].state(),
            &BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Ambiguous
            }
        );
        assert_eq!(
            items[0].target_change_id(),
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv")
        );
        assert_eq!(
            items[0].target_commit_id(),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6")
        );
        assert_eq!(items[1].bookmark_name(), "prototype");
    }

    #[test]
    fn remote_bookmark_rows_do_not_advance_local_metadata() {
        let lines = b"main: okrnpmzv d10e26b6 Update agent repository guidance\nmain@origin: okrnpmzv d10e26b6 Update agent repository guidance\nprototype: nqvrkyps f65c4354 docs: add explicit unsupported warning\n"
                .to_vec()
                .into_text()
                .unwrap()
                .lines;
        let metadata = vec![
            bookmark_metadata(
                "main",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            remote_bookmark_metadata(
                "main",
                "origin",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            bookmark_metadata(
                "prototype",
                Some("nqvrkypsnqvrkypsnqvrkypsnqvrkyps"),
                Some("f65c4354f65c4354f65c4354f65c4354f65c4354"),
            ),
        ];

        let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::AllRemotes);

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].bookmark_name(), "main");
        assert!(items[0].is_local());
        assert_eq!(
            items[0].state(),
            &BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Tracked {
                    untracked_remote_present: false,
                }
            }
        );
        assert_eq!(items[1].bookmark_name(), "main");
        assert!(!items[1].is_local());
        assert_eq!(
            items[1].state(),
            &BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Present,
            }
        );
        assert_eq!(
            items[1].target_change_id(),
            Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv")
        );
        assert_eq!(
            items[1].target_commit_id(),
            Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6")
        );
        assert_eq!(items[2].bookmark_name(), "prototype");
        assert!(items[2].is_local());
    }

    #[test]
    fn all_remote_bookmark_metadata_marks_local_only_and_untracked_remote_rows() {
        let lines = b"local-only: okrnpmzv d10e26b6\nremote-only@origin: okrnpmzv d10e26b6\nlocal-with-untracked: okrnpmzv d10e26b6\nlocal-with-untracked@origin: okrnpmzv d10e26b6\n"
                .to_vec()
                .into_text()
                .unwrap()
                .lines;
        let metadata = vec![
            bookmark_metadata(
                "local-only",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            remote_bookmark_metadata(
                "remote-only",
                "origin",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            )
            .with_tracking(false, false, false),
            bookmark_metadata(
                "local-with-untracked",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            remote_bookmark_metadata(
                "local-with-untracked",
                "origin",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            )
            .with_tracking(false, false, false),
        ];

        let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::AllRemotes);

        assert_eq!(
            items[0].state(),
            &BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::LocalOnly
            }
        );
        assert_eq!(
            items[1].state(),
            &BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Absent,
            }
        );
        assert_eq!(
            items[2].state(),
            &BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::UntrackedRemotePresent
            }
        );
        assert_eq!(
            items[3].state(),
            &BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Present,
            }
        );
    }

    #[test]
    fn bookmark_rows_without_metadata_are_not_marked_local() {
        let lines = b"remote-looking-but-not-trusted: okrnpmzv d10e26b6\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;

        let items =
            pair_bookmark_lines(lines, Vec::new(), BookmarkMetadataCoverage::VisibleRowsOnly);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].bookmark_name(), "remote-looking-but-not-trusted");
        assert!(!items[0].is_local());
        assert_eq!(items[0].state(), &BookmarkRowState::Unknown);
    }

    #[test]
    fn bookmark_rows_with_extra_metadata_are_not_marked_local() {
        let lines = b"main: okrnpmzv d10e26b6\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![
            bookmark_metadata(
                "main",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            remote_bookmark_metadata(
                "main",
                "origin",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
        ];

        let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::AllRemotes);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].bookmark_name(), "main");
        assert_eq!(items[0].target_change_id(), None);
        assert_eq!(items[0].state(), &BookmarkRowState::Unknown);
    }

    #[test]
    fn tracked_local_bookmark_state_preserves_untracked_remote_peer() {
        let lines = b"main: okrnpmzv d10e26b6\nmain@origin: okrnpmzv d10e26b6\nmain@upstream: okrnpmzv d10e26b6\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![
            bookmark_metadata(
                "main",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            remote_bookmark_metadata(
                "main",
                "origin",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            ),
            remote_bookmark_metadata(
                "main",
                "upstream",
                Some("okrnpmzvokrnpmzvokrnpmzvokrnpmzv"),
                Some("d10e26b6d10e26b6d10e26b6d10e26b6d10e26b6"),
            )
            .with_tracking(false, false, false),
        ];

        let items = pair_bookmark_lines(lines, metadata, BookmarkMetadataCoverage::AllRemotes);

        assert_eq!(
            items[0].state(),
            &BookmarkRowState::Local {
                tracking: LocalBookmarkRemoteState::Tracked {
                    untracked_remote_present: true,
                }
            }
        );
        assert_eq!(
            items[1].state(),
            &BookmarkRowState::Remote {
                remote: "origin".to_owned(),
                tracking: RemoteBookmarkTrackingState::Tracked {
                    local_present: true,
                    synced: true,
                },
                local_peer: BookmarkLocalPeerState::Present,
            }
        );
        assert_eq!(
            items[2].state(),
            &BookmarkRowState::Remote {
                remote: "upstream".to_owned(),
                tracking: RemoteBookmarkTrackingState::Untracked { synced: false },
                local_peer: BookmarkLocalPeerState::Present,
            }
        );
    }
    fn metadata(change_id: &str, commit_id: &str) -> RevisionMetadata {
        RevisionMetadata {
            change_id: change_id.to_owned(),
            commit_id: Some(commit_id.to_owned()),
        }
    }

    fn bookmark_metadata(
        name: &str,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkMetadata {
        bookmark_metadata_with_remote(name, None, target_change_id, target_commit_id)
    }

    fn remote_bookmark_metadata(
        name: &str,
        remote: &str,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkMetadata {
        bookmark_metadata_with_remote(name, Some(remote), target_change_id, target_commit_id)
    }

    fn bookmark_metadata_with_remote(
        name: &str,
        remote: Option<&str>,
        target_change_id: Option<&str>,
        target_commit_id: Option<&str>,
    ) -> BookmarkMetadata {
        BookmarkMetadata {
            name: name.to_owned(),
            remote: remote.map(str::to_owned),
            tracked: remote.is_some(),
            tracking_present: remote.is_some(),
            synced: remote.is_some(),
            target_change_id: target_change_id.map(str::to_owned),
            target_commit_id: target_commit_id.map(str::to_owned),
        }
    }

    trait BookmarkMetadataTestExt {
        fn with_tracking(self, tracked: bool, tracking_present: bool, synced: bool) -> Self;
    }

    impl BookmarkMetadataTestExt for BookmarkMetadata {
        fn with_tracking(mut self, tracked: bool, tracking_present: bool, synced: bool) -> Self {
            self.tracked = tracked;
            self.tracking_present = tracking_present;
            self.synced = synced;
            self
        }
    }

    fn operation_id(digit: char) -> String {
        std::iter::repeat_n(digit, 128).collect()
    }
}
