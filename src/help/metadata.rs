use crate::command::{Command, ViewCommand};

use super::{HelpContext, HelpSectionKind};

/// Maps one app-level command to a help section and action string for the given context.
pub fn help_metadata(
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

/// Maps one view command to a help section and action string for the given context.
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
