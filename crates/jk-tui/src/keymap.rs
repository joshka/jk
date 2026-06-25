//! Display metadata for contextual key help and hotbar text.
//!
//! Dispatch still lives in the binary's terminal adapter. This module is only the visible binding
//! registry used by TUI chrome until input dispatch can share the same data model.

/// Screen-specific binding set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingContext {
    /// The revision log view.
    Log,
    /// The selected-change diff view.
    Diff,
    /// A rendered read-only inspection view, such as show or status.
    Inspection,
    /// The workspace list view.
    Workspaces,
    /// The command-history list view.
    CommandHistory,
    /// The operation log list view.
    OperationLog,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActionId {
    Move,
    LineScroll,
    PageDown,
    PageUp,
    JumpTop,
    JumpBottom,
    Mark,
    ClearMarks,
    Expand,
    Collapse,
    OpenShow,
    OpenDiff,
    OpenLog,
    OpenDescribe,
    OpenEvolog,
    OpenStatus,
    OpenOperation,
    OpenOperationLog,
    OpenCommandHistory,
    OpenCommandDetails,
    CopyCommand,
    CommandMode,
    NewChange,
    EditChange,
    Abandon,
    Undo,
    Redo,
    UpdateStale,
    ViewOptions,
    Refresh,
    SwitchLogCommand,
    OpenFileList,
    File,
    Hunk,
    FoldFile,
    FoldAll,
    FoldHunk,
    HorizontalScroll,
    Search,
    ReturnToLog,
    ReturnBack,
    CloseHelp,
    Quit,
}

const fn default_help_group(action: ActionId) -> HelpGroup {
    match action {
        ActionId::Move
        | ActionId::LineScroll
        | ActionId::PageDown
        | ActionId::PageUp
        | ActionId::JumpTop
        | ActionId::JumpBottom
        | ActionId::Expand
        | ActionId::Collapse
        | ActionId::HorizontalScroll
        | ActionId::Search
        | ActionId::ReturnToLog
        | ActionId::ReturnBack => HelpGroup::Navigation,
        ActionId::OpenShow
        | ActionId::OpenDiff
        | ActionId::OpenLog
        | ActionId::OpenEvolog
        | ActionId::OpenStatus
        | ActionId::SwitchLogCommand
        | ActionId::ViewOptions
        | ActionId::OpenFileList
        | ActionId::File
        | ActionId::Hunk
        | ActionId::FoldFile
        | ActionId::FoldAll
        | ActionId::FoldHunk => HelpGroup::Views,
        ActionId::OpenDescribe
        | ActionId::NewChange
        | ActionId::EditChange
        | ActionId::Abandon
        | ActionId::Mark
        | ActionId::ClearMarks => HelpGroup::Mutations,
        ActionId::OpenCommandHistory
        | ActionId::OpenCommandDetails
        | ActionId::CopyCommand
        | ActionId::OpenOperation
        | ActionId::OpenOperationLog
        | ActionId::Undo
        | ActionId::Redo => HelpGroup::Recovery,
        ActionId::CommandMode
        | ActionId::Refresh
        | ActionId::UpdateStale
        | ActionId::CloseHelp
        | ActionId::Quit => HelpGroup::Session,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HelpGroup {
    Navigation,
    Views,
    Mutations,
    Recovery,
    Session,
}

impl HelpGroup {
    const fn label(self) -> &'static str {
        match self {
            Self::Navigation => "Move and find",
            Self::Views => "Open and inspect",
            Self::Mutations => "Change actions",
            Self::Recovery => "History and recovery",
            Self::Session => "Session",
        }
    }
}

impl ActionId {
    const fn label(self) -> &'static str {
        match self {
            Self::Move => "Move selection",
            Self::LineScroll => "Scroll line",
            Self::PageDown => "Page down",
            Self::PageUp => "Page up",
            Self::JumpTop => "Jump to top",
            Self::JumpBottom => "Jump to bottom",
            Self::Mark => "Mark revision",
            Self::ClearMarks => "Clear marks",
            Self::Expand => "Expand change",
            Self::Collapse => "Collapse change",
            Self::OpenShow => "Open show",
            Self::OpenDiff => "Open diff",
            Self::OpenLog => "Open log",
            Self::OpenDescribe => "Describe revision",
            Self::OpenEvolog => "Open evolog",
            Self::OpenStatus => "Open status",
            Self::OpenOperation => "Open operation",
            Self::OpenOperationLog => "Open operation log",
            Self::OpenCommandHistory => "Open command history",
            Self::OpenCommandDetails => "Open command details",
            Self::CopyCommand => "Copy command",
            Self::CommandMode => "Run jj command",
            Self::NewChange => "New change",
            Self::EditChange => "Edit change",
            Self::Abandon => "Abandon revision",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::UpdateStale => "Update stale",
            Self::ViewOptions => "View options",
            Self::Refresh => "Refresh",
            Self::SwitchLogCommand => "Switch log command",
            Self::OpenFileList => "Open file list",
            Self::File => "Move file",
            Self::Hunk => "Move hunk",
            Self::FoldFile => "Fold file",
            Self::FoldAll => "Fold all files",
            Self::FoldHunk => "Fold hunk",
            Self::HorizontalScroll => "Horizontal scroll",
            Self::Search => "Search output",
            Self::ReturnToLog => "Return to log",
            Self::ReturnBack => "Return back",
            Self::CloseHelp => "Close help",
            Self::Quit => "Quit / exit",
        }
    }
}

/// Command or interaction family used by contextual help.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandFamily {
    /// Commands and actions related to `jj log`.
    JjLog,
    /// Commands and actions related to `jj diff`.
    JjDiff,
    /// Commands and actions related to `jj describe`.
    JjDescribe,
    /// Commands and actions related to `jj new`.
    JjNew,
    /// Commands and actions related to `jj edit`.
    JjEdit,
    /// Commands and actions related to `jj evolog`.
    JjEvolog,
    /// Commands and actions related to `jj show`.
    JjShow,
    /// Commands and actions related to `jj status`.
    JjStatus,
    /// Commands and actions related to `jj workspace`.
    JjWorkspace,
    /// Commands and actions related to `jj operation`.
    JjOperation,
    /// Command history and transcripts.
    History,
    /// In-view output search.
    Search,
    /// Refreshing the active view.
    Refresh,
    /// Selection, scrolling, and viewport movement.
    Navigation,
    /// Ordered revision mark actions.
    Mark,
    /// Diff folding actions.
    Fold,
    /// Diff file-section movement.
    File,
    /// Diff hunk movement.
    Hunk,
    /// View-scoped display and template options.
    ViewOptions,
    /// User-entered `jj` command mode.
    CommandMode,
    /// Help controls.
    Help,
    /// Quitting the application.
    Quit,
}

