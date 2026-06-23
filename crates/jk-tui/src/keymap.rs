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
    Page,
    FirstLast,
    Mark,
    ClearMarks,
    Expand,
    Collapse,
    OpenShow,
    OpenDiff,
    OpenDescribe,
    OpenEvolog,
    OpenStatus,
    OpenOperation,
    OpenOperationLog,
    OpenCommandHistory,
    UpdateStale,
    ViewOptions,
    Refresh,
    SwitchLogCommand,
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

impl ActionId {
    const fn label(self) -> &'static str {
        match self {
            Self::Move => "Move selection",
            Self::LineScroll => "Scroll line",
            Self::Page => "Page",
            Self::FirstLast => "Jump to edge",
            Self::Mark => "Mark revision",
            Self::ClearMarks => "Clear marks",
            Self::Expand => "Expand change",
            Self::Collapse => "Collapse change",
            Self::OpenShow => "Open show",
            Self::OpenDiff => "Open diff",
            Self::OpenDescribe => "Describe revision",
            Self::OpenEvolog => "Open evolog",
            Self::OpenStatus => "Open status",
            Self::OpenOperation => "Open operation",
            Self::OpenOperationLog => "Open operation log",
            Self::OpenCommandHistory => "Open command history",
            Self::UpdateStale => "Update stale",
            Self::ViewOptions => "View options",
            Self::Refresh => "Refresh",
            Self::SwitchLogCommand => "Switch log command",
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
            Self::Quit => "Quit",
        }
    }
}

/// Command or interaction family used by searchable discovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandFamily {
    /// Commands and actions related to `jj log`.
    JjLog,
    /// Commands and actions related to `jj diff`.
    JjDiff,
    /// Commands and actions related to `jj describe`.
    JjDescribe,
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
    /// Help and discovery controls.
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

    fn help_line(self) -> String {
        format!("{:<21}{}", self.keys, self.help)
    }
}

const LOG_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "current row"])
        .with_hotbar(11, "j/k move"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
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
    .with_hotbar(10, "c clear"),
    KeyBinding::new(ActionId::Page, "b, Ctrl-f/b, PgUp/Dn", "page down/up")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"]),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::OpenShow, "enter", "open selected-change show")
        .with_family(CommandFamily::JjShow)
        .with_aliases(&["details", "inspect"])
        .with_hotbar(4, "enter show"),
    KeyBinding::new(ActionId::Expand, "right, l", "expand selected change")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Collapse, "left, h", "collapse selected change")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::OpenDiff, "d", "open selected-change diff")
        .with_family(CommandFamily::JjDiff)
        .with_hotbar(5, "d diff"),
    KeyBinding::new(ActionId::OpenDescribe, "m", "describe selected revision")
        .with_family(CommandFamily::JjDescribe)
        .with_aliases(&["message", "description", "mutation", "preview"])
        .with_hotbar(6, "m describe"),
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
        .with_hotbar(9, "o ops"),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "open command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "template", "jj log"])
        .with_hotbar(13, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(3, "r refresh"),
    KeyBinding::new(ActionId::SwitchLogCommand, "H / L", "home command / jj log")
        .with_family(CommandFamily::JjLog)
        .with_aliases(&["home", "current screen"])
        .with_hotbar(2, "H home  L log"),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(14, "q quit")
        .hotbar_only(),
];

const DIFF_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "scroll one line")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(4, "j/k line"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Page, "space / b, Ctrl-f/b", "page down/up")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(5, "space/b page"),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom")
        .with_family(CommandFamily::Navigation),
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
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "display"])
        .with_hotbar(2, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(3, "r refresh"),
    KeyBinding::new(ActionId::ReturnToLog, "H / L", "go back")
        .with_family(CommandFamily::JjLog)
        .with_aliases(&["back"]),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(6, "q quit")
        .hotbar_only(),
];

const INSPECTION_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "scroll one line")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(4, "j/k line"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Page, "space / b, Ctrl-f/b", "page down/up")
        .with_family(CommandFamily::Navigation)
        .with_hotbar(5, "space/b page"),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Search, "/, n, N", "search, next, previous")
        .with_family(CommandFamily::Search)
        .with_aliases(&["find", "filter", "details", "status"]),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "open command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "display"])
        .with_hotbar(2, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(3, "r refresh"),
    KeyBinding::new(ActionId::ReturnToLog, "H / L", "go back")
        .with_family(CommandFamily::JjLog)
        .with_aliases(&["back"]),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(6, "q quit")
        .hotbar_only(),
];

