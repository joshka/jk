use super::super::path_actions::{FileActionContext, StatusPathActionAvailability};

/// Exact selection state carried from revision-owned surfaces into action-menu policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactActionContext {
    /// Exact current revision selected on the log/detail surface, if any.
    pub current_revision: Option<String>,
    /// Additional exact selected revisions used for multi-source rewrite menus.
    pub source_revisions: Vec<String>,
    /// Optional selected path carried from detail, file, or status surfaces.
    pub selected_path: Option<String>,
    /// Path-scoped action policy derived from the current surface and selection.
    pub file_action: Option<FileActionContext>,
    /// Whether the selected current revision is the visible working-copy change.
    pub current_is_visible_working_copy: bool,
    /// Surface whose action vocabulary and ordering should be built.
    pub surface: ActionSurface,
}

impl ExactActionContext {
    /// Build a log-surface context with one exact current revision selected.
    pub fn with_current(current_revision: impl Into<String>) -> Self {
        Self {
            current_revision: Some(current_revision.into()),
            source_revisions: Vec::new(),
            selected_path: None,
            file_action: None,
            current_is_visible_working_copy: false,
            surface: ActionSurface::Log,
        }
    }

    /// Build a detail-surface context with one exact current revision selected.
    pub fn detail(current_revision: impl Into<String>) -> Self {
        Self {
            current_revision: Some(current_revision.into()),
            source_revisions: Vec::new(),
            selected_path: None,
            file_action: None,
            current_is_visible_working_copy: false,
            surface: ActionSurface::Detail,
        }
    }

    /// Build a status-surface context for a tracked working-copy path.
    pub fn status_tracked_path(
        path: impl Into<String>,
        availability: StatusPathActionAvailability,
    ) -> Self {
        let path = path.into();
        Self {
            current_revision: Some("@".to_owned()),
            source_revisions: Vec::new(),
            selected_path: Some(path.clone()),
            file_action: Some(FileActionContext::status_tracked_path(path, availability)),
            current_is_visible_working_copy: false,
            surface: ActionSurface::Status,
        }
    }

    #[cfg(test)]
    pub fn status_path(path: impl Into<String>) -> Self {
        Self::status_tracked_path(
            path,
            StatusPathActionAvailability {
                restore_allowed: true,
                chmod_allowed: true,
            },
        )
    }

    /// Build a status-surface context for an untracked working-copy path.
    pub fn status_untracked_path(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            current_revision: Some("@".to_owned()),
            source_revisions: Vec::new(),
            selected_path: Some(path.clone()),
            file_action: Some(FileActionContext::working_copy_untracked(path)),
            current_is_visible_working_copy: false,
            surface: ActionSurface::Status,
        }
    }

    /// Build a file-surface context for the working-copy path view.
    pub fn working_copy_file_path(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            current_revision: None,
            source_revisions: Vec::new(),
            selected_path: Some(path.clone()),
            file_action: Some(FileActionContext::working_copy_file_path(path)),
            current_is_visible_working_copy: false,
            surface: ActionSurface::File,
        }
    }

    #[cfg(test)]
    pub fn none() -> Self {
        Self {
            current_revision: None,
            source_revisions: Vec::new(),
            selected_path: None,
            file_action: None,
            current_is_visible_working_copy: false,
            surface: ActionSurface::Log,
        }
    }

    /// Add the exact source revisions selected alongside the current destination.
    pub fn with_sources<I, S>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.source_revisions = sources.into_iter().map(Into::into).collect();
        self
    }

    /// Add the currently selected path and derive detail-surface file actions when needed.
    pub fn with_selected_path(mut self, path: impl Into<String>) -> Self {
        let path = path.into();
        if self.is_detail_surface()
            && let Some(revision) = self.current_revision.clone()
        {
            self.file_action = Some(FileActionContext::exact_revision_tracked(
                revision,
                path.clone(),
            ));
        }
        self.selected_path = Some(path);
        self
    }

    /// Mark that the selected current revision is the visible working-copy change.
    pub fn with_visible_working_copy(mut self) -> Self {
        self.current_is_visible_working_copy = true;
        self
    }

    /// Return the exact current revision selected on the surface, if any.
    pub fn current_revision(&self) -> Option<&str> {
        self.current_revision.as_deref()
    }

    /// Return the additional selected source revisions used for rewrite menus.
    pub fn source_revisions(&self) -> &[String] {
        &self.source_revisions
    }

    /// Return the optional selected path carried with this context.
    pub fn selected_path(&self) -> Option<&str> {
        self.selected_path.as_deref()
    }

    pub fn current_is_visible_working_copy(&self) -> bool {
        self.current_is_visible_working_copy
    }

    pub fn is_detail_surface(&self) -> bool {
        matches!(self.surface, ActionSurface::Detail)
    }

    pub fn is_status_surface(&self) -> bool {
        matches!(self.surface, ActionSurface::Status)
    }
}

/// Internal classification of which surface owns the current selection semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionSurface {
    /// Graph/log row with a current revision and optional multiselect sources.
    Log,
    /// Detail/document surface rooted at one exact revision.
    Detail,
    /// Working-copy status row for a selected path.
    Status,
    /// File-list surface rooted in the working copy.
    File,
}