impl CommandFamily {
    const fn label(self) -> &'static str {
        match self {
            Self::JjLog => "jj log",
            Self::JjDiff => "jj diff",
            Self::JjDescribe => "jj describe",
            Self::JjNew => "jj new",
            Self::JjEdit => "jj edit",
            Self::JjEvolog => "jj evolog",
            Self::JjShow => "jj show",
            Self::JjStatus => "jj status",
            Self::JjWorkspace => "jj workspace",
            Self::JjOperation => "jj operation",
            Self::History => "history",
            Self::Search => "search",
            Self::Refresh => "refresh",
            Self::Navigation => "navigation",
            Self::Mark => "mark",
            Self::Fold => "fold",
            Self::File => "file",
            Self::Hunk => "hunk",
            Self::ViewOptions => "view options",
            Self::CommandMode => "jj command",
            Self::Help => "help",
            Self::Quit => "quit",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KeyBinding {
    action: ActionId,
    keys: &'static str,
    help: &'static str,
    help_group: HelpGroup,
    command_family: Option<CommandFamily>,
    aliases: &'static [&'static str],
    hotbar: Option<&'static str>,
    hotbar_rank: Option<u8>,
    show_in_help: bool,
    show_in_discovery: bool,
}

impl KeyBinding {
    const fn new(action: ActionId, keys: &'static str, help: &'static str) -> Self {
        Self {
            action,
            keys,
            help,
            help_group: default_help_group(action),
            command_family: None,
            aliases: &[],
            hotbar: None,
            hotbar_rank: None,
            show_in_help: true,
            show_in_discovery: true,
        }
    }

    const fn with_family(mut self, command_family: CommandFamily) -> Self {
        self.command_family = Some(command_family);
        self
    }

    const fn with_aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }

    const fn with_hotbar(mut self, rank: u8, hotbar: &'static str) -> Self {
        self.hotbar = Some(hotbar);
        self.hotbar_rank = Some(rank);
        self
    }

    const fn hotbar_only(mut self) -> Self {
        self.show_in_help = false;
        self
    }

    fn help_line(self, key_width: usize) -> String {
        format!("  {:<key_width$} {}", self.keys, self.help)
    }
}

const LOG_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::OpenShow, "enter", "open change / drill into ~")
        .with_family(CommandFamily::JjShow)
        .with_aliases(&["details", "inspect"])
        .with_hotbar(4, "enter open"),
    KeyBinding::new(ActionId::OpenDiff, "d", "open selected-change diff")
        .with_family(CommandFamily::JjDiff)
        .with_hotbar(5, "d diff"),
    KeyBinding::new(ActionId::OpenEvolog, "v", "open selected-change evolog")
        .with_family(CommandFamily::JjEvolog)
        .with_aliases(&[
            "evolution",
            "history",
            "change",
            "version",
            "selected change",
        ])
        .with_hotbar(7, "v evolog"),
    KeyBinding::new(ActionId::OpenStatus, "s", "open repository status")
        .with_family(CommandFamily::JjStatus)
        .with_hotbar(8, "s status"),
    KeyBinding::new(ActionId::OpenOperationLog, "o", "open operation log")
        .with_family(CommandFamily::JjOperation)
        .with_aliases(&["operation", "op log", "undo", "redo", "recovery"])
        .with_hotbar(10, "o ops"),
    KeyBinding::new(ActionId::OpenDescribe, "m", "describe selected revision")
        .with_family(CommandFamily::JjDescribe)
        .with_aliases(&["message", "description", "mutation", "preview"])
        .with_hotbar(6, "m describe"),
    KeyBinding::new(ActionId::NewChange, "n", "preview jj new")
        .with_family(CommandFamily::JjNew)
        .with_aliases(&["new", "change", "parent", "mutation", "preview"])
        .with_hotbar(15, "n new"),
    KeyBinding::new(ActionId::EditChange, "e", "preview jj edit")
        .with_family(CommandFamily::JjEdit)
        .with_aliases(&["edit", "checkout", "working copy", "mutation", "preview"])
        .with_hotbar(17, "e edit"),
    KeyBinding::new(ActionId::Abandon, "a", "preview jj abandon")
        .with_family(CommandFamily::JjOperation)
        .with_aliases(&["abandon", "delete", "destructive", "mutation", "preview"])
        .with_hotbar(16, "a abandon"),
    KeyBinding::new(ActionId::Undo, "u", "preview jj undo")
        .with_family(CommandFamily::JjOperation)
        .with_aliases(&["undo", "operation", "recovery"])
        .with_hotbar(12, "u undo"),
    KeyBinding::new(ActionId::Redo, "U", "preview jj redo")
        .with_family(CommandFamily::JjOperation)
        .with_aliases(&["redo", "operation", "recovery"])
        .with_hotbar(14, "U redo"),
    KeyBinding::new(ActionId::Mark, "space", "mark/unmark selected revision")
        .with_family(CommandFamily::Mark)
        .with_aliases(&["selected", "revision", "toggle"])
        .with_hotbar(9, "space mark"),
    KeyBinding::new(
        ActionId::ClearMarks,
        "c",
        "clear revision marks when marks exist",
    )
    .with_family(CommandFamily::Mark)
    .with_aliases(&["clear", "unmark", "selected", "revision"])
    .with_hotbar(11, "c clear"),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "open command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::CommandMode, ":", "run jj command")
        .with_family(CommandFamily::CommandMode)
        .with_aliases(&["command", "prompt", "colon", "jj"]),
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "template", "jj log"])
        .with_hotbar(18, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(3, "r refresh"),
    KeyBinding::new(ActionId::SwitchLogCommand, "H / L", "home command / jj log")
        .with_family(CommandFamily::JjLog)
        .with_aliases(&["home", "current screen"])
        .with_hotbar(2, "H home  L log"),
    KeyBinding::new(ActionId::Move, "↑/↓, j/k", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "current row"])
        .with_hotbar(13, "j/k move"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::PageDown, "PgDn, Ctrl-f", "page down")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"]),
    KeyBinding::new(ActionId::PageUp, "PgUp, Ctrl-b", "page up")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"]),
    KeyBinding::new(ActionId::JumpTop, "Home, g", "jump to top")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::JumpBottom, "End, G", "jump to bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Expand, "→, l", "expand change / drill into ~")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Collapse, "←, h", "collapse selected change")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::CloseHelp, "?, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(19, "q quit")
        .hotbar_only(),
];

