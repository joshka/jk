//! Key binding metadata and command effects.
//!
//! Bindings are static Rust data. `tui.rs` renders the help and status chrome,
//! while this module owns the command metadata and labels that feed those
//! views and dispatch.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::action_menu::ActionMenu;
use crate::copy::CopyOption;
use crate::jj::{JjCommand, JjOperationRecoveryKind};
use crate::search::SearchQuery;

/// App-level dispatch vocabulary for global bindings and view-facing effects.
///
/// `App` matches these variants to top-level bindings first, then routes the
/// view-specific variants through `ViewCommand`. Keep the enum aligned with the
/// dispatcher because the variants define the commands the app can execute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    Quit,
    Help,
    SearchPrompt,
    PromptLogRevset,
    OpenStatus,
    OpenResolve,
    OpenBookmarks,
    OpenWorkspaces,
    OpenOperationLog,
    OperationUndo,
    OperationRedo,
    Edit,
    NextEdit,
    PrevEdit,
    Describe,
    Commit,
    BookmarkCreate,
    BookmarkSet,
    BookmarkMove,
    BookmarkRename,
    BookmarkDelete,
    BookmarkForget,
    BookmarkTrack,
    BookmarkUntrack,
    Fetch,
    FetchRemote,
    Push,
    Copy,
    ViewFormat,
    Refresh,
    Back,
    SwitchLog,
    SwitchDefault,
    View(ViewCommand),
}

/// View-local commands that may inspect the current viewport and search state.
///
/// These commands stay on the presentation side of the boundary: they can
/// return a `ViewEffect`, but `App` owns the actual state transition that
/// follows.
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
    ToggleWrap,
    ScrollLeft,
    ScrollRight,
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
    key: KeySequence,
    command: Command,
}

impl Binding {
    pub const fn new(key: KeyPattern, command: Command) -> Self {
        Self {
            key: KeySequence::Single(key),
            command,
        }
    }

    pub const fn sequence(keys: &'static [KeyPattern], command: Command) -> Self {
        Self {
            key: KeySequence::Multi(keys),
            command,
        }
    }

    #[cfg(test)]
    pub fn matches(self, key: KeyEvent) -> bool {
        self.key.matches(key)
    }

    pub fn command(self) -> Command {
        self.command
    }

