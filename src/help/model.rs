#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpContext {
    /// Help shown for the log/default graph view.
    Log,
    /// Help shown for the `jj show` detail view.
    Show,
    /// Help shown for the `jj diff` detail view.
    Diff,
    /// Help shown for the status view.
    Status,
    /// Help shown for the resolve/conflict list view.
    Resolve,
    /// Help shown for the file-list view.
    FileList,
    /// Help shown for the file-show view.
    FileShow,
    /// Help shown for the bookmarks view.
    Bookmarks,
    /// Help shown for the workspaces view.
    Workspaces,
    /// Help shown for the operation-log list view.
    OperationLog,
    /// Help shown for operation show/diff detail views.
    OperationDetail,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpSectionKind {
    /// Movement and viewport navigation commands.
    Navigation,
    /// Commands that switch views or change view presentation.
    Views,
    /// Search and copy related commands.
    SearchCopy,
    /// Repo-wide non-view actions such as fetch or new from trunk.
    RepositoryActions,
    /// Contextual action-preview and mutation commands.
    Actions,
    /// Recovery and undo/redo commands.
    Recovery,
    /// App-level commands such as refresh.
    App,
}

impl HelpSectionKind {
    /// Returns the user-facing title shown for this help section.
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
    /// Comma-joined key labels that trigger the same action in this context.
    keys: String,
    /// User-facing action description shown in the overlay.
    action: &'static str,
}

impl HelpRow {
    /// Builds one projected help row from key labels and action text.
    pub fn new(keys: impl Into<String>, action: &'static str) -> Self {
        Self {
            keys: keys.into(),
            action,
        }
    }

    /// Returns the formatted key label string for this help row.
    pub fn keys(&self) -> &str {
        &self.keys
    }

    /// Returns the action description for this help row.
    pub fn action(&self) -> &str {
        self.action
    }

    pub fn push_key_label(&mut self, key: &str) {
        self.keys.push_str(", ");
        self.keys.push_str(key);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HelpSection {
    /// Logical help grouping for the rows in this section.
    kind: HelpSectionKind,
    /// Projected help rows shown under this section title.
    rows: Vec<HelpRow>,
}

impl HelpSection {
    /// Builds one help section from its grouping kind and projected rows.
    pub fn new(kind: HelpSectionKind, rows: Vec<HelpRow>) -> Self {
        Self { kind, rows }
    }

    /// Returns the user-facing section title.
    pub fn title(&self) -> &'static str {
        self.kind.title()
    }

    /// Returns the projected help rows for this section.
    pub fn rows(&self) -> &[HelpRow] {
        &self.rows
    }
}