const DIFF_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::OpenFileList, "f", "open file list")
        .with_family(CommandFamily::File)
        .with_aliases(&["files", "paths", "jump", "file list"])
        .with_hotbar(5, "f files"),
    KeyBinding::new(ActionId::File, "[ / ]", "previous/next file").with_family(CommandFamily::File),
    KeyBinding::new(ActionId::Hunk, "{ / }", "previous/next hunk").with_family(CommandFamily::Hunk),
    KeyBinding::new(ActionId::FoldFile, "h / l", "fold/unfold current file")
        .with_family(CommandFamily::Fold),
    KeyBinding::new(
        ActionId::FoldAll,
        "Ctrl-left/right",
        "fold/unfold all files",
    )
    .with_family(CommandFamily::Fold),
    KeyBinding::new(ActionId::FoldHunk, "- / +", "fold/unfold current hunk")
        .with_family(CommandFamily::Fold),
    KeyBinding::new(ActionId::HorizontalScroll, "< / >", "horizontal scroll")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Search, "/, n, N", "search, next, previous")
        .with_family(CommandFamily::Search)
        .with_aliases(&["find", "filter"]),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "open command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::CommandMode, ":", "run jj command")
        .with_family(CommandFamily::CommandMode)
        .with_aliases(&["command", "prompt", "colon", "jj"]),
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "display"])
        .with_hotbar(2, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(3, "r refresh"),
    KeyBinding::new(ActionId::Move, "↑/↓, j/k", "scroll one line")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(4, "j/k line"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::PageDown, "Space, PgDn, Ctrl-f", "page down")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(6, "space page"),
    KeyBinding::new(ActionId::PageUp, "PgUp, Ctrl-b", "page up")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::JumpTop, "Home, g", "jump to top")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::JumpBottom, "End, G", "jump to bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::ReturnToLog, "H / L", "go back")
        .with_family(CommandFamily::JjLog)
        .with_aliases(&["back"]),
    KeyBinding::new(ActionId::CloseHelp, "?, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(7, "q quit")
        .hotbar_only(),
];

const INSPECTION_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Search, "/, n, N", "search, next, previous")
        .with_family(CommandFamily::Search)
        .with_aliases(&["find", "filter", "details", "status"]),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "open command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::CommandMode, ":", "run jj command")
        .with_family(CommandFamily::CommandMode)
        .with_aliases(&["command", "prompt", "colon", "jj"]),
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "display"])
        .with_hotbar(2, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(3, "r refresh"),
    KeyBinding::new(ActionId::Move, "↑/↓, j/k", "scroll one line")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(4, "j/k line"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::PageDown, "Space, PgDn, Ctrl-f", "page down")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(5, "space page"),
    KeyBinding::new(ActionId::PageUp, "PgUp, Ctrl-b", "page up")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::JumpTop, "Home, g", "jump to top")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::JumpBottom, "End, G", "jump to bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::ReturnToLog, "H / L", "go back")
        .with_family(CommandFamily::JjLog)
        .with_aliases(&["back"]),
    KeyBinding::new(ActionId::CloseHelp, "?, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(6, "q quit")
        .hotbar_only(),
];

const WORKSPACES_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::OpenLog, "l", "open selected workspace log")
        .with_family(CommandFamily::JjLog)
        .with_aliases(&["log", "workspace", "selected"])
        .with_hotbar(3, "l log"),
    KeyBinding::new(
        ActionId::OpenStatus,
        "enter, s",
        "open selected workspace status",
    )
    .with_family(CommandFamily::JjStatus)
    .with_aliases(&["status", "workspace", "selected"])
    .with_hotbar(4, "s status"),
    KeyBinding::new(ActionId::OpenDiff, "d", "open selected workspace diff")
        .with_family(CommandFamily::JjDiff)
        .with_aliases(&["diff", "workspace", "selected"])
        .with_hotbar(5, "d diff"),
    KeyBinding::new(
        ActionId::UpdateStale,
        "u",
        "update selected stale workspace",
    )
    .with_family(CommandFamily::JjWorkspace)
    .with_aliases(&["update", "stale", "workspace", "selected", "refresh"])
    .with_hotbar(7, "u update-stale"),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "open command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::CommandMode, ":", "run jj command")
        .with_family(CommandFamily::CommandMode)
        .with_aliases(&["command", "prompt", "colon", "jj"]),
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "display"])
        .with_hotbar(8, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh workspaces")
        .with_family(CommandFamily::Refresh)
        .with_aliases(&["reload", "workspace"])
        .with_hotbar(2, "r refresh"),
    KeyBinding::new(ActionId::Move, "↑/↓, j/k", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "workspace", "current row"])
        .with_hotbar(5, "j/k move"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(
        ActionId::ReturnBack,
        "Backspace, Esc, H/L",
        "return to previous view",
    )
    .with_family(CommandFamily::Navigation)
    .with_aliases(&["back", "return", "previous"])
    .with_hotbar(9, "Esc back"),
    KeyBinding::new(ActionId::CloseHelp, "?, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(10, "q quit")
        .hotbar_only(),
];

const COMMAND_HISTORY_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(
        ActionId::OpenCommandDetails,
        "enter",
        "open command details",
    )
    .with_family(CommandFamily::History)
    .with_aliases(&["details", "output", "stdout", "stderr", "argv"])
    .with_hotbar(2, "enter details"),
    KeyBinding::new(
        ActionId::OpenOperation,
        "o",
        "open operation or operation log",
    )
    .with_family(CommandFamily::JjOperation)
    .with_aliases(&["operation", "op log", "recovery", "selected command"])
    .with_hotbar(6, "o operation"),
    KeyBinding::new(ActionId::CopyCommand, "y", "copy selected command")
        .with_family(CommandFamily::History)
        .with_aliases(&["copy", "clipboard", "command", "argv"])
        .with_hotbar(7, "y copy"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh history")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(5, "r refresh"),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "refresh command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::CommandMode, ":", "run jj command")
        .with_family(CommandFamily::CommandMode)
        .with_aliases(&["command", "prompt", "colon", "jj"]),
    KeyBinding::new(ActionId::Move, "↑/↓, j/k", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "command", "current row"])
        .with_hotbar(3, "j/k move"),
    KeyBinding::new(ActionId::PageDown, "Space, PgDn, Ctrl-f", "page down")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"])
        .with_hotbar(4, "space page"),
    KeyBinding::new(ActionId::PageUp, "PgUp, Ctrl-b", "page up")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"]),
    KeyBinding::new(ActionId::JumpTop, "Home, g", "jump to top")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::JumpBottom, "End, G", "jump to bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(
        ActionId::ReturnBack,
        "Backspace, Esc",
        "return to previous view",
    )
    .with_family(CommandFamily::Navigation)
    .with_aliases(&["back", "return", "previous"])
    .with_hotbar(8, "Esc back"),
    KeyBinding::new(ActionId::CloseHelp, "?, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(9, "q quit")
        .hotbar_only(),
];

