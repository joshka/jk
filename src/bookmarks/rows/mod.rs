//! Rendered bookmark rows plus trusted metadata pairing.
//!
//! This module owns the bookmark-specific bridge between rendered `jj bookmark
//! list` rows and the template-driven metadata used for exact bookmark names,
//! target ids, and local/remote classification. Pairing stays fail-closed: if
//! the metadata drifts or row counts do not match the rendered rows, items keep
//! their rendered text but degrade to unknown bookmark state and empty targets.

use std::collections::HashSet;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;
use serde_json::Value;

use crate::jj::{ColorMode, ViewSpec, run_jj, run_jj_template_lines};
use crate::rendered_rows::{
    boolean_field, line_text, non_empty_string_field, optional_string_field, string_field,
};

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

fn run_jj_bookmark_metadata(spec: &ViewSpec) -> Result<Vec<BookmarkMetadata>> {
    Ok(
        run_jj_template_lines(spec, BOOKMARK_METADATA_TEMPLATE, false)?
            .into_iter()
            .filter_map(|line| parse_bookmark_metadata_line(&line))
            .collect(),
    )
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
    UnfilteredAllRemotes,
}

fn bookmark_metadata_coverage(spec: &ViewSpec) -> BookmarkMetadataCoverage {
    if !spec.args().is_empty()
        && spec
            .args()
            .iter()
            .all(|arg| matches!(arg.as_str(), "--all-remotes" | "-a"))
    {
        BookmarkMetadataCoverage::UnfilteredAllRemotes
    } else {
        BookmarkMetadataCoverage::VisibleRowsOnly
    }
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
    } else if coverage == BookmarkMetadataCoverage::UnfilteredAllRemotes {
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
    } else if coverage == BookmarkMetadataCoverage::UnfilteredAllRemotes {
        BookmarkLocalPeerState::Absent
    } else {
        BookmarkLocalPeerState::Unknown
    }
}

fn bookmark_name_from_rendered_row(text: &str) -> String {
    text.split_once(':')
        .map(|(name, _)| name.trim())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| text.trim())
        .to_owned()
}

#[cfg(test)]
mod tests;
