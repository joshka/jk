//! Shared log records used by `jk` integration and TUI crates.
//!
//! `jk-core` keeps the boundary between opaque `jj` rendering and semantic log state explicit. The
//! rendered log body is still owned by `jj`; `jk` only keeps enough structured metadata to move by
//! change, preserve selection across refresh, and insert inline details at the right rendered line.

mod command;

pub use command::{ExecutionMode, JjCommandSpec, RefreshPlan, SafetyClass};

/// A rendered `jj` log view plus semantic records for navigation.
///
/// The `rendered` body is opaque terminal text produced by `jj`. Each [`LogEntry`] in `entries`
/// must correspond to one commit row in that rendered text through [`LogEntry::rendered_line`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LogSnapshot {
    title: String,
    rendered: String,
    entries: Vec<LogEntry>,
}

impl LogSnapshot {
    /// Creates a log snapshot from rendered terminal text and semantic entries.
    ///
    /// The caller is responsible for ensuring each entry's rendered line points at the matching
    /// commit row in `rendered`.
    #[must_use]
    pub fn new(rendered: impl Into<String>, entries: Vec<LogEntry>) -> Self {
        Self {
            title: String::new(),
            rendered: rendered.into(),
            entries,
        }
    }

    /// Sets the command context shown in the title bar.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Returns the human-readable command context for the current view.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the opaque output rendered by `jj`.
    #[must_use]
    pub fn rendered(&self) -> &str {
        &self.rendered
    }

    /// Returns semantic records aligned to the rendered log body.
    #[must_use]
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Consumes the snapshot into its owned title, rendered body, and entries.
    #[must_use]
    pub fn into_parts(self) -> (String, String, Vec<LogEntry>) {
        (self.title, self.rendered, self.entries)
    }
}

/// A rendered `jj diff` view for one change.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiffSnapshot {
    title: String,
    change_id: String,
    rendered: String,
    file_stats: Vec<DiffFileStat>,
}

impl DiffSnapshot {
    /// Creates a diff snapshot from the target change and rendered terminal text.
    #[must_use]
    pub fn new(change_id: impl Into<String>, rendered: impl Into<String>) -> Self {
        Self {
            title: String::new(),
            change_id: change_id.into(),
            rendered: rendered.into(),
            file_stats: Vec::new(),
        }
    }

    /// Sets the command context shown in the title bar.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets per-file line counts for the rendered diff.
    #[must_use]
    pub fn with_file_stats(mut self, file_stats: Vec<DiffFileStat>) -> Self {
        self.file_stats = file_stats;
        self
    }

    /// Returns the human-readable command context for the current view.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the `jj` change identifier being inspected.
    #[must_use]
    pub fn change_id(&self) -> &str {
        &self.change_id
    }

    /// Returns the opaque output rendered by `jj`.
    #[must_use]
    pub fn rendered(&self) -> &str {
        &self.rendered
    }

    /// Returns per-file line counts for changed files.
    #[must_use]
    pub fn file_stats(&self) -> &[DiffFileStat] {
        &self.file_stats
    }

    /// Consumes the snapshot into its owned title, target change, and rendered body.
    #[must_use]
    pub fn into_parts(self) -> (String, String, String, Vec<DiffFileStat>) {
        (self.title, self.change_id, self.rendered, self.file_stats)
    }
}

/// Per-file line counts for a rendered diff.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiffFileStat {
    path: String,
    added: usize,
    removed: usize,
    rendered: String,
}

impl DiffFileStat {
    /// Creates line-count stats for one displayed diff path.
    #[must_use]
    pub fn new(path: impl Into<String>, added: usize, removed: usize) -> Self {
        Self {
            path: path.into(),
            added,
            removed,
            rendered: String::new(),
        }
    }

    /// Sets the rendered `--stat` suffix for this path, including the separator.
    #[must_use]
    pub fn with_rendered(mut self, rendered: impl Into<String>) -> Self {
        self.rendered = rendered.into();
        self
    }

    /// Returns the displayed diff path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the number of added lines.
    #[must_use]
    pub const fn added(&self) -> usize {
        self.added
    }

    /// Returns the number of removed lines.
    #[must_use]
    pub const fn removed(&self) -> usize {
        self.removed
    }

    /// Returns the rendered `--stat` suffix for this path.
    #[must_use]
    pub fn rendered(&self) -> &str {
        &self.rendered
    }
}

/// A semantic log item that the TUI can navigate without knowing how `jj` produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogEntry {
    change_id: String,
    commit_id: String,
    description: String,
    details: String,
    rendered_line: usize,
}

impl LogEntry {
    /// Creates a log entry from `jj` change metadata.
    ///
    /// New entries default to rendered line `0`; callers that build a [`LogSnapshot`] should set
    /// the aligned rendered line with [`Self::with_rendered_line`] while constructing entries.
    #[must_use]
    pub fn new(
        change_id: impl Into<String>,
        commit_id: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            change_id: change_id.into(),
            commit_id: commit_id.into(),
            description: description.into(),
            details: String::new(),
            rendered_line: 0,
        }
    }

    /// Sets the template-derived detail text for inline expansion.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = details.into();
        self
    }

    /// Sets the rendered line for this entry.
    #[must_use]
    pub const fn with_rendered_line(mut self, rendered_line: usize) -> Self {
        self.rendered_line = rendered_line;
        self
    }

    /// Returns the stable `jj` change identifier.
    #[must_use]
    pub fn change_id(&self) -> &str {
        &self.change_id
    }

    /// Returns the commit identifier for follow-up inspection commands.
    #[must_use]
    pub fn commit_id(&self) -> &str {
        &self.commit_id
    }

    /// Returns the full commit description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns template-derived detail text for inline expansion.
    #[must_use]
    pub fn details(&self) -> &str {
        &self.details
    }

    /// Returns the zero-based line in the rendered `jj` output for this entry.
    #[must_use]
    pub const fn rendered_line(&self) -> usize {
        self.rendered_line
    }

    /// Returns the first line of the commit description, or a placeholder for empty descriptions.
    #[must_use]
    pub fn summary(&self) -> &str {
        let first_line = self.description.lines().next().unwrap_or_default();
        if first_line.is_empty() {
            "(no description set)"
        } else {
            first_line
        }
    }
}