const WORKSPACES_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "workspace", "current row"])
        .with_hotbar(5, "j/k move"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(
        ActionId::OpenStatus,
        "enter, s",
        "open selected workspace status",
    )
    .with_family(CommandFamily::JjStatus)
    .with_aliases(&["status", "workspace", "selected"])
    .with_hotbar(3, "s status"),
    KeyBinding::new(ActionId::OpenDiff, "d", "open selected workspace diff")
        .with_family(CommandFamily::JjDiff)
        .with_aliases(&["diff", "workspace", "selected"])
        .with_hotbar(4, "d diff"),
    KeyBinding::new(
        ActionId::UpdateStale,
        "u",
        "update selected stale workspace",
    )
    .with_family(CommandFamily::JjWorkspace)
    .with_aliases(&["update", "stale", "workspace", "selected", "refresh"])
    .with_hotbar(5, "u update-stale"),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "open command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(ActionId::ViewOptions, "V", "open view options")
        .with_family(CommandFamily::ViewOptions)
        .with_aliases(&["view", "options", "display"])
        .with_hotbar(7, "V options"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh workspaces")
        .with_family(CommandFamily::Refresh)
        .with_aliases(&["reload", "workspace"])
        .with_hotbar(2, "r refresh"),
    KeyBinding::new(
        ActionId::ReturnBack,
        "Backspace, Esc, H/L",
        "return to previous view",
    )
    .with_family(CommandFamily::Navigation)
    .with_aliases(&["back", "return", "previous"])
    .with_hotbar(8, "Esc back"),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(9, "q quit")
        .hotbar_only(),
];

const COMMAND_HISTORY_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "command", "current row"])
        .with_hotbar(2, "j/k move"),
    KeyBinding::new(ActionId::Page, "space / b, Ctrl-f/b", "page down/up")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"])
        .with_hotbar(3, "space/b page"),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom")
        .with_family(CommandFamily::Navigation),
    KeyBinding::new(ActionId::Refresh, "r", "refresh history")
        .with_family(CommandFamily::Refresh)
        .with_hotbar(4, "r refresh"),
    KeyBinding::new(
        ActionId::OpenOperation,
        "o",
        "open operation or operation log",
    )
    .with_family(CommandFamily::JjOperation)
    .with_aliases(&["operation", "op log", "recovery", "selected command"])
    .with_hotbar(5, "o operation"),
    KeyBinding::new(ActionId::OpenCommandHistory, "C", "refresh command history")
        .with_family(CommandFamily::History)
        .with_aliases(&["commands", "history", "recent"]),
    KeyBinding::new(
        ActionId::ReturnBack,
        "Backspace, Esc",
        "return to previous view",
    )
    .with_family(CommandFamily::Navigation)
    .with_aliases(&["back", "return", "previous"])
    .with_hotbar(6, "Esc back"),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help")
        .with_family(CommandFamily::Help)
        .with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_family(CommandFamily::Quit)
        .with_hotbar(7, "q quit")
        .hotbar_only(),
];

const OPERATION_LOG_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "move selection")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["selection", "operation", "current row"])
        .with_hotbar(4, "j/k move"),
    KeyBinding::new(ActionId::Page, "space / b, Ctrl-f/b", "page down/up")
        .with_family(CommandFamily::Navigation)
        .with_aliases(&["page", "pagedown", "pageup"])
        .with_hotbar(5, "space/b page"),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom")
        .with_family(CommandFamily::Navigation),
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
    KeyBinding::new(
        ActionId::ReturnBack,
        "Backspace, Esc",
        "return to previous view",
    )
    .with_family(CommandFamily::Navigation)
    .with_aliases(&["back", "return", "previous"])
    .with_hotbar(7, "Esc back"),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help")
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
    bindings(context)
        .iter()
        .filter(|binding| binding.show_in_help)
        .map(|binding| binding.help_line())
        .collect()
}

/// A searchable command-discovery row derived from visible keymap metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscoveryRow {
    /// Key text shown in the first column.
    pub keys: &'static str,
    /// Short action label shown in the second column.
    pub action: &'static str,
    /// Help text used for filtering and future detail displays.
    pub help: &'static str,
    /// Screen context where the row applies.
    pub context: BindingContext,
    /// Optional command or interaction family.
    pub command_family: Option<CommandFamily>,
    aliases: &'static [&'static str],
}

