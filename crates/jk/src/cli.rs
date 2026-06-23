use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use jk_cli::{
    DiffFormat, DiffQuery, JjAbandon, JjDescribe, JjDiff, JjEdit, JjEvolog, JjLog, JjLogCommand,
    JjNew, JjOperation, JjRecovery, JjShow, JjStatus, JjWorkspaces, LogTemplateSelection,
    ShowQuery, StatusQuery,
};

/// Command-line options for the first log-oriented `jk` surface.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// Repository path to pass to jj.
    #[arg(short = 'R', long = "repository")]
    pub(crate) repository: Option<PathBuf>,

    /// Maximum number of log entries to render for the default command.
    #[arg(short = 'n', long)]
    pub(crate) limit: Option<usize>,

    /// View to open. If omitted, jk follows jj's configured default command.
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

impl Args {
    /// Builds the log source that matches the requested command-line view.
    ///
    /// Bare `jk` intentionally starts from jj's configured default command, while `jk log` forces
    /// the explicit log command. The top-level limit applies to both forms unless the subcommand
    /// provides a narrower value.
    pub(crate) fn log_source(&self) -> JjLog {
        let (command, limit, template) = match &self.command {
            Some(Command::Log(log_args)) => (
                JjLogCommand::Log,
                log_args.limit.or(self.limit),
                log_args.template.clone().map_or(
                    LogTemplateSelection::Configured,
                    LogTemplateSelection::Custom,
                ),
            ),
            Some(
                Command::Diff(_) | Command::Show(_) | Command::Status(_) | Command::Workspaces,
            )
            | None => (
                JjLogCommand::ConfiguredDefault,
                self.limit,
                LogTemplateSelection::Configured,
            ),
        };

        let source = JjLog::default()
            .with_command(command)
            .with_limit(limit)
            .with_template(template);
        self.with_repository(source)
    }

    /// Builds the diff source for selected-change inspection.
    pub(crate) fn diff_source(&self) -> JjDiff {
        self.with_repository(JjDiff::default())
    }

    /// Builds the show source for selected-change inspection.
    pub(crate) fn show_source(&self) -> JjShow {
        self.with_repository(JjShow::default())
    }

    /// Builds the evolog source for selected-change inspection.
    pub(crate) fn evolog_source(&self) -> JjEvolog {
        self.with_repository(JjEvolog::default())
    }

    /// Builds the status source for repository inspection.
    pub(crate) fn status_source(&self) -> JjStatus {
        self.with_repository(JjStatus::default())
    }

    /// Builds the describe source for selected-change mutation preview.
    pub(crate) fn describe_source(&self) -> JjDescribe {
        self.with_repository(JjDescribe::default())
    }

    /// Builds the abandon source for selected-change mutation preview.
    pub(crate) fn abandon_source(&self) -> JjAbandon {
        self.with_repository(JjAbandon::default())
    }

    /// Builds the new source for selected-change mutation preview.
    pub(crate) fn new_source(&self) -> JjNew {
        self.with_repository(JjNew::default())
    }

    /// Builds the edit source for selected-change mutation preview.
    pub(crate) fn edit_source(&self) -> JjEdit {
        self.with_repository(JjEdit::default())
    }

    /// Builds the operation source for operation log/show/diff inspection.
    pub(crate) fn operation_source(&self) -> JjOperation {
        self.with_repository(JjOperation::default())
    }

    /// Builds the recovery source for undo/redo mutation previews.
    pub(crate) fn recovery_source(&self) -> JjRecovery {
        self.with_repository(JjRecovery::default())
    }

    /// Builds the workspace source for workspace list and selected-workspace inspection.
    pub(crate) fn workspaces_source(&self) -> JjWorkspaces {
        self.with_repository(JjWorkspaces::default())
    }

    fn with_repository<T>(&self, source: T) -> T
    where
        T: WithRepository,
    {
        if let Some(repository) = self.repository.as_deref() {
            source.with_repository(repository)
        } else {
            source
        }
    }
}

trait WithRepository: Sized {
    fn with_repository(self, repository: &Path) -> Self;
}

macro_rules! impl_with_repository {
    ($($source:ty),+ $(,)?) => {
        $(
            impl WithRepository for $source {
                fn with_repository(self, repository: &Path) -> Self {
                    <$source>::with_repository(self, repository)
                }
            }
        )+
    };
}

