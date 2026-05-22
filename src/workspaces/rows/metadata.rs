//! Workspace metadata transport and fail-closed pairing.
//!
//! `jk` treats rendered workspace rows as opaque. Exact workspace names and
//! target ids come from this side channel only when the machine-oriented
//! template still matches the expected schema and row count.

use color_eyre::Result;
use ratatui::text::Line;
use serde_json::Value;

use crate::jj::{ViewSpec, run_jj_template_lines};
use crate::rendered_rows::{non_empty_string_field, string_field};

use super::WorkspaceItem;

pub const WORKSPACE_METADATA_TEMPLATE: &str = concat!(
    r#""{\"name\":" ++ json(name)"#,
    r#" ++ ",\"target_change_id\":" ++ json(target.change_id())"#,
    r#" ++ ",\"target_commit_id\":" ++ json(target.commit_id())"#,
    r#" ++ "}\n""#,
);

/// Loads workspace metadata rows through the workspace-specific template side channel.
pub fn run_workspace_metadata(spec: &ViewSpec) -> Result<Vec<WorkspaceMetadata>> {
    parse_workspace_metadata_lines(run_jj_template_lines(
        spec,
        WORKSPACE_METADATA_TEMPLATE,
        false,
    )?)
}

/// Pairs rendered workspace rows with metadata and degrades safely when parsing drifts.
pub fn pair_workspace_lines(
    lines: Vec<Line<'static>>,
    metadata: Result<Vec<WorkspaceMetadata>, String>,
) -> (Vec<WorkspaceItem>, Option<String>) {
    let rendered_count = lines.len();
    let metadata = match metadata {
        Ok(metadata) if metadata.len() == rendered_count => metadata,
        Ok(metadata) => {
            let metadata_count = metadata.len();
            let entries = lines
                .into_iter()
                .map(|line| WorkspaceItem::new(vec![line], None, None, None))
                .collect();
            return (
                entries,
                Some(format!(
                    "workspace metadata row-count mismatch: rendered {} rows, metadata {} rows",
                    rendered_count, metadata_count
                )),
            );
        }
        Err(error) => {
            let entries = lines
                .into_iter()
                .map(|line| WorkspaceItem::new(vec![line], None, None, None))
                .collect();
            return (entries, Some(error));
        }
    };

    let entries = lines
        .into_iter()
        .zip(metadata)
        .map(|(line, metadata)| {
            WorkspaceItem::new(
                vec![line],
                Some(metadata.name),
                metadata.target_change_id,
                metadata.target_commit_id,
            )
        })
        .collect();
    (entries, None)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMetadata {
    /// Exact workspace name from metadata.
    name: String,
    /// Exact target change id when present.
    target_change_id: Option<String>,
    /// Exact target commit id when present.
    target_commit_id: Option<String>,
}

/// Parses metadata lines and reports the first malformed row as an error.
fn parse_workspace_metadata_lines(lines: Vec<String>) -> Result<Vec<WorkspaceMetadata>> {
    let mut metadata = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some(row) = parse_workspace_metadata_line(&line) else {
            return Err(color_eyre::eyre::eyre!(
                "workspace metadata parse failed for line: {line}"
            ));
        };
        metadata.push(row);
    }
    Ok(metadata)
}

/// Parses one metadata line only when it matches the exact workspace schema.
fn parse_workspace_metadata_line(line: &str) -> Option<WorkspaceMetadata> {
    let Value::Object(fields) = serde_json::from_str::<Value>(line).ok()? else {
        return None;
    };

    let name = string_field(&fields, "name")?;
    if name.is_empty() {
        return None;
    }

    Some(WorkspaceMetadata {
        name,
        target_change_id: non_empty_string_field(&fields, "target_change_id"),
        target_commit_id: non_empty_string_field(&fields, "target_commit_id"),
    })
}

#[cfg(test)]
mod tests {
    use ansi_to_tui::IntoText as _;

    use super::*;

    #[test]
    fn parses_workspace_metadata_lines() {
        let metadata = parse_workspace_metadata_lines(vec![
            r#"{"name":"default","target_change_id":"znpvoytkxsukywrolvkxnsnlpzypvmry","target_commit_id":"419cab141b2a748d4e7d91f0322a0dd499b57669","future":"ignored"}"#.to_owned(),
        ])
        .unwrap();

        assert_eq!(
            metadata,
            vec![WorkspaceMetadata {
                name: "default".to_owned(),
                target_change_id: Some("znpvoytkxsukywrolvkxnsnlpzypvmry".to_owned()),
                target_commit_id: Some("419cab141b2a748d4e7d91f0322a0dd499b57669".to_owned()),
            }]
        );
        assert!(parse_workspace_metadata_lines(vec!["{not json".to_owned()]).is_err());
        assert!(
            parse_workspace_metadata_lines(vec![
                r#"{"name":"","target_change_id":"x"}"#.to_owned()
            ])
            .is_err()
        );
    }

    #[test]
    fn pairs_workspace_rows_without_parsing_rendered_labels() {
        let lines = b"display name: abc123 description\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;
        let metadata = vec![WorkspaceMetadata {
            name: "exact-name".to_owned(),
            target_change_id: Some("change-id".to_owned()),
            target_commit_id: Some("commit-id".to_owned()),
        }];

        let (items, error) = pair_workspace_lines(lines, Ok(metadata));

        assert_eq!(error, None);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name(), Some("exact-name"));
        assert_eq!(items[0].target_change_id(), Some("change-id"));
        assert_eq!(items[0].target_commit_id(), Some("commit-id"));
        assert_eq!(items[0].row_text(), "display name: abc123 description");
    }

    #[test]
    fn workspace_rows_degrade_when_metadata_is_malformed() {
        let lines = b"default: rendered\n".to_vec().into_text().unwrap().lines;

        let (items, error) = pair_workspace_lines(
            lines,
            Err("workspace metadata parse failed for line: {not json".to_owned()),
        );

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name(), None);
        assert_eq!(items[0].row_text(), "default: rendered");
        assert_eq!(
            error.as_deref(),
            Some("workspace metadata parse failed for line: {not json")
        );
    }

    #[test]
    fn workspace_rows_degrade_on_row_count_mismatch() {
        let lines = b"default: rendered\nother: rendered\n"
            .to_vec()
            .into_text()
            .unwrap()
            .lines;

        let (items, error) = pair_workspace_lines(
            lines,
            Ok(vec![WorkspaceMetadata {
                name: "default".to_owned(),
                target_change_id: None,
                target_commit_id: None,
            }]),
        );

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.name().is_none()));
        assert_eq!(
            error.as_deref(),
            Some("workspace metadata row-count mismatch: rendered 2 rows, metadata 1 rows")
        );
    }
}