impl DiscoveryRow {
    /// Returns the context label used for rendering and filtering.
    #[must_use]
    pub const fn context_label(self) -> &'static str {
        context_label(self.context)
    }

    /// Returns the command family label used for rendering and filtering.
    #[must_use]
    pub const fn command_family_label(self) -> Option<&'static str> {
        match self.command_family {
            Some(family) => Some(family.label()),
            None => None,
        }
    }

    fn matches_token(self, token: &str) -> bool {
        let token = token.to_lowercase();
        self.keys.to_lowercase().contains(&token)
            || self.action.to_lowercase().contains(&token)
            || self.help.to_lowercase().contains(&token)
            || self.context_label().to_lowercase().contains(&token)
            || self
                .command_family_label()
                .is_some_and(|family| family.to_lowercase().contains(&token))
            || self
                .aliases
                .iter()
                .any(|alias| alias.to_lowercase().contains(&token))
    }
}

/// Returns all discoverable rows for the current context.
#[must_use]
pub fn discovery_rows(context: BindingContext) -> Vec<DiscoveryRow> {
    bindings(context)
        .iter()
        .filter(|binding| binding.show_in_discovery)
        .map(|binding| DiscoveryRow {
            keys: binding.keys,
            action: binding.action.label(),
            help: binding.help,
            context,
            command_family: binding.command_family,
            aliases: binding.aliases,
        })
        .collect()
}

/// Returns context rows whose searchable fields match every query token.
#[must_use]
pub fn filter_discovery_rows(context: BindingContext, query: &str) -> Vec<DiscoveryRow> {
    let tokens = query.split_whitespace().collect::<Vec<_>>();
    discovery_rows(context)
        .into_iter()
        .filter(|row| tokens.iter().all(|token| row.matches_token(token)))
        .collect()
}

/// Formats command-discovery popup lines for the current query and selected row.
#[must_use]
pub fn discovery_lines(context: BindingContext, query: &str, selected: usize) -> Vec<String> {
    let rows = filter_discovery_rows(context, query);
    let selected = selected.min(rows.len().saturating_sub(1));
    let mut lines = vec![format!("? {query}")];

    if rows.is_empty() {
        lines.push("  no matching commands".to_owned());
    } else {
        lines.extend(rows.into_iter().enumerate().map(|(index, row)| {
            let marker = if index == selected { ">" } else { " " };
            let family = row.command_family_label().unwrap_or("");
            format!(
                "{marker} {:<18} {:<24} {:<10} {}",
                row.keys,
                row.action,
                row.context_label(),
                family
            )
        }));
    }

    lines.push(String::new());
    lines.push("type to filter   j/k or arrows move   enter/esc close".to_owned());
    lines
}

