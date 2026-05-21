//! Generated help projection policy.
//!
//! This module decides which commands are shown in each view's help overlay and
//! how those commands are grouped. Key binding matching stays in `command.rs`;
//! help projection only consumes the binding vocabulary.

use crate::command::{Binding, Command, ViewCommand};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpContext {
    Graph,
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
            (context == HelpContext::Graph).then_some((HelpSectionKind::Views, "custom revset"))
        }
        Command::OpenStatus => Some((HelpSectionKind::Views, "status")),
        Command::OpenResolve => Some((HelpSectionKind::Views, "resolve")),
        Command::OpenBookmarks => Some((HelpSectionKind::Views, "bookmarks")),
        Command::OpenWorkspaces => Some((HelpSectionKind::Views, "workspaces")),
        Command::OpenOperationLog => Some((HelpSectionKind::Views, "operation log")),
        Command::Edit => (context == HelpContext::Graph)
            .then_some((HelpSectionKind::Actions, "edit selected revision")),
        Command::NextEdit => (context == HelpContext::Graph).then_some((
            HelpSectionKind::Actions,
            "next editable change from @ (ignores selection)",
        )),
        Command::PrevEdit => (context == HelpContext::Graph).then_some((
            HelpSectionKind::Actions,
            "previous editable change from @ (ignores selection)",
        )),
        Command::Describe => match context {
            HelpContext::Graph => Some((HelpSectionKind::Actions, "describe selected revision")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "describe @")),
            _ => None,
        },
        Command::Commit => match context {
            HelpContext::Graph => Some((
                HelpSectionKind::Actions,
                "commit @ and create new change (ignores selection)",
            )),
            HelpContext::Status => {
                Some((HelpSectionKind::Actions, "commit @ and create new change"))
            }
            _ => None,
        },
        Command::BookmarkCreate => match context {
            HelpContext::Graph => Some((HelpSectionKind::Actions, "create bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "create bookmark at @")),
            _ => None,
        },
        Command::BookmarkSet => match context {
            HelpContext::Graph => Some((HelpSectionKind::Actions, "set bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Actions, "set bookmark to @")),
            _ => None,
        },
        Command::BookmarkMove => match context {
            HelpContext::Graph => Some((HelpSectionKind::Actions, "move bookmark here")),
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
            HelpContext::Graph => Some((HelpSectionKind::Actions, "push selected revision")),
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
                HelpContext::Graph
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
                HelpContext::Graph
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
                HelpContext::Graph | HelpContext::Diff => "open show",
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
                HelpContext::Graph | HelpContext::Show => "open diff",
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
        ViewCommand::ToggleSelect => (context == HelpContext::Graph).then_some((
            HelpSectionKind::Actions,
            "toggle exact revision selection (preview target)",
        )),
        ViewCommand::OpenActionMenu => matches!(
            context,
            HelpContext::Graph
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
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};

    use super::*;
    use crate::command::KeyPattern;

    #[test]
    fn project_help_groups_bindings_by_command() {
        let global = [
            Binding::new(KeyPattern::char('r'), Command::Refresh),
            Binding::new(KeyPattern::code(KeyCode::Char('R')), Command::Refresh),
        ];
        let view = [Binding::new(
            KeyPattern::char('s'),
            Command::View(ViewCommand::OpenShow),
        )];

        let sections = project_help(&global, &view, HelpContext::Graph);

        assert_eq!(sections[0].title(), "View Switching");
        assert_eq!(sections[0].rows()[0], HelpRow::new("s", "open show"));
        assert_eq!(sections[1].title(), "App");
        assert_eq!(sections[1].rows()[0], HelpRow::new("r, R", "refresh"));
    }

    #[test]
    fn project_help_exposes_push_only_in_supported_contexts() {
        let global = [Binding::new(KeyPattern::char('p'), Command::Push)];

        let graph_help = project_help(&global, &[], HelpContext::Graph);
        let status_help = project_help(&global, &[], HelpContext::Status);
        let bookmarks_help = project_help(&global, &[], HelpContext::Bookmarks);
        let show_help = project_help(&global, &[], HelpContext::Show);

        assert_eq!(graph_help[0].title(), "Action Previews");
        assert_eq!(
            graph_help[0].rows()[0],
            HelpRow::new("p", "push selected revision")
        );
        assert_eq!(status_help[0].title(), "Action Previews");
        assert_eq!(status_help[0].rows()[0], HelpRow::new("p", "push status"));
        assert_eq!(
            bookmarks_help[0].rows()[0],
            HelpRow::new("p", "push selected bookmark")
        );
        assert!(!show_help.iter().any(|section| {
            section
                .rows()
                .iter()
                .any(|row| row.keys() == "p" && row.action().contains("push"))
        }));
    }

    #[test]
    fn project_help_exposes_describe_and_commit_in_honest_contexts() {
        let global = [
            Binding::new(KeyPattern::char('D'), Command::Describe),
            Binding::new(KeyPattern::char('C'), Command::Commit),
        ];

        let graph_help = project_help(&global, &[], HelpContext::Graph);
        let status_help = project_help(&global, &[], HelpContext::Status);
        let show_help = project_help(&global, &[], HelpContext::Show);

        assert_eq!(graph_help[0].title(), "Action Previews");
        assert_eq!(
            graph_help[0].rows()[0],
            HelpRow::new("D", "describe selected revision")
        );
        assert_eq!(
            graph_help[0].rows()[1],
            HelpRow::new("C", "commit @ and create new change (ignores selection)")
        );
        assert_eq!(status_help[0].rows()[0], HelpRow::new("D", "describe @"));
        assert_eq!(
            status_help[0].rows()[1],
            HelpRow::new("C", "commit @ and create new change")
        );
        assert!(!show_help.iter().any(|section| {
            section
                .rows()
                .iter()
                .any(|row| row.keys() == "D" || row.keys() == "C")
        }));
    }

    #[test]
    fn project_help_exposes_graph_edit_next_and_prev_only_in_graph() {
        let view = [
            Binding::new(KeyPattern::char('e'), Command::Edit),
            Binding::new(KeyPattern::char(']'), Command::NextEdit),
            Binding::new(KeyPattern::char('['), Command::PrevEdit),
        ];

        let graph_help = project_help(&[], &view, HelpContext::Graph);
        let show_help = project_help(&[], &view, HelpContext::Show);

        assert_eq!(graph_help[0].title(), "Action Previews");
        assert_eq!(
            graph_help[0].rows(),
            &[
                HelpRow::new("e", "edit selected revision"),
                HelpRow::new("]", "next editable change from @ (ignores selection)"),
                HelpRow::new("[", "previous editable change from @ (ignores selection)"),
            ]
        );
        assert!(show_help.is_empty());
    }

    #[test]
    fn project_help_exposes_bookmark_mutations_only_in_honest_contexts() {
        const BOOKMARK_RENAME: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('r')];
        const BOOKMARK_FORGET: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('f')];
        const BOOKMARK_TRACK: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('t')];
        const BOOKMARK_UNTRACK: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('u')];
        let global = [
            Binding::new(KeyPattern::char('b'), Command::BookmarkCreate),
            Binding::new(KeyPattern::char('='), Command::BookmarkSet),
            Binding::new(KeyPattern::char('m'), Command::BookmarkMove),
            Binding::sequence(BOOKMARK_RENAME, Command::BookmarkRename),
            Binding::new(KeyPattern::char('x'), Command::BookmarkDelete),
            Binding::sequence(BOOKMARK_FORGET, Command::BookmarkForget),
            Binding::sequence(BOOKMARK_TRACK, Command::BookmarkTrack),
            Binding::sequence(BOOKMARK_UNTRACK, Command::BookmarkUntrack),
        ];

        let graph_help = project_help(&global, &[], HelpContext::Graph);
        let status_help = project_help(&global, &[], HelpContext::Status);
        let bookmarks_help = project_help(&global, &[], HelpContext::Bookmarks);
        let show_help = project_help(&global, &[], HelpContext::Show);

        assert_eq!(
            graph_help[0].rows(),
            &[
                HelpRow::new("b", "create bookmark here"),
                HelpRow::new("=", "set bookmark here"),
                HelpRow::new("m", "move bookmark here"),
            ]
        );
        assert_eq!(
            status_help[0].rows(),
            &[
                HelpRow::new("b", "create bookmark at @"),
                HelpRow::new("=", "set bookmark to @"),
                HelpRow::new("m", "move bookmark to @"),
            ]
        );
        assert_eq!(
            bookmarks_help[0].rows(),
            &[
                HelpRow::new("br", "rename local bookmark"),
                HelpRow::new("x", "delete local bookmark"),
                HelpRow::new("bf", "forget tracked or single remote-only bookmark"),
                HelpRow::new("bt", "track exact remote bookmark"),
                HelpRow::new("bu", "untrack exact remote bookmark"),
            ]
        );
        assert!(!show_help.iter().any(|section| {
            section
                .rows()
                .iter()
                .any(|row| row.action().contains("bookmark") || row.action().contains("track"))
        }));
    }

    #[test]
    fn operation_help_exposes_show_and_diff_without_placeholder_text() {
        let view = [
            Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow)),
            Binding::new(KeyPattern::char('d'), Command::View(ViewCommand::OpenDiff)),
        ];

        let sections = project_help(&[], &view, HelpContext::OperationLog);

        assert_eq!(sections[0].title(), "View Switching");
        assert_eq!(sections[0].rows()[0], HelpRow::new("s", "operation show"));
        assert_eq!(sections[0].rows()[1], HelpRow::new("d", "operation diff"));
    }

    #[test]
    fn operation_help_exposes_global_undo_and_redo_recovery_actions() {
        let view = [
            Binding::new(KeyPattern::char('u'), Command::OperationUndo),
            Binding::new(
                KeyPattern::modified_char('r', KeyModifiers::CONTROL),
                Command::OperationRedo,
            ),
        ];

        let sections = project_help(&[], &view, HelpContext::OperationLog);

        assert_eq!(sections[0].title(), "Recovery");
        assert_eq!(
            sections[0].rows()[0],
            HelpRow::new("u", "undo last repo operation (global)")
        );
        assert_eq!(
            sections[0].rows()[1],
            HelpRow::new("C-r", "redo most recently undone operation (global)")
        );
    }

    #[test]
    fn operation_help_exposes_exact_id_action_menu_separately_from_global_recovery() {
        let view = [Binding::new(
            KeyPattern::char('a'),
            Command::View(ViewCommand::OpenActionMenu),
        )];

        let sections = project_help(&[], &view, HelpContext::OperationLog);

        assert_eq!(sections[0].title(), "Action Previews");
        assert_eq!(
            sections[0].rows()[0],
            HelpRow::new("a", "open action menu (preview required)")
        );
    }

    #[test]
    fn document_help_exposes_wrap_commands_only_in_document_contexts() {
        const TOGGLE_WRAP: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('w')];
        const SCROLL_LEFT: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('h')];
        const SCROLL_RIGHT: &[KeyPattern] = &[KeyPattern::char('z'), KeyPattern::char('l')];
        let view = [
            Binding::sequence(TOGGLE_WRAP, Command::View(ViewCommand::ToggleWrap)),
            Binding::sequence(SCROLL_LEFT, Command::View(ViewCommand::ScrollLeft)),
            Binding::sequence(SCROLL_RIGHT, Command::View(ViewCommand::ScrollRight)),
        ];

        let file_show_help = project_help(&[], &view, HelpContext::FileShow);
        let graph_help = project_help(&[], &view, HelpContext::Graph);

        assert_eq!(file_show_help[0].title(), "Navigation");
        assert_eq!(
            file_show_help[0].rows(),
            &[
                HelpRow::new("zh", "scroll left"),
                HelpRow::new("zl", "scroll right"),
            ]
        );
        assert_eq!(file_show_help[1].title(), "View Switching");
        assert_eq!(
            file_show_help[1].rows(),
            &[HelpRow::new("zw", "toggle wrap")]
        );
        assert!(graph_help.is_empty());
    }

    #[test]
    fn resolve_help_exposes_global_entry_and_inspect_action() {
        let global = [Binding::new(KeyPattern::char('R'), Command::OpenResolve)];
        let view = [Binding::new(
            KeyPattern::code(KeyCode::Enter),
            Command::View(ViewCommand::OpenItem),
        )];

        let sections = project_help(&global, &view, HelpContext::Resolve);

        assert_eq!(sections[0].title(), "View Switching");
        assert_eq!(sections[0].rows()[0], HelpRow::new("R", "resolve"));
        assert_eq!(
            sections[0].rows()[1],
            HelpRow::new("Enter", "inspect conflict")
        );
    }
}
