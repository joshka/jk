//! Status-file action contracts.
//!
//! These are feature-owned action targets derived from selected `jj status` rows. Command-plan
//! execution still lives in the shared action planning modules after the status slice has chosen a
//! safe path target.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StatusFileAction {
    Track {
        path: String,
    },
    Tracked {
        path: String,
        restore_allowed: bool,
        chmod_allowed: bool,
    },
}

impl StatusFileAction {
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
