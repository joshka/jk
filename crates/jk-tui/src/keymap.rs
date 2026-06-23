//! Display metadata for contextual key help and hotbar text.
//!
//! Dispatch still lives in the binary's terminal adapter. This module is only the visible binding
//! registry used by TUI chrome until input dispatch can share the same data model.

/// Screen-specific binding set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingContext {
    Log,
    Diff,
    Inspection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActionId {
    Move,
    LineScroll,
    Page,
    FirstLast,
    Expand,
    Collapse,
    OpenShow,
    OpenDiff,
    OpenStatus,
    SwitchTemplate,
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
    CloseHelp,
    Quit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct KeyBinding {
    action: ActionId,
    keys: &'static str,
    help: &'static str,
    hotbar: Option<&'static str>,
    hotbar_rank: Option<u8>,
    show_in_help: bool,
}

impl KeyBinding {
    const fn new(action: ActionId, keys: &'static str, help: &'static str) -> Self {
        Self {
            action,
            keys,
            help,
            hotbar: None,
            hotbar_rank: None,
            show_in_help: true,
        }
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
    KeyBinding::new(ActionId::Move, "j/k or arrows", "move selection").with_hotbar(8, "j/k move"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line"),
    KeyBinding::new(ActionId::Page, "space / b, Ctrl-f/b", "page down/up")
        .with_hotbar(9, "space/b page"),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom"),
    KeyBinding::new(ActionId::OpenShow, "enter", "open selected-change show")
        .with_hotbar(4, "enter show"),
    KeyBinding::new(ActionId::Expand, "right, l", "expand selected change"),
    KeyBinding::new(ActionId::Collapse, "left, h", "collapse selected change"),
    KeyBinding::new(ActionId::OpenDiff, "d", "open selected-change diff").with_hotbar(5, "d diff"),
    KeyBinding::new(ActionId::OpenStatus, "s", "open repository status").with_hotbar(6, "s status"),
    KeyBinding::new(ActionId::SwitchTemplate, "T", "switch log template")
        .with_hotbar(7, "T template"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh").with_hotbar(3, "r refresh"),
    KeyBinding::new(ActionId::SwitchLogCommand, "H / L", "home command / jj log")
        .with_hotbar(2, "H home  L log"),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help").with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_hotbar(10, "q quit")
        .hotbar_only(),
];

const DIFF_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "scroll one line").with_hotbar(3, "j/k line"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line"),
    KeyBinding::new(ActionId::Page, "space / b, Ctrl-f/b", "page down/up")
        .with_hotbar(4, "space/b page"),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom"),
    KeyBinding::new(ActionId::File, "[ / ]", "previous/next file"),
    KeyBinding::new(ActionId::Hunk, "{ / }", "previous/next hunk"),
    KeyBinding::new(ActionId::FoldFile, "h / l", "fold/unfold current file"),
    KeyBinding::new(
        ActionId::FoldAll,
        "Ctrl-left/right",
        "fold/unfold all files",
    ),
    KeyBinding::new(ActionId::FoldHunk, "- / +", "fold/unfold current hunk"),
    KeyBinding::new(ActionId::HorizontalScroll, "< / >", "horizontal scroll"),
    KeyBinding::new(ActionId::Search, "/, n, N", "search, next, previous"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh").with_hotbar(2, "r refresh"),
    KeyBinding::new(ActionId::ReturnToLog, "H / L", "return to log"),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help").with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_hotbar(5, "q quit")
        .hotbar_only(),
];

const INSPECTION_BINDINGS: &[KeyBinding] = &[
    KeyBinding::new(ActionId::Move, "j/k or arrows", "scroll one line").with_hotbar(3, "j/k line"),
    KeyBinding::new(ActionId::LineScroll, "Ctrl-j/k", "scroll one line"),
    KeyBinding::new(ActionId::Page, "space / b, Ctrl-f/b", "page down/up")
        .with_hotbar(4, "space/b page"),
    KeyBinding::new(ActionId::FirstLast, "g/G or Home/End", "jump to top/bottom"),
    KeyBinding::new(ActionId::Search, "/, n, N", "search, next, previous"),
    KeyBinding::new(ActionId::Refresh, "r", "refresh").with_hotbar(2, "r refresh"),
    KeyBinding::new(ActionId::ReturnToLog, "H / L", "return to log"),
    KeyBinding::new(ActionId::CloseHelp, "?, q, Esc", "close help").with_hotbar(1, "? help"),
    KeyBinding::new(ActionId::Quit, "q", "quit")
        .with_hotbar(5, "q quit")
        .hotbar_only(),
];

/// Returns hotbar text for the current binding context.
pub(crate) fn hotbar(context: BindingContext) -> String {
    let mut hotbar = bindings(context)
        .iter()
        .filter_map(|binding| binding.hotbar_rank.zip(binding.hotbar))
        .collect::<Vec<_>>();
    hotbar.sort_by_key(|(rank, _)| *rank);
    hotbar
        .into_iter()
        .map(|(_, label)| label)
        .collect::<Vec<_>>()
        .join("  ")
}

/// Returns the help overlay title for the current binding context.
pub(crate) const fn help_title(context: BindingContext) -> &'static str {
    match context {
        BindingContext::Log => "Log keys",
        BindingContext::Diff => "Diff keys",
        BindingContext::Inspection => "Inspection keys",
    }
}

/// Returns generated help lines for the current binding context.
pub(crate) fn help_lines(context: BindingContext) -> Vec<String> {
    bindings(context)
        .iter()
        .filter(|binding| binding.show_in_help)
        .map(|binding| binding.help_line())
        .collect()
}

const fn bindings(context: BindingContext) -> &'static [KeyBinding] {
    match context {
        BindingContext::Log => LOG_BINDINGS,
        BindingContext::Diff => DIFF_BINDINGS,
        BindingContext::Inspection => INSPECTION_BINDINGS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Log),
            "? help  H home  L log  r refresh  enter show  d diff  s status  T template  j/k move  space/b page  q quit"
        );
    }

    #[test]
    fn diff_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Diff),
            "? help  r refresh  j/k line  space/b page  q quit"
        );
    }

    #[test]
    fn inspection_hotbar_matches_current_status_text() {
        assert_eq!(
            hotbar(BindingContext::Inspection),
            "? help  r refresh  j/k line  space/b page  q quit"
        );
    }

    #[test]
    fn log_help_lines_match_current_overlay() {
        assert_eq!(
            help_lines(BindingContext::Log),
            vec![
                "j/k or arrows        move selection",
                "Ctrl-j/k             scroll one line",
                "space / b, Ctrl-f/b  page down/up",
                "g/G or Home/End      jump to top/bottom",
                "enter                open selected-change show",
                "right, l             expand selected change",
                "left, h              collapse selected change",
                "d                    open selected-change diff",
                "s                    open repository status",
                "T                    switch log template",
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
                "r                    refresh",
                "H / L                return to log",
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
                "r                    refresh",
                "H / L                return to log",
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
        ] {
            for binding in bindings(context) {
                if binding.hotbar.is_some() {
                    assert!(!binding.help.is_empty());
                }
            }
        }
    }
}