impl_with_repository!(
    JjAbandon,
    JjDescribe,
    JjDiff,
    JjEdit,
    JjEvolog,
    JjLog,
    JjNew,
    JjOperation,
    JjRecovery,
    JjShow,
    JjStatus,
    JjWorkspaces,
);

/// Top-level view commands supported by the binary.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show the jj log view.
    Log(LogArgs),

    /// Show the jj diff view for a revision.
    Diff(DiffArgs),

    /// Show one or more revisions.
    Show(ShowArgs),

    /// Show repository status.
    Status(StatusArgs),

    /// Show jj workspaces.
    Workspaces,
}

/// Options for the explicit `jk log` command.
#[derive(Debug, Parser)]
pub struct LogArgs {
    /// Maximum number of log entries to render.
    #[arg(short = 'n', long)]
    limit: Option<usize>,

    /// Rendered jj log template to pass to the explicit log command.
    #[arg(short = 'T', long = "template", value_name = "TEMPLATE")]
    pub(crate) template: Option<String>,
}

/// Options for the explicit `jk diff` command.
#[derive(Debug, Parser)]
pub struct DiffArgs {
    /// Compatibility sugar for `jk diff -r REV`.
    #[arg(value_name = "REV", conflicts_with_all = ["revision", "from", "to"])]
    compatibility_revision: Option<String>,

    /// Revision to diff against its parent.
    #[arg(short = 'r', long = "revision", value_name = "REV", conflicts_with_all = ["from", "to"])]
    revision: Option<String>,

    /// Starting revision for a two-revision diff.
    #[arg(
        long,
        value_name = "FROM",
        requires = "to",
        conflicts_with = "revision"
    )]
    from: Option<String>,

    /// Ending revision for a two-revision diff.
    #[arg(
        long,
        value_name = "TO",
        requires = "from",
        conflicts_with = "revision"
    )]
    to: Option<String>,

    /// Render `jj diff --stat`.
    #[arg(long, conflicts_with_all = ["summary", "types", "name_only", "git", "color_words"])]
    stat: bool,

    /// Render `jj diff --summary`.
    #[arg(short = 's', long, conflicts_with_all = ["stat", "types", "name_only", "git", "color_words"])]
    summary: bool,

    /// Render `jj diff --types`.
    #[arg(long, conflicts_with_all = ["stat", "summary", "name_only", "git", "color_words"])]
    types: bool,

    /// Render `jj diff --name-only`.
    #[arg(long, conflicts_with_all = ["stat", "summary", "types", "git", "color_words"])]
    name_only: bool,

    /// Render `jj diff --git`.
    #[arg(long, conflicts_with_all = ["stat", "summary", "types", "name_only", "color_words"])]
    git: bool,

    /// Render `jj diff --color-words`.
    #[arg(long, conflicts_with_all = ["stat", "summary", "types", "name_only", "git"])]
    color_words: bool,
}

impl DiffArgs {
    pub(crate) fn query(&self) -> DiffQuery {
        let format = self.format();

        if let (Some(from), Some(to)) = (&self.from, &self.to) {
            return DiffQuery::FromTo {
                from: from.clone(),
                to: to.clone(),
                format,
            };
        }

        let rev = self
            .revision
            .as_ref()
            .or(self.compatibility_revision.as_ref())
            .cloned()
            .unwrap_or_else(|| "@".to_owned());
        DiffQuery::Revision { rev, format }
    }

    const fn format(&self) -> DiffFormat {
        if self.summary {
            DiffFormat::Summary
        } else if self.stat {
            DiffFormat::Stat
        } else if self.types {
            DiffFormat::Types
        } else if self.name_only {
            DiffFormat::NameOnly
        } else if self.git {
            DiffFormat::Git
        } else if self.color_words {
            DiffFormat::ColorWords
        } else {
            DiffFormat::Patch
        }
    }
}

/// Options for the explicit `jk show` command.
#[derive(Debug, Parser)]
pub struct ShowArgs {
    /// Revisions to show.
    #[arg(value_name = "REV", required = true)]
    revs: Vec<String>,
}

impl ShowArgs {
    pub(crate) fn query(&self) -> ShowQuery {
        ShowQuery::new(self.revs.clone())
    }
}

/// Options for the explicit `jk status` command.
#[derive(Debug, Parser)]
pub struct StatusArgs {
    /// Filesets to pass to jj status.
    #[arg(value_name = "FILESET")]
    filesets: Vec<String>,
}

impl StatusArgs {
    pub(crate) fn query(&self) -> StatusQuery {
        StatusQuery::new(self.filesets.clone())
    }
}
