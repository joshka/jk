//! Command vocabulary, binding metadata, and view dispatch effects.
//!
//! This module owns the app/view command vocabulary, key patterns and labels,
//! key-sequence matching, and the effects views may return to app dispatch.
//! Keep help/menu presentation policy in `help.rs`; this module only exposes the
//! command metadata and filtered match helpers those projections need.

mod bindings;
mod vocabulary;

#[cfg(test)]
pub use crate::help::HelpSectionKind;
/// Help projection types re-exported from the command vocabulary surface.
///
/// Command tables are the source of key identity. `help.rs` decides which rows
/// are visible for a context and how they are grouped for display.
pub use crate::help::{HelpContext, HelpRow, HelpSection, project_help};
#[cfg(test)]
pub use bindings::find_binding;
pub use bindings::{
    Binding, BindingMatch, KeyPattern, binding_prefix_next_labels, help_binding_prefix_next_labels,
    match_binding_sequence, match_help_binding_sequence,
};
pub use vocabulary::{Command, CommandContext, ViewCommand, ViewEffect};

#[cfg(test)]
mod tests;