const OPERATION_LOG_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::OpenShow, "enter", "open selected operation show")
        .with_family(CommandFamily::JjOperation)
        .with_aliases(&["show", "details", "inspect", "operation"])
        .with_hotbar(2, "enter show"),
    KeyBinding::new(ActionId::OpenDiff, "d", "open selected operation diff")
        .with_family(CommandFamily::JjOperation)
        .with_aliases(&["diff", "operation", "selected"])
        .with_hotbar(3, "d diff"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh operation log")
        .with_family(CommandFamily::Refresh)
        .with_aliases(&["reload", "operation"])
        .with_hotbar(6, "r refresh"),
    KeyBinding::new(ActionId::CommandMode, ":", "run jj command")
        .with_family(CommandFamily::CommandMode)
        .with_aliases(&["command", "prompt", "colon", "jj"]),
    KeyBinding::new(ActionId::Move, "↑/↓, j/k", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "operation", "current row"])
        .with_hotbar(4, "j/k move"),
    KeyBinding::new(ActionId::PageDown, "Space, PgDn, Ctrl-f", "page down")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"])
        .with_hotbar(5, "space page"),
    KeyBinding::new(ActionId::PageUp, "PgUp, Ctrl-b", "page up")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"]),
    KeyBinding::new(ActionId::JumpTop, "Home, g", "jump to top")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::JumpBottom, "End, G", "jump to bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(
        ActionId::ReturnBack,
        "Backspace, Esc",
        "return to previous view",
    )
    .with_family(CommandFamily::Navigation)
    .with_aliases(&["back", "return", "previous"])
    .with_hotbar(7, "Esc back"),
    KeyBinding::new(ActionId::CloseHelp, "?, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(8, "q quit")
        .hotbar_only(),
];

/// Returns hotbar text for the current binding context.
pub fn hotbar(context: BindingContext) -> String {
    hotbar_items(context)
        .into_iter()
        .map(|item| item.label)
        .collect::<Vec<_>>()
        .join("  ")
}

/// Returns hotbar text that fits the available row width.
pub fn adaptive_hotbar(context: BindingContext, width: u16) -> String {
    let full_hotbar = hotbar(context);
    let width = usize::from(width);
    if width == 0 {
        return String::new();
    }
    if visible_width(&full_hotbar) <= width {
        return full_hotbar;
    }

    let help = pinned_label(context, CommandFamily::Help).unwrap_or("?");
    let quit = pinned_label(context, CommandFamily::Quit).unwrap_or("q");

    let optional = hotbar_items(context)
        .into_iter()
        .filter(|item| {
            !matches!(
                item.command_family,
                Some(CommandFamily::Help | CommandFamily::Quit)
            )
        })
        .collect::<Vec<_>>();
    let mut selected = Vec::new();
    for index in 0..optional.len() {
        selected.push(optional[index].label);
        let omitted = optional.len().saturating_sub(selected.len());
        let candidate = narrowed_hotbar(help, &selected, omitted, quit);
        if visible_width(&candidate) > width {
            selected.pop();
            break;
        }
    }

    let omitted = optional.len().saturating_sub(selected.len());
    let candidate = narrowed_hotbar(help, &selected, omitted, quit);
    if visible_width(&candidate) <= width {
        return candidate;
    }

    let candidate = narrowed_hotbar(help, &[], optional.len(), quit);
    if visible_width(&candidate) <= width {
        return candidate;
    }

    let candidate = join_hotbar_labels(&["?", "...", "q"]);
    if omitted > 0 && visible_width(&candidate) <= width {
        return candidate;
    }

    let candidate = join_hotbar_labels(&["?", "q"]);
    if visible_width(&candidate) <= width {
        return candidate;
    }

    if width >= visible_width("? q") {
        return "? q".to_owned();
    }

    fit_label(help, width)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HotbarItem {
    label: &'static str,
    command_family: Option<CommandFamily>,
}

fn hotbar_items(context: BindingContext) -> Vec<HotbarItem> {
    let mut hotbar = bindings(context)
        .iter()
        .filter_map(|binding| {
            let label = binding.hotbar?;
            let rank = binding.hotbar_rank?;
            Some((rank, binding.command_family, label))
        })
        .collect::<Vec<_>>();
    hotbar.sort_by_key(|(rank, _, _)| *rank);
    hotbar
        .into_iter()
        .map(|(_, command_family, label)| HotbarItem {
            label,
            command_family,
        })
        .collect()
}

fn pinned_label(context: BindingContext, family: CommandFamily) -> Option<&'static str> {
    hotbar_items(context)
        .into_iter()
        .find(|item| item.command_family == Some(family))
        .map(|item| item.label)
}

fn narrowed_hotbar(
    help: &'static str,
    selected: &[&'static str],
    omitted: usize,
    quit: &'static str,
) -> String {
    let mut labels = Vec::with_capacity(selected.len().saturating_add(3));
    labels.push(help);
    labels.extend_from_slice(selected);
    if omitted > 0 {
        labels.push("...");
    }
    labels.push(quit);
    join_hotbar_labels(&labels)
}

fn join_hotbar_labels(labels: &[&str]) -> String {
    labels.join("  ")
}

fn visible_width(text: &str) -> usize {
    text.chars().count()
}

fn fit_label(label: &str, width: usize) -> String {
    label.chars().take(width).collect()
}

/// Returns the help overlay title for the current binding context.
pub const fn help_title(context: BindingContext) -> &'static str {
    match context {
        BindingContext::Log => "Log keys",
        BindingContext::Diff => "Diff keys",
        BindingContext::Inspection => "Inspection keys",
        BindingContext::Workspaces => "Workspaces keys",
        BindingContext::CommandHistory => "Command History keys",
        BindingContext::OperationLog => "Operation Log keys",
    }
}

/// Returns generated help lines for the current binding context.
pub fn help_lines(context: BindingContext) -> Vec<String> {
    let visible = bindings(context)
        .iter()
        .filter(|binding| binding.show_in_help)
        .copied()
        .collect::<Vec<_>>();
    let key_width = visible
        .iter()
        .map(|binding| visible_width(binding.keys))
        .max()
        .unwrap_or(0);
    let mut lines = vec![
        "Contextual help for the current screen.".to_owned(),
        String::new(),
    ];

    let mut first_group = true;
    for group in help_groups_for_context(context) {
        let group_bindings = visible
            .iter()
            .filter(|binding| binding.help_group == *group)
            .collect::<Vec<_>>();
        if group_bindings.is_empty() {
            continue;
        }

        if !first_group {
            lines.push(String::new());
        }
        first_group = false;
        lines.push(format!("{}:", group.label()));
        lines.extend(
            group_bindings
                .into_iter()
                .map(|binding| binding.help_line(key_width)),
        );
    }

    lines
}

/// A contextual help row derived from visible keymap metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscoveryRow {
    action_id: ActionId,
    /// Key text shown in the first column.
    pub keys: &'static str,
    /// Short action label shown in the second column.
    pub action: &'static str,
    /// Help text used for legacy help and future detail displays.
    pub help: &'static str,
    /// Screen context where the row applies.
    pub context: BindingContext,
    /// Optional command or interaction family.
    pub command_family: Option<CommandFamily>,
    help_group: HelpGroup,
    aliases: &'static [&'static str],
}

