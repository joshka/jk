//! `jj rebase` mutation command integration.

use std::fmt;

use jk_core::{GlobalOptions, JjCommandSpec, RefreshPlan, SafetyClass};

const REBASE_COMMAND: &str = "rebase";

/// Which revisions a rebase moves.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RebaseSourceRole {
    /// Move a branch relative to the destination.
    Branch,
    /// Move a revision and all of its descendants.
    Source,
    /// Move only the selected revision.
    Revision,
}

impl RebaseSourceRole {
    /// Returns the canonical short `jj rebase` flag.
    #[must_use]
    pub const fn flag(self) -> &'static str {
        match self {
            Self::Branch => "-b",
            Self::Source => "-s",
            Self::Revision => "-r",
        }
    }

    /// Returns the role name shown in selection UI.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Branch => "branch",
            Self::Source => "source + descendants",
            Self::Revision => "revision only",
        }
    }
}

/// Where a rebase places the moved revisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RebaseDestinationRole {
    /// Move the revisions onto a new parent.
    Onto,
    /// Insert the revisions after the destination.
    InsertAfter,
    /// Insert the revisions before the destination.
    InsertBefore,
}

impl RebaseDestinationRole {
    /// Returns the canonical short `jj rebase` flag.
    #[must_use]
    pub const fn flag(self) -> &'static str {
        match self {
            Self::Onto => "-o",
            Self::InsertAfter => "-A",
            Self::InsertBefore => "-B",
        }
    }

    /// Returns the role name shown in selection UI.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Onto => "onto",
            Self::InsertAfter => "insert after",
            Self::InsertBefore => "insert before",
        }
    }
}

/// A validated rebase role assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebaseQuery {
    source_role: RebaseSourceRole,
    sources: Vec<String>,
    destination_role: RebaseDestinationRole,
    destination: String,
}

impl RebaseQuery {
    /// Creates a rebase query after checking that every required role is distinct and present.
    ///
    /// # Errors
    ///
    /// Returns [`RebaseQueryError`] when a source or destination is missing, empty, or duplicated
    /// across the two roles.
    pub fn new(
        source_role: RebaseSourceRole,
        sources: impl IntoIterator<Item = impl Into<String>>,
        destination_role: RebaseDestinationRole,
        destination: impl Into<String>,
    ) -> Result<Self, RebaseQueryError> {
        let sources = sources.into_iter().map(Into::into).collect::<Vec<_>>();
        if sources.is_empty() {
            return Err(RebaseQueryError::MissingSource);
        }
        if sources.iter().any(String::is_empty) {
            return Err(RebaseQueryError::EmptySource);
        }

        let destination = destination.into();
        if destination.is_empty() {
            return Err(RebaseQueryError::MissingDestination);
        }
        if sources.iter().any(|source| source == &destination) {
            return Err(RebaseQueryError::DestinationIsSource(destination));
        }

        Ok(Self {
            source_role,
            sources,
            destination_role,
            destination,
        })
    }

    /// Returns the selected source role.
    #[must_use]
    pub const fn source_role(&self) -> RebaseSourceRole {
        self.source_role
    }

    /// Returns source revisions in their selected order.
    #[must_use]
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Returns the selected destination role.
    #[must_use]
    pub const fn destination_role(&self) -> RebaseDestinationRole {
        self.destination_role
    }

    /// Returns the destination revision.
    #[must_use]
    pub fn destination(&self) -> &str {
        &self.destination
    }
}

/// Invalid role assignment rejected before command preview.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RebaseQueryError {
    /// No source revisions were selected.
    MissingSource,
    /// A source revision was empty.
    EmptySource,
    /// No destination revision was selected.
    MissingDestination,
    /// The selected destination was also a source.
    DestinationIsSource(String),
}

impl fmt::Display for RebaseQueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSource => formatter.write_str("rebase needs a source revision"),
            Self::EmptySource => formatter.write_str("rebase source revisions cannot be empty"),
            Self::MissingDestination => formatter.write_str("rebase needs a destination revision"),
            Self::DestinationIsSource(revision) => {
                write!(formatter, "destination {revision} is also a rebase source")
            }
        }
    }
}

