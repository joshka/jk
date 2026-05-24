use crate::jj::command::{ALL_REPO_REVSET, RECENT_WORK_REVSET, TRUNK_WORK_REVSET, option_value};
use crate::jj::{Command, ViewSpec};

/// Named log revset modes that cycle within the default/log surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogViewMode {
    /// Home log mode used by the default startup surface.
    Default,
    /// Work relative to `trunk()`.
    Trunk,
    /// Recent mutable work plus the working copy and trunk.
    Recent,
    /// Repository-wide overview.
    All,
    /// User-provided explicit revset carried through the log view state.
    CustomRevset(String),
}

impl LogViewMode {
    pub fn label(&self) -> &str {
        match self {
            Self::Default => "default work",
            Self::Trunk => "trunk work",
            Self::Recent => "recent work",
            Self::All => "repo overview",
            Self::CustomRevset(_) => "custom revset",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Default => Self::Trunk,
            Self::Trunk => Self::Recent,
            Self::Recent => Self::All,
            Self::All | Self::CustomRevset(_) => Self::Default,
        }
    }

    pub fn from_spec(spec: &ViewSpec) -> Self {
        if spec.command() == Command::Default {
            return Self::Default;
        }

        revset_from_log_args(spec.args())
            .map(Self::from_revset)
            .unwrap_or(Self::Default)
    }

    fn from_revset(revset: &str) -> Self {
        match revset {
            TRUNK_WORK_REVSET => Self::Trunk,
            RECENT_WORK_REVSET => Self::Recent,
            ALL_REPO_REVSET => Self::All,
            _ => Self::CustomRevset(revset.to_owned()),
        }
    }

    pub fn args(&self) -> Vec<String> {
        match self {
            Self::Default => Vec::new(),
            Self::Trunk => vec!["-r".to_owned(), TRUNK_WORK_REVSET.to_owned()],
            Self::Recent => vec!["-r".to_owned(), RECENT_WORK_REVSET.to_owned()],
            Self::All => vec!["-r".to_owned(), ALL_REPO_REVSET.to_owned()],
            Self::CustomRevset(revset) => vec!["-r".to_owned(), revset.clone()],
        }
    }
}

fn revset_from_log_args(args: &[String]) -> Option<&str> {
    option_value(args, &["-r", "--revisions"], &["--revisions="])
}
