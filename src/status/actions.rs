//! Status-file action contracts.
//!
//! These are feature-owned action targets derived from selected `jj status` rows. Command-plan
//! execution still lives in the shared action planning modules after the status slice has chosen a
//! safe path target.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusFileAction {
    /// Untracked path that may be added to version control.
    Track {
        /// Exact repo-relative path selected from status output.
        path: String,
    },
    /// Tracked path that may support restore and/or chmod actions.
    Tracked {
        /// Exact repo-relative path selected from status output.
        path: String,
        /// Whether status semantics allow restore for this path.
        restore_allowed: bool,
        /// Whether status semantics allow chmod mutations for this path.
        chmod_allowed: bool,
    },
}

impl StatusFileAction {
    /// Return the exact repo-relative path selected from status output.
    pub fn path(&self) -> &str {
        match self {
            Self::Track { path }
            | Self::Tracked {
                path,
                restore_allowed: _,
                chmod_allowed: _,
            } => path,
        }
    }

    #[cfg(test)]
    pub fn restore_allowed(&self) -> bool {
        matches!(
            self,
            Self::Tracked {
                restore_allowed: true,
                ..
            }
        )
    }

    #[cfg(test)]
    pub fn chmod_allowed(&self) -> bool {
        matches!(
            self,
            Self::Tracked {
                chmod_allowed: true,
                ..
            }
        )
    }
}