/// Returns the number of visible discovery rows for clamping selection state.
#[must_use]
pub fn filtered_discovery_len(context: BindingContext, query: &str) -> usize {
    filter_discovery_rows(context, query).len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Log),
            "? help  H home  L log  r refresh  enter show  d diff  m describe  v evolog  s status  space mark  o ops  c clear  j/k move  V options  q quit"
        );
    }

    #[test]
    fn diff_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Diff),
            "? help  V options  r refresh  j/k line  space/b page  q quit"
        );
    }

    #[test]
    fn inspection_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Inspection),
            "? help  V options  r refresh  j/k line  space/b page  q quit"
        );
    }

    #[test]
    fn command_history_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::CommandHistory),
            "? help  j/k move  space/b page  r refresh  o operation  Esc back  q quit"
        );
    }

    #[test]
    fn operation_log_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::OperationLog),
            "? help  enter show  d diff  j/k move  space/b page  r refresh  Esc back  q quit"
        );
    }

    #[test]
    fn workspaces_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Workspaces),
            "? help  r refresh  s status  d diff  j/k move  u update-stale  V options  Esc back  q quit"
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
        assert!(status.contains("enter show"));
        assert!(status.contains("d diff"));
        assert!(status.contains("m describe"));
        assert!(status.contains("v evolog"));
        assert!(status.contains("s status"));
        assert!(status.contains("..."));
        assert!(!status.contains("space mark"));
        assert!(!status.contains("j/k move"));
    }

    #[test]
    fn adaptive_log_hotbar_preserves_rank_order() {
        let status = adaptive_hotbar(BindingContext::Log, 75);

        assert_eq!(
            status,
            "? help  H home  L log  r refresh  enter show  d diff  ...  q quit"
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
    fn log_help_lines_match_current_overlay() {
        assert_eq!(
            help_lines(BindingContext::Log),
            vec![
                "j/k or arrows        move selection",
                "Ctrl-j/k             scroll one line",
                "space                mark/unmark selected revision",
                "c                    clear revision marks when marks exist",
                "b, Ctrl-f/b, PgUp/Dn page down/up",
                "g/G or Home/End      jump to top/bottom",
                "enter                open selected-change show",
                "right, l             expand selected change",
                "left, h              collapse selected change",
                "d                    open selected-change diff",
                "m                    describe selected revision",
                "v                    open selected-change evolog",
                "s                    open repository status",
                "o                    open operation log",
                "C                    open command history",
                "V                    open view options",
                "r                    refresh",
                "H / L                home command / jj log",
                "?, q, Esc            close help",
            ]
        );
    }

    #[test]
    fn diff_help_lines_match_current_overlay() {
        assert_eq!(
            help_lines(BindingContext::Diff),
            vec![
                "j/k or arrows        scroll one line",
                "Ctrl-j/k             scroll one line",
                "space / b, Ctrl-f/b  page down/up",
                "g/G or Home/End      jump to top/bottom",
                "[ / ]                previous/next file",
                "{ / }                previous/next hunk",
                "h / l                fold/unfold current file",
                "Ctrl-left/right      fold/unfold all files",
                "- / +                fold/unfold current hunk",
                "< / >                horizontal scroll",
                "/, n, N              search, next, previous",
                "C                    open command history",
                "V                    open view options",
                "r                    refresh",
                "H / L                go back",
                "?, q, Esc            close help",
            ]
        );
    }

    #[test]
    fn inspection_help_lines_match_current_overlay() {
        assert_eq!(
            help_lines(BindingContext::Inspection),
            vec![
                "j/k or arrows        scroll one line",
                "Ctrl-j/k             scroll one line",
                "space / b, Ctrl-f/b  page down/up",
                "g/G or Home/End      jump to top/bottom",
                "/, n, N              search, next, previous",
                "C                    open command history",
                "V                    open view options",
                "r                    refresh",
                "H / L                go back",
                "?, q, Esc            close help",
            ]
        );
    }

    #[test]
    fn workspaces_help_lines_match_current_overlay() {
        assert_eq!(
            help_lines(BindingContext::Workspaces),
            vec![
                "j/k or arrows        move selection",
                "Ctrl-j/k             scroll one line",
                "enter, s             open selected workspace status",
                "d                    open selected workspace diff",
                "u                    update selected stale workspace",
                "C                    open command history",
                "V                    open view options",
                "r                    refresh workspaces",
                "Backspace, Esc, H/L  return to previous view",
                "?, q, Esc            close help",
            ]
        );
    }

    #[test]
    fn command_history_help_lines_match_current_overlay() {
        assert_eq!(
            help_lines(BindingContext::CommandHistory),
            vec![
                "j/k or arrows        move selection",
                "space / b, Ctrl-f/b  page down/up",
                "g/G or Home/End      jump to top/bottom",
                "r                    refresh history",
                "o                    open operation or operation log",
                "C                    refresh command history",
                "Backspace, Esc       return to previous view",
                "?, q, Esc            close help",
            ]
        );
    }

    #[test]
    fn operation_log_help_lines_match_current_overlay() {
        assert_eq!(
            help_lines(BindingContext::OperationLog),
            vec![
                "j/k or arrows        move selection",
                "space / b, Ctrl-f/b  page down/up",
                "g/G or Home/End      jump to top/bottom",
                "enter                open selected operation show",
                "d                    open selected operation diff",
                "r                    refresh operation log",
                "Backspace, Esc       return to previous view",
                "?, q, Esc            close help",
            ]
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

    #[test]
    fn discovery_filter_matches_all_tokens_case_insensitively() {
        let rows = filter_discovery_rows(BindingContext::Log, "JJ SHOW");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].keys, "enter");
        assert_eq!(rows[0].action, "Open show");
    }

    #[test]
    fn discovery_filter_searches_view_options_aliases() {
        let rows = filter_discovery_rows(BindingContext::Log, "template log");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].keys, "V");
        assert_eq!(rows[0].command_family_label(), Some("view options"));

        let rows = filter_discovery_rows(BindingContext::Log, "view options");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].keys, "V");
    }

    #[test]
    fn log_discovery_finds_mark_and_clear_bindings() {
        let mark_rows = filter_discovery_rows(BindingContext::Log, "space mark");
        assert_eq!(mark_rows.len(), 1);
        assert_eq!(mark_rows[0].keys, "space");
        assert_eq!(mark_rows[0].command_family_label(), Some("mark"));

        let clear_rows = filter_discovery_rows(BindingContext::Log, "clear marks");
        assert_eq!(clear_rows.len(), 1);
        assert_eq!(clear_rows[0].keys, "c");
    }

    #[test]
    fn log_discovery_finds_evolog_binding() {
        for query in [
            "v",
            "evolog",
            "jj evolog",
            "evolution history",
            "selected change",
        ] {
            let rows = filter_discovery_rows(BindingContext::Log, query);
            let evolog_row = rows
                .iter()
                .find(|row| row.keys == "v")
                .copied()
                .unwrap_or_else(|| panic!("missing evolog row for query {query:?}"));

            assert_eq!(evolog_row.action, "Open evolog");
            assert_eq!(evolog_row.command_family_label(), Some("jj evolog"));
        }
    }

    #[test]
    fn workspaces_discovery_finds_workspace_actions() {
        let status_rows = filter_discovery_rows(BindingContext::Workspaces, "workspace status");
        assert_eq!(status_rows.len(), 1);
        assert_eq!(status_rows[0].keys, "enter, s");
        assert_eq!(status_rows[0].command_family_label(), Some("jj status"));

        let back_rows = filter_discovery_rows(BindingContext::Workspaces, "back previous");
        assert_eq!(back_rows.len(), 1);
        assert_eq!(back_rows[0].keys, "Backspace, Esc, H/L");
        assert_eq!(back_rows[0].context_label(), "workspaces");

        let update_rows = filter_discovery_rows(BindingContext::Workspaces, "update stale");
        assert_eq!(update_rows.len(), 1);
        assert_eq!(update_rows[0].keys, "u");
        assert_eq!(update_rows[0].command_family_label(), Some("jj workspace"));
    }

    #[test]
    fn discovery_finds_command_history_entry_point() {
        for context in [
            BindingContext::Log,
            BindingContext::Diff,
            BindingContext::Inspection,
            BindingContext::Workspaces,
        ] {
            let rows = filter_discovery_rows(context, "command history");

            assert_eq!(rows.len(), 1, "{context:?}");
            assert_eq!(rows[0].keys, "C");
            assert_eq!(rows[0].command_family_label(), Some("history"));
        }
    }

    #[test]
    fn command_history_discovery_finds_operation_route() {
        for query in ["operation", "op log", "recovery selected command"] {
            let rows = filter_discovery_rows(BindingContext::CommandHistory, query);

            assert_eq!(rows.len(), 1, "{query}");
            assert_eq!(rows[0].keys, "o");
            assert_eq!(rows[0].action, "Open operation");
            assert_eq!(rows[0].command_family_label(), Some("jj operation"));
        }
    }

    #[test]
    fn operation_log_discovery_finds_show_diff_and_refresh() {
        let show_rows = filter_discovery_rows(BindingContext::OperationLog, "operation show");
        assert_eq!(show_rows.len(), 1);
        assert_eq!(show_rows[0].keys, "enter");
        assert_eq!(show_rows[0].command_family_label(), Some("jj operation"));

        let diff_rows = filter_discovery_rows(BindingContext::OperationLog, "operation diff");
        assert_eq!(diff_rows.len(), 1);
        assert_eq!(diff_rows[0].keys, "d");
        assert_eq!(diff_rows[0].command_family_label(), Some("jj operation"));

        let refresh_rows = filter_discovery_rows(BindingContext::OperationLog, "operation reload");
        assert_eq!(refresh_rows.len(), 1);
        assert_eq!(refresh_rows[0].keys, "r");
        assert_eq!(refresh_rows[0].command_family_label(), Some("refresh"));
    }

    #[test]
    fn discovery_filter_keeps_binding_order_without_scoring() {
        let rows = filter_discovery_rows(BindingContext::Diff, "fold");
        let keys = rows.iter().map(|row| row.keys).collect::<Vec<_>>();

        assert_eq!(keys, vec!["h / l", "Ctrl-left/right", "- / +"]);
    }

    #[test]
    fn discovery_lines_render_empty_state_for_no_results() {
        assert_eq!(
            discovery_lines(BindingContext::Inspection, "definitely-not-present", 99),
            vec![
                "? definitely-not-present",
                "  no matching commands",
                "",
                "type to filter   j/k or arrows move   enter/esc close",
            ]
        );
    }
}
