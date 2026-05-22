//! Workspace row loading and metadata pairing.
//!
//! This module owns the read-only `jj root` / `jj workspace list` loading path and the
//! machine-oriented metadata template used to attach workspace names and target ids to rendered
//! rows. Shared rendered-row helpers stay in `rows`; workspace-specific row/context policy lives
//! under this feature root.

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;
use serde_json::Value;

use crate::jj::{ColorMode, ViewSpec, run_jj, run_jj_template_lines};
use crate::rendered_rows::{line_text, non_empty_string_field, string_field};

pub(crate) const WORKSPACE_METADATA_TEMPLATE: &str = concat!(
    r#""{\"name\":" ++ json(name)"#,
    r#" ++ ",\"target_change_id\":" ++ json(target.change_id())"#,
    r#" ++ ",\"target_commit_id\":" ++ json(target.commit_id())"#,
    r#" ++ "}\n""#,
);

/// Read-only workspace/root context loaded from separate `jj` commands.
#[derive(Clone, Debug, Default)]
pub struct WorkspaceContext {
    /// Current repository root reported by `jj root`, when available.
    root: Option<String>,
    /// Error text from loading the current root, if `jj root` failed.
    root_error: Option<String>,
    /// Rendered workspace rows paired with trusted metadata when available.
    entries: Vec<WorkspaceItem>,
    /// Error text from `jj workspace list`, if the rendered list failed to load.
    list_error: Option<String>,
    /// Warning or error text from workspace metadata pairing or parsing.
    metadata_error: Option<String>,
}

impl WorkspaceContext {
    /// Builds the full read-only workspace/root context from its loaded parts.
    pub fn new(
        root: Option<String>,
        root_error: Option<String>,
        entries: Vec<WorkspaceItem>,
        list_error: Option<String>,
        metadata_error: Option<String>,
    ) -> Self {
        Self {
            root,
            root_error,
            entries,
            list_error,
            metadata_error,
        }
    }

    /// Returns the current repository root, if `jj root` succeeded.
    pub fn root(&self) -> Option<&str> {
        self.root.as_deref()
    }

    /// Returns the `jj root` error message, if root loading failed.
    pub fn root_error(&self) -> Option<&str> {
        self.root_error.as_deref()
    }

    /// Returns the rendered workspace rows for the current context.
    pub fn entries(&self) -> &[WorkspaceItem] {
        &self.entries
    }

    /// Returns the rendered workspace-list error, if the list failed to load.
    pub fn list_error(&self) -> Option<&str> {
        self.list_error.as_deref()
    }

    /// Returns the metadata parse or pairing warning, if metadata degraded.
    pub fn metadata_error(&self) -> Option<&str> {
        self.metadata_error.as_deref()
    }
}

/// One selectable row from rendered `jj workspace list` output.
#[derive(Clone, Debug)]
pub struct WorkspaceItem {
    /// Preserved rendered lines for one workspace row.
    lines: Vec<Line<'static>>,
    /// Exact workspace name when metadata still proves it.
    name: Option<String>,
    /// Exact target change id when metadata still proves it.
    target_change_id: Option<String>,
    /// Exact target commit id when metadata still proves it.
    target_commit_id: Option<String>,
}

impl WorkspaceItem {
    /// Builds one rendered workspace row and its trusted metadata fields.
    pub fn new(
        lines: Vec<Line<'static>>,
        name: Option<String>,
        target_change_id: Option<String>,
        target_commit_id: Option<String>,
    ) -> Self {
        Self {
            lines,
            name,
            target_change_id,
            target_commit_id,
        }
    }

    /// Returns the preserved rendered lines for this workspace row.
    pub fn lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    /// Returns the number of rendered lines in this workspace row.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the exact workspace name when metadata still proves it.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the exact target change id when metadata still proves it.
    pub fn target_change_id(&self) -> Option<&str> {
        self.target_change_id.as_deref()
    }

    /// Returns the exact target commit id when metadata still proves it.
    pub fn target_commit_id(&self) -> Option<&str> {
        self.target_commit_id.as_deref()
    }

    /// Returns plain rendered row text for copy and search surfaces.
    pub fn row_text(&self) -> String {
        self.lines
            .iter()
            .map(line_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Loads the current root, rendered workspace rows, and metadata side channel together.
pub fn load_workspace_context(spec: &ViewSpec) -> Result<WorkspaceContext> {
    let (root, root_error) = match crate::jj::load_workspace_root() {
        Ok(root) => (Some(root), None),
        Err(error) => (None, Some(error.to_string())),
    };

    let rendered_rows = match run_jj(spec, ColorMode::Always) {
        Ok(output) => Ok(output.stdout.into_text()?.lines),
        Err(error) => Err(error.to_string()),
    };
    let metadata_rows = run_workspace_metadata(spec).map_err(|error| error.to_string());

    let (entries, list_error, metadata_error) = match rendered_rows {
        Ok(lines) => {
            let (entries, metadata_error) = pair_workspace_lines(lines, metadata_rows);
            (entries, None, metadata_error)
        }
        Err(error) => (Vec::new(), Some(error), metadata_rows.err()),
    };

    Ok(WorkspaceContext::new(
        root,
        root_error,
        entries,
        list_error,
        metadata_error,
    ))
}

/// Loads workspace metadata rows through the workspace-specific template side channel.
fn run_workspace_metadata(spec: &ViewSpec) -> Result<Vec<WorkspaceMetadata>> {
    parse_workspace_metadata_lines(run_jj_template_lines(
        spec,
        WORKSPACE_METADATA_TEMPLATE,
        false,
    )?)
}

/// Pairs rendered workspace rows with metadata and degrades safely when parsing drifts.
fn pair_workspace_lines(
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
struct WorkspaceMetadata {
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