impl DiscoveryRow {
    /// Returns the context label used for rendering.
    #[must_use]
    pub const fn context_label(self) -> &'static str {
        context_label(self.context)
    }

    /// Returns the command family label used for rendering.
    #[must_use]
    pub const fn command_family_label(self) -> Option<&'static str> {
        match self.command_family {
            Some(family) => Some(family.label()),
            None => None,
        }
    }
}

/// Returns all discoverable rows for the current context.
#[must_use]
pub fn discovery_rows(context: BindingContext) -> Vec<DiscoveryRow> {
    bindings(context)
        .iter()
        .filter(|binding| binding.show_in_discovery)
        .map(|binding| DiscoveryRow {
            action_id: binding.action,
            keys: binding.keys,
            action: binding.action.label(),
            help: binding.help,
            context,
            command_family: binding.command_family,
            help_group: binding.help_group,
            aliases: binding.aliases,
        })
        .collect()
}

/// Number of command rows shown in the discovery popup before scrolling.
pub const DISCOVERY_VISIBLE_ROWS: usize = 12;

const DISCOVERY_DEFAULT_WIDTH: usize = 80;
const DISCOVERY_COLUMN_GUTTER: usize = 6;
const DISCOVERY_TWO_COLUMN_WIDTH: usize = 80;
const DISCOVERY_WIDE_VISIBLE_ROWS: usize = 34;

/// Formats command-discovery popup lines for the current context and scroll offset.
#[must_use]
pub fn discovery_lines(context: BindingContext, _query: &str, scroll_offset: usize) -> Vec<String> {
    discovery_lines_for_width(context, "", scroll_offset, DISCOVERY_DEFAULT_WIDTH)
}

/// Formats command-discovery popup lines for the current context, scroll offset, and width.
#[must_use]
pub fn discovery_lines_for_width(
    context: BindingContext,
    _query: &str,
    scroll_offset: usize,
    width: usize,
) -> Vec<String> {
    discovery_lines_for_width_and_rows(
        context,
        "",
        scroll_offset,
        width,
        default_discovery_visible_rows(width),
    )
}

/// Formats command-discovery popup lines for the current context, width, and body-row budget.
#[must_use]
pub fn discovery_lines_for_width_and_rows(
    context: BindingContext,
    _query: &str,
    scroll_offset: usize,
    width: usize,
    visible_rows: usize,
) -> Vec<String> {
    let rows = discovery_rows(context);
    let body_lines = discovery_body_lines(context, &rows, width);
    let body_line_count = body_lines.len();
    let visible_body_lines = discovery_visible_body_lines(visible_rows);
    let (start, end) = visible_discovery_range(body_line_count, scroll_offset, visible_body_lines);
    let mut lines = body_lines[start..end].to_vec();
    lines.extend(discovery_footer_lines(width, start, end, body_line_count));
    lines
}

fn discovery_body_lines(
    context: BindingContext,
    rows: &[DiscoveryRow],
    width: usize,
) -> Vec<String> {
    let mut lines = Vec::new();
    let spaced_groups = discovery_uses_group_spacing(width);
    let key_width = discovery_key_width_for_width(rows, width);
    let column_width = discovery_global_column_width(rows, key_width);
    let column_count = discovery_column_count(width, column_width);
    for group in help_groups_for_context(context) {
        let group_rows = rows
            .iter()
            .filter(|row| row.help_group == *group)
            .copied()
            .collect::<Vec<_>>();
        if group_rows.is_empty() {
            continue;
        }

        if spaced_groups && !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("{}:", group.label()));
        lines.extend(
            group_rows.chunks(column_count).map(|chunk| {
                discovery_row_line(chunk, column_count, column_width, key_width, width)
            }),
        );
    }
    lines
}

const fn discovery_uses_group_spacing(width: usize) -> bool {
    width >= 120
}

const fn discovery_column_count(width: usize, column_width: usize) -> usize {
    if width >= DISCOVERY_TWO_COLUMN_WIDTH
        && column_width
            .saturating_mul(2)
            .saturating_add(DISCOVERY_COLUMN_GUTTER)
            <= width
    {
        2
    } else {
        1
    }
}

fn discovery_global_column_width(rows: &[DiscoveryRow], key_width: usize) -> usize {
    rows.iter()
        .map(|row| discovery_cell_width(*row, key_width))
        .max()
        .unwrap_or(0)
}

fn discovery_row_line(
    rows: &[DiscoveryRow],
    column_count: usize,
    column_width: usize,
    key_width: usize,
    width: usize,
) -> String {
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            let cell = discovery_cell(*row, key_width, width);
            if index + 1 == column_count || index + 1 == rows.len() {
                cell
            } else {
                format!("{cell:<column_width$}")
            }
        })
        .collect::<Vec<_>>()
        .join(&" ".repeat(DISCOVERY_COLUMN_GUTTER))
}

fn discovery_cell(row: DiscoveryRow, key_width: usize, width: usize) -> String {
    let full_action = discovery_action_label(row);
    let compact_action = row.action.to_owned();
    let full = discovery_cell_with_action(row, key_width, &full_action);
    if visible_width(&full) <= width {
        return full;
    }

    discovery_cell_with_action(row, key_width, &compact_action)
}

fn discovery_cell_with_action(row: DiscoveryRow, key_width: usize, action: &str) -> String {
    format!("  {:<key_width$}  {action}", row.keys)
}

fn discovery_key_width(rows: &[DiscoveryRow]) -> usize {
    rows.iter()
        .map(|row| visible_width(row.keys))
        .max()
        .unwrap_or(0)
}

fn discovery_key_width_for_width(rows: &[DiscoveryRow], width: usize) -> usize {
    let key_width = discovery_key_width(rows);
    if width >= DISCOVERY_TWO_COLUMN_WIDTH {
        return key_width;
    }

    key_width.min(12)
}

fn discovery_cell_width(row: DiscoveryRow, key_width: usize) -> usize {
    4_usize
        .saturating_add(key_width)
        .saturating_add(visible_width(&discovery_action_label(row)))
}

fn discovery_action_label(row: DiscoveryRow) -> String {
    // Prefer one clear label per row. A concrete jj command teaches the command shape better than
    // repeating it as `jj describe (Describe revision)`; app-only actions keep the human action
    // label.
    discovery_command_display_label(row)
        .map_or_else(|| row.action.to_owned(), std::borrow::ToOwned::to_owned)
}

fn discovery_command_display_label(row: DiscoveryRow) -> Option<&'static str> {
    match (row.command_family, row.action_id) {
        (Some(CommandFamily::JjOperation), ActionId::OpenOperationLog) => Some("jj op log"),
        (Some(CommandFamily::JjOperation), ActionId::OpenShow) => Some("jj op show"),
        (Some(CommandFamily::JjOperation), ActionId::OpenDiff) => Some("jj op diff"),
        (Some(CommandFamily::JjOperation), ActionId::Undo) => Some("jj undo"),
        (Some(CommandFamily::JjOperation), ActionId::Redo) => Some("jj redo"),
        (Some(CommandFamily::JjOperation), ActionId::Abandon) => Some("jj abandon"),
        (Some(CommandFamily::JjOperation) | None, _) => None,
        (Some(family), _) => {
            let label = family.label();
            if label.starts_with("jj ") {
                Some(label)
            } else {
                None
            }
        }
    }
}

