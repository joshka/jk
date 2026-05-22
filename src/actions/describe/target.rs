use crate::jj::exact_change_id_revset;

/// Target for non-interactive `jj describe --message` finalization.
///
/// Exact changes come from rendered row metadata and are quoted before argv
/// construction; the working-copy target stays as jj's `@` revset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JjDescribeTarget {
    /// One exact selected revision from rendered metadata.
    ExactChange(String),
    /// The current working-copy revset (`@`).
    CurrentWorkingCopy,
}

impl JjDescribeTarget {
    /// Target an exact rendered change id, quoted during argv construction.
    pub fn exact_change(change_id: impl Into<String>) -> Self {
        Self::ExactChange(change_id.into())
    }

    /// Target jj's current working-copy revset (`@`) without exact-change quoting.
    pub fn current_working_copy() -> Self {
        Self::CurrentWorkingCopy
    }

    /// Returns the user-facing target label used in preview and command labels.
    pub fn label(&self) -> &str {
        match self {
            Self::ExactChange(change_id) => change_id,
            Self::CurrentWorkingCopy => "@",
        }
    }

    /// Expose the exact change id only when the target came from rendered row metadata.
    pub fn exact_change_id(&self) -> Option<&str> {
        match self {
            Self::ExactChange(change_id) => Some(change_id),
            Self::CurrentWorkingCopy => None,
        }
    }

    /// Returns the revset argv fragment for this target.
    pub fn command_arg(&self) -> String {
        match self {
            Self::ExactChange(change_id) => exact_change_id_revset(change_id),
            Self::CurrentWorkingCopy => "@".to_owned(),
        }
    }

    /// Returns user-facing preview wording for this target.
    pub fn preview_target(&self) -> String {
        match self {
            Self::ExactChange(change_id) => format!("exact selected revision {change_id}"),
            Self::CurrentWorkingCopy => "current working-copy change (@)".to_owned(),
        }
    }
}
