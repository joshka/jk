/// One role/value pair in an action prompt that needs an explicit source or destination choice.
///
/// Roles are presentation labels and dispatcher cues, not parsed revsets. The follow-up action plan
/// is responsible for quoting selected values before passing them to `jj`. Values are the exact
/// revision strings selected by the builder, so callers should not normalize them while the prompt is
/// open.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePromptOption {
    /// Presentation label naming how the selected revision will be used.
    role: &'static str,
    /// Exact revision string preserved for later preview-plan construction.
    value: String,
}

impl RolePromptOption {
    /// Build one immutable role/value row for a rewrite prompt.
    pub fn new(role: &'static str, value: impl Into<String>) -> Self {
        Self {
            role,
            value: value.into(),
        }
    }

    /// Return the presentation role shown beside the selected revision.
    pub fn role(&self) -> &'static str {
        self.role
    }

    /// Return the exact revision string that the builder selected.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Render the role/value pair for status text or list rows.
    pub fn label(&self) -> String {
        format!("{}: {}", self.role, self.value)
    }
}

/// Prompt model for actions that need a role choice before preview.
///
/// The prompt is immutable UI state owned by `InteractionMode`; choosing an option only creates the
/// next follow-up, and never executes `jj` directly. The role names currently consumed by app
/// reducers are `"source"` and `"destination"`; additional role semantics belong with the reducer
/// that turns a chosen prompt into a preview plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePrompt {
    /// Prompt title describing the pending rewrite action.
    title: &'static str,
    /// Immutable list of role assignments that the user can inspect or accept.
    options: Vec<RolePromptOption>,
    /// User-facing safety reminder appended beneath the role list.
    preview_required_message: &'static str,
}

impl RolePrompt {
    /// Build the immutable prompt model carried by `InteractionMode::RolePrompt`.
    pub fn new(
        title: &'static str,
        options: Vec<RolePromptOption>,
        preview_required_message: &'static str,
    ) -> Self {
        Self {
            title,
            options,
            preview_required_message,
        }
    }

    /// Return the user-facing action title for the prompt.
    pub fn title(&self) -> &str {
        self.title
    }

    /// Return the ordered role rows shown in the prompt.
    pub fn options(&self) -> &[RolePromptOption] {
        &self.options
    }

    /// Return the safety reminder shown below the role rows.
    pub fn preview_required_message(&self) -> &str {
        self.preview_required_message
    }

    /// Render the prompt rows and preview reminder into a status-text block.
    pub fn status_message(&self) -> String {
        let mut lines = self
            .options
            .iter()
            .map(RolePromptOption::label)
            .collect::<Vec<_>>();
        lines.push(self.preview_required_message.to_owned());
        lines.join("\n")
    }

    /// Return every selected revision whose role is `"source"`.
    pub fn source_revisions(&self) -> Vec<&str> {
        self.options
            .iter()
            .filter(|option| option.role() == "source")
            .map(RolePromptOption::value)
            .collect()
    }

    /// Return the selected revision whose role is `"destination"`, if present.
    pub fn destination_revision(&self) -> Option<&str> {
        self.options
            .iter()
            .find(|option| option.role() == "destination")
            .map(RolePromptOption::value)
    }
}
