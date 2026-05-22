use crate::jj::{DiffFormat, JjCommand};

/// One static row in the view-switching menu.
///
/// The option is copied into overlay projection only; dispatch remains in `app.rs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ViewMenuOption {
    /// User-visible label rendered in the global view menu.
    label: &'static str,
    /// App-owned action requested when this row is accepted.
    action: ViewMenuAction,
}

impl ViewMenuOption {
    /// Returns the user-visible menu label without implying any dispatch behavior.
    ///
    /// `tui` renders this text as provided; changing wording here is a user-visible screen change.
    pub fn label(self) -> &'static str {
        self.label
    }

    /// Returns the app-owned action requested by this static menu row.
    ///
    /// The action is data for dispatch only. Opening views, changing diff format, refreshing, and
    /// surfacing errors remain in the app navigation/lifecycle code.
    pub fn action(self) -> ViewMenuAction {
        self.action
    }
}

/// Action selected from the global view menu.
///
/// Opening a view and changing diff format are app-owned effects; the menu only supplies the
/// user's requested target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewMenuAction {
    /// Open one shipped top-level view.
    Open(JjCommand),
    /// Apply one app-level show/diff format choice.
    DiffFormat(DiffFormat),
}

/// Static view menu entries shown by shared TUI chrome.
///
/// This table is the shared source for overlay rendering, selected-index clamping, and app
/// navigation lookup. The labels are user-visible command names. Feature-specific view behavior and
/// loading still belong to `ViewState` and the individual view modules.
pub fn view_menu_options() -> &'static [ViewMenuOption] {
    &[
        ViewMenuOption {
            label: "log",
            action: ViewMenuAction::Open(JjCommand::Log),
        },
        ViewMenuOption {
            label: "jj default",
            action: ViewMenuAction::Open(JjCommand::Default),
        },
        ViewMenuOption {
            label: "status",
            action: ViewMenuAction::Open(JjCommand::Status),
        },
        ViewMenuOption {
            label: "resolve",
            action: ViewMenuAction::Open(JjCommand::Resolve),
        },
        ViewMenuOption {
            label: "bookmarks",
            action: ViewMenuAction::Open(JjCommand::Bookmarks),
        },
        ViewMenuOption {
            label: "workspaces",
            action: ViewMenuAction::Open(JjCommand::Workspaces),
        },
        ViewMenuOption {
            label: "operation log",
            action: ViewMenuAction::Open(JjCommand::OperationLog),
        },
        ViewMenuOption {
            label: "show/diff format: default jj",
            action: ViewMenuAction::DiffFormat(DiffFormat::Default),
        },
        ViewMenuOption {
            label: "show/diff format: git (--git)",
            action: ViewMenuAction::DiffFormat(DiffFormat::Git),
        },
    ]
}
