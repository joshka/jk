/// Preflight result for an abandon confirmation screen.
///
/// The preview keeps jj's diff summary text and only classifies empty versus
/// non-empty changes for confirmation strength. It does not decide refresh or
/// reveal behavior after abandon completes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JjAbandonPreview {
    /// Exact selected revision shown in the confirmation screen.
    revision: String,
    /// First description line loaded from `jj log`, when available.
    title: Option<String>,
    /// `jj diff --summary` output preserved for the confirmation screen.
    summary: String,
    /// Empty versus non-empty classification derived from the summary text.
    change_state: AbandonChangeState,
}

impl JjAbandonPreview {
    /// Classify preflight output only by whether jj reported a non-empty diff summary.
    /// Builds the confirmation preview and classifies empty versus non-empty change state.
    pub fn new(revision: String, title: Option<String>, summary: String) -> Self {
        let change_state = if summary.trim().is_empty() {
            AbandonChangeState::Empty
        } else {
            AbandonChangeState::NonEmpty
        };

        Self {
            revision,
            title,
            summary,
            change_state,
        }
    }

    #[cfg(test)]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Return whether abandon can use the weaker empty-change confirmation flow.
    pub fn is_empty_change(&self) -> bool {
        self.change_state == AbandonChangeState::Empty
    }

    /// Build the confirmation text from jj preflight output without simulating abandon results.
    pub fn preview_text(&self) -> String {
        let title = self.title.as_deref().unwrap_or("<no description>");
        let summary = if self.summary.trim().is_empty() {
            "empty change".to_owned()
        } else {
            self.summary.trim().to_owned()
        };
        let confirmation = if self.is_empty_change() {
            "press Enter to abandon this empty change".to_owned()
        } else {
            format!(
                "type exact revision '{}' before abandon runs",
                self.revision
            )
        };

        format!(
            "change: {}\ntitle: {}\ndiff summary:\n{}\n\neffect: abandon removes the selected change from the visible history; recovery stays available through jj undo\nconfirmation: {}\nundo path: jj undo",
            self.revision, title, summary, confirmation
        )
    }
}

/// Preflight only cares whether the diff summary is empty or not.
///
/// More detailed abandon policy belongs in the preview builder and app
/// confirmation flow, not in this local classifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AbandonChangeState {
    /// `jj diff --summary` reported no visible content changes.
    Empty,
    /// `jj diff --summary` reported content changes.
    NonEmpty,
}