impl std::error::Error for RebaseQueryError {}

/// Builds typed `jj rebase` mutation specs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JjRebase {
    global_options: GlobalOptions,
}

impl JjRebase {
    /// Sets the repository path passed to `jj --repository`.
    #[must_use]
    pub fn with_repository(mut self, repository: impl Into<std::path::PathBuf>) -> Self {
        self.global_options = self.global_options.with_repository(repository);
        self
    }

    /// Returns the command spec for `query`.
    #[must_use]
    pub fn spec_for(&self, query: &RebaseQuery) -> JjCommandSpec {
        let mut argv = Vec::with_capacity(query.sources.len() * 2 + 3);
        argv.push(REBASE_COMMAND.to_owned());
        for source in &query.sources {
            argv.push(query.source_role.flag().to_owned());
            argv.push(source.clone());
        }
        argv.push(query.destination_role.flag().to_owned());
        argv.push(query.destination.clone());

        JjCommandSpec::confirm_mutation(argv, SafetyClass::LocalRewrite)
            .with_global_options(self.global_options.clone())
            .with_title("Rebase revisions")
            .with_refresh_plan(RefreshPlan::None)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use jk_core::{ExecutionMode, RefreshPlan};

    use super::*;

    fn strings(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn branch_onto_builds_confirmed_local_rewrite() {
        let query = RebaseQuery::new(
            RebaseSourceRole::Branch,
            ["source"],
            RebaseDestinationRole::Onto,
            "destination",
        )
        .unwrap_or_else(|error| panic!("roles are distinct: {error}"));
        let spec = JjRebase::default().spec_for(&query);

        assert_eq!(
            strings(spec.argv()),
            ["rebase", "-b", "source", "-o", "destination"]
        );
        assert_eq!(spec.mode(), ExecutionMode::ConfirmMutation);
        assert_eq!(spec.safety(), SafetyClass::LocalRewrite);
        assert_eq!(spec.refresh_plan(), RefreshPlan::None);
    }

    #[test]
    fn repeated_sources_preserve_selection_order() {
        let query = RebaseQuery::new(
            RebaseSourceRole::Source,
            ["first", "second"],
            RebaseDestinationRole::InsertBefore,
            "destination",
        )
        .unwrap_or_else(|error| panic!("roles are distinct: {error}"));

        assert_eq!(
            strings(JjRebase::default().spec_for(&query).argv()),
            ["rebase", "-s", "first", "-s", "second", "-B", "destination",]
        );
    }

    #[test]
    fn destination_role_flags_are_explicit() {
        let after = RebaseQuery::new(
            RebaseSourceRole::Revision,
            ["source"],
            RebaseDestinationRole::InsertAfter,
            "destination",
        )
        .unwrap_or_else(|error| panic!("roles are distinct: {error}"));

        assert_eq!(
            strings(JjRebase::default().spec_for(&after).argv()),
            ["rebase", "-r", "source", "-A", "destination"]
        );
    }

    #[test]
    fn ambiguous_role_assignments_are_rejected() {
        assert_eq!(
            RebaseQuery::new(
                RebaseSourceRole::Branch,
                Vec::<String>::new(),
                RebaseDestinationRole::Onto,
                "destination",
            ),
            Err(RebaseQueryError::MissingSource)
        );
        assert_eq!(
            RebaseQuery::new(
                RebaseSourceRole::Branch,
                ["same"],
                RebaseDestinationRole::Onto,
                "same",
            ),
            Err(RebaseQueryError::DestinationIsSource("same".to_owned()))
        );
    }

    #[test]
    fn repository_renders_before_rebase() {
        let query = RebaseQuery::new(
            RebaseSourceRole::Branch,
            ["source"],
            RebaseDestinationRole::Onto,
            "destination",
        )
        .unwrap_or_else(|error| panic!("roles are distinct: {error}"));
        let argv = JjRebase::default()
            .with_repository("/tmp/repo")
            .spec_for(&query)
            .process_argv()
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(
            argv,
            [
                "--no-pager",
                "--color",
                "always",
                "--repository",
                "/tmp/repo",
                "rebase",
                "-b",
                "source",
                "-o",
                "destination",
            ]
        );
    }
}
