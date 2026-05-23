//! Rendered `jj status` rows and selected-path action contracts.
//!
//! The status view keeps every rendered row visible, but only rows with one validated
//! repo-relative path become file-action targets. Ambiguous rows fail closed with an explanatory
//! status message.

use color_eyre::Result;
use ratatui::text::Line;

use super::actions::StatusFileAction;
use crate::documents::{DocumentLines, load_document};
use crate::jj::ViewSpec;

#[derive(Clone, Debug)]
pub struct StatusRow {
    /// Rendered status line preserved for display, search, and copy.
    line: Line<'static>,
    /// Exact-path action contract, disabled reason, or absence for this row.
    path: StatusPathContract,
}

impl StatusRow {
    /// Build one rendered status row plus its selected-path contract.
    fn new(line: Line<'static>, path: StatusPathContract) -> Self {
        Self { line, path }
    }

    /// Return the rendered line shown in the status view.
    pub fn line(&self) -> &Line<'static> {
        &self.line
    }

    #[cfg(test)]
    pub fn exact_path(&self) -> std::result::Result<&str, String> {
        match &self.path {
            StatusPathContract::Action(action) if action.restore_allowed() => Ok(action.path()),
            StatusPathContract::Action(_) => Err(
                "status file action unavailable: selected status row is not path-restorable"
                    .to_owned(),
            ),
            StatusPathContract::Disabled(message) => Err((*message).to_owned()),
            StatusPathContract::None => Err(
                "status file action unavailable: selected row has no exact file path".to_owned(),
            ),
        }
    }

    pub fn exact_path_option(&self) -> Option<&str> {
        match &self.path {
            StatusPathContract::Action(action) => Some(action.path()),
            StatusPathContract::Disabled(_) | StatusPathContract::None => None,
        }
    }

    /// Return the file-action target for this row, or the disabled reason when unavailable.
    pub fn file_action(&self) -> std::result::Result<StatusFileAction, String> {
        match &self.path {
            StatusPathContract::Action(action) => Ok(action.clone()),
            StatusPathContract::Disabled(message) => Err((*message).to_owned()),
            StatusPathContract::None => Err(
                "status file action unavailable: selected row has no exact file path".to_owned(),
            ),
        }
    }

    /// Return the rendered row text for selection-restore fallback.
    pub fn row_text(&self) -> String {
        line_text(&self.line)
    }
}

/// Exact-path contract derived from one rendered status row.
#[derive(Clone, Debug)]
enum StatusPathContract {
    /// Row safely identifies one file-action target.
    Action(StatusFileAction),
    /// Row looks path-like but must fail closed with a specific reason.
    Disabled(&'static str),
    /// Row does not represent a file path target.
    None,
}

/// Load rendered status rows and their file-action contracts for one `ViewSpec`.
pub fn load_status_rows(spec: &ViewSpec) -> Result<Vec<StatusRow>> {
    Ok(status_rows_from_document(load_document(spec)?))
}

/// Convert a rendered document into feature-owned status rows.
fn status_rows_from_document(document: DocumentLines) -> Vec<StatusRow> {
    document
        .lines()
        .iter()
        .cloned()
        .map(parse_status_row)
        .collect()
}

/// Parse one rendered line into a status row plus file-action contract.
pub fn parse_status_row(line: Line<'static>) -> StatusRow {
    let text = line_text(&line);
    let path = parse_status_path_contract(&text);
    StatusRow::new(line, path)
}

/// Classify one rendered status line as an actionable path, disabled path, or non-path row.
fn parse_status_path_contract(text: &str) -> StatusPathContract {
    let Some(separator_index) = text.find(char::is_whitespace) else {
        return StatusPathContract::None;
    };
    let status = &text[..separator_index];
    if status.is_empty()
        || status.len() > 2
        || status.chars().any(|character| !is_status_code(character))
    {
        return StatusPathContract::None;
    }
    let separator_and_path = &text[separator_index..];
    let Some(path) = separator_and_path.strip_prefix(' ') else {
        return StatusPathContract::Disabled(
            "status file action unavailable: selected path separator is ambiguous",
        );
    };
    if path.is_empty() {
        return StatusPathContract::Disabled(
            "status file action unavailable: selected status row has no file path",
        );
    }
    if status == "R" {
        return StatusPathContract::Disabled(
            "status file action unavailable: renamed status rows contain multiple paths",
        );
    }
    if status.contains('U') {
        return StatusPathContract::Disabled(
            "status file action unavailable: conflicted status rows are not file hygiene targets",
        );
    }

    if let Err(message) = validate_repo_relative_path(path) {
        return StatusPathContract::Disabled(message);
    }

    match status {
        "?" => StatusPathContract::Action(StatusFileAction::Track {
            path: path.to_owned(),
        }),
        "M" | "A" => StatusPathContract::Action(StatusFileAction::Tracked {
            path: path.to_owned(),
            restore_allowed: true,
            chmod_allowed: true,
        }),
        "D" => StatusPathContract::Action(StatusFileAction::Tracked {
            path: path.to_owned(),
            restore_allowed: true,
            chmod_allowed: false,
        }),
        "!" => StatusPathContract::Action(StatusFileAction::Tracked {
            path: path.to_owned(),
            restore_allowed: false,
            chmod_allowed: false,
        }),
        _ => StatusPathContract::Disabled(
            "status file action unavailable: selected status kind is not a file hygiene target",
        ),
    }
}

/// Return whether a character may appear in the short `jj status` kind prefix.
fn is_status_code(character: char) -> bool {
    character.is_ascii_alphabetic() || matches!(character, '?' | '!')
}

/// Validate that a rendered path is an exact, clean repo-relative path target.
fn validate_repo_relative_path(path: &str) -> std::result::Result<(), &'static str> {
    if path.trim() != path {
        return Err(
            "status file action unavailable: selected path has ambiguous surrounding whitespace",
        );
    }
    if path.starts_with('/') {
        return Err("status file action unavailable: selected path is absolute");
    }
    if path.contains('\0') || path.contains('\n') {
        return Err("status file action unavailable: selected path contains invalid control text");
    }
    if path.contains(" => ") || (path.starts_with('{') && path.ends_with('}')) {
        return Err(
            "status file action unavailable: selected row appears to contain multiple paths",
        );
    }
    if path
        .split('/')
        .any(|component| matches!(component, "" | "." | ".."))
    {
        return Err(
            "status file action unavailable: selected path is not a clean repo-relative path",
        );
    }
    Ok(())
}

/// Flatten a rendered line into plain text for parsing and selection restore.
fn line_text(line: &Line<'_>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}
