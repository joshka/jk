//! Copy-menu options shared by view slices.

/// A label/value pair that can be offered in the copy modal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyOption {
    /// User-facing label shown in the copy modal for this value.
    label: String,
    /// Exact text copied to the clipboard when this option is chosen.
    value: String,
}

impl CopyOption {
    /// Builds one copyable label/value pair for the modal shown by app dispatch.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }

    /// Returns the user-facing label for this copy target.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the exact text written when this option is copied.
    pub fn value(&self) -> &str {
        &self.value
    }
}
