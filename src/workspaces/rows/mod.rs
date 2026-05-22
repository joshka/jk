//! Workspace row loading and metadata pairing.
//!
//! This root owns the read-only `jj root` / `jj workspace list` loading path
//! plus the rendered row and context types exposed to the rest of the
//! `workspaces` feature. The metadata side channel and its fail-closed pairing
//! rules live in `metadata.rs` so exact-schema parsing does not crowd the
//! higher-level row/context contract.

mod metadata;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use ratatui::text::Line;

use crate::jj::{ColorMode, ViewSpec, run_jj};
use crate::rendered_rows::line_text;

pub(crate) use self::metadata::WORKSPACE_METADATA_TEMPLATE;
use self::metadata::{pair_workspace_lines, run_workspace_metadata};

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