fn discovery_footer_lines(width: usize, start: usize, end: usize, row_count: usize) -> Vec<String> {
    let position = discovery_position_line(start, end, row_count);
    if row_count <= end.saturating_sub(start) {
        return Vec::new();
    }

    let scroll_controls = "j/k scroll";
    let full = format!("{position}   {scroll_controls}");
    if visible_width(&full) <= width {
        return vec![full];
    }

    vec![position, scroll_controls.to_owned()]
}

fn visible_discovery_range(
    row_count: usize,
    selected: usize,
    visible_rows: usize,
) -> (usize, usize) {
    if row_count <= visible_rows {
        return (0, row_count);
    }

    let start = selected.min(discovery_scroll_limit_for_len(row_count, visible_rows));
    (start, start + visible_rows)
}

fn discovery_position_line(start: usize, end: usize, row_count: usize) -> String {
    let more_above = start > 0;
    let more_below = end < row_count;
    let affordance = match (more_above, more_below) {
        (true, true) => "more above/below",
        (true, false) => "more above",
        (false, true) => "more below",
        (false, false) => "",
    };
    if affordance.is_empty() {
        format!(
            "showing lines {}-{} of {}",
            start.saturating_add(1),
            end,
            row_count
        )
    } else {
        format!(
            "showing lines {}-{} of {}  {affordance}",
            start.saturating_add(1),
            end,
            row_count
        )
    }
}

/// Returns the number of visible help rows for clamping scroll state.
#[must_use]
pub fn discovery_len(context: BindingContext, query: &str) -> usize {
    let _ = query;
    discovery_rows(context).len()
}

/// Returns the highest scroll offset for the contextual help overlay.
#[must_use]
pub fn discovery_scroll_limit(context: BindingContext, query: &str) -> usize {
    let _ = query;
    let rows = discovery_rows(context);
    let body_line_count = discovery_body_lines(context, &rows, 40).len();
    discovery_scroll_limit_for_len(
        body_line_count,
        discovery_visible_body_lines(DISCOVERY_VISIBLE_ROWS),
    )
}

const fn default_discovery_visible_rows(width: usize) -> usize {
    if width >= DISCOVERY_TWO_COLUMN_WIDTH {
        DISCOVERY_WIDE_VISIBLE_ROWS
    } else {
        DISCOVERY_VISIBLE_ROWS
    }
}

const fn discovery_visible_body_lines(visible_rows: usize) -> usize {
    visible_rows
}

const fn discovery_scroll_limit_for_len(row_count: usize, visible_rows: usize) -> usize {
    row_count.saturating_sub(visible_rows)
}

const fn bindings(context: BindingContext) -> &'static [KeyBinding] {
    match context {
        BindingContext::Log => LOG_BINDINGS,
        BindingContext::Diff => DIFF_BINDINGS,
        BindingContext::Inspection => INSPECTION_BINDINGS,
        BindingContext::Workspaces => WORKSPACES_BINDINGS,
        BindingContext::CommandHistory => COMMAND_HISTORY_BINDINGS,
        BindingContext::OperationLog => OPERATION_LOG_BINDINGS,
    }
}

const fn context_label(context: BindingContext) -> &'static str {
    match context {
        BindingContext::Log => "log",
        BindingContext::Diff => "diff",
        BindingContext::Inspection => "inspection",
        BindingContext::Workspaces => "workspaces",
        BindingContext::CommandHistory => "history",
        BindingContext::OperationLog => "operation log",
    }
}

