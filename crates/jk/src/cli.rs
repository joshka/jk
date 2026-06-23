use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

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
    /// Resolves an omitted repository argument when started from a no-working-copy repo container.
    pub(crate) fn resolve_container_repository(&mut self) {
        if self.repository.is_some() {
            return;
        }

        self.repository = resolve_container_repository();
    }

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

/// Resolves a no-working-copy repo container to a child workspace repository.
fn resolve_container_repository() -> Option<PathBuf> {
    if !current_workspace_has_no_working_copy() {
        return None;
    }

    let repo_root = jj_output(&["root"])?;
    let repo_root = PathBuf::from(repo_root.trim());
    let workspaces = workspace_candidates()?;

    select_workspace_root(&repo_root, &workspaces)
}

/// Checks whether jj sees the current directory as a repo without a working-copy commit.
fn current_workspace_has_no_working_copy() -> bool {
    let Some(status) = jj_output(&["status"]) else {
        return false;
    };

    status.lines().any(|line| line.trim() == "No working copy")
}

/// Runs a read-only jj query used before the TUI chooses its repository target.
fn jj_output(args: &[&str]) -> Option<String> {
    let output = ProcessCommand::new("jj")
        .args(["--ignore-working-copy", "--no-pager", "--color", "never"])
        .args(args)
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Loads known jj workspace names and roots.
fn workspace_candidates() -> Option<Vec<WorkspaceCandidate>> {
    let output = jj_output(&[
        "workspace",
        "list",
        "-T",
        r#"self.name() ++ "\t" ++ self.root() ++ "\n""#,
    ])?;

    Some(parse_workspace_candidates(&output))
}

/// Parses `jj workspace list` rows produced by [`workspace_candidates`].
fn parse_workspace_candidates(output: &str) -> Vec<WorkspaceCandidate> {
    output
        .lines()
        .filter_map(|line| {
            let (name, root) = line.split_once('\t')?;
            Some(WorkspaceCandidate {
                name: name.to_owned(),
                root: PathBuf::from(root),
            })
        })
        .collect()
}

/// Selects the child workspace that should stand in for a repo container.
fn select_workspace_root(repo_root: &Path, workspaces: &[WorkspaceCandidate]) -> Option<PathBuf> {
    workspaces
        .iter()
        .filter(|workspace| workspace.root != repo_root && workspace.root.starts_with(repo_root))
        .find(|workspace| workspace.name == "default")
        .or_else(|| {
            workspaces
                .iter()
                .filter(|workspace| {
                    workspace.root != repo_root && workspace.root.starts_with(repo_root)
                })
                .min_by(|left, right| left.name.cmp(&right.name))
        })
        .map(|workspace| workspace.root.clone())
}

/// Workspace row returned by `jj workspace list`.
#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceCandidate {
    name: String,
    root: PathBuf,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_workspace_candidates_from_jj_template_rows() {
        let candidates = parse_workspace_candidates(
            "/bad row\n\
             cli\t/Users/joshka/local/jk/cli\n\
             default\t/Users/joshka/local/jk/default\n",
        );

        assert_eq!(
            candidates,
            vec![
                WorkspaceCandidate {
                    name: "cli".to_owned(),
                    root: PathBuf::from("/Users/joshka/local/jk/cli"),
                },
                WorkspaceCandidate {
                    name: "default".to_owned(),
                    root: PathBuf::from("/Users/joshka/local/jk/default"),
                },
            ]
        );
    }

    #[test]
    fn selects_default_child_workspace_for_container_root() {
        let repo_root = Path::new("/Users/joshka/local/jk");
        let workspaces = vec![
            WorkspaceCandidate {
                name: "cli".to_owned(),
                root: repo_root.join("cli"),
            },
            WorkspaceCandidate {
                name: "default".to_owned(),
                root: repo_root.join("default"),
            },
        ];

        let selected = select_workspace_root(repo_root, &workspaces);

        assert_eq!(selected, Some(repo_root.join("default")));
    }

    #[test]
    fn ignores_container_root_workspace_when_selecting_child() {
        let repo_root = Path::new("/Users/joshka/local/jk");
        let workspaces = vec![
            WorkspaceCandidate {
                name: "container".to_owned(),
                root: repo_root.to_path_buf(),
            },
            WorkspaceCandidate {
                name: "spike".to_owned(),
                root: repo_root.join("spike"),
            },
        ];

        let selected = select_workspace_root(repo_root, &workspaces);

        assert_eq!(selected, Some(repo_root.join("spike")));
    }

    #[test]
    fn does_not_select_workspace_outside_container_root() {
        let repo_root = Path::new("/Users/joshka/local/jk");
        let workspaces = vec![WorkspaceCandidate {
            name: "other".to_owned(),
            root: PathBuf::from("/Users/joshka/local/jk-other"),
        }];

        let selected = select_workspace_root(repo_root, &workspaces);

        assert_eq!(selected, None);
    }
}

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
