//! Rendered log revision row loading and metadata pairing.
//!
//! This module owns the revision-specific row contract: rendered `jj log` rows
//! are preserved as styled lines, while change and commit ids come from a
//! narrow metadata template and are paired only when row counts still match.

mod metadata;
mod pairing;

use ansi_to_tui::IntoText as _;
use color_eyre::Result;
use metadata::run_jj_with_template;
use pairing::group_lines;
use ratatui::text::Line;

use crate::jj::{self, ColorMode, ViewSpec, run_jj};
use crate::rendered_rows::{RowMetadata, first_content_char, line_text};

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
    let spec = ViewSpec::new(jj::Command::Log, vec!["-r".to_owned(), revset.to_owned()]);
    let output = run_jj(&spec, ColorMode::Always)?;
    let lines = output.stdout.into_text()?.lines;

    Ok(group_lines(lines, RowMetadata::Valid(Vec::new()))
        .into_iter()
        .next()
        .map(|item| item.lines().into_iter().take(2).collect())
        .unwrap_or_default())
}

#[cfg(test)]
mod tests;