const fn help_groups_for_context(context: BindingContext) -> &'static [HelpGroup] {
    match context {
        BindingContext::Log => &[
            HelpGroup::Views,
            HelpGroup::Navigation,
            HelpGroup::Mutations,
            HelpGroup::Recovery,
            HelpGroup::Session,
        ],
        BindingContext::Diff
        | BindingContext::Inspection
        | BindingContext::Workspaces
        | BindingContext::OperationLog => &[
            HelpGroup::Views,
            HelpGroup::Navigation,
            HelpGroup::Recovery,
            HelpGroup::Session,
        ],
        BindingContext::CommandHistory => &[
            HelpGroup::Recovery,
            HelpGroup::Navigation,
            HelpGroup::Session,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_help_header(lines: &[String]) {
        assert_eq!(lines[0], "Contextual help for the current screen.");
    }

    fn assert_ordered_lines(lines: &[String], expected: &[&str]) {
        let mut start = 0;
        for expected_line in expected {
            let relative_index = lines[start..]
                .iter()
                .position(|line| line == expected_line)
                .unwrap_or_else(|| panic!("missing ordered line {expected_line:?} in {lines:?}"));
            start += relative_index + 1;
        }
    }

    #[test]
    fn log_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Log),
            "? help  H home  L log  r refresh  enter open  d diff  m describe  v evolog  s status  space mark  o ops  c clear  u undo  j/k move  U redo  n new  a abandon  e edit  V options  q quit"
        );
    }

    #[test]
    fn diff_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Diff),
            "? help  V options  r refresh  j/k line  f files  space page  q quit"
        );
    }

    #[test]
    fn inspection_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Inspection),
            "? help  V options  r refresh  j/k line  space page  q quit"
        );
    }

    #[test]
    fn command_history_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::CommandHistory),
            "? help  enter details  j/k move  space page  r refresh  o operation  y copy  Esc back  q quit"
        );
    }

    #[test]
    fn operation_log_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::OperationLog),
            "? help  enter show  d diff  j/k move  space page  r refresh  Esc back  q quit"
        );
    }

    #[test]
    fn workspaces_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Workspaces),
            "? help  r refresh  l log  s status  d diff  j/k move  u update-stale  V options  Esc back  q quit"
        );
    }

    #[test]
    fn adaptive_log_hotbar_keeps_full_text_when_it_fits() {
        assert_eq!(
            adaptive_hotbar(BindingContext::Log, 200),
            hotbar(BindingContext::Log)
        );
    }

    #[test]
    fn adaptive_log_hotbar_keeps_primary_commands_at_betamax_width() {
        let status = adaptive_hotbar(BindingContext::Log, 104);

        assert!(status.chars().count() <= 104);
        assert!(status.contains("? help"));
        assert!(status.contains("q quit"));
        assert!(status.contains("enter open"));
        assert!(status.contains("d diff"));
        assert!(status.contains("m describe"));
        assert!(status.contains("v evolog"));
        assert!(status.contains("s status"));
        assert!(status.contains("..."));
        assert!(!status.contains("space mark"));
        assert!(!status.contains("n new"));
        assert!(!status.contains("a abandon"));
        assert!(!status.contains("e edit"));
        assert!(!status.contains("j/k move"));
    }

    #[test]
    fn adaptive_log_hotbar_preserves_rank_order() {
        let status = adaptive_hotbar(BindingContext::Log, 75);

        assert_eq!(
            status,
            "? help  H home  L log  r refresh  enter open  d diff  ...  q quit"
        );
    }

    #[test]
    fn adaptive_hotbar_prefers_help_at_tiny_width() {
        assert_eq!(adaptive_hotbar(BindingContext::Log, 9), "?  ...  q");
        assert_eq!(adaptive_hotbar(BindingContext::Log, 4), "?  q");
        assert_eq!(adaptive_hotbar(BindingContext::Log, 3), "? q");
        assert_eq!(adaptive_hotbar(BindingContext::Log, 1), "?");
        assert_eq!(adaptive_hotbar(BindingContext::Log, 0), "");
    }

    #[test]
    fn adaptive_diff_hotbar_keeps_action_affordances() {
        let status = adaptive_hotbar(BindingContext::Diff, 51);

        assert!(status.chars().count() <= 51);
        assert_eq!(
            status,
            "? help  V options  r refresh  j/k line  ...  q quit"
        );
    }

    #[test]
    fn adaptive_inspection_hotbar_matches_diff_policy() {
        let status = adaptive_hotbar(BindingContext::Inspection, 41);

        assert!(status.chars().count() <= 41);
        assert_eq!(status, "? help  V options  r refresh  ...  q quit");
    }

    #[test]
    fn adaptive_hotbars_never_exceed_requested_width() {
        for context in [
            BindingContext::Log,
            BindingContext::Diff,
            BindingContext::Inspection,
            BindingContext::Workspaces,
            BindingContext::CommandHistory,
            BindingContext::OperationLog,
        ] {
            for width in 0..=140 {
                let status = adaptive_hotbar(context, width);
                assert!(
                    status.chars().count() <= usize::from(width),
                    "{context:?} width {width}: {status}"
                );
            }
        }
    }

    #[test]
    fn log_help_lines_group_contextual_commands() {
        let lines = help_lines(BindingContext::Log);

        assert_help_header(&lines);
        assert_ordered_lines(
            &lines,
            &[
                "Open and inspect:",
                "  enter        open change / drill into ~",
                "Move and find:",
                "  ↑/↓, j/k     move selection",
                "  PgDn, Ctrl-f page down",
                "  PgUp, Ctrl-b page up",
                "  →, l         expand change / drill into ~",
                "Change actions:",
                "  m            describe selected revision",
                "  space        mark/unmark selected revision",
                "History and recovery:",
                "  o            open operation log",
                "Session:",
                "  ?, Esc       close help",
            ],
        );
    }

    #[test]
    fn diff_help_lines_group_file_search_and_navigation_commands() {
        let lines = help_lines(BindingContext::Diff);

        assert_help_header(&lines);
        assert_ordered_lines(
            &lines,
            &[
                "Open and inspect:",
                "  f                   open file list",
                "  V                   open view options",
                "Move and find:",
                "  < / >               horizontal scroll",
                "  /, n, N             search, next, previous",
                "Session:",
                "  ?, Esc              close help",
            ],
        );
    }

    #[test]
    fn inspection_help_lines_lead_with_search() {
        let lines = help_lines(BindingContext::Inspection);

        assert_help_header(&lines);
        assert_ordered_lines(
            &lines,
            &[
                "Open and inspect:",
                "  V                   open view options",
                "Move and find:",
                "  /, n, N             search, next, previous",
                "  H / L               go back",
                "Session:",
                "  ?, Esc              close help",
            ],
        );
    }

    #[test]
    fn workspaces_help_lines_keep_workspace_actions_together() {
        let lines = help_lines(BindingContext::Workspaces);

        assert_help_header(&lines);
        assert_ordered_lines(
            &lines,
            &[
                "Open and inspect:",
                "  l                   open selected workspace log",
                "Move and find:",
                "  Backspace, Esc, H/L return to previous view",
                "History and recovery:",
                "  C                   open command history",
                "Session:",
                "  u                   update selected stale workspace",
            ],
        );
    }

    #[test]
    fn command_history_help_lines_group_history_and_recovery() {
        let lines = help_lines(BindingContext::CommandHistory);

        assert_help_header(&lines);
        assert_ordered_lines(
            &lines,
            &[
                "History and recovery:",
                "  enter               open command details",
                "  o                   open operation or operation log",
                "Move and find:",
                "  Backspace, Esc      return to previous view",
                "Session:",
                "  :                   run jj command",
            ],
        );
    }

    #[test]
    fn operation_log_help_lines_keep_operation_routes_visible() {
        let lines = help_lines(BindingContext::OperationLog);

        assert_help_header(&lines);
        assert_ordered_lines(
            &lines,
            &[
                "Open and inspect:",
                "  enter               open selected operation show",
                "  d                   open selected operation diff",
                "Move and find:",
                "  Backspace, Esc      return to previous view",
                "Session:",
                "  r                   refresh operation log",
                "  :                   run jj command",
            ],
        );
    }

    #[test]
    fn hotbar_bindings_have_help() {
        for context in [
            BindingContext::Log,
            BindingContext::Diff,
            BindingContext::Inspection,
            BindingContext::Workspaces,
            BindingContext::CommandHistory,
            BindingContext::OperationLog,
        ] {
            for binding in bindings(context) {
                if binding.hotbar.is_some() {
                    assert!(!binding.help.is_empty());
                }
            }
        }
    }

    fn discovery_row_for_key(rows: &[DiscoveryRow], key: &str) -> DiscoveryRow {
        rows.iter()
            .find(|row| row.keys == key)
            .map_or_else(|| panic!("missing discovery row for key {key}"), |row| *row)
    }

    #[test]
    fn log_discovery_finds_mark_and_clear_bindings() {
        let rows = discovery_rows(BindingContext::Log);
        let mark_row = discovery_row_for_key(&rows, "space");
        assert_eq!(mark_row.command_family_label(), Some("mark"));

        let clear_row = discovery_row_for_key(&rows, "c");
        assert_eq!(clear_row.action, "Clear marks");
    }

    #[test]
    fn log_discovery_keeps_evolog_binding() {
        let rows = discovery_rows(BindingContext::Log);
        let evolog_row = discovery_row_for_key(&rows, "v");

        assert_eq!(evolog_row.action, "Open evolog");
        assert_eq!(evolog_row.command_family_label(), Some("jj evolog"));
    }

    #[test]
    fn workspaces_discovery_keeps_workspace_actions() {
        let rows = discovery_rows(BindingContext::Workspaces);

        let log_row = discovery_row_for_key(&rows, "l");
        assert_eq!(log_row.command_family_label(), Some("jj log"));

        let status_row = discovery_row_for_key(&rows, "enter, s");
        assert_eq!(status_row.command_family_label(), Some("jj status"));

        let back_row = discovery_row_for_key(&rows, "Backspace, Esc, H/L");
        assert_eq!(back_row.context_label(), "workspaces");

        let update_row = discovery_row_for_key(&rows, "u");
        assert_eq!(update_row.command_family_label(), Some("jj workspace"));
    }

    #[test]
    fn discovery_keeps_command_history_entry_point() {
        for context in [
            BindingContext::Log,
            BindingContext::Diff,
            BindingContext::Inspection,
            BindingContext::Workspaces,
        ] {
            let rows = discovery_rows(context);
            let row = discovery_row_for_key(&rows, "C");

            assert_eq!(row.command_family_label(), Some("history"), "{context:?}");
        }
    }

    #[test]
    fn discovery_keeps_colon_command_mode() {
        for context in [
            BindingContext::Log,
            BindingContext::Diff,
            BindingContext::Inspection,
            BindingContext::Workspaces,
            BindingContext::CommandHistory,
            BindingContext::OperationLog,
        ] {
            let rows = discovery_rows(context);
            let row = discovery_row_for_key(&rows, ":");

            assert_eq!(row.action, "Run jj command");
            assert_eq!(row.command_family_label(), Some("jj command"));
        }
    }

    #[test]
    fn command_history_discovery_keeps_operation_route() {
        let rows = discovery_rows(BindingContext::CommandHistory);
        let row = discovery_row_for_key(&rows, "o");

        assert_eq!(row.action, "Open operation");
        assert_eq!(row.command_family_label(), Some("jj operation"));
    }

    #[test]
    fn operation_log_discovery_keeps_show_diff_and_refresh() {
        let rows = discovery_rows(BindingContext::OperationLog);

        let show_row = discovery_row_for_key(&rows, "enter");
        assert_eq!(show_row.command_family_label(), Some("jj operation"));

        let diff_row = discovery_row_for_key(&rows, "d");
        assert_eq!(diff_row.command_family_label(), Some("jj operation"));

        let refresh_row = discovery_row_for_key(&rows, "r");
        assert_eq!(refresh_row.command_family_label(), Some("refresh"));
    }

    #[test]
    fn discovery_lines_use_inline_context_and_scroll_affordance() {
        let lines = discovery_lines(BindingContext::Log, "", 0);

        assert!(!lines.iter().any(|line| line.contains("family")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("enter") && line.contains("jj show"))
        );
        assert!(!lines.iter().any(|line| line.contains('>')));
        assert!(
            lines
                .iter()
                .any(|line| line.contains('d') && line.contains("jj diff"))
        );
        assert!(!lines.iter().any(|line| line.starts_with("showing lines")));
        assert!(!lines.iter().any(String::is_empty));
    }

    #[test]
    fn discovery_lines_use_aligned_columns_when_wide() {
        let lines = discovery_lines_for_width(BindingContext::Log, "", 0, 120);

        let view_line = lines
            .iter()
            .find(|line| line.contains("jj show") && line.contains("jj diff"))
            .unwrap_or_else(|| panic!("missing two-column view line in {lines:?}"));

        assert!(view_line.contains("enter"));
        assert!(view_line.contains('d'));
        assert!(visible_width(view_line) <= 120);
    }

    #[test]
    fn discovery_lines_do_not_truncate_keys_or_instructions() {
        let lines = discovery_lines_for_width(BindingContext::Log, "", 0, DISCOVERY_DEFAULT_WIDTH);

        assert!(lines.iter().any(|line| line.contains("PgDn, Ctrl-f")));
        assert!(lines.iter().any(|line| line.contains("PgUp, Ctrl-b")));
        assert!(lines.iter().any(|line| line.contains("jj log")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains('q') && line.contains("Quit / exit"))
        );
        assert!(!lines.iter().any(|line| line.contains("...")));
    }

    #[test]
    fn discovery_lines_compress_chrome_in_compact_width() {
        let lines = discovery_lines_for_width(BindingContext::Log, "", 0, 40);

        assert_eq!(lines.first().map(String::as_str), Some("Open and inspect:"));
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("showing lines 1-"))
        );
        assert!(lines.iter().any(|line| line.contains("j/k scroll")));
        assert!(lines.iter().any(|line| line.contains("↑/↓, j/k")));
        assert!(!lines.iter().any(|line| line.contains("...")));
    }

    #[test]
    fn discovery_lines_fit_common_terminal_widths() {
        for context in [
            BindingContext::Log,
            BindingContext::Diff,
            BindingContext::Inspection,
            BindingContext::Workspaces,
            BindingContext::CommandHistory,
            BindingContext::OperationLog,
        ] {
            for width in [40, 60, 80, 100, 120, 130] {
                for visible_rows in [8, 12, 20, 34] {
                    let lines =
                        discovery_lines_for_width_and_rows(context, "", 0, width, visible_rows);

                    for line in &lines {
                        assert!(
                            visible_width(line) <= width,
                            "{context:?} width {width} rows {visible_rows}: {line:?} in {lines:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn discovery_lines_adapt_columns_to_width_and_height() {
        let compact = discovery_lines_for_width_and_rows(BindingContext::Log, "", 0, 40, 12);
        assert!(
            !compact
                .iter()
                .any(|line| line.contains("jj show") && line.contains("jj diff"))
        );
        assert!(!compact.iter().any(String::is_empty));
        assert!(compact.iter().any(|line| line.contains("more below")));

        let wide_short = discovery_lines_for_width_and_rows(BindingContext::Log, "", 0, 120, 12);
        assert!(
            wide_short
                .iter()
                .any(|line| line.contains("jj show") && line.contains("jj diff"))
        );
        assert!(wide_short.iter().any(|line| line.contains("more below")));

        let wide_tall = discovery_lines_for_width_and_rows(BindingContext::Log, "", 0, 120, 34);
        assert!(
            wide_tall
                .iter()
                .any(|line| line.contains("jj show") && line.contains("jj diff"))
        );
        assert!(
            !wide_tall
                .iter()
                .any(|line| line.starts_with("showing lines"))
        );
        assert!(wide_tall.iter().any(String::is_empty));
    }

    #[test]
    fn discovery_lines_scroll_by_rendered_line_when_wide() {
        let top_lines = discovery_lines_for_width_and_rows(BindingContext::Log, "", 0, 120, 12);
        let scrolled_lines =
            discovery_lines_for_width_and_rows(BindingContext::Log, "", 1, 120, 12);

        assert_eq!(top_lines[0], "Open and inspect:");
        assert_eq!(scrolled_lines[0], top_lines[1]);
        assert!(
            scrolled_lines
                .iter()
                .any(|line| line.starts_with("showing lines 2-13 of ")
                    && line.contains("more above/below"))
        );
    }
}
