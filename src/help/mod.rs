//! Generated help projection policy.
//!
//! This module decides which commands are shown in each view's help overlay and
//! how those commands are grouped. Key binding matching stays in `command.rs`;
//! help projection only consumes the binding vocabulary.

use crate::command::{Binding, Command, ViewCommand};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpContext {
    Log,
    Show,
    Diff,
    Status,
    Resolve,
    FileList,
    FileShow,
    Bookmarks,
    Workspaces,
    OperationLog,
    OperationDetail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpSectionKind {
    Navigation,
    Views,
    SearchCopy,
    RepositoryActions,
    Actions,
    Recovery,
    App,
}

impl HelpSectionKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Navigation => "Navigation",
            Self::Views => "View Switching",
            Self::SearchCopy => "Search / Copy",
            Self::RepositoryActions => "Repository Actions",
            Self::Actions => "Action Previews",
            Self::Recovery => "Recovery",
            Self::App => "App",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HelpRow {
    keys: String,
    action: &'static str,
}

impl HelpRow {
    pub fn new(keys: impl Into<String>, action: &'static str) -> Self {
        Self {
            keys: keys.into(),
            action,
        }
    }

    pub fn keys(&self) -> &str {
        &self.keys
    }

    pub fn action(&self) -> &str {
        self.action
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HelpSection {
    kind: HelpSectionKind,
    rows: Vec<HelpRow>,
}

impl HelpSection {
    pub fn new(kind: HelpSectionKind, rows: Vec<HelpRow>) -> Self {
        Self { kind, rows }
    }

    pub fn title(&self) -> &'static str {
        self.kind.title()
    }

    pub fn rows(&self) -> &[HelpRow] {
        &self.rows
    }
}

pub fn project_help(
    global_bindings: &[Binding],
    view_bindings: &[Binding],
    context: HelpContext,
) -> Vec<HelpSection> {
    let global_rows = collect_help_rows(global_bindings, context);
    let view_rows = collect_help_rows(view_bindings, context);

    [
        HelpSectionKind::Navigation,
        HelpSectionKind::Views,
        HelpSectionKind::SearchCopy,
        HelpSectionKind::RepositoryActions,
        HelpSectionKind::Actions,
        HelpSectionKind::Recovery,
        HelpSectionKind::App,
    ]
    .into_iter()
    .filter_map(|kind| {
        let rows = global_rows
            .iter()
            .chain(&view_rows)
            .filter(|(row_kind, _)| *row_kind == kind)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        (!rows.is_empty()).then(|| HelpSection::new(kind, rows))
    })
    .collect()
}

fn collect_help_rows(
    bindings: &[Binding],
    context: HelpContext,
) -> Vec<(HelpSectionKind, HelpRow)> {
    let mut rows: Vec<(HelpSectionKind, Command, HelpRow)> = Vec::new();

    for binding in bindings {
        let command = binding.command();
        let Some((kind, action)) = help_metadata(command, context) else {
            continue;
        };
        let key = binding.key_label();

        if let Some((_, _, row)) = rows.iter_mut().find(|(row_kind, row_command, row)| {
            *row_kind == kind && *row_command == command && row.action == action
        }) {
            row.keys.push_str(", ");
            row.keys.push_str(&key);
        } else {
            rows.push((kind, command, HelpRow::new(key, action)));
        }
    }

    rows.into_iter().map(|(kind, _, row)| (kind, row)).collect()
}

pub(crate) fn command_is_visible_in_help(command: Command, context: HelpContext) -> bool {
    help_metadata(command, context).is_some()
}

fn help_metadata(
    command: Command,
    context: HelpContext,
) -> Option<(HelpSectionKind, &'static str)> {
    match command {
        Command::Quit | Command::Help => None,
        Command::SearchPrompt => Some((HelpSectionKind::SearchCopy, "search")),
        Command::PromptLogRevset => {
            (context == HelpContext::Log).then_some((HelpSectionKind::Views, "custom revset"))
        }
        Command::OpenStatus => Some((HelpSectionKind::Views, "status")),
        Command::OpenResolve => Some((HelpSectionKind::Views, "resolve")),
        Command::OpenBookmarks => Some((HelpSectionKind::Views, "bookmarks")),
        Command::OpenWorkspaces => Some((HelpSectionKind::Views, "workspaces")),
        Command::OpenOperationLog => Some((HelpSectionKind::Views, "operation log")),
        Command::Edit => (context == HelpContext::Log)
            .then_some((HelpSectionKind::Actions, "edit selected revision")),
        Command::NextEdit => (context == HelpContext::Log).then_some((
            HelpSectionKind::Actions,
            "next editable change from @ (ignores selection)",
        )),
        Command::PrevEdit => (context == HelpContext::Log).then_some((
            HelpSectionKind::Actions,
            "previous editable change from @ (ignores selection)",
        )),
        Command::Describe => match context {
            HelpContext::Log => Some((HelpSectionKind::Actions, "describe selected revision")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "describe @")),
            _ => None,
        },
        Command::Commit => match context {
            HelpContext::Log => Some((
                HelpSectionKind::Actions,
                "commit @ and create new change (ignores selection)",
            )),
            HelpContext::Status => {
                Some((HelpSectionKind::Actions, "commit @ and create new change"))
            }
            _ => None,
        },
        Command::BookmarkCreate => match context {
            HelpContext::Log => Some((HelpSectionKind::Actions, "create bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "create bookmark at @")),
            _ => None,
        },
        Command::BookmarkSet => match context {
            HelpContext::Log => Some((HelpSectionKind::Actions, "set bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "set bookmark to @")),
            _ => None,
        },
        Command::BookmarkMove => match context {
            HelpContext::Log => Some((HelpSectionKind::Actions, "move bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "move bookmark to @")),
            _ => None,
        },
        Command::BookmarkRename => match context {
            HelpContext::Bookmarks => Some((HelpSectionKind::Actions, "rename local bookmark")),
            _ => None,
        },
        Command::BookmarkDelete => match context {
            HelpContext::Bookmarks => Some((HelpSectionKind::Actions, "delete local bookmark")),
            _ => None,
        },
        Command::BookmarkForget => match context {
            HelpContext::Bookmarks => Some((
                HelpSectionKind::Actions,
                "forget tracked or single remote-only bookmark",
            )),
            _ => None,
        },
        Command::BookmarkTrack => match context {
            HelpContext::Bookmarks => {
                Some((HelpSectionKind::Actions, "track exact remote bookmark"))
            }
            _ => None,
        },
        Command::BookmarkUntrack => match context {
            HelpContext::Bookmarks => {
                Some((HelpSectionKind::Actions, "untrack exact remote bookmark"))
            }
            _ => None,
        },
        Command::OperationUndo => (context == HelpContext::OperationLog).then_some((
            HelpSectionKind::Recovery,
            "undo last repo operation (global)",
        )),
        Command::OperationRedo => (context == HelpContext::OperationLog).then_some((
            HelpSectionKind::Recovery,
            "redo most recently undone operation (global)",
        )),
        Command::Push => match context {
            HelpContext::Log => Some((HelpSectionKind::Actions, "push selected revision")),
            HelpContext::Bookmarks => Some((HelpSectionKind::Actions, "push selected bookmark")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "push status")),
            _ => None,
        },
        Command::Fetch => Some((HelpSectionKind::RepositoryActions, "fetch")),
        Command::FetchRemote => Some((HelpSectionKind::RepositoryActions, "fetch remote")),
        Command::Copy => Some((HelpSectionKind::SearchCopy, "copy")),
        Command::ViewFormat => Some((HelpSectionKind::Views, "view menu")),
        Command::Refresh => Some((HelpSectionKind::App, "refresh")),
        Command::Back => Some((HelpSectionKind::Views, "back")),
        Command::SwitchLog => Some((HelpSectionKind::Views, "log")),
        Command::SwitchDefault => Some((HelpSectionKind::Views, "jj")),
        Command::View(command) => view_help_metadata(command, context),
    }
}

fn view_help_metadata(
    command: ViewCommand,
    context: HelpContext,
) -> Option<(HelpSectionKind, &'static str)> {
    match command {
        ViewCommand::CycleMode => Some((HelpSectionKind::Views, "cycle view mode")),
        ViewCommand::NewTrunk => Some((
            HelpSectionKind::RepositoryActions,
            "new from trunk (jj undo)",
        )),
        ViewCommand::MoveDown => Some((HelpSectionKind::Navigation, "move down")),
        ViewCommand::MoveUp => Some((HelpSectionKind::Navigation, "move up")),
        ViewCommand::PageDown => Some((HelpSectionKind::Navigation, "page down")),
        ViewCommand::PageUp => Some((HelpSectionKind::Navigation, "page up")),
        ViewCommand::MoveFirst => Some((HelpSectionKind::Navigation, "jump to first")),
        ViewCommand::MoveLast => Some((HelpSectionKind::Navigation, "jump to last")),
        ViewCommand::ToggleWrap => matches!(
            context,
            HelpContext::Show | HelpContext::Diff | HelpContext::FileShow
        )
        .then_some((HelpSectionKind::Views, "toggle wrap")),
        ViewCommand::ScrollLeft => matches!(
            context,
            HelpContext::Show | HelpContext::Diff | HelpContext::FileShow
        )
        .then_some((HelpSectionKind::Navigation, "scroll left")),
        ViewCommand::ScrollRight => matches!(
            context,
            HelpContext::Show | HelpContext::Diff | HelpContext::FileShow
        )
        .then_some((HelpSectionKind::Navigation, "scroll right")),
        ViewCommand::NextFile => Some((HelpSectionKind::Navigation, "next file")),
        ViewCommand::PreviousFile => Some((HelpSectionKind::Navigation, "previous file")),
        ViewCommand::OpenFiles => {
            let action = match context {
                HelpContext::Show | HelpContext::Diff | HelpContext::Status => "open file list",
                HelpContext::Log
                | HelpContext::Resolve
                | HelpContext::FileList
                | HelpContext::FileShow
                | HelpContext::Bookmarks
                | HelpContext::Workspaces
                | HelpContext::OperationLog
                | HelpContext::OperationDetail => return None,
            };
            Some((HelpSectionKind::Views, action))
        }
        ViewCommand::OpenItem => {
            let action = match context {
                HelpContext::FileList => "open file",
                HelpContext::Resolve => "inspect conflict",
                HelpContext::Log
                | HelpContext::Show
                | HelpContext::Diff
                | HelpContext::Status
                | HelpContext::FileShow
                | HelpContext::Bookmarks
                | HelpContext::Workspaces
                | HelpContext::OperationLog
                | HelpContext::OperationDetail => return None,
            };
            Some((HelpSectionKind::Views, action))
        }
        ViewCommand::OpenShow => {
            let action = match context {
                HelpContext::Log | HelpContext::Diff => "open show",
                HelpContext::Bookmarks => "open show",
                HelpContext::OperationLog | HelpContext::OperationDetail => "operation show",
                HelpContext::Show
                | HelpContext::Resolve
                | HelpContext::Status
                | HelpContext::FileList
                | HelpContext::FileShow
                | HelpContext::Workspaces => return None,
            };
            Some((HelpSectionKind::Views, action))
        }
        ViewCommand::OpenDiff => {
            let action = match context {
                HelpContext::Log | HelpContext::Show => "open diff",
                HelpContext::OperationLog | HelpContext::OperationDetail => "operation diff",
                HelpContext::Bookmarks
                | HelpContext::Workspaces
                | HelpContext::Diff
                | HelpContext::Resolve
                | HelpContext::Status
                | HelpContext::FileList
                | HelpContext::FileShow => return None,
            };
            Some((HelpSectionKind::Views, action))
        }
        ViewCommand::StartSearch => None,
        ViewCommand::NextSearchMatch => Some((HelpSectionKind::SearchCopy, "next match")),
        ViewCommand::PreviousSearchMatch => Some((HelpSectionKind::SearchCopy, "previous match")),
        ViewCommand::ToggleSelect => (context == HelpContext::Log).then_some((
            HelpSectionKind::Actions,
            "toggle exact revision selection (preview target)",
        )),
        ViewCommand::OpenActionMenu => matches!(
            context,
            HelpContext::Log
                | HelpContext::Show
                | HelpContext::Diff
                | HelpContext::Status
                | HelpContext::FileList
                | HelpContext::FileShow
                | HelpContext::OperationLog
        )
        .then_some((
            HelpSectionKind::Actions,
            "open action menu (preview required)",
        )),
        ViewCommand::Copy => None,
    }
}

#[cfg(test)]
mod tests;