    pub fn key_label(self) -> String {
        self.key.label()
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
            | Self::OpenWorkspaces
            | Self::OpenOperationLog
            | Self::Edit
            | Self::NextEdit
            | Self::PrevEdit
            | Self::Describe
            | Self::Commit
            | Self::BookmarkCreate
            | Self::BookmarkSet
            | Self::BookmarkMove
            | Self::BookmarkRename
            | Self::BookmarkDelete
            | Self::BookmarkForget
            | Self::BookmarkTrack
            | Self::BookmarkUntrack
            | Self::Fetch
            | Self::FetchRemote
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
        key.code == self.code
            && (key.modifiers == self.modifiers
                || (self.modifiers.is_empty()
                    && key.modifiers == KeyModifiers::SHIFT
                    && shifted_character_is_encoded_in_key_code(self.code)))
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
enum KeySequence {
    Single(KeyPattern),
    Multi(&'static [KeyPattern]),
}

impl KeySequence {
    #[cfg(test)]
    fn matches(self, key: KeyEvent) -> bool {
        match self {
            Self::Single(pattern) => pattern.matches(key),
            Self::Multi([pattern]) => pattern.matches(key),
            Self::Multi(_) => false,
        }
    }

    fn label(self) -> String {
        match self {
            Self::Single(pattern) => pattern.label(),
            Self::Multi(patterns) if patterns.iter().all(|pattern| pattern.is_plain_char()) => {
                patterns
                    .iter()
                    .filter_map(|pattern| pattern.plain_char())
                    .collect()
            }
            Self::Multi(patterns) => patterns
                .iter()
                .map(|pattern| pattern.label())
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    fn len(self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multi(patterns) => patterns.len(),
        }
    }

    fn matches_prefix(self, keys: &[KeyEvent]) -> bool {
        if keys.len() > self.len() {
            return false;
        }

        match self {
            Self::Single(pattern) => keys
                .first()
                .is_some_and(|key| keys.len() == 1 && pattern.matches(*key)),
            Self::Multi(patterns) => keys
                .iter()
                .zip(patterns)
                .all(|(key, pattern)| pattern.matches(*key)),
        }
    }

    fn next_pattern(self, key_count: usize) -> Option<KeyPattern> {
        match self {
            Self::Single(_) => None,
            Self::Multi(patterns) => patterns.get(key_count).copied(),
        }
    }
}

fn shifted_character_is_encoded_in_key_code(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Char(character)
            if character.is_ascii_uppercase()
                || (!character.is_ascii_alphanumeric() && !character.is_ascii_whitespace())
    )
}

impl KeyPattern {
    fn is_plain_char(self) -> bool {
        matches!(self.code, KeyCode::Char(_)) && self.modifiers.is_empty()
    }

    fn plain_char(self) -> Option<char> {
        match self.code {
            KeyCode::Char(character) if self.modifiers.is_empty() => Some(character),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingMatch {
    Exact(Binding),
    Prefix { fallback: Option<Binding> },
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

/// Snapshot of the live viewport and search state for one view dispatch.
///
/// `App` rebuilds this for each key event, so view code must treat it as
/// read-only input for the current dispatch instead of retained state.
pub struct CommandContext<'a> {
    pub viewport_height: u16,
    pub viewport_width: u16,
    pub search: Option<&'a SearchQuery>,
}

impl CommandContext<'_> {
    pub fn page_size(&self) -> usize {
        usize::from(self.viewport_height.saturating_sub(1).max(1))
    }
}

/// One-way output from a view command back to the app dispatcher.
///
/// The app interprets these effects and performs the resulting navigation,
/// refresh, or status update; views do not mutate app-owned state directly.
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

#[cfg(test)]
pub fn find_binding(bindings: &[Binding], key: KeyEvent) -> Option<Binding> {
    bindings
        .iter()
        .copied()
        .find(|binding| binding.matches(key))
}

pub fn match_binding_sequence(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
) -> Option<BindingMatch> {
    match_binding_sequence_by(binding_groups, keys, |_| true)
}

pub fn match_help_binding_sequence(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    context: HelpContext,
) -> Option<BindingMatch> {
    match_binding_sequence_by(binding_groups, keys, |binding| {
        command_is_visible_in_help(binding.command(), context)
    })
}

pub fn binding_prefix_next_labels(binding_groups: &[&[Binding]], keys: &[KeyEvent]) -> Vec<String> {
    binding_prefix_next_labels_by(binding_groups, keys, |_| true)
}

pub fn help_binding_prefix_next_labels(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    context: HelpContext,
) -> Vec<String> {
    binding_prefix_next_labels_by(binding_groups, keys, |binding| {
        command_is_visible_in_help(binding.command(), context)
    })
}

fn match_binding_sequence_by(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    is_available: impl Fn(Binding) -> bool,
) -> Option<BindingMatch> {
    if keys.is_empty() {
        return None;
    }

    let mut exact = None;
    let mut has_prefix = false;

    for bindings in binding_groups {
        for binding in *bindings {
            if !is_available(*binding) {
                continue;
            }

            if !binding.key.matches_prefix(keys) {
                continue;
            }

            if binding.key.len() == keys.len() {
                exact.get_or_insert(*binding);
            } else {
                has_prefix = true;
            }
        }
    }

    if has_prefix {
        Some(BindingMatch::Prefix { fallback: exact })
    } else {
        exact.map(BindingMatch::Exact)
    }
}

fn binding_prefix_next_labels_by(
    binding_groups: &[&[Binding]],
    keys: &[KeyEvent],
    is_available: impl Fn(Binding) -> bool,
) -> Vec<String> {
    if keys.is_empty() {
        return Vec::new();
    }

    let mut labels = Vec::new();
    for bindings in binding_groups {
        for binding in *bindings {
            if !is_available(*binding) || !binding.key.matches_prefix(keys) {
                continue;
            }

            let Some(pattern) = binding.key.next_pattern(keys.len()) else {
                continue;
            };
            let label = pattern.label();
            if !labels.iter().any(|existing| existing == &label) {
                labels.push(label);
            }
        }
    }
    labels
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

fn command_is_visible_in_help(command: Command, context: HelpContext) -> bool {
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
    fn uppercase_bindings_accept_shifted_character_events() {
        let uppercase = Binding::new(KeyPattern::char('S'), Command::OpenStatus);
        let lowercase = Binding::new(KeyPattern::char('s'), Command::View(ViewCommand::OpenShow));

        assert!(uppercase.matches(key(KeyCode::Char('S'), KeyModifiers::SHIFT)));
        assert!(!lowercase.matches(key(KeyCode::Char('S'), KeyModifiers::SHIFT)));
        assert!(!uppercase.matches(key(KeyCode::Char('s'), KeyModifiers::NONE)));
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
    fn match_binding_sequence_reports_prefix_and_exact_fallback() {
        const BOOKMARK_CREATE: &[KeyPattern] = &[KeyPattern::char('b'), KeyPattern::char('c')];
        let bindings = [
            Binding::new(KeyPattern::char('b'), Command::BookmarkCreate),
            Binding::sequence(BOOKMARK_CREATE, Command::BookmarkCreate),
        ];

        let pending =
            match_binding_sequence(&[&bindings], &[key(KeyCode::Char('b'), KeyModifiers::NONE)]);
        let complete = match_binding_sequence(
            &[&bindings],
            &[
                key(KeyCode::Char('b'), KeyModifiers::NONE),
                key(KeyCode::Char('c'), KeyModifiers::NONE),
            ],
        );

        assert_eq!(
            pending,
            Some(BindingMatch::Prefix {
                fallback: Some(bindings[0])
            })
        );
        assert_eq!(complete, Some(BindingMatch::Exact(bindings[1])));
        assert_eq!(bindings[1].key_label(), "bc");
    }

    #[test]
    fn match_binding_sequence_allows_global_prefix_over_view_fallback() {
        const GIT_FETCH: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('f')];
        let global = [Binding::sequence(GIT_FETCH, Command::Fetch)];
        let view = [Binding::new(
            KeyPattern::char('g'),
            Command::View(ViewCommand::MoveFirst),
        )];

        let pending = match_binding_sequence(
            &[&global, &view],
            &[key(KeyCode::Char('g'), KeyModifiers::NONE)],
        );

        assert_eq!(
            pending,
            Some(BindingMatch::Prefix {
                fallback: Some(view[0])
            })
        );
    }

    #[test]
    fn binding_prefix_next_labels_list_available_suffixes() {
        const GIT_FETCH: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('f')];
        const GIT_PUSH: &[KeyPattern] = &[KeyPattern::char('g'), KeyPattern::char('p')];
        let bindings = [
            Binding::new(KeyPattern::char('g'), Command::View(ViewCommand::MoveFirst)),
            Binding::sequence(GIT_FETCH, Command::Fetch),
            Binding::sequence(GIT_PUSH, Command::Push),
        ];

        assert_eq!(
            binding_prefix_next_labels(
                &[&bindings],
                &[key(KeyCode::Char('g'), KeyModifiers::NONE)]
            ),
            vec!["f", "p"]
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
    fn help_binding_match_uses_visible_help_metadata() {
        let bindings = [
            Binding::new(KeyPattern::char('q'), Command::Quit),
            Binding::new(KeyPattern::char('D'), Command::Describe),
            Binding::new(KeyPattern::char('S'), Command::OpenStatus),
        ];

        assert_eq!(
            match_help_binding_sequence(
                &[&bindings],
                &[key(KeyCode::Char('S'), KeyModifiers::NONE)],
                HelpContext::Show,
            ),
            Some(BindingMatch::Exact(bindings[2]))
        );
        assert_eq!(
            match_help_binding_sequence(
                &[&bindings],
                &[key(KeyCode::Char('D'), KeyModifiers::NONE)],
                HelpContext::Show,
            ),
            None
        );
        assert_eq!(
            match_help_binding_sequence(
                &[&bindings],
                &[key(KeyCode::Char('q'), KeyModifiers::NONE)],
                HelpContext::Graph,
            ),
            None
        );
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

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}
