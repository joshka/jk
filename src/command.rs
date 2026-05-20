//! Key binding metadata and command effects.
//!
//! Bindings are static Rust data. Help and status text live in `tui.rs`, so
//! this module only owns the key-to-command mapping used by dispatch.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action_menu::ActionMenu;
use crate::copy::CopyOption;
use crate::jj::{JjCommand, JjOperationRecoveryKind};
use crate::search::SearchQuery;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Quit,
    Help,
    SearchPrompt,
    PromptLogRevset,
    OpenStatus,
    OpenResolve,
    OpenBookmarks,
    OpenOperationLog,
    OperationUndo,
    OperationRedo,
    Describe,
    Commit,
    BookmarkCreate,
    BookmarkSet,
    BookmarkMove,
    BookmarkDelete,
    Fetch,
    Push,
    Copy,
    ViewFormat,
    Refresh,
    Back,
    SwitchLog,
    SwitchDefault,
    View(ViewCommand),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewCommand {
    CycleMode,
    NewTrunk,
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    MoveFirst,
    MoveLast,
    NextFile,
    PreviousFile,
    OpenFiles,
    OpenItem,
    OpenShow,
    OpenDiff,
    StartSearch,
    NextSearchMatch,
    PreviousSearchMatch,
    ToggleSelect,
    OpenActionMenu,
    Copy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Binding {
    key: KeyPattern,
    command: Command,
}

impl Binding {
    pub const fn new(key: KeyPattern, command: Command) -> Self {
        Self { key, command }
    }

    pub fn matches(self, key: KeyEvent) -> bool {
        self.key.matches(key)
    }

    pub fn command(self) -> Command {
        self.command
    }
}

impl Command {
    pub fn operation_recovery(self) -> Option<JjOperationRecoveryKind> {
        match self {
            Self::OperationUndo => Some(JjOperationRecoveryKind::Undo),
            Self::OperationRedo => Some(JjOperationRecoveryKind::Redo),
            Self::Quit
            | Self::Help
            | Self::SearchPrompt
            | Self::PromptLogRevset
            | Self::OpenStatus
            | Self::OpenResolve
            | Self::OpenBookmarks
            | Self::OpenOperationLog
            | Self::Describe
            | Self::Commit
            | Self::BookmarkCreate
            | Self::BookmarkSet
            | Self::BookmarkMove
            | Self::BookmarkDelete
            | Self::Fetch
            | Self::Push
            | Self::Copy
            | Self::ViewFormat
            | Self::Refresh
            | Self::Back
            | Self::SwitchLog
            | Self::SwitchDefault
            | Self::View(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPattern {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyPattern {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub const fn char(character: char) -> Self {
        Self::new(KeyCode::Char(character), KeyModifiers::NONE)
    }

    pub const fn modified_char(character: char, modifiers: KeyModifiers) -> Self {
        Self::new(KeyCode::Char(character), modifiers)
    }

    pub const fn code(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    fn matches(self, key: KeyEvent) -> bool {
        key.code == self.code && key.modifiers == self.modifiers
    }

    pub fn label(self) -> String {
        let code = match self.code {
            KeyCode::Backspace => "Backspace".to_owned(),
            KeyCode::Down => "Down".to_owned(),
            KeyCode::End => "End".to_owned(),
            KeyCode::Enter => "Enter".to_owned(),
            KeyCode::Esc => "Esc".to_owned(),
            KeyCode::Home => "Home".to_owned(),
            KeyCode::Left => "Left".to_owned(),
            KeyCode::PageDown => "PageDown".to_owned(),
            KeyCode::PageUp => "PageUp".to_owned(),
            KeyCode::Right => "Right".to_owned(),
            KeyCode::Up => "Up".to_owned(),
            KeyCode::Char(' ') => "Space".to_owned(),
            KeyCode::Char(character) => character.to_string(),
            _ => format!("{:?}", self.code),
        };

        if self.modifiers.is_empty() {
            code
        } else {
            format!("{}-{code}", key_modifier_label(self.modifiers))
        }
    }
}

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
    OperationLog,
    OperationDetail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpSectionKind {
    Global,
    View,
    Direct,
    Preview,
}

impl HelpSectionKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::View => "Current View",
            Self::Direct => "Direct Actions",
            Self::Preview => "Preview / Confirm",
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

pub struct CommandContext<'a> {
    pub viewport_height: u16,
    pub search: Option<&'a SearchQuery>,
}

impl CommandContext<'_> {
    pub fn page_size(&self) -> usize {
        usize::from(self.viewport_height.saturating_sub(1).max(1))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ViewEffect {
    Ignored,
    Handled,
    StatusMessage(String),
    StatusError(String),
    RunNewTrunk,
    OpenDetail(JjCommand, String),
    OpenView(crate::jj::ViewSpec),
    SearchMoved,
    SearchStarted { matches: usize },
    CopyOptions(Vec<CopyOption>),
    OpenActionMenu(ActionMenu),
}

pub fn find_binding(bindings: &[Binding], key: KeyEvent) -> Option<Binding> {
    bindings
        .iter()
        .copied()
        .find(|binding| binding.matches(key))
}

pub fn project_help(
    global_bindings: &[Binding],
    view_bindings: &[Binding],
    context: HelpContext,
) -> Vec<HelpSection> {
    let global_rows = collect_help_rows(global_bindings, context);
    let view_rows = collect_help_rows(view_bindings, context);

    [
        HelpSectionKind::Global,
        HelpSectionKind::View,
        HelpSectionKind::Direct,
        HelpSectionKind::Preview,
    ]
    .into_iter()
    .filter_map(|kind| {
        let mut rows = global_rows
            .iter()
            .chain(&view_rows)
            .filter(|(row_kind, _)| *row_kind == kind)
            .map(|(_, row)| row.clone())
            .collect::<Vec<_>>();
        if kind == HelpSectionKind::Preview && rows.is_empty() {
            rows.push(HelpRow::new("-", "none yet"));
        }
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
        let key = binding.key.label();

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

fn help_metadata(
    command: Command,
    context: HelpContext,
) -> Option<(HelpSectionKind, &'static str)> {
    match command {
        Command::Quit => Some((HelpSectionKind::Global, "quit")),
        Command::Help => Some((HelpSectionKind::Global, "help")),
        Command::SearchPrompt => Some((HelpSectionKind::Global, "search")),
        Command::PromptLogRevset => {
            (context == HelpContext::Graph).then_some((HelpSectionKind::Direct, "custom revset"))
        }
        Command::OpenStatus => Some((HelpSectionKind::Global, "status")),
        Command::OpenResolve => Some((HelpSectionKind::Global, "resolve")),
        Command::OpenBookmarks => Some((HelpSectionKind::Global, "bookmarks")),
        Command::OpenOperationLog => Some((HelpSectionKind::Global, "operation log")),
        Command::Describe => match context {
            HelpContext::Graph => Some((HelpSectionKind::Preview, "describe selected revision")),
            HelpContext::Status => Some((HelpSectionKind::Preview, "describe @")),
            _ => None,
        },
        Command::Commit => match context {
            HelpContext::Graph => Some((
                HelpSectionKind::Preview,
                "commit @ and create new change (ignores selection)",
            )),
            HelpContext::Status => {
                Some((HelpSectionKind::Preview, "commit @ and create new change"))
            }
            _ => None,
        },
        Command::BookmarkCreate => match context {
            HelpContext::Graph => Some((HelpSectionKind::Preview, "create bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Preview, "create bookmark at @")),
            _ => None,
        },
        Command::BookmarkSet => match context {
            HelpContext::Graph => Some((HelpSectionKind::Preview, "set bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Preview, "set bookmark to @")),
            _ => None,
        },
        Command::BookmarkMove => match context {
            HelpContext::Graph => Some((HelpSectionKind::Preview, "move bookmark here")),
            HelpContext::Status => Some((HelpSectionKind::Preview, "move bookmark to @")),
            _ => None,
        },
        Command::BookmarkDelete => match context {
            HelpContext::Bookmarks => Some((HelpSectionKind::Preview, "delete local bookmark")),
            _ => None,
        },
        Command::OperationUndo => (context == HelpContext::OperationLog).then_some((
            HelpSectionKind::Preview,
            "undo last repo operation (global)",
        )),
        Command::OperationRedo => (context == HelpContext::OperationLog).then_some((
            HelpSectionKind::Preview,
            "redo most recently undone operation (global)",
        )),
        Command::Push => match context {
            HelpContext::Graph => Some((HelpSectionKind::Preview, "push selected revision")),
            HelpContext::Bookmarks => Some((HelpSectionKind::Preview, "push selected bookmark")),
            HelpContext::Status => Some((HelpSectionKind::Preview, "push status")),
            _ => None,
        },
        Command::Fetch => Some((HelpSectionKind::Direct, "fetch")),
        Command::Copy => Some((HelpSectionKind::Global, "copy")),
        Command::ViewFormat => Some((HelpSectionKind::Global, "view format")),
        Command::Refresh => Some((HelpSectionKind::Global, "refresh")),
        Command::Back => Some((HelpSectionKind::Global, "back")),
        Command::SwitchLog => Some((HelpSectionKind::Global, "log")),
        Command::SwitchDefault => Some((HelpSectionKind::Global, "jj")),
        Command::View(command) => view_help_metadata(command, context),
    }
}

fn view_help_metadata(
    command: ViewCommand,
    context: HelpContext,
) -> Option<(HelpSectionKind, &'static str)> {
    match command {
        ViewCommand::CycleMode => Some((HelpSectionKind::Direct, "cycle view mode")),
        ViewCommand::NewTrunk => Some((HelpSectionKind::Direct, "new from trunk (jj undo)")),
        ViewCommand::MoveDown => Some((HelpSectionKind::View, "move down")),
        ViewCommand::MoveUp => Some((HelpSectionKind::View, "move up")),
        ViewCommand::PageDown => Some((HelpSectionKind::View, "page down")),
        ViewCommand::PageUp => Some((HelpSectionKind::View, "page up")),
        ViewCommand::MoveFirst => Some((HelpSectionKind::View, "jump to first")),
        ViewCommand::MoveLast => Some((HelpSectionKind::View, "jump to last")),
        ViewCommand::NextFile => Some((HelpSectionKind::View, "next file")),
        ViewCommand::PreviousFile => Some((HelpSectionKind::View, "previous file")),
        ViewCommand::OpenFiles => {
            let action = match context {
                HelpContext::Show | HelpContext::Diff | HelpContext::Status => "open file list",
                HelpContext::Graph
                | HelpContext::Resolve
                | HelpContext::FileList
                | HelpContext::FileShow
                | HelpContext::Bookmarks
                | HelpContext::OperationLog
                | HelpContext::OperationDetail => return None,
            };
            Some((HelpSectionKind::Direct, action))
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
                | HelpContext::OperationLog
                | HelpContext::OperationDetail => return None,
            };
            Some((HelpSectionKind::Direct, action))
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
                | HelpContext::FileShow => return None,
            };
            Some((HelpSectionKind::Direct, action))
        }
        ViewCommand::OpenDiff => {
            let action = match context {
                HelpContext::Graph | HelpContext::Show => "open diff",
                HelpContext::OperationLog | HelpContext::OperationDetail => "operation diff",
                HelpContext::Bookmarks
                | HelpContext::Diff
                | HelpContext::Resolve
                | HelpContext::Status
                | HelpContext::FileList
                | HelpContext::FileShow => return None,
            };
            Some((HelpSectionKind::Direct, action))
        }
        ViewCommand::StartSearch => None,
        ViewCommand::NextSearchMatch => Some((HelpSectionKind::View, "next match")),
        ViewCommand::PreviousSearchMatch => Some((HelpSectionKind::View, "previous match")),
        ViewCommand::ToggleSelect => (context == HelpContext::Graph).then_some((
            HelpSectionKind::Preview,
            "toggle exact revision selection (preview target)",
        )),
        ViewCommand::OpenActionMenu => matches!(
            context,
            HelpContext::Graph
                | HelpContext::Show
                | HelpContext::Diff
                | HelpContext::FileList
                | HelpContext::FileShow
        )
        .then_some((
            HelpSectionKind::Preview,
            "open action menu (preview required)",
        )),
        ViewCommand::Copy => None,
    }
}

fn key_modifier_label(modifiers: KeyModifiers) -> &'static str {
    if modifiers == KeyModifiers::CONTROL {
        "C"
    } else if modifiers == KeyModifiers::SHIFT {
        "S"
    } else if modifiers == KeyModifiers::ALT {
        "A"
    } else {
        "M"
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

    use super::*;

    #[test]
    fn binding_matches_key_code_and_modifiers() {
        let binding = Binding::new(
            KeyPattern::modified_char('f', KeyModifiers::CONTROL),
            Command::View(ViewCommand::PageDown),
        );

        assert!(binding.matches(key(KeyCode::Char('f'), KeyModifiers::CONTROL)));
        assert!(!binding.matches(key(KeyCode::Char('f'), KeyModifiers::NONE)));
    }

    #[test]
    fn find_binding_returns_first_matching_command() {
        let bindings = [
            Binding::new(KeyPattern::char('j'), Command::View(ViewCommand::MoveDown)),
            Binding::new(KeyPattern::char('q'), Command::Quit),
        ];

        assert_eq!(
            find_binding(&bindings, key(KeyCode::Char('q'), KeyModifiers::NONE))
                .map(Binding::command),
            Some(Command::Quit)
        );
    }

    #[test]
    fn key_pattern_labels_special_keys() {
        assert_eq!(KeyPattern::char(' ').label(), "Space");
        assert_eq!(
            KeyPattern::modified_char('f', KeyModifiers::CONTROL).label(),
            "C-f"
        );
        assert_eq!(KeyPattern::code(KeyCode::Down).label(), "Down");
    }

    #[test]
    fn project_help_groups_bindings_by_command() {
        let global = [
            Binding::new(KeyPattern::char('q'), Command::Quit),
            Binding::new(KeyPattern::code(KeyCode::Esc), Command::Quit),
        ];
        let view = [Binding::new(
            KeyPattern::char('s'),
            Command::View(ViewCommand::OpenShow),
        )];

        let sections = project_help(&global, &view, HelpContext::Graph);

        assert_eq!(sections[0].title(), "Global");
        assert_eq!(sections[0].rows()[0], HelpRow::new("q, Esc", "quit"));
        assert_eq!(sections[1].title(), "Direct Actions");
        assert_eq!(sections[1].rows()[0], HelpRow::new("s", "open show"));
        assert_eq!(sections[2].rows()[0], HelpRow::new("-", "none yet"));
    }

    #[test]
    fn project_help_exposes_push_only_in_supported_contexts() {
        let global = [Binding::new(KeyPattern::char('p'), Command::Push)];

        let graph_help = project_help(&global, &[], HelpContext::Graph);
        let status_help = project_help(&global, &[], HelpContext::Status);
        let bookmarks_help = project_help(&global, &[], HelpContext::Bookmarks);
        let show_help = project_help(&global, &[], HelpContext::Show);

        assert_eq!(graph_help[0].title(), "Preview / Confirm");
        assert_eq!(
            graph_help[0].rows()[0],
            HelpRow::new("p", "push selected revision")
        );
        assert_eq!(status_help[0].title(), "Preview / Confirm");
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

        assert_eq!(graph_help[0].title(), "Preview / Confirm");
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
    fn project_help_exposes_bookmark_mutations_only_in_honest_contexts() {
        let global = [
            Binding::new(KeyPattern::char('b'), Command::BookmarkCreate),
            Binding::new(KeyPattern::char('='), Command::BookmarkSet),
            Binding::new(KeyPattern::char('m'), Command::BookmarkMove),
            Binding::new(KeyPattern::char('x'), Command::BookmarkDelete),
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
            &[HelpRow::new("x", "delete local bookmark")]
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

        assert_eq!(sections[0].title(), "Direct Actions");
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

        assert_eq!(sections[0].title(), "Preview / Confirm");
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
    fn resolve_help_exposes_global_entry_and_inspect_action() {
        let global = [Binding::new(KeyPattern::char('R'), Command::OpenResolve)];
        let view = [Binding::new(
            KeyPattern::code(KeyCode::Enter),
            Command::View(ViewCommand::OpenItem),
        )];

        let sections = project_help(&global, &view, HelpContext::Resolve);

        assert_eq!(sections[0].title(), "Global");
        assert_eq!(sections[0].rows()[0], HelpRow::new("R", "resolve"));
        assert_eq!(sections[1].title(), "Direct Actions");
        assert_eq!(
            sections[1].rows()[0],
            HelpRow::new("Enter", "inspect conflict")
        );
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}
