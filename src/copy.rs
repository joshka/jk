//! Copy-menu options shared by view slices.

/// A label/value pair that can be offered in the copy modal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyOption {
    label: String,
    value: String,
}

impl CopyOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
